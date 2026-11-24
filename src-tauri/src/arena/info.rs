use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Deserializer};

use crate::model::{ArenaInfo, ArenaVehicle, Relation};

pub const FILE_NAME: &str = "tempArenaInfo.json";

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

static GAME_DATE_TIME: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"^([0-9]{1,2})\.([0-9]{1,2})\.([0-9]{4}) ([0-9]{1,2}):([0-9]{2}):([0-9]{2})$").ok()
});

#[derive(Debug, thiserror::Error)]
pub enum ArenaError {
    #[error("tempArenaInfo.json is not valid JSON yet: {0}")]
    Json(#[from] serde_json::Error),
    #[error("tempArenaInfo.json has no vehicles")]
    Empty,
}

pub fn parse(bytes: &[u8]) -> Result<ArenaInfo, ArenaError> {
    let document = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let arena: GameArena = serde_json::from_slice(document)?;
    Some(arena)
        .filter(|arena| !arena.vehicles.is_empty())
        .map(ArenaInfo::from)
        .ok_or(ArenaError::Empty)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameArena {
    #[serde(default, deserialize_with = "null_as_default")]
    date_time: String,
    #[serde(default)]
    map_id: Option<i64>,
    #[serde(default, deserialize_with = "null_as_default")]
    map_name: String,
    #[serde(default, deserialize_with = "null_as_default")]
    map_display_name: String,
    #[serde(default, deserialize_with = "null_as_default")]
    match_group: String,
    #[serde(default, deserialize_with = "null_as_default")]
    game_type: String,
    #[serde(default, deserialize_with = "null_as_default")]
    scenario: String,
    #[serde(default, deserialize_with = "null_as_default")]
    players_per_team: u32,
    #[serde(default, deserialize_with = "null_as_default")]
    player_name: String,
    #[serde(default, deserialize_with = "null_as_default")]
    player_vehicle: String,
    #[serde(default, deserialize_with = "null_as_default")]
    client_version_from_exe: String,
    #[serde(default, deserialize_with = "null_as_default")]
    client_version_from_xml: String,
    #[serde(default, deserialize_with = "null_as_default")]
    vehicles: Vec<GameVehicle>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameVehicle {
    #[serde(default, deserialize_with = "null_as_default")]
    id: i64,
    #[serde(default, deserialize_with = "null_as_default")]
    name: String,
    #[serde(default = "enemy_code", deserialize_with = "relation_code")]
    relation: u8,
    #[serde(default, deserialize_with = "null_as_default")]
    ship_id: u64,
}

fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

fn enemy_code() -> u8 {
    u8::MAX
}

fn relation_code<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer).map(|code| {
        code.and_then(|code| u8::try_from(code).ok())
            .unwrap_or_else(enemy_code)
    })
}

impl From<GameVehicle> for ArenaVehicle {
    fn from(vehicle: GameVehicle) -> ArenaVehicle {
        ArenaVehicle {
            player_id: vehicle.id,
            name: vehicle.name,
            relation: Relation::from_code(vehicle.relation),
            ship_id: vehicle.ship_id,
        }
    }
}

impl From<GameArena> for ArenaInfo {
    fn from(arena: GameArena) -> ArenaInfo {
        ArenaInfo {
            started_at: iso_local_date_time(&arena.date_time).unwrap_or(arena.date_time),
            map_id: arena.map_id,
            map_name: arena.map_name,
            map_display_name: arena.map_display_name,
            match_group: arena.match_group,
            game_type: arena.game_type,
            scenario: arena.scenario,
            players_per_team: arena.players_per_team,
            player_name: arena.player_name,
            player_vehicle: arena.player_vehicle,
            client_version: first_present(
                arena.client_version_from_exe,
                arena.client_version_from_xml,
            ),
            vehicles: arena.vehicles.into_iter().map(ArenaVehicle::from).collect(),
        }
    }
}

fn first_present(preferred: String, fallback: String) -> String {
    Some(preferred)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
}

fn iso_local_date_time(game_date_time: &str) -> Option<String> {
    let captures = GAME_DATE_TIME.as_ref()?.captures(game_date_time.trim())?;
    let field = |index: usize| {
        captures
            .get(index)
            .and_then(|found| found.as_str().parse::<u32>().ok())
    };
    let (day, month, year) = (field(1)?, field(2)?, field(3)?);
    let (hour, minute, second) = (field(4)?, field(5)?, field(6)?);
    let plausible = (1..=31).contains(&day)
        && (1..=12).contains(&month)
        && hour < 24
        && minute < 60
        && second < 60;
    plausible.then(|| format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!("fixtures/tempArenaInfo.json");

    fn fixture() -> ArenaInfo {
        parse(FIXTURE).expect("the fixture parses")
    }

    fn with_bom(document: &[u8]) -> Vec<u8> {
        UTF8_BOM.iter().chain(document).copied().collect()
    }

    #[test]
    fn parses_the_header_of_a_random_battle() {
        let arena = fixture();

        assert_eq!(arena.started_at, "2026-10-29T19:21:49");
        assert_eq!(arena.map_id, Some(38));
        assert_eq!(arena.map_name, "spaces/38_Canada");
        assert_eq!(arena.map_display_name, "38_Canada");
        assert_eq!(arena.match_group, "pvp");
        assert_eq!(arena.game_type, "RandomBattle");
        assert_eq!(arena.scenario, "Domination");
        assert_eq!(arena.players_per_team, 12);
        assert_eq!(arena.player_name, "Player01");
        assert_eq!(arena.player_vehicle, "PRSB108-Vladivostok");
        assert_eq!(arena.client_version, "15,10,0,13533378");
    }

    #[test]
    fn parses_twelve_versus_twelve_with_the_player_as_me() {
        let arena = fixture();
        let count = |relation: Relation| {
            arena
                .vehicles
                .iter()
                .filter(|vehicle| vehicle.relation == relation)
                .count()
        };

        assert_eq!(arena.vehicles.len(), 24);
        assert_eq!(count(Relation::Me), 1);
        assert_eq!(count(Relation::Ally), 11);
        assert_eq!(count(Relation::Enemy), 12);
        assert_eq!(
            arena.vehicles.first(),
            Some(&ArenaVehicle {
                player_id: 1043512,
                name: "Player01".into(),
                relation: Relation::Me,
                ship_id: 4181604176,
            })
        );
    }

    #[test]
    fn keeps_a_bot_name_wrapped_in_colons() {
        let bots: Vec<String> = fixture()
            .vehicles
            .into_iter()
            .filter(ArenaVehicle::is_bot)
            .map(|vehicle| vehicle.name)
            .collect();

        assert_eq!(bots, vec![":Player24:".to_string()]);
    }

    #[test]
    fn strips_a_utf8_byte_order_mark() {
        assert_eq!(parse(&with_bom(FIXTURE)).ok(), Some(fixture()));
    }

    #[test]
    fn rejects_a_document_without_vehicles() {
        assert!(matches!(
            parse(br#"{"playerName": "Player01", "vehicles": []}"#),
            Err(ArenaError::Empty)
        ));
        assert!(matches!(
            parse(br#"{"playerName": "Player01"}"#),
            Err(ArenaError::Empty)
        ));
    }

    #[test]
    fn reports_a_half_written_file_as_invalid_json() {
        assert!(matches!(
            parse(&FIXTURE[..FIXTURE.len() / 2]),
            Err(ArenaError::Json(_))
        ));
        assert!(matches!(parse(b""), Err(ArenaError::Json(_))));
        assert!(matches!(parse(&with_bom(b"")), Err(ArenaError::Json(_))));
    }

    #[test]
    fn defaults_missing_and_null_keys_and_ignores_unknown_ones() {
        let arena = parse(
            br#"{
                "mapName": null,
                "clientVersionFromXml": "15,10,0,13533378",
                "someFutureKey": {"nested": [1, 2, 3]},
                "vehicles": [{"name": "Player01", "relation": 0, "extra": true}]
            }"#,
        )
        .expect("a sparse document parses");

        assert_eq!(
            arena,
            ArenaInfo {
                started_at: String::new(),
                map_id: None,
                map_name: String::new(),
                map_display_name: String::new(),
                match_group: String::new(),
                game_type: String::new(),
                scenario: String::new(),
                players_per_team: 0,
                player_name: String::new(),
                player_vehicle: String::new(),
                client_version: "15,10,0,13533378".into(),
                vehicles: vec![ArenaVehicle {
                    player_id: 0,
                    name: "Player01".into(),
                    relation: Relation::Me,
                    ship_id: 0,
                }],
            }
        );
    }

    #[test]
    fn treats_a_missing_or_unknown_relation_as_enemy() {
        let arena = parse(br#"{"vehicles": [{"name": "A"}, {"name": "B", "relation": 900}]}"#)
            .expect("vehicles without relations parse");
        let relations: Vec<Relation> = arena
            .vehicles
            .iter()
            .map(|vehicle| vehicle.relation)
            .collect();

        assert_eq!(relations, vec![Relation::Enemy, Relation::Enemy]);
    }

    #[test]
    fn converts_the_game_date_time_to_iso_local_form() {
        assert_eq!(
            iso_local_date_time("29.10.2026 19:21:49").as_deref(),
            Some("2026-10-29T19:21:49")
        );
        assert_eq!(
            iso_local_date_time("1.2.2026 7:05:09").as_deref(),
            Some("2026-02-01T07:05:09")
        );
    }

    #[test]
    fn keeps_an_unrecognised_date_time_as_it_was() {
        let parsed_with = |date_time: &str| {
            let document = format!(
                r#"{{"dateTime": "{date_time}", "vehicles": [{{"name": "A", "relation": 0}}]}}"#
            );
            parse(document.as_bytes())
                .map(|arena| arena.started_at)
                .ok()
        };

        assert_eq!(
            parsed_with("2026-10-29 19:21:49").as_deref(),
            Some("2026-10-29 19:21:49")
        );
        assert_eq!(
            parsed_with("31.13.2026 19:21:49").as_deref(),
            Some("31.13.2026 19:21:49")
        );
        assert_eq!(iso_local_date_time("yesterday"), None);
    }
}
