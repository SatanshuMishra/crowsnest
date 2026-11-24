use std::ffi::OsStr;
use std::fs;
use std::io;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer,
};

use crate::arena::info::{self, FILE_NAME};
use crate::model::ArenaInfo;

const DEBOUNCE: Duration = Duration::from_millis(250);
const RETRY_INTERVAL: Duration = Duration::from_millis(200);
const READ_PATIENCE: Duration = Duration::from_secs(3);

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArenaEvent {
    Started(ArenaInfo),
    Ended,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchError {
    #[error("the replays folder {0} does not exist")]
    MissingDir(PathBuf),
    #[error("could not watch the replays folder: {0}")]
    Notify(String),
}

pub struct ArenaWatcher {
    stopping: Arc<AtomicBool>,
    debouncer: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    worker: Option<JoinHandle<()>>,
}

impl ArenaWatcher {
    pub fn start(
        replays_dir: PathBuf,
        on_event: impl Fn(ArenaEvent) + Send + Sync + 'static,
    ) -> Result<ArenaWatcher, WatchError> {
        if !replays_dir.is_dir() {
            return Err(WatchError::MissingDir(replays_dir));
        }
        let (batches, received) = mpsc::channel();
        let mut debouncer = new_debouncer(DEBOUNCE, None, batches).map_err(watch_failure)?;
        debouncer
            .watch(&replays_dir, RecursiveMode::Recursive)
            .map_err(watch_failure)?;
        let stopping = Arc::new(AtomicBool::new(false));
        let worker = Worker {
            replays_dir,
            stopping: Arc::clone(&stopping),
            on_event,
        };
        let handle = thread::Builder::new()
            .name("arena-watcher".into())
            .spawn(move || worker.run(received))
            .map_err(watch_failure)?;
        Ok(ArenaWatcher {
            stopping,
            debouncer: Some(debouncer),
            worker: Some(handle),
        })
    }
}

impl Drop for ArenaWatcher {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(debouncer) = self.debouncer.take() {
            debouncer.stop();
        }
        let joinable = self
            .worker
            .take()
            .filter(|worker| worker.thread().id() != thread::current().id());
        if let Some(worker) = joinable
            && worker.join().is_err()
        {
            log::error!("the arena watcher thread panicked");
        }
    }
}

fn watch_failure(error: impl ToString) -> WatchError {
    WatchError::Notify(error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Change {
    Written(PathBuf),
    Removed,
    NeedsRescan,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Observation {
    Arena(ArenaInfo),
    Removed,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Decision {
    emit: Option<ArenaEvent>,
    last_fingerprint: Option<String>,
}

fn decide(last_fingerprint: Option<&str>, observation: Observation) -> Decision {
    match observation {
        Observation::Arena(arena) => {
            let fingerprint = arena.fingerprint();
            let emit = (last_fingerprint != Some(fingerprint.as_str()))
                .then_some(ArenaEvent::Started(arena));
            Decision {
                emit,
                last_fingerprint: Some(fingerprint),
            }
        }
        Observation::Removed => Decision {
            emit: Some(ArenaEvent::Ended),
            last_fingerprint: None,
        },
        Observation::Absent => Decision {
            emit: last_fingerprint.map(|_| ArenaEvent::Ended),
            last_fingerprint: None,
        },
    }
}

fn is_arena_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.eq_ignore_ascii_case(FILE_NAME))
}

fn change_for(kind: EventKind, path: &Path) -> Option<Change> {
    match kind {
        EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            Some(Change::Removed)
        }
        EventKind::Create(_) | EventKind::Modify(_) => Some(Change::Written(path.to_path_buf())),
        EventKind::Access(_) | EventKind::Any | EventKind::Other => None,
    }
}

fn changes_for(kind: EventKind, paths: &[PathBuf]) -> Vec<Change> {
    match kind {
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            let renamed_away = paths
                .first()
                .filter(|from| is_arena_file(from))
                .map(|_| Change::Removed);
            let renamed_into = paths
                .get(1)
                .filter(|to| is_arena_file(to))
                .map(|to| Change::Written(to.clone()));
            renamed_away.into_iter().chain(renamed_into).collect()
        }
        _ => paths
            .iter()
            .filter(|path| is_arena_file(path))
            .filter_map(|path| change_for(kind, path))
            .collect(),
    }
}

fn changes_in(events: &[DebouncedEvent]) -> Vec<Change> {
    let changes: Vec<Change> = events
        .iter()
        .flat_map(|event| {
            if event.need_rescan() {
                vec![Change::NeedsRescan]
            } else {
                changes_for(event.kind, &event.paths)
            }
        })
        .collect();
    without_repeats(&changes)
}

fn without_repeats(changes: &[Change]) -> Vec<Change> {
    changes
        .iter()
        .enumerate()
        .filter(|(index, change)| {
            index
                .checked_sub(1)
                .and_then(|previous| changes.get(previous))
                != Some(*change)
        })
        .map(|(_, change)| change.clone())
        .collect()
}

#[allow(clippy::large_enum_variant)]
enum ReadAttempt {
    Parsed(ArenaInfo),
    Missing,
    NotReady(String),
}

fn read_arena(path: &Path) -> ReadAttempt {
    match fs::read(path) {
        Ok(bytes) => info::parse(&bytes).map_or_else(
            |error| ReadAttempt::NotReady(error.to_string()),
            ReadAttempt::Parsed,
        ),
        Err(error) if error.kind() == io::ErrorKind::NotFound => ReadAttempt::Missing,
        Err(error) => ReadAttempt::NotReady(error.to_string()),
    }
}

fn read_when_ready(path: &Path, stopping: &AtomicBool) -> Option<ArenaInfo> {
    let deadline = Instant::now() + READ_PATIENCE;
    loop {
        match read_arena(path) {
            ReadAttempt::Parsed(arena) => return Some(arena),
            ReadAttempt::Missing => {
                log::debug!("{} vanished before it could be read", path.display());
                return None;
            }
            ReadAttempt::NotReady(_) if stopping.load(Ordering::SeqCst) => return None,
            ReadAttempt::NotReady(reason) if Instant::now() >= deadline => {
                log::warn!("gave up reading {}: {reason}", path.display());
                return None;
            }
            ReadAttempt::NotReady(reason) => {
                log::debug!("{} is not ready yet: {reason}", path.display());
            }
        }
        thread::sleep(RETRY_INTERVAL);
    }
}

fn entries_of(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .inspect_err(|error| log::warn!("could not list {}: {error}", dir.display()))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default()
}

fn existing_arena_file(replays_dir: &Path) -> Option<PathBuf> {
    let top_level = entries_of(replays_dir);
    let one_level_down: Vec<PathBuf> = top_level
        .iter()
        .filter(|path| path.is_dir())
        .flat_map(|dir| entries_of(dir))
        .collect();
    top_level
        .into_iter()
        .chain(one_level_down)
        .filter(|path| is_arena_file(path) && path.is_file())
        .max_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok())
}

fn batch_changes(batch: DebounceEventResult) -> Vec<Change> {
    match batch {
        Ok(events) => changes_in(&events),
        Err(errors) => {
            errors
                .iter()
                .for_each(|error| log::warn!("replays folder watch error: {error}"));
            Vec::new()
        }
    }
}

struct Worker<F> {
    replays_dir: PathBuf,
    stopping: Arc<AtomicBool>,
    on_event: F,
}

impl<F: Fn(ArenaEvent)> Worker<F> {
    fn run(self, received: Receiver<DebounceEventResult>) {
        let at_start_up: Vec<Change> = existing_arena_file(&self.replays_dir)
            .map(Change::Written)
            .into_iter()
            .collect();
        let later = received.iter().map(batch_changes);
        let _final_fingerprint = iter::once(at_start_up)
            .chain(later)
            .take_while(|_| !self.is_stopping())
            .flatten()
            .fold(None, |last_fingerprint, change| {
                self.apply(last_fingerprint, change)
            });
    }

    fn apply(&self, last_fingerprint: Option<String>, change: Change) -> Option<String> {
        let Some(observation) = self.observe(change).filter(|_| !self.is_stopping()) else {
            return last_fingerprint;
        };
        let decision = decide(last_fingerprint.as_deref(), observation);
        if let Some(event) = decision.emit {
            (self.on_event)(event);
        }
        decision.last_fingerprint
    }

    fn observe(&self, change: Change) -> Option<Observation> {
        match change {
            Change::Written(path) => self.read(&path),
            Change::Removed => Some(Observation::Removed),
            Change::NeedsRescan => existing_arena_file(&self.replays_dir)
                .map_or(Some(Observation::Absent), |path| self.read(&path)),
        }
    }

    fn read(&self, path: &Path) -> Option<Observation> {
        read_when_ready(path, &self.stopping).map(Observation::Arena)
    }

    fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::RecvTimeoutError;

    use notify::Event;
    use notify::event::{AccessKind, CreateKind, DataChange, Flag, RemoveKind};

    use super::*;

    const FIXTURE: &[u8] = include_bytes!("fixtures/tempArenaInfo.json");
    const PATIENCE: Duration = Duration::from_secs(15);

    fn fixture() -> ArenaInfo {
        info::parse(FIXTURE).expect("the fixture parses")
    }

    fn replays() -> PathBuf {
        PathBuf::from("replays")
    }

    fn arena_path() -> PathBuf {
        replays().join(FILE_NAME)
    }

    fn debounced(kind: EventKind, paths: &[PathBuf]) -> DebouncedEvent {
        DebouncedEvent::new(
            paths
                .iter()
                .cloned()
                .fold(Event::new(kind), Event::add_path),
            Instant::now(),
        )
    }

    #[test]
    fn creating_or_modifying_the_arena_file_is_a_write() {
        let kinds = [
            EventKind::Create(CreateKind::File),
            EventKind::Create(CreateKind::Any),
            EventKind::Modify(ModifyKind::Any),
            EventKind::Modify(ModifyKind::Data(DataChange::Content)),
            EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        ];

        kinds.into_iter().for_each(|kind| {
            assert_eq!(
                changes_for(kind, &[arena_path()]),
                vec![Change::Written(arena_path())],
                "{kind:?}"
            );
        });
    }

    #[test]
    fn removing_or_renaming_away_the_arena_file_is_a_removal() {
        let kinds = [
            EventKind::Remove(RemoveKind::File),
            EventKind::Remove(RemoveKind::Any),
            EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        ];

        kinds.into_iter().for_each(|kind| {
            assert_eq!(
                changes_for(kind, &[arena_path()]),
                vec![Change::Removed],
                "{kind:?}"
            );
        });
    }

    #[test]
    fn matches_the_file_name_case_insensitively() {
        let shouted = replays().join("TEMPARENAINFO.JSON");

        assert_eq!(
            changes_for(
                EventKind::Create(CreateKind::File),
                std::slice::from_ref(&shouted)
            ),
            vec![Change::Written(shouted)]
        );
    }

    #[test]
    fn ignores_other_files_and_access_events() {
        let replay = replays().join("20261029_192149_PRSB108-Vladivostok_38_Canada.wowsreplay");

        assert!(changes_for(EventKind::Create(CreateKind::File), &[replay]).is_empty());
        assert!(changes_for(EventKind::Access(AccessKind::Any), &[arena_path()]).is_empty());
        assert!(changes_for(EventKind::Other, &[arena_path()]).is_empty());
        assert!(changes_for(EventKind::Any, &[arena_path()]).is_empty());
    }

    #[test]
    fn a_rename_with_both_paths_reports_each_side() {
        let other = replays().join("temp.json");
        let both = EventKind::Modify(ModifyKind::Name(RenameMode::Both));

        assert_eq!(
            changes_for(both, &[other.clone(), arena_path()]),
            vec![Change::Written(arena_path())]
        );
        assert_eq!(
            changes_for(both, &[arena_path(), other]),
            vec![Change::Removed]
        );
    }

    #[test]
    fn a_rescan_notice_asks_for_a_rescan() {
        let rescan = DebouncedEvent::new(
            Event::new(EventKind::Other).set_flag(Flag::Rescan),
            Instant::now(),
        );

        assert_eq!(changes_in(&[rescan]), vec![Change::NeedsRescan]);
    }

    #[test]
    fn repeated_changes_in_one_batch_collapse() {
        let events = [
            debounced(EventKind::Create(CreateKind::File), &[arena_path()]),
            debounced(EventKind::Modify(ModifyKind::Any), &[arena_path()]),
            debounced(EventKind::Remove(RemoveKind::File), &[arena_path()]),
            debounced(EventKind::Remove(RemoveKind::Any), &[arena_path()]),
            debounced(EventKind::Create(CreateKind::File), &[arena_path()]),
        ];

        assert_eq!(
            changes_in(&events),
            vec![
                Change::Written(arena_path()),
                Change::Removed,
                Change::Written(arena_path()),
            ]
        );
    }

    #[test]
    fn a_new_arena_starts_a_battle_and_is_remembered() {
        let arena = fixture();

        assert_eq!(
            decide(None, Observation::Arena(arena.clone())),
            Decision {
                emit: Some(ArenaEvent::Started(arena.clone())),
                last_fingerprint: Some(arena.fingerprint()),
            }
        );
    }

    #[test]
    fn the_same_arena_is_not_announced_twice() {
        let arena = fixture();
        let fingerprint = arena.fingerprint();

        assert_eq!(
            decide(Some(&fingerprint), Observation::Arena(arena)),
            Decision {
                emit: None,
                last_fingerprint: Some(fingerprint.clone()),
            }
        );
    }

    #[test]
    fn a_different_arena_starts_a_new_battle() {
        let previous = fixture().fingerprint();
        let next = ArenaInfo {
            started_at: "2026-10-29T19:45:02".into(),
            ..fixture()
        };

        assert_eq!(
            decide(Some(&previous), Observation::Arena(next.clone())),
            Decision {
                emit: Some(ArenaEvent::Started(next.clone())),
                last_fingerprint: Some(next.fingerprint()),
            }
        );
    }

    #[test]
    fn removal_ends_the_battle_and_forgets_the_arena() {
        let ended = Decision {
            emit: Some(ArenaEvent::Ended),
            last_fingerprint: None,
        };

        assert_eq!(
            decide(Some(&fixture().fingerprint()), Observation::Removed),
            ended
        );
        assert_eq!(decide(None, Observation::Removed), ended);
    }

    #[test]
    fn an_absent_file_after_a_rescan_ends_only_a_known_battle() {
        assert_eq!(
            decide(Some(&fixture().fingerprint()), Observation::Absent),
            Decision {
                emit: Some(ArenaEvent::Ended),
                last_fingerprint: None,
            }
        );
        assert_eq!(
            decide(None, Observation::Absent),
            Decision {
                emit: None,
                last_fingerprint: None,
            }
        );
    }

    fn start_collecting(replays_dir: &Path) -> (ArenaWatcher, Receiver<ArenaEvent>) {
        let (sender, events) = mpsc::channel();
        let watcher = ArenaWatcher::start(replays_dir.to_path_buf(), move |event| {
            let _ = sender.send(event);
        })
        .expect("the watcher starts");
        (watcher, events)
    }

    #[test]
    fn a_missing_replays_folder_is_rejected() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let missing = dir.path().join("replays");

        assert!(matches!(
            ArenaWatcher::start(missing.clone(), |_| {}),
            Err(WatchError::MissingDir(path)) if path == missing
        ));
    }

    #[test]
    fn reports_a_battle_starting_and_ending_in_the_replays_folder() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (watcher, events) = start_collecting(dir.path());
        let arena_file = dir.path().join(FILE_NAME);

        fs::write(&arena_file, FIXTURE).expect("the fixture is written");
        assert_eq!(
            events.recv_timeout(PATIENCE),
            Ok(ArenaEvent::Started(fixture()))
        );

        fs::remove_file(&arena_file).expect("the fixture is removed");
        assert_eq!(events.recv_timeout(PATIENCE), Ok(ArenaEvent::Ended));

        drop(watcher);
    }

    #[test]
    fn an_arena_file_present_at_start_up_counts_as_created() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        fs::write(dir.path().join(FILE_NAME), FIXTURE).expect("the fixture is written");

        let (_watcher, events) = start_collecting(dir.path());

        assert_eq!(
            events.recv_timeout(PATIENCE),
            Ok(ArenaEvent::Started(fixture()))
        );
    }

    #[test]
    fn dropping_the_watcher_stops_its_thread() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (watcher, events) = start_collecting(dir.path());

        drop(watcher);
        fs::write(dir.path().join(FILE_NAME), FIXTURE).expect("the fixture is written");

        assert_eq!(
            events.recv_timeout(PATIENCE),
            Err(RecvTimeoutError::Disconnected)
        );
    }

    #[test]
    fn a_battle_in_a_version_folder_created_after_start_is_reported() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (_watcher, events) = start_collecting(dir.path());
        let version_folder = dir.path().join("15.11.0.0");

        fs::create_dir(&version_folder).expect("the version folder is created");
        fs::write(version_folder.join(FILE_NAME), FIXTURE).expect("the fixture is written");

        assert_eq!(
            events.recv_timeout(PATIENCE),
            Ok(ArenaEvent::Started(fixture()))
        );
    }

    #[test]
    fn an_arena_file_in_a_version_folder_at_start_up_counts_as_created() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let version_folder = dir.path().join("15.10.0.0");
        fs::create_dir(&version_folder).expect("the version folder is created");
        fs::write(version_folder.join(FILE_NAME), FIXTURE).expect("the fixture is written");

        let (_watcher, events) = start_collecting(dir.path());

        assert_eq!(
            events.recv_timeout(PATIENCE),
            Ok(ArenaEvent::Started(fixture()))
        );
    }
}
