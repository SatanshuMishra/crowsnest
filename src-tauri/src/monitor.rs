use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use futures::stream::{self, StreamExt};
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};

use crate::arena::division::{self, Divisions, SpecCache};
use crate::arena::watch::{ArenaEvent, ArenaWatcher};
use crate::game::{config, install};
use crate::model::{
    ArenaInfo, ArenaVehicle, Battle, GameInstall, InstallSource, Notice, NoticeLevel, Phase,
    Player, PlayerStats, Region, ShipStatsState, Snapshot, StatLine, Team, Totals,
};
use crate::rating::expected::ExpectedValues;
use crate::rating::pr;
use crate::settings::Settings;
use crate::wg::catalog::ShipCatalog;
use crate::wg::client::{AccountRecord, ShipRecord, WgClient, WgError};

pub const SNAPSHOT_EVENT: &str = "snapshot";
const SHIP_STATS_CONCURRENCY: usize = 4;
const EXPECTED_VALUES_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const EXPECTED_VALUES_RETRY_AFTER: Duration = Duration::from_secs(10 * 60);
const SUPERVISE_INTERVAL: Duration = Duration::from_secs(5);
const INSTALL_CHECK_EVERY: u64 = 12;
const DIVISION_RETRY: Duration = Duration::from_secs(3);
const DIVISION_ATTEMPTS: u32 = 30;

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("Crowsnest/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

#[derive(Default)]
struct CatalogCache {
    game_version: Option<String>,
    ships: ShipCatalog,
}

#[derive(Clone)]
enum ExpectedCache {
    Empty,
    Loaded { values: ExpectedValues, at: Instant },
    Failed { at: Instant },
}

#[derive(Debug, PartialEq, Eq)]
enum ExpectedPlan {
    Reuse,
    Download,
    SkipAfterFailure,
}

fn expected_plan(cache: &ExpectedCache, now: Instant) -> ExpectedPlan {
    match cache {
        ExpectedCache::Loaded { at, .. } if now.duration_since(*at) < EXPECTED_VALUES_MAX_AGE => {
            ExpectedPlan::Reuse
        }
        ExpectedCache::Failed { at } if now.duration_since(*at) < EXPECTED_VALUES_RETRY_AFTER => {
            ExpectedPlan::SkipAfterFailure
        }
        _ => ExpectedPlan::Download,
    }
}

struct SharedClient {
    region: Region,
    application_id: String,
    client: WgClient,
}

struct Shared {
    app: AppHandle,
    http: reqwest::Client,
    cache_dir: PathBuf,
    snapshot: Mutex<Snapshot>,
    settings: Mutex<Settings>,
    watcher: Mutex<Option<ArenaWatcher>>,
    attachment: AtomicU64,
    generation: AtomicU64,
    battle_live: AtomicBool,
    load_task: Mutex<Option<JoinHandle<()>>>,
    division_task: Mutex<Option<JoinHandle<()>>>,
    specs: Arc<SpecCache>,
    client: Mutex<Option<SharedClient>>,
    catalog: tokio::sync::Mutex<CatalogCache>,
    expected: tokio::sync::Mutex<ExpectedCache>,
}

#[derive(Clone)]
pub struct Monitor(Arc<Shared>);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Monitor {
    pub fn new(app: AppHandle, cache_dir: PathBuf, settings: Settings) -> Monitor {
        Monitor(Arc::new(Shared {
            app,
            http: http_client(),
            cache_dir,
            snapshot: Mutex::new(Snapshot::initial()),
            settings: Mutex::new(settings),
            watcher: Mutex::new(None),
            attachment: AtomicU64::new(0),
            generation: AtomicU64::new(0),
            battle_live: AtomicBool::new(false),
            load_task: Mutex::new(None),
            division_task: Mutex::new(None),
            specs: Arc::new(SpecCache::default()),
            client: Mutex::new(None),
            catalog: tokio::sync::Mutex::new(CatalogCache::default()),
            expected: tokio::sync::Mutex::new(ExpectedCache::Empty),
        }))
    }

    pub fn snapshot(&self) -> Snapshot {
        lock(&self.0.snapshot).clone()
    }

    pub fn settings(&self) -> Settings {
        lock(&self.0.settings).clone()
    }

    pub fn start(&self) {
        self.reconfigure(self.settings());
    }

    pub fn reconfigure(&self, settings: Settings) {
        *lock(&self.0.settings) = settings.clone();
        let attachment = self.0.attachment.fetch_add(1, Ordering::SeqCst) + 1;
        lock(&self.0.watcher).take();
        self.0.generation.fetch_add(1, Ordering::SeqCst);
        self.0.battle_live.store(false, Ordering::SeqCst);
        self.abort_load();

        let monitor = self.clone();
        tauri::async_runtime::spawn_blocking(move || monitor.attach(settings, attachment));
    }

    pub fn apply_settings(&self, settings: Settings) {
        if needs_reattach(&self.settings(), &settings) {
            self.reconfigure(settings);
        } else {
            *lock(&self.0.settings) = settings;
        }
    }

    pub fn refresh(&self) {
        let current = self.snapshot();
        if let Some(battle) = current.battle {
            self.begin_battle(battle.arena);
        }
    }

    fn is_attachment(&self, attachment: u64) -> bool {
        self.0.attachment.load(Ordering::SeqCst) == attachment
    }

    fn attach(&self, settings: Settings, attachment: u64) {
        let (install, install_notice) = resolve_install(&settings);
        let Some(install) = install else {
            self.update_if(
                || self.is_attachment(attachment),
                |_| Snapshot {
                    notice: install_notice,
                    ..Snapshot::initial()
                },
            );
            return;
        };

        let replays_dir = settings
            .replays_dir_override
            .clone()
            .unwrap_or_else(|| install.replays_dir.clone());
        let phase = if settings.effective_application_id().is_none() {
            Phase::NeedsApplicationId
        } else if !install.replays_enabled && settings.replays_dir_override.is_none() {
            Phase::ReplaysDisabled
        } else {
            Phase::Waiting
        };
        self.update_if(
            || self.is_attachment(attachment),
            |_| Snapshot {
                phase,
                install: Some(install.clone()),
                battle: None,
                notice: install_notice,
            },
        );
        if phase != Phase::Waiting {
            return;
        }
        self.supervise(attachment, watch_root(&replays_dir), install);
    }

    fn supervise(&self, attachment: u64, root: PathBuf, install: GameInstall) {
        let monitor = self.clone();
        tauri::async_runtime::spawn(async move {
            let mut watching = monitor.watch(&root, attachment);
            for tick in 1u64.. {
                tokio::time::sleep(SUPERVISE_INTERVAL).await;
                if !monitor.is_attachment(attachment) {
                    return;
                }
                if watching == WatchState::FolderMissing && root.is_dir() {
                    watching = monitor.watch(&root, attachment);
                }
                let idle = !monitor.0.battle_live.load(Ordering::SeqCst);
                if tick % INSTALL_CHECK_EVERY == 0 && idle && install_moved(&install).await {
                    log::info!("the game installation changed; attaching again");
                    monitor.reconfigure(monitor.settings());
                    return;
                }
            }
        });
    }

    fn watch(&self, root: &Path, attachment: u64) -> WatchState {
        if !root.is_dir() {
            let message = format!(
                "Waiting for the replays folder {} to appear. The game creates it when your first battle starts.",
                root.display()
            );
            self.update_if(
                || self.is_attachment(attachment),
                |snapshot| Snapshot {
                    notice: Some(notice(NoticeLevel::Info, message)),
                    ..snapshot
                },
            );
            return WatchState::FolderMissing;
        }
        let monitor = self.clone();
        match ArenaWatcher::start(root.to_path_buf(), move |event| monitor.on_arena(event)) {
            Ok(watcher) => {
                let mut slot = lock(&self.0.watcher);
                if self.is_attachment(attachment) {
                    *slot = Some(watcher);
                }
                drop(slot);
                self.update_if(
                    || self.is_attachment(attachment),
                    |snapshot| Snapshot {
                        notice: snapshot
                            .notice
                            .filter(|notice| notice.level != NoticeLevel::Info),
                        ..snapshot
                    },
                );
                WatchState::Watching
            }
            Err(error) => {
                log::warn!("arena watcher failed to start: {error}");
                self.update_if(
                    || self.is_attachment(attachment),
                    |snapshot| Snapshot {
                        notice: Some(notice(NoticeLevel::Error, error.to_string())),
                        ..snapshot
                    },
                );
                WatchState::Failed
            }
        }
    }

    fn on_arena(&self, event: ArenaEvent) {
        match event {
            ArenaEvent::Started(arena) => {
                self.0.battle_live.store(true, Ordering::SeqCst);
                self.begin_battle(arena);
            }
            ArenaEvent::Ended => {
                self.0.battle_live.store(false, Ordering::SeqCst);
                self.update(|snapshot| Snapshot {
                    phase: Phase::Waiting,
                    ..snapshot
                });
            }
        }
    }

    fn begin_battle(&self, arena: ArenaInfo) {
        let settings = self.settings();
        let Some(application_id) = settings.effective_application_id() else {
            return;
        };
        self.abort_load();
        let generation = self.0.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let install = lock(&self.0.snapshot).install.clone();
        let install_region = install
            .as_ref()
            .and_then(|install| config::active_region(&install.root).or(install.region));
        let region = settings.region_override.or(install_region);
        let region_notice = region.is_none().then(|| {
            notice(
                NoticeLevel::Warning,
                "Could not tell which server you play on, so stats come from EU. Pick your server in Settings.".into(),
            )
        });
        let region = region.unwrap_or(Region::Eu);

        let placeholder = assemble_battle(
            &arena,
            Some(region),
            &ShipCatalog::default(),
            None,
            |vehicle| {
                if vehicle.is_bot() {
                    PlayerStats::Bot
                } else {
                    PlayerStats::Pending
                }
            },
        );
        let live = self.0.battle_live.load(Ordering::SeqCst);
        self.update(|snapshot| Snapshot {
            phase: if live { Phase::Loading } else { snapshot.phase },
            install: snapshot.install.map(|install| GameInstall {
                region: install_region.or(install.region),
                ..install
            }),
            battle: Some(keep_divisions(snapshot.battle.as_ref(), placeholder)),
            notice: region_notice.clone(),
        });

        let replays_dir = settings
            .replays_dir_override
            .clone()
            .or_else(|| install.as_ref().map(|install| install.replays_dir.clone()));
        if let (Some(install), Some(replays_dir)) = (install.as_ref(), replays_dir) {
            let monitor = self.clone();
            let game_root = install.root.clone();
            let arena = arena.clone();
            let task = tauri::async_runtime::spawn(async move {
                monitor
                    .find_divisions(generation, game_root, watch_root(&replays_dir), arena)
                    .await;
            });
            *lock(&self.0.division_task) = Some(task);
        }

        let own_application_id = settings.has_own_application_id();
        let client = self.client_for(region, application_id);
        let monitor = self.clone();
        let task = tauri::async_runtime::spawn(async move {
            monitor
                .load_battle(generation, client, arena, region_notice, own_application_id)
                .await;
        });
        *lock(&self.0.load_task) = Some(task);
    }

    fn abort_load(&self) {
        for slot in [&self.0.load_task, &self.0.division_task] {
            if let Some(task) = lock(slot).take() {
                task.abort();
            }
        }
    }

    async fn find_divisions(
        &self,
        generation: u64,
        game_root: PathBuf,
        watch_root: PathBuf,
        arena: ArenaInfo,
    ) {
        for _ in 0..DIVISION_ATTEMPTS {
            if !self.is_current(generation) {
                return;
            }
            let found = division::read_divisions(
                Arc::clone(&self.0.specs),
                game_root.clone(),
                watch_root.clone(),
                arena.clone(),
            )
            .await;
            if let Some(divisions) = found {
                self.update_current(generation, |snapshot| Snapshot {
                    battle: snapshot
                        .battle
                        .map(|battle| with_found_divisions(battle, &divisions)),
                    ..snapshot
                });
                return;
            }
            tokio::time::sleep(DIVISION_RETRY).await;
        }
        if self.is_current(generation) {
            log::info!("could not read divisions for this battle from the replays folder");
        }
    }

    fn client_for(&self, region: Region, application_id: String) -> WgClient {
        let mut slot = lock(&self.0.client);
        let reusable = slot
            .as_ref()
            .filter(|shared| shared.region == region && shared.application_id == application_id)
            .map(|shared| shared.client.clone());
        reusable.unwrap_or_else(|| {
            let client = WgClient::new(self.0.http.clone(), region, application_id.clone());
            *slot = Some(SharedClient {
                region,
                application_id,
                client: client.clone(),
            });
            client
        })
    }

    fn settled_phase(&self) -> Phase {
        if self.0.battle_live.load(Ordering::SeqCst) {
            Phase::InBattle
        } else {
            Phase::Waiting
        }
    }

    fn is_current(&self, generation: u64) -> bool {
        self.0.generation.load(Ordering::SeqCst) == generation
    }

    async fn load_battle(
        &self,
        generation: u64,
        client: WgClient,
        arena: ArenaInfo,
        region_notice: Option<Notice>,
        own_application_id: bool,
    ) {
        let region = client.region();
        let ship_ids: Vec<u64> = arena
            .vehicles
            .iter()
            .filter(|vehicle| !vehicle.is_bot())
            .map(|vehicle| vehicle.ship_id)
            .collect();
        let catalog = self.ships(&client, &arena.client_version, &ship_ids).await;
        if !self.is_current(generation) {
            return;
        }

        let roster = match fetch_roster(&client, &arena).await {
            Ok(roster) => roster,
            Err(error) => {
                let reason = error.to_string();
                let failed = assemble_battle(&arena, Some(region), &catalog, None, |vehicle| {
                    if vehicle.is_bot() {
                        PlayerStats::Bot
                    } else {
                        PlayerStats::Failed {
                            reason: reason.clone(),
                        }
                    }
                });
                let phase = self.settled_phase();
                self.update_current(generation, |snapshot| Snapshot {
                    phase,
                    battle: Some(keep_divisions(snapshot.battle.as_ref(), failed)),
                    notice: Some(notice(
                        NoticeLevel::Error,
                        roster_failure_message(&error, own_application_id),
                    )),
                    ..snapshot
                });
                return;
            }
        };

        let early = with_clans(
            assemble_battle(&arena, Some(region), &catalog, None, |vehicle| {
                roster.stats_for(vehicle, None, None)
            }),
            &roster,
        );
        self.update_current(generation, |snapshot| Snapshot {
            battle: Some(keep_divisions(snapshot.battle.as_ref(), early)),
            ..snapshot
        });

        let ((expected, expected_notice), ship_records) =
            futures::join!(self.expected_values(), fetch_ship_records(&client, &roster));
        if !self.is_current(generation) {
            return;
        }
        let ship_notice = ship_stats_notice(&roster, &ship_records);

        let complete = with_clans(
            assemble_battle(
                &arena,
                Some(region),
                &catalog,
                expected.as_ref(),
                |vehicle| roster.stats_for(vehicle, Some(&ship_records), expected.as_ref()),
            ),
            &roster,
        );
        let phase = self.settled_phase();
        self.update_current(generation, |snapshot| Snapshot {
            phase,
            battle: Some(keep_divisions(snapshot.battle.as_ref(), complete)),
            notice: region_notice
                .clone()
                .or(ship_notice.clone())
                .or(expected_notice.clone()),
            ..snapshot
        });
    }

    async fn ships(&self, client: &WgClient, game_version: &str, required: &[u64]) -> ShipCatalog {
        let mut cached = self.0.catalog.lock().await;
        let same_version = cached.game_version.as_deref() == Some(game_version);
        if same_version && !cached.ships.is_empty() {
            return cached.ships.clone();
        }
        match ShipCatalog::load(&self.0.cache_dir, client, game_version, required).await {
            Ok(catalog) => {
                *cached = CatalogCache {
                    game_version: Some(game_version.to_owned()),
                    ships: catalog.clone(),
                };
                catalog
            }
            Err(error) => {
                log::warn!("ship catalog unavailable: {error}");
                cached.ships.clone()
            }
        }
    }

    async fn expected_values(&self) -> (Option<ExpectedValues>, Option<Notice>) {
        let unavailable = || {
            let message = "Personal rating is unavailable because the expected values could not be downloaded.";
            Some(notice(NoticeLevel::Warning, message.into()))
        };
        let mut cached = self.0.expected.lock().await;
        let now = Instant::now();
        match (expected_plan(&cached, now), cached.clone()) {
            (ExpectedPlan::Reuse, ExpectedCache::Loaded { values, .. }) => {
                return (Some(values), None);
            }
            (ExpectedPlan::SkipAfterFailure, _) => return (None, unavailable()),
            _ => {}
        }
        match ExpectedValues::load(&self.0.cache_dir, &self.0.http).await {
            Ok(values) => {
                *cached = ExpectedCache::Loaded {
                    values: values.clone(),
                    at: now,
                };
                (Some(values), None)
            }
            Err(error) => {
                log::warn!("expected values unavailable: {error}");
                *cached = ExpectedCache::Failed { at: now };
                (None, unavailable())
            }
        }
    }

    fn update(&self, change: impl FnOnce(Snapshot) -> Snapshot) {
        self.update_if(|| true, change);
    }

    fn update_current(&self, generation: u64, change: impl FnOnce(Snapshot) -> Snapshot) {
        self.update_if(|| self.is_current(generation), change);
    }

    fn update_if(
        &self,
        still_wanted: impl FnOnce() -> bool,
        change: impl FnOnce(Snapshot) -> Snapshot,
    ) {
        let next = {
            let mut guard = lock(&self.0.snapshot);
            if !still_wanted() {
                return;
            }
            let next = change(guard.clone());
            *guard = next.clone();
            next
        };
        if let Err(error) = self.0.app.emit(SNAPSHOT_EVENT, next) {
            log::warn!("could not send the snapshot to the window: {error}");
        }
    }
}

const BUNDLED_ID_BLOCKED: &str = "Wargaming has blocked the application ID built into Crowsnest. Enter your own free application ID in Settings to load stats again.";
const OWN_ID_BLOCKED: &str = "Wargaming has blocked your application ID. Create a new Standalone application on developers.wargaming.net and enter its ID in Settings.";

fn roster_failure_message(error: &WgError, own_application_id: bool) -> String {
    match (error, own_application_id) {
        (WgError::ApplicationBlocked, false) => BUNDLED_ID_BLOCKED.to_owned(),
        (WgError::ApplicationBlocked, true) => OWN_ID_BLOCKED.to_owned(),
        _ => error.to_string(),
    }
}

fn notice(level: NoticeLevel, message: String) -> Notice {
    Notice { level, message }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatchState {
    Watching,
    FolderMissing,
    Failed,
}

fn needs_reattach(current: &Settings, next: &Settings) -> bool {
    current.install_path != next.install_path
        || current.replays_dir_override != next.replays_dir_override
        || current.effective_application_id() != next.effective_application_id()
}

async fn install_moved(install: &GameInstall) -> bool {
    let root = install.root.clone();
    let source = install.source;
    let fresh = tauri::async_runtime::spawn_blocking(move || install::inspect(&root, source)).await;
    match fresh {
        Ok(Ok(fresh)) => {
            fresh.replays_dir != install.replays_dir
                || fresh.replays_enabled != install.replays_enabled
        }
        _ => false,
    }
}

fn resolve_install(settings: &Settings) -> (Option<GameInstall>, Option<Notice>) {
    let saved = settings
        .install_path
        .as_ref()
        .map(|path| install::inspect(path, InstallSource::Manual));
    match saved {
        Some(Ok(found)) => (Some(found), None),
        Some(Err(error)) => {
            let fallback = install::detect().into_iter().next();
            let tail = if fallback.is_some() {
                " Using the installation found automatically instead."
            } else {
                ""
            };
            let message = format!("The saved game folder can no longer be used ({error}).{tail}");
            (fallback, Some(notice(NoticeLevel::Warning, message)))
        }
        None => (install::detect().into_iter().next(), None),
    }
}

struct Roster {
    ids_by_name: HashMap<String, u64>,
    accounts: HashMap<u64, AccountRecord>,
    clans: HashMap<u64, String>,
}

impl Roster {
    fn account_for(&self, vehicle: &ArenaVehicle) -> Option<&AccountRecord> {
        self.ids_by_name
            .get(&vehicle.name.to_lowercase())
            .and_then(|id| self.accounts.get(id))
    }

    fn stats_for(
        &self,
        vehicle: &ArenaVehicle,
        ships: Option<&HashMap<u64, Vec<ShipRecord>>>,
        expected: Option<&ExpectedValues>,
    ) -> PlayerStats {
        if vehicle.is_bot() {
            return PlayerStats::Bot;
        }
        let Some(account) = self.account_for(vehicle) else {
            return PlayerStats::NotFound;
        };
        let Some(pvp) = account.pvp.filter(|_| !account.hidden) else {
            return PlayerStats::Hidden {
                account_id: account.account_id,
            };
        };
        let records = ships.and_then(|ships| ships.get(&account.account_id));
        let ship_state = match (ships, records) {
            (None, _) => ShipStatsState::Pending,
            (Some(_), Some(_)) => ShipStatsState::Ready,
            (Some(_), None) => ShipStatsState::Unavailable,
        };
        let rating_over = |pick: fn(&ShipRecord) -> Option<Totals>| {
            expected.zip(records).and_then(|(expected, records)| {
                let per_ship: Vec<(u64, Totals)> = records
                    .iter()
                    .filter_map(|record| pick(record).map(|totals| (record.ship_id, totals)))
                    .collect();
                pr::personal_rating(&per_ship, expected)
            })
        };
        let account_line = |totals: Option<Totals>, pick: fn(&ShipRecord) -> Option<Totals>| {
            totals.map(|totals| StatLine::from_totals(totals, rating_over(pick)))
        };
        let ship_record = records.and_then(|records| {
            records
                .iter()
                .find(|record| record.ship_id == vehicle.ship_id)
        });
        let ship_line = |totals: Totals| {
            let rating =
                expected.and_then(|expected| pr::ship_rating(vehicle.ship_id, totals, expected));
            StatLine::from_totals(totals, rating)
        };
        PlayerStats::Loaded {
            account_id: account.account_id,
            overall: StatLine::from_totals(pvp, rating_over(|record| Some(record.pvp))),
            overall_solo: account_line(account.pvp_solo, |record| record.pvp_solo),
            overall_div2: account_line(account.pvp_div2, |record| record.pvp_div2),
            overall_div3: account_line(account.pvp_div3, |record| record.pvp_div3),
            ship: ship_record.map(|record| ship_line(record.pvp)),
            ship_solo: ship_record
                .and_then(|record| record.pvp_solo)
                .map(ship_line),
            ship_div2: ship_record
                .and_then(|record| record.pvp_div2)
                .map(ship_line),
            ship_div3: ship_record
                .and_then(|record| record.pvp_div3)
                .map(ship_line),
            ship_state,
            created_at: account.created_at,
        }
    }
}

async fn fetch_roster(client: &WgClient, arena: &ArenaInfo) -> Result<Roster, WgError> {
    let names: Vec<String> = arena
        .vehicles
        .iter()
        .filter(|vehicle| !vehicle.is_bot())
        .map(|vehicle| vehicle.name.clone())
        .collect();
    let ids_by_name: HashMap<String, u64> = client
        .account_ids(&names)
        .await?
        .into_iter()
        .map(|(name, id)| (name.to_lowercase(), id))
        .collect();
    let ids: Vec<u64> = ids_by_name.values().copied().collect();
    let (accounts, clans) = futures::join!(client.accounts(&ids), client.clan_tags(&ids));
    let clans = clans.unwrap_or_else(|error| {
        log::warn!("clan tags unavailable: {error}");
        HashMap::new()
    });
    Ok(Roster {
        ids_by_name,
        accounts: accounts?,
        clans,
    })
}

async fn fetch_ship_records(client: &WgClient, roster: &Roster) -> HashMap<u64, Vec<ShipRecord>> {
    let visible: Vec<u64> = roster
        .accounts
        .values()
        .filter(|account| !account.hidden && account.pvp.is_some())
        .map(|account| account.account_id)
        .collect();
    stream::iter(visible)
        .map(|account_id| async move { (account_id, client.ship_stats(account_id).await) })
        .buffer_unordered(SHIP_STATS_CONCURRENCY)
        .filter_map(|(account_id, result)| async move {
            match result {
                Ok(records) => Some((account_id, records)),
                Err(error) => {
                    log::warn!("ship stats unavailable for one player: {error}");
                    None
                }
            }
        })
        .collect()
        .await
}

fn ship_stats_notice(roster: &Roster, fetched: &HashMap<u64, Vec<ShipRecord>>) -> Option<Notice> {
    let missing = roster
        .accounts
        .values()
        .filter(|account| !account.hidden && account.pvp.is_some())
        .filter(|account| !fetched.contains_key(&account.account_id))
        .count();
    let players = if missing == 1 { "player" } else { "players" };
    (missing > 0).then(|| {
        notice(
            NoticeLevel::Warning,
            format!(
                "Wargaming did not return ship statistics for {missing} {players}. Press Refresh to try again."
            ),
        )
    })
}

fn with_clans(battle: Battle, roster: &Roster) -> Battle {
    let tag = |team: Team| {
        let players = team
            .players
            .into_iter()
            .map(|player| {
                let clan_tag = player
                    .stats
                    .account_id()
                    .and_then(|id| roster.clans.get(&id).cloned())
                    .or(player.clan_tag);
                Player { clan_tag, ..player }
            })
            .collect();
        Team::of(players)
    };
    Battle {
        allies: tag(battle.allies),
        enemies: tag(battle.enemies),
        ..battle
    }
}

fn with_divisions(battle: Battle, division_of: impl Fn(&str) -> Option<u8>) -> Battle {
    let mark = |team: Team| {
        let players = team
            .players
            .into_iter()
            .map(|player| Player {
                division: division_of(&player.name).or(player.division),
                ..player
            })
            .collect();
        Team::of(players)
    };
    Battle {
        allies: mark(battle.allies),
        enemies: mark(battle.enemies),
        ..battle
    }
}

fn with_found_divisions(battle: Battle, divisions: &Divisions) -> Battle {
    with_divisions(battle, |name| divisions.get(name).copied())
}

fn keep_divisions(previous: Option<&Battle>, next: Battle) -> Battle {
    let known: Divisions = previous
        .filter(|previous| previous.id == next.id)
        .into_iter()
        .flat_map(|previous| {
            previous
                .allies
                .players
                .iter()
                .chain(&previous.enemies.players)
        })
        .filter_map(|player| {
            player
                .division
                .map(|division| (player.name.clone(), division))
        })
        .collect();
    with_found_divisions(next, &known)
}

fn assemble_battle(
    arena: &ArenaInfo,
    region: Option<Region>,
    catalog: &ShipCatalog,
    expected: Option<&ExpectedValues>,
    stats_for: impl Fn(&ArenaVehicle) -> PlayerStats,
) -> Battle {
    let players: Vec<Player> = arena
        .vehicles
        .iter()
        .map(|vehicle| Player {
            name: vehicle.name.clone(),
            relation: vehicle.relation,
            clan_tag: None,
            ship: catalog.get(vehicle.ship_id).cloned(),
            ship_id: vehicle.ship_id,
            expected: expected.and_then(|values| values.for_ship(vehicle.ship_id)),
            division: None,
            stats: stats_for(vehicle),
        })
        .collect();
    let (allies, enemies): (Vec<Player>, Vec<Player>) = players
        .into_iter()
        .partition(|player| player.relation.is_friendly());
    Battle {
        id: arena.fingerprint(),
        region,
        arena: arena.clone(),
        allies: Team::of(sorted(allies)),
        enemies: Team::of(sorted(enemies)),
    }
}

fn sorted(players: Vec<Player>) -> Vec<Player> {
    let class = |player: &Player| {
        player
            .ship
            .as_ref()
            .map_or(u8::MAX, |ship| ship.class.sort_rank())
    };
    let tier = |player: &Player| player.ship.as_ref().map_or(0, |ship| ship.tier);
    let mut ordered = players;
    ordered.sort_by(|a, b| {
        class(a)
            .cmp(&class(b))
            .then(tier(b).cmp(&tier(a)))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    ordered
}

fn is_version_folder(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.contains('.')
                && name
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        })
}

fn watch_root(replays_dir: &Path) -> PathBuf {
    replays_dir
        .parent()
        .filter(|_| is_version_folder(replays_dir))
        .unwrap_or(replays_dir)
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Relation, Ship, ShipClass};

    #[test]
    fn a_blocked_bundled_id_points_to_settings() {
        assert_eq!(
            roster_failure_message(&WgError::ApplicationBlocked, false),
            BUNDLED_ID_BLOCKED
        );
    }

    #[test]
    fn a_blocked_own_id_points_to_a_new_application() {
        assert_eq!(
            roster_failure_message(&WgError::ApplicationBlocked, true),
            OWN_ID_BLOCKED
        );
    }

    #[test]
    fn other_roster_failures_keep_their_own_message() {
        let error = WgError::Network("the request timed out".into());
        assert_eq!(roster_failure_message(&error, false), error.to_string());
    }

    fn vehicle(name: &str, relation: Relation, ship_id: u64) -> ArenaVehicle {
        ArenaVehicle {
            player_id: 1,
            name: name.into(),
            relation,
            ship_id,
        }
    }

    fn arena(vehicles: Vec<ArenaVehicle>) -> ArenaInfo {
        ArenaInfo {
            started_at: "2026-11-23T19:00:00".into(),
            map_id: Some(1),
            map_name: "spaces/38_Canada".into(),
            map_display_name: "38_Canada".into(),
            match_group: "pvp".into(),
            game_type: "RandomBattle".into(),
            scenario: "Domination".into(),
            players_per_team: 12,
            player_name: "Me".into(),
            player_vehicle: "PASB001".into(),
            client_version: "15,10,0,13533378".into(),
            vehicles,
        }
    }

    fn ship(id: u64, tier: u8, class: ShipClass) -> Ship {
        Ship {
            id,
            name: format!("Ship {id}"),
            tier,
            class,
            nation: "usa".into(),
            is_premium: false,
            is_special: false,
        }
    }

    fn empty_roster() -> Roster {
        Roster {
            ids_by_name: HashMap::new(),
            accounts: HashMap::new(),
            clans: HashMap::new(),
        }
    }

    #[test]
    fn players_split_into_teams_with_me_among_allies() {
        let battle = assemble_battle(
            &arena(vec![
                vehicle("Me", Relation::Me, 1),
                vehicle("Friend", Relation::Ally, 2),
                vehicle("Foe", Relation::Enemy, 3),
            ]),
            Some(Region::Eu),
            &ShipCatalog::default(),
            None,
            |_| PlayerStats::Pending,
        );
        let ally_names: Vec<&str> = battle
            .allies
            .players
            .iter()
            .map(|player| player.name.as_str())
            .collect();
        assert_eq!(ally_names, ["Friend", "Me"]);
        assert_eq!(battle.enemies.players.len(), 1);
    }

    #[test]
    fn players_sort_by_class_then_higher_tier_first() {
        let catalog = ShipCatalog::from_ships(vec![
            ship(1, 10, ShipClass::Destroyer),
            ship(2, 8, ShipClass::Battleship),
            ship(3, 10, ShipClass::Battleship),
            ship(4, 10, ShipClass::AirCarrier),
        ]);
        let battle = assemble_battle(
            &arena(vec![
                vehicle("dd", Relation::Ally, 1),
                vehicle("bb8", Relation::Ally, 2),
                vehicle("bb10", Relation::Ally, 3),
                vehicle("cv", Relation::Ally, 4),
            ]),
            None,
            &catalog,
            None,
            |_| PlayerStats::Pending,
        );
        let order: Vec<&str> = battle
            .allies
            .players
            .iter()
            .map(|player| player.name.as_str())
            .collect();
        assert_eq!(order, ["cv", "bb10", "bb8", "dd"]);
    }

    #[test]
    fn a_hidden_profile_never_shows_stats() {
        let roster = Roster {
            ids_by_name: HashMap::from([("shy".to_string(), 7)]),
            accounts: HashMap::from([(
                7,
                AccountRecord {
                    account_id: 7,
                    hidden: true,
                    created_at: None,
                    pvp: Some(Totals {
                        battles: 10,
                        ..Totals::default()
                    }),
                    pvp_solo: None,
                    pvp_div2: None,
                    pvp_div3: None,
                },
            )]),
            clans: HashMap::new(),
        };
        let stats = roster.stats_for(&vehicle("Shy", Relation::Enemy, 1), None, None);
        assert_eq!(stats, PlayerStats::Hidden { account_id: 7 });
    }

    #[test]
    fn a_name_wargaming_does_not_know_is_reported_as_not_found() {
        let stats = empty_roster().stats_for(&vehicle("Ghost", Relation::Ally, 1), None, None);
        assert_eq!(stats, PlayerStats::NotFound);
    }

    #[test]
    fn bots_are_never_looked_up() {
        let bot = ArenaVehicle {
            player_id: 0,
            name: ":IDS_OP_BOT".into(),
            relation: Relation::Enemy,
            ship_id: 1,
        };
        assert_eq!(empty_roster().stats_for(&bot, None, None), PlayerStats::Bot);
    }

    #[test]
    fn clan_tags_attach_by_account() {
        let roster = Roster {
            ids_by_name: HashMap::new(),
            accounts: HashMap::new(),
            clans: HashMap::from([(9, "NEST".to_string())]),
        };
        let battle = assemble_battle(
            &arena(vec![vehicle("Tagged", Relation::Ally, 1)]),
            None,
            &ShipCatalog::default(),
            None,
            |_| PlayerStats::Hidden { account_id: 9 },
        );
        let tagged = with_clans(battle, &roster);
        assert_eq!(tagged.allies.players[0].clan_tag.as_deref(), Some("NEST"));
    }

    #[test]
    fn versioned_replays_folders_are_watched_from_their_parent() {
        let base = Path::new("C:/Games/World_of_Warships/replays");
        assert_eq!(watch_root(&base.join("15.10.0.0")), base);
        assert_eq!(watch_root(base), base);
        assert_eq!(
            watch_root(Path::new("D:/my.replays")),
            Path::new("D:/my.replays")
        );
        assert_eq!(
            watch_root(Path::new("D:/replays/15..0")),
            Path::new("D:/replays/15..0")
        );
    }

    fn visible_roster() -> Roster {
        Roster {
            ids_by_name: HashMap::from([("seen".to_string(), 5)]),
            accounts: HashMap::from([(
                5,
                AccountRecord {
                    account_id: 5,
                    hidden: false,
                    created_at: None,
                    pvp: Some(Totals {
                        battles: 10,
                        wins: 6,
                        ..Totals::default()
                    }),
                    pvp_solo: None,
                    pvp_div2: None,
                    pvp_div3: None,
                },
            )]),
            clans: HashMap::new(),
        }
    }

    fn ship_state_of(stats: PlayerStats) -> Option<ShipStatsState> {
        match stats {
            PlayerStats::Loaded { ship_state, .. } => Some(ship_state),
            _ => None,
        }
    }

    #[test]
    fn ship_stats_are_pending_until_they_are_fetched() {
        let stats = visible_roster().stats_for(&vehicle("Seen", Relation::Ally, 1), None, None);
        assert_eq!(ship_state_of(stats), Some(ShipStatsState::Pending));
    }

    #[test]
    fn a_failed_ship_stats_request_is_unavailable_not_empty() {
        let fetched = HashMap::new();
        let stats =
            visible_roster().stats_for(&vehicle("Seen", Relation::Ally, 1), Some(&fetched), None);
        assert_eq!(ship_state_of(stats), Some(ShipStatsState::Unavailable));
    }

    #[test]
    fn a_player_without_battles_in_the_ship_is_ready_with_no_ship_line() {
        let other_ship = ShipRecord {
            ship_id: 2,
            pvp: Totals {
                battles: 3,
                ..Totals::default()
            },
            pvp_solo: None,
            pvp_div2: None,
            pvp_div3: None,
        };
        let fetched = HashMap::from([(5, vec![other_ship])]);
        let stats =
            visible_roster().stats_for(&vehicle("Seen", Relation::Ally, 1), Some(&fetched), None);
        assert!(matches!(
            stats,
            PlayerStats::Loaded {
                ship: None,
                ship_state: ShipStatsState::Ready,
                ..
            }
        ));
    }

    #[test]
    fn division_lines_come_from_the_account_and_the_ship_in_battle() {
        let base = visible_roster();
        let account = AccountRecord {
            pvp_div2: Some(Totals {
                battles: 4,
                wins: 3,
                ..Totals::default()
            }),
            ..base.accounts[&5].clone()
        };
        let roster = Roster {
            accounts: HashMap::from([(5, account)]),
            ..base
        };
        let in_battle = ShipRecord {
            ship_id: 1,
            pvp: Totals {
                battles: 9,
                wins: 5,
                ..Totals::default()
            },
            pvp_solo: None,
            pvp_div2: None,
            pvp_div3: Some(Totals {
                battles: 2,
                wins: 2,
                ..Totals::default()
            }),
        };
        let fetched = HashMap::from([(5, vec![in_battle])]);
        let stats = roster.stats_for(&vehicle("Seen", Relation::Ally, 1), Some(&fetched), None);
        let PlayerStats::Loaded {
            overall_div2,
            overall_div3,
            ship_div2,
            ship_div3,
            ..
        } = stats
        else {
            panic!("expected loaded stats");
        };
        assert_eq!(overall_div2.map(|line| line.battles), Some(4));
        assert_eq!(overall_div3, None);
        assert_eq!(ship_div2, None);
        assert_eq!(ship_div3.map(|line| line.battles), Some(2));
    }

    fn divisions_by_name(battle: &Battle) -> Vec<(String, Option<u8>)> {
        battle
            .allies
            .players
            .iter()
            .chain(&battle.enemies.players)
            .map(|player| (player.name.clone(), player.division))
            .collect()
    }

    fn pending_battle(arena: &ArenaInfo) -> Battle {
        assemble_battle(arena, None, &ShipCatalog::default(), None, |_| {
            PlayerStats::Pending
        })
    }

    #[test]
    fn found_divisions_survive_a_later_assembly_of_the_same_battle() {
        let battle_arena = arena(vec![
            vehicle("Me", Relation::Me, 1),
            vehicle("Mate", Relation::Ally, 2),
            vehicle("Foe", Relation::Enemy, 3),
        ]);
        let found: Divisions = [("Me".to_string(), 1), ("Mate".to_string(), 1)]
            .into_iter()
            .collect();
        let shown = with_found_divisions(pending_battle(&battle_arena), &found);
        let reassembled = keep_divisions(Some(&shown), pending_battle(&battle_arena));
        assert_eq!(
            divisions_by_name(&reassembled),
            vec![
                ("Mate".to_string(), Some(1)),
                ("Me".to_string(), Some(1)),
                ("Foe".to_string(), None),
            ]
        );
    }

    #[test]
    fn divisions_never_carry_into_another_battle() {
        let first = arena(vec![
            vehicle("Me", Relation::Me, 1),
            vehicle("Mate", Relation::Ally, 2),
        ]);
        let next = ArenaInfo {
            started_at: "2026-11-23T19:30:00".into(),
            ..first.clone()
        };
        let found: Divisions = [("Me".to_string(), 1), ("Mate".to_string(), 1)]
            .into_iter()
            .collect();
        let shown = with_found_divisions(pending_battle(&first), &found);
        let following = keep_divisions(Some(&shown), pending_battle(&next));
        assert!(
            divisions_by_name(&following)
                .iter()
                .all(|(_, division)| division.is_none())
        );
    }

    #[test]
    fn saving_display_settings_keeps_the_battle() {
        let current = Settings::default();
        let display_only = Settings {
            theme: crate::settings::Theme::Light,
            always_on_top: true,
            solo_stats: true,
            region_override: Some(Region::Na),
            ..Settings::default()
        };
        assert!(!needs_reattach(&current, &display_only));
    }

    #[test]
    fn changing_the_game_folder_replays_folder_or_application_id_reattaches() {
        let current = Settings::default();
        let folder = Settings {
            install_path: Some(PathBuf::from("D:/Games/World_of_Warships")),
            ..Settings::default()
        };
        let replays = Settings {
            replays_dir_override: Some(PathBuf::from("D:/replays")),
            ..Settings::default()
        };
        let application_id = Settings {
            application_id: Some("0123456789abcdef0123456789abcdef".into()),
            ..Settings::default()
        };
        assert!(needs_reattach(&current, &folder));
        assert!(needs_reattach(&current, &replays));
        assert!(needs_reattach(&current, &application_id));
    }

    #[test]
    fn expected_values_are_reused_for_a_day_and_retried_ten_minutes_after_a_failure() {
        let start = Instant::now();
        let loaded = ExpectedCache::Loaded {
            values: ExpectedValues {
                ships: HashMap::new(),
            },
            at: start,
        };
        let failed = ExpectedCache::Failed { at: start };
        let hour = Duration::from_secs(60 * 60);

        assert_eq!(
            expected_plan(&ExpectedCache::Empty, start),
            ExpectedPlan::Download
        );
        assert_eq!(expected_plan(&loaded, start + hour), ExpectedPlan::Reuse);
        assert_eq!(
            expected_plan(&loaded, start + 25 * hour),
            ExpectedPlan::Download
        );
        assert_eq!(
            expected_plan(&failed, start + Duration::from_secs(60)),
            ExpectedPlan::SkipAfterFailure
        );
        assert_eq!(expected_plan(&failed, start + hour), ExpectedPlan::Download);
    }
}
