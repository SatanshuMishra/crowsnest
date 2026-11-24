use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Eu,
    Na,
    Asia,
}

impl Region {
    pub const ALL: [Region; 3] = [Region::Eu, Region::Na, Region::Asia];

    pub fn api_host(self) -> &'static str {
        match self {
            Region::Eu => "api.worldofwarships.eu",
            Region::Na => "api.worldofwarships.com",
            Region::Asia => "api.worldofwarships.asia",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallSource {
    GameCenter,
    Steam,
    DefaultPath,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInstall {
    pub root: PathBuf,
    pub source: InstallSource,
    pub build: Option<u32>,
    pub client_version: Option<String>,
    pub replays_dir: PathBuf,
    pub replays_enabled: bool,
    pub region: Option<Region>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    Me,
    Ally,
    Enemy,
}

impl Relation {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => Relation::Me,
            1 => Relation::Ally,
            _ => Relation::Enemy,
        }
    }

    pub fn is_friendly(self) -> bool {
        matches!(self, Relation::Me | Relation::Ally)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShipClass {
    Destroyer,
    Cruiser,
    Battleship,
    AirCarrier,
    Submarine,
    Unknown,
}

impl ShipClass {
    pub fn from_api(value: &str) -> Self {
        match value {
            "Destroyer" => ShipClass::Destroyer,
            "Cruiser" => ShipClass::Cruiser,
            "Battleship" => ShipClass::Battleship,
            "AirCarrier" => ShipClass::AirCarrier,
            "Submarine" => ShipClass::Submarine,
            _ => ShipClass::Unknown,
        }
    }

    pub fn sort_rank(self) -> u8 {
        match self {
            ShipClass::AirCarrier => 0,
            ShipClass::Battleship => 1,
            ShipClass::Cruiser => 2,
            ShipClass::Destroyer => 3,
            ShipClass::Submarine => 4,
            ShipClass::Unknown => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ship {
    pub id: u64,
    pub name: String,
    pub tier: u8,
    pub class: ShipClass,
    pub nation: String,
    pub is_premium: bool,
    pub is_special: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaVehicle {
    pub player_id: i64,
    pub name: String,
    pub relation: Relation,
    pub ship_id: u64,
}

impl ArenaVehicle {
    pub fn is_bot(&self) -> bool {
        self.name.starts_with(':') || self.player_id <= 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaInfo {
    pub started_at: String,
    pub map_id: Option<i64>,
    pub map_name: String,
    pub map_display_name: String,
    pub match_group: String,
    pub game_type: String,
    pub scenario: String,
    pub players_per_team: u32,
    pub player_name: String,
    pub player_vehicle: String,
    pub client_version: String,
    pub vehicles: Vec<ArenaVehicle>,
}

impl ArenaInfo {
    pub fn fingerprint(&self) -> String {
        format!("{}|{}|{}", self.started_at, self.map_name, self.player_name)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub battles: u64,
    pub wins: u64,
    pub survived: u64,
    pub damage: u64,
    pub frags: u64,
    pub xp: u64,
}

impl Totals {
    pub fn plus(self, other: Totals) -> Totals {
        Totals {
            battles: self.battles + other.battles,
            wins: self.wins + other.wins,
            survived: self.survived + other.survived,
            damage: self.damage + other.damage,
            frags: self.frags + other.frags,
            xp: self.xp + other.xp,
        }
    }

    fn per_battle(self, value: u64) -> Option<f64> {
        (self.battles > 0).then(|| value as f64 / self.battles as f64)
    }

    pub fn win_rate(self) -> Option<f64> {
        self.per_battle(self.wins).map(|ratio| ratio * 100.0)
    }

    pub fn survival_rate(self) -> Option<f64> {
        self.per_battle(self.survived).map(|ratio| ratio * 100.0)
    }

    pub fn avg_damage(self) -> Option<f64> {
        self.per_battle(self.damage)
    }

    pub fn avg_frags(self) -> Option<f64> {
        self.per_battle(self.frags)
    }

    pub fn avg_xp(self) -> Option<f64> {
        self.per_battle(self.xp)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatLine {
    pub battles: u64,
    pub wins: u64,
    pub win_rate: Option<f64>,
    pub avg_damage: Option<f64>,
    pub avg_frags: Option<f64>,
    pub avg_xp: Option<f64>,
    pub survival_rate: Option<f64>,
    pub personal_rating: Option<f64>,
}

impl StatLine {
    pub fn from_totals(totals: Totals, personal_rating: Option<f64>) -> StatLine {
        StatLine {
            battles: totals.battles,
            wins: totals.wins,
            win_rate: totals.win_rate(),
            avg_damage: totals.avg_damage(),
            avg_frags: totals.avg_frags(),
            avg_xp: totals.avg_xp(),
            survival_rate: totals.survival_rate(),
            personal_rating,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum PlayerStats {
    Pending,
    Bot,
    Hidden {
        account_id: u64,
    },
    NotFound,
    Failed {
        reason: String,
    },
    Loaded {
        account_id: u64,
        overall: StatLine,
        overall_solo: Option<StatLine>,
        ship: Option<StatLine>,
        ship_solo: Option<StatLine>,
        created_at: Option<i64>,
    },
}

impl PlayerStats {
    pub fn account_id(&self) -> Option<u64> {
        match self {
            PlayerStats::Hidden { account_id } | PlayerStats::Loaded { account_id, .. } => {
                Some(*account_id)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedStats {
    pub damage: f64,
    pub frags: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub name: String,
    pub relation: Relation,
    pub clan_tag: Option<String>,
    pub ship: Option<Ship>,
    pub ship_id: u64,
    pub expected: Option<ExpectedStats>,
    pub division: Option<u8>,
    pub stats: PlayerStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub players: Vec<Player>,
}

impl Team {
    pub fn of(players: Vec<Player>) -> Team {
        Team { players }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
    pub id: String,
    pub region: Option<Region>,
    pub arena: ArenaInfo,
    pub allies: Team,
    pub enemies: Team,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    NeedsInstall,
    NeedsApplicationId,
    ReplaysDisabled,
    Waiting,
    Loading,
    InBattle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub level: NoticeLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: Phase,
    pub install: Option<GameInstall>,
    pub battle: Option<Battle>,
    pub notice: Option<Notice>,
}

impl Snapshot {
    pub fn initial() -> Snapshot {
        Snapshot {
            phase: Phase::NeedsInstall,
            install: None,
            battle: None,
            notice: None,
        }
    }
}
