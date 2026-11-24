use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::Read;
use std::iter;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use tokio::sync::oneshot;
use wows_replays::ReplayFile;
use wows_replays::analyzer::arena_scan::scan_arena_state;
use wowsunpack::data::Version;
use wowsunpack::game_data::load_game_resources;
use wowsunpack::rpc::entitydefs::EntitySpec;

use crate::arena::info;
use crate::model::ArenaInfo;

const REPLAY_FILE_NAME: &str = "temp.wowsreplay";
const ARENA_FILE_NAME: &str = "tempArenaInfo.json";
const CONTAINER_MAGIC: [u8; 4] = [0x12, 0x32, 0x34, 0x11];
const READ_LIMIT: u64 = 1 << 20;
const PARSER_STACK: usize = 256 << 20;
const LARGEST_DIVISION: usize = 3;
const ON_START_AFTER_NAME: i64 = 2;
const PREBATTLE_AFTER_NAME: i64 = 4;

pub type Divisions = HashMap<String, u8>;
type RawPlayer = HashMap<i64, String>;
type Specs = Arc<Vec<EntitySpec>>;

#[derive(Default)]
pub struct SpecCache(Mutex<Option<(u32, Option<Specs>)>>);

impl SpecCache {
    fn specs_for(&self, game_root: &Path, version: &Version) -> Option<Specs> {
        let build = version.build_number()?;
        let mut slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, specs)) = slot.as_ref().filter(|(cached, _)| *cached == build) {
            return specs.clone();
        }
        let specs = load_specs(game_root, version);
        *slot = Some((build, specs.clone()));
        specs
    }
}

fn load_specs(game_root: &Path, version: &Version) -> Option<Specs> {
    match panic::catch_unwind(AssertUnwindSafe(|| load_game_resources(game_root, version))) {
        Ok(Ok(resources)) => Some(Arc::new(resources.specs)),
        Ok(Err(error)) => {
            log::warn!(
                "divisions are unavailable: the game's entity definitions did not load: {error}"
            );
            None
        }
        Err(_) => {
            log::warn!(
                "divisions are unavailable: this game build's entity definitions are in a shape Crowsnest cannot read"
            );
            None
        }
    }
}

pub async fn read_divisions(
    specs: Arc<SpecCache>,
    game_root: PathBuf,
    watch_root: PathBuf,
    arena: ArenaInfo,
) -> Option<Divisions> {
    let (sender, receiver) = oneshot::channel();
    thread::Builder::new()
        .name("division reader".into())
        .stack_size(PARSER_STACK)
        .spawn(move || sender.send(read_now(&specs, &game_root, &watch_root, &arena)))
        .ok()?;
    receiver.await.ok().flatten()
}

fn read_now(
    specs: &SpecCache,
    game_root: &Path,
    watch_root: &Path,
    arena: &ArenaInfo,
) -> Option<Divisions> {
    battle_folders(watch_root, arena)
        .iter()
        .find_map(|(folder, meta)| read_folder(specs, game_root, folder, meta, arena))
}

fn battle_folders(watch_root: &Path, arena: &ArenaInfo) -> Vec<(PathBuf, Vec<u8>)> {
    let nested = fs::read_dir(watch_root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir());
    iter::once(watch_root.to_path_buf())
        .chain(nested)
        .filter_map(|folder| {
            let meta = fs::read(folder.join(ARENA_FILE_NAME)).ok()?;
            let written = info::parse(&meta).ok()?;
            (written.fingerprint() == arena.fingerprint()).then_some((folder, meta))
        })
        .collect()
}

fn read_folder(
    specs: &SpecCache,
    game_root: &Path,
    folder: &Path,
    meta: &[u8],
    arena: &ArenaInfo,
) -> Option<Divisions> {
    let stream = read_prefix(&folder.join(REPLAY_FILE_NAME))?;
    let replay = if stream.starts_with(&CONTAINER_MAGIC) {
        ReplayFile::from_partial_bytes(&stream).ok()?
    } else {
        ReplayFile::from_decrypted_parts(meta.to_vec(), stream).ok()?
    };
    let version = Version::try_from_client_exe(&replay.meta.clientVersionFromExe)?;
    let specs = specs.specs_for(game_root, &version)?;
    let state = scan_arena_state(&specs, version, &replay)?;
    let players: Vec<&RawPlayer> = state.players.iter().map(|player| player.raw()).collect();
    divisions_in(&players, arena)
}

fn read_prefix(path: &Path) -> Option<Vec<u8>> {
    let mut prefix = Vec::new();
    File::open(path)
        .ok()?
        .take(READ_LIMIT)
        .read_to_end(&mut prefix)
        .ok()?;
    Some(prefix)
}

struct Seat<'a> {
    name: &'a str,
    friendly: bool,
    name_index: i64,
    prebattle: Option<i64>,
    on_start: Option<i64>,
}

fn divisions_in(players: &[&RawPlayer], arena: &ArenaInfo) -> Option<Divisions> {
    let humans: HashMap<String, (&str, bool)> = arena
        .vehicles
        .iter()
        .filter(|vehicle| !vehicle.is_bot())
        .map(|vehicle| {
            let seat = (vehicle.name.as_str(), vehicle.relation.is_friendly());
            (format!("{:?}", vehicle.name), seat)
        })
        .collect();
    let seats: Vec<Seat> = players
        .iter()
        .filter_map(|raw| seat_of(raw, &humans))
        .collect();
    let names: BTreeSet<&str> = seats.iter().map(|seat| seat.name).collect();
    let name_indices: BTreeSet<i64> = seats.iter().map(|seat| seat.name_index).collect();
    let calibrated = names.len() == humans.len()
        && seats.len() == humans.len()
        && name_indices.len() == 1
        && seats
            .iter()
            .all(|seat| seat.prebattle.is_some() && seat.prebattle == seat.on_start);
    if !calibrated {
        return None;
    }
    let ids: BTreeSet<i64> = seats
        .iter()
        .filter_map(|seat| seat.prebattle)
        .filter(|id| *id != 0)
        .collect();
    let groups: Vec<Vec<&Seat>> = ids
        .iter()
        .map(|id| {
            seats
                .iter()
                .filter(|seat| seat.prebattle == Some(*id))
                .collect()
        })
        .collect();
    let crosses_teams = groups
        .iter()
        .any(|group| group.iter().any(|seat| seat.friendly != group[0].friendly));
    if crosses_teams {
        return None;
    }
    let lettered = |friendly: bool| {
        groups
            .iter()
            .filter(move |group| group[0].friendly == friendly)
            .filter(|group| (2..=LARGEST_DIVISION).contains(&group.len()))
            .zip(1u8..)
            .flat_map(|(group, number)| {
                group
                    .iter()
                    .map(move |seat| (seat.name.to_string(), number))
            })
    };
    Some(lettered(true).chain(lettered(false)).collect())
}

fn seat_of<'a>(raw: &RawPlayer, humans: &HashMap<String, (&'a str, bool)>) -> Option<Seat<'a>> {
    let (name_index, (name, friendly)) = raw
        .iter()
        .filter_map(|(index, value)| {
            let human = quoted_string(value).and_then(|quoted| humans.get(quoted))?;
            Some((*index, *human))
        })
        .min_by_key(|(index, _)| *index)?;
    Some(Seat {
        name,
        friendly,
        name_index,
        prebattle: integer_at(raw, name_index + PREBATTLE_AFTER_NAME),
        on_start: integer_at(raw, name_index + ON_START_AFTER_NAME),
    })
}

fn quoted_string(value: &str) -> Option<&str> {
    let inner = value.strip_prefix("String(")?;
    let start = inner.find('"')?;
    let end = inner.rfind('"')?;
    (end > start).then(|| &inner[start..=end])
}

fn integer_at(raw: &RawPlayer, index: i64) -> Option<i64> {
    raw.get(&index)?
        .strip_prefix("I64(")?
        .strip_suffix(')')?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use pickled::Value;

    use super::*;
    use crate::model::{ArenaVehicle, Relation};

    const ARENA_FILE: &str = include_str!("fixtures/tempArenaInfo.json");

    const NAME_AT: i64 = 26;

    fn arena(seats: &[(&str, Relation)]) -> ArenaInfo {
        ArenaInfo {
            started_at: "2026-11-23T19:00:00".into(),
            map_id: Some(1),
            map_name: "spaces/38_Canada".into(),
            map_display_name: "38_Canada".into(),
            match_group: "pvp".into(),
            game_type: "RandomBattle".into(),
            scenario: "Domination".into(),
            players_per_team: 12,
            player_name: "Rook".into(),
            player_vehicle: "PASB001".into(),
            client_version: "15,10,0,13533378".into(),
            vehicles: seats
                .iter()
                .enumerate()
                .map(|(seat, (name, relation))| ArenaVehicle {
                    player_id: if name.starts_with(':') {
                        0
                    } else {
                        seat as i64 + 1
                    },
                    name: (*name).into(),
                    relation: *relation,
                    ship_id: 4_000_000 + seat as u64,
                })
                .collect(),
        }
    }

    fn raw_at(name_at: i64, name: &str, on_start: i64, prebattle: i64) -> RawPlayer {
        (0..40)
            .map(|index| (index, pickled(Value::I64(1000 + index))))
            .chain([
                (name_at, pickled(Value::String(name.to_string().into()))),
                (name_at + ON_START_AFTER_NAME, pickled(Value::I64(on_start))),
                (
                    name_at + PREBATTLE_AFTER_NAME,
                    pickled(Value::I64(prebattle)),
                ),
            ])
            .collect()
    }

    fn pickled(value: Value) -> String {
        format!("{value:?}")
    }

    fn raw(name: &str, prebattle: i64) -> RawPlayer {
        raw_at(NAME_AT, name, prebattle, prebattle)
    }

    fn read(players: &[RawPlayer], arena: &ArenaInfo) -> Option<Divisions> {
        let borrowed: Vec<&RawPlayer> = players.iter().collect();
        divisions_in(&borrowed, arena)
    }

    fn expect(pairs: &[(&str, u8)]) -> Option<Divisions> {
        Some(
            pairs
                .iter()
                .map(|(name, number)| ((*name).to_string(), *number))
                .collect(),
        )
    }

    #[test]
    fn division_mates_are_numbered_per_team_in_order_of_their_division_id() {
        let battle = arena(&[
            ("Rook", Relation::Me),
            ("Bishop", Relation::Ally),
            ("Knight", Relation::Ally),
            ("Pawn", Relation::Ally),
            ("Queen", Relation::Ally),
            ("King", Relation::Enemy),
            ("Castle", Relation::Enemy),
        ]);
        let players = [
            raw("Rook", 900),
            raw("Bishop", 900),
            raw("Knight", 500),
            raw("Pawn", 500),
            raw("Queen", 0),
            raw("King", 700),
            raw("Castle", 700),
        ];
        let expected = expect(&[
            ("Knight", 1),
            ("Pawn", 1),
            ("Rook", 2),
            ("Bishop", 2),
            ("King", 1),
            ("Castle", 1),
        ]);
        assert_eq!(read(&players, &battle), expected);
    }

    #[test]
    fn the_name_field_is_found_wherever_the_game_version_puts_it() {
        let battle = arena(&[("Rook", Relation::Me), ("Bishop", Relation::Ally)]);
        let players = [raw_at(25, "Rook", 42, 42), raw_at(25, "Bishop", 42, 42)];
        assert_eq!(
            read(&players, &battle),
            expect(&[("Rook", 1), ("Bishop", 1)])
        );
    }

    #[test]
    fn a_battle_without_divisions_reads_as_known_and_empty() {
        let battle = arena(&[("Rook", Relation::Me), ("King", Relation::Enemy)]);
        assert_eq!(
            read(&[raw("Rook", 0), raw("King", 0)], &battle),
            expect(&[])
        );
    }

    #[test]
    fn a_roster_from_another_battle_reads_as_unknown() {
        let battle = arena(&[("Rook", Relation::Me), ("Bishop", Relation::Ally)]);
        assert_eq!(read(&[raw("Rook", 42), raw("Stranger", 42)], &battle), None);
    }

    #[test]
    fn a_division_field_that_disagrees_with_its_start_value_reads_as_unknown() {
        let battle = arena(&[("Rook", Relation::Me), ("Bishop", Relation::Ally)]);
        let players = [raw("Rook", 42), raw_at(NAME_AT, "Bishop", 41, 42)];
        assert_eq!(read(&players, &battle), None);
    }

    #[test]
    fn players_whose_name_sits_at_another_field_read_as_unknown() {
        let battle = arena(&[("Rook", Relation::Me), ("Bishop", Relation::Ally)]);
        let players = [raw("Rook", 42), raw_at(NAME_AT - 1, "Bishop", 42, 42)];
        assert_eq!(read(&players, &battle), None);
    }

    #[test]
    fn an_id_shared_across_teams_reads_as_unknown() {
        let battle = arena(&[("Rook", Relation::Me), ("King", Relation::Enemy)]);
        assert_eq!(read(&[raw("Rook", 42), raw("King", 42)], &battle), None);
    }

    #[test]
    fn a_whole_team_sharing_one_id_is_not_a_division() {
        let names = ["Rook", "Bishop", "Knight", "Pawn"];
        let seats: Vec<(&str, Relation)> =
            names.iter().map(|name| (*name, Relation::Ally)).collect();
        let players: Vec<RawPlayer> = names.iter().map(|name| raw(name, 42)).collect();
        assert_eq!(read(&players, &arena(&seats)), expect(&[]));
    }

    #[test]
    fn only_the_folder_holding_this_battles_arena_file_is_read() {
        let root = tempfile::tempdir().unwrap();
        let current = root.path().join("15.10.0.0");
        let left_over = root.path().join("15.9.0.0");
        let earlier = ARENA_FILE.replace("29.10.2026 19:21:49", "28.10.2026 18:02:11");
        for (folder, contents) in [(&current, ARENA_FILE), (&left_over, earlier.as_str())] {
            fs::create_dir(folder).unwrap();
            fs::write(folder.join(ARENA_FILE_NAME), contents).unwrap();
            fs::write(folder.join(REPLAY_FILE_NAME), b"").unwrap();
        }
        fs::create_dir(root.path().join("no battle")).unwrap();
        let battle = info::parse(ARENA_FILE.as_bytes()).unwrap();
        let folders: Vec<PathBuf> = battle_folders(root.path(), &battle)
            .into_iter()
            .map(|(folder, _)| folder)
            .collect();
        assert_eq!(folders, vec![current]);
    }

    #[test]
    fn a_replay_prefix_never_exceeds_the_read_limit() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(REPLAY_FILE_NAME);
        fs::write(&path, vec![7u8; READ_LIMIT as usize + 4096]).unwrap();
        assert_eq!(
            read_prefix(&path).map(|prefix| prefix.len()),
            Some(READ_LIMIT as usize)
        );
    }

    #[test]
    fn bots_are_left_out_of_the_match() {
        let battle = arena(&[
            ("Rook", Relation::Me),
            ("Bishop", Relation::Ally),
            (":IJN_bot:", Relation::Enemy),
        ]);
        let bot: RawPlayer = [(3, pickled(Value::String(":IJN_bot:".to_string().into())))]
            .into_iter()
            .collect();
        let players = [raw("Rook", 42), raw("Bishop", 42), bot];
        assert_eq!(
            read(&players, &battle),
            expect(&[("Rook", 1), ("Bishop", 1)])
        );
    }
}
