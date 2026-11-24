use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use futures::stream::{self, StreamExt};
use tauri::{AppHandle, Emitter};

use crate::arena::watch::{ArenaEvent, ArenaWatcher};
use crate::game::install;
use crate::model::{
    ArenaInfo, ArenaVehicle, Battle, GameInstall, InstallSource, Notice, NoticeLevel, Phase,
    Player, PlayerStats, Region, Snapshot, StatLine, Team, Totals,
};
use crate::rating::expected::ExpectedValues;
use crate::rating::pr;
use crate::settings::Settings;
use crate::wg::catalog::ShipCatalog;
use crate::wg::client::{AccountRecord, ShipRecord, WgClient, WgError};

pub const SNAPSHOT_EVENT: &str = "snapshot";
const SHIP_STATS_CONCURRENCY: usize = 4;

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("Crowsnest/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

struct Shared {
    app: AppHandle,
    http: reqwest::Client,
    cache_dir: PathBuf,
    snapshot: Mutex<Snapshot>,
    settings: Mutex<Settings>,
    watcher: Mutex<Option<ArenaWatcher>>,
    generation: AtomicU64,
    catalog: tokio::sync::Mutex<ShipCatalog>,
    expected: tokio::sync::Mutex<Option<ExpectedValues>>,
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
            generation: AtomicU64::new(0),
            catalog: tokio::sync::Mutex::new(ShipCatalog::default()),
            expected: tokio::sync::Mutex::new(None),
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
        lock(&self.0.watcher).take();
        self.0.generation.fetch_add(1, Ordering::SeqCst);

        let monitor = self.clone();
        tauri::async_runtime::spawn_blocking(move || monitor.attach(settings));
    }

    pub fn refresh(&self) {
        let current = self.snapshot();
        if let Some(battle) = current.battle {
            self.begin_battle(battle.arena);
        }
    }

    fn attach(&self, settings: Settings) {
        let (install, install_notice) = resolve_install(&settings);
        let Some(install) = install else {
            self.publish(Snapshot {
                notice: install_notice,
                ..Snapshot::initial()
            });
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
        self.publish(Snapshot {
            phase,
            install: Some(install),
            battle: None,
            notice: install_notice,
        });
        if phase != Phase::Waiting {
            return;
        }

        let monitor = self.clone();
        match ArenaWatcher::start(watch_root(&replays_dir), move |event| {
            monitor.on_arena(event)
        }) {
            Ok(watcher) => *lock(&self.0.watcher) = Some(watcher),
            Err(error) => {
                log::warn!("arena watcher failed to start: {error}");
                self.update(|snapshot| Snapshot {
                    notice: Some(notice(NoticeLevel::Error, error.to_string())),
                    ..snapshot
                });
            }
        }
    }

    fn on_arena(&self, event: ArenaEvent) {
        match event {
            ArenaEvent::Started(arena) => self.begin_battle(arena),
            ArenaEvent::Ended => {
                self.0.generation.fetch_add(1, Ordering::SeqCst);
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
        let generation = self.0.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let install_region = lock(&self.0.snapshot)
            .install
            .as_ref()
            .and_then(|install| install.region);
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
        self.update(|snapshot| Snapshot {
            phase: Phase::Loading,
            battle: Some(placeholder),
            notice: region_notice.clone(),
            ..snapshot
        });

        let monitor = self.clone();
        tauri::async_runtime::spawn(async move {
            let client = WgClient::new(monitor.0.http.clone(), region, application_id);
            monitor
                .load_battle(generation, client, arena, region_notice)
                .await;
        });
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
    ) {
        let region = client.region();
        let ship_ids: Vec<u64> = arena
            .vehicles
            .iter()
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
                if self.is_current(generation) {
                    self.update(|snapshot| Snapshot {
                        phase: Phase::InBattle,
                        battle: Some(failed),
                        notice: Some(notice(NoticeLevel::Error, reason.clone())),
                        ..snapshot
                    });
                }
                return;
            }
        };

        let early = with_clans(
            assemble_battle(&arena, Some(region), &catalog, None, |vehicle| {
                roster.stats_for(vehicle, &HashMap::new(), None)
            }),
            &roster,
        );
        if !self.is_current(generation) {
            return;
        }
        self.update(|snapshot| Snapshot {
            battle: Some(early),
            ..snapshot
        });

        let (expected, expected_notice) = self.expected_values().await;
        let ship_records = fetch_ship_records(&client, &roster).await;
        if !self.is_current(generation) {
            return;
        }

        let complete = with_clans(
            assemble_battle(
                &arena,
                Some(region),
                &catalog,
                expected.as_ref(),
                |vehicle| roster.stats_for(vehicle, &ship_records, expected.as_ref()),
            ),
            &roster,
        );
        self.update(|snapshot| Snapshot {
            phase: Phase::InBattle,
            battle: Some(complete),
            notice: region_notice.clone().or(expected_notice.clone()),
            ..snapshot
        });
    }

    async fn ships(&self, client: &WgClient, game_version: &str, required: &[u64]) -> ShipCatalog {
        let mut cached = self.0.catalog.lock().await;
        let complete = required.iter().all(|id| cached.get(*id).is_some());
        if complete && !cached.is_empty() {
            return cached.clone();
        }
        match ShipCatalog::load(&self.0.cache_dir, client, game_version, required).await {
            Ok(catalog) => {
                *cached = catalog.clone();
                catalog
            }
            Err(error) => {
                log::warn!("ship catalog unavailable: {error}");
                cached.clone()
            }
        }
    }

    async fn expected_values(&self) -> (Option<ExpectedValues>, Option<Notice>) {
        let mut cached = self.0.expected.lock().await;
        if let Some(values) = cached.as_ref() {
            return (Some(values.clone()), None);
        }
        match ExpectedValues::load(&self.0.cache_dir, &self.0.http).await {
            Ok(values) => {
                *cached = Some(values.clone());
                (Some(values), None)
            }
            Err(error) => {
                log::warn!("expected values unavailable: {error}");
                let message = "Personal rating is unavailable because the expected values could not be downloaded.";
                (None, Some(notice(NoticeLevel::Warning, message.into())))
            }
        }
    }

    fn publish(&self, snapshot: Snapshot) {
        self.update(|_| snapshot);
    }

    fn update(&self, change: impl FnOnce(Snapshot) -> Snapshot) {
        let next = {
            let mut guard = lock(&self.0.snapshot);
            let next = change(guard.clone());
            *guard = next.clone();
            next
        };
        if let Err(error) = self.0.app.emit(SNAPSHOT_EVENT, next) {
            log::warn!("could not send the snapshot to the window: {error}");
        }
    }
}

fn notice(level: NoticeLevel, message: String) -> Notice {
    Notice { level, message }
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
        ships: &HashMap<u64, Vec<ShipRecord>>,
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
        let records = ships.get(&account.account_id);
        let overall_rating = expected.zip(records).and_then(|(expected, records)| {
            let per_ship: Vec<(u64, Totals)> = records
                .iter()
                .map(|record| (record.ship_id, record.pvp))
                .collect();
            pr::personal_rating(&per_ship, expected)
        });
        let solo_rating = expected.zip(records).and_then(|(expected, records)| {
            let per_ship: Vec<(u64, Totals)> = records
                .iter()
                .filter_map(|record| record.pvp_solo.map(|solo| (record.ship_id, solo)))
                .collect();
            pr::personal_rating(&per_ship, expected)
        });
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
            overall: StatLine::from_totals(pvp, overall_rating),
            overall_solo: account
                .pvp_solo
                .map(|solo| StatLine::from_totals(solo, solo_rating)),
            ship: ship_record.map(|record| ship_line(record.pvp)),
            ship_solo: ship_record
                .and_then(|record| record.pvp_solo)
                .map(ship_line),
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
                },
            )]),
            clans: HashMap::new(),
        };
        let stats = roster.stats_for(&vehicle("Shy", Relation::Enemy, 1), &HashMap::new(), None);
        assert_eq!(stats, PlayerStats::Hidden { account_id: 7 });
    }

    #[test]
    fn a_name_wargaming_does_not_know_is_reported_as_not_found() {
        let stats =
            empty_roster().stats_for(&vehicle("Ghost", Relation::Ally, 1), &HashMap::new(), None);
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
        assert_eq!(
            empty_roster().stats_for(&bot, &HashMap::new(), None),
            PlayerStats::Bot
        );
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
}
