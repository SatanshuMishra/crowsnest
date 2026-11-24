use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::iter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use wows_replays::ReplayFile;
use wows_replays::analyzer::arena_scan::scan_arena_state;
use wowsunpack::data::Version;
use wowsunpack::game_data::load_game_resources;
use wowsunpack::rpc::entitydefs::EntitySpec;

use crate::model::ArenaInfo;

const REPLAY_FILE_NAME: &str = "temp.wowsreplay";
const ARENA_FILE_NAME: &str = "tempArenaInfo.json";
const CONTAINER_MAGIC: [u8; 4] = [0x12, 0x32, 0x34, 0x11];
const LARGEST_DIVISION: usize = 3;
const ON_START_AFTER_NAME: i64 = 2;
const PREBATTLE_AFTER_NAME: i64 = 4;

pub type Divisions = HashMap<String, u8>;
type RawPlayer = HashMap<i64, String>;

#[derive(Default)]
pub struct SpecCache(Mutex<Option<(u32, Arc<Vec<EntitySpec>>)>>);

impl SpecCache {
    fn specs_for(&self, game_root: &Path, version: &Version) -> Option<Arc<Vec<EntitySpec>>> {
        let build = version.build_number()?;
        let mut slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, specs)) = slot.as_ref().filter(|(cached, _)| *cached == build) {
            return Some(Arc::clone(specs));
        }
        let specs = Arc::new(load_game_resources(game_root, version).ok()?.specs);
        *slot = Some((build, Arc::clone(&specs)));
        Some(specs)
    }
}

pub fn read_divisions(
    specs: &SpecCache,
    game_root: &Path,
    watch_root: &Path,
    arena: &ArenaInfo,
) -> Option<Divisions> {
    replay_folders(watch_root)
        .iter()
        .find_map(|folder| read_folder(specs, game_root, folder, arena))
}

fn replay_folders(watch_root: &Path) -> Vec<PathBuf> {
    let nested = fs::read_dir(watch_root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir());
    let newest_first: BTreeSet<(Reverse<SystemTime>, PathBuf)> =
        iter::once(watch_root.to_path_buf())
            .chain(nested)
            .filter_map(|folder| {
                let written = fs::metadata(folder.join(REPLAY_FILE_NAME))
                    .and_then(|meta| meta.modified())
                    .ok()?;
                Some((Reverse(written), folder))
            })
            .collect();
    newest_first.into_iter().map(|(_, folder)| folder).collect()
}

fn read_folder(
    specs: &SpecCache,
    game_root: &Path,
    folder: &Path,
    arena: &ArenaInfo,
) -> Option<Divisions> {
    let stream = fs::read(folder.join(REPLAY_FILE_NAME)).ok()?;
    let replay = if stream.starts_with(&CONTAINER_MAGIC) {
        ReplayFile::from_partial_bytes(&stream).ok()?
    } else {
        let meta = fs::read(folder.join(ARENA_FILE_NAME)).ok()?;
        ReplayFile::from_decrypted_parts(meta, stream).ok()?
    };
    let version = Version::try_from_client_exe(&replay.meta.clientVersionFromExe)?;
    let specs = specs.specs_for(game_root, &version)?;
    let state = scan_arena_state(&specs, version, &replay)?;
    let players: Vec<&RawPlayer> = state.players.iter().map(|player| player.raw()).collect();
    divisions_in(&players, arena)
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
    let groups = seats
        .iter()
        .filter_map(|seat| seat.prebattle.filter(|id| *id != 0).map(|id| (id, seat)))
        .fold(
            BTreeMap::<i64, Vec<&Seat>>::new(),
            |mut groups, (id, seat)| {
                groups.entry(id).or_default().push(seat);
                groups
            },
        );
    let crosses_teams = groups.values().any(|group| {
        group.iter().any(|seat| seat.friendly) && group.iter().any(|seat| !seat.friendly)
    });
    if crosses_teams {
        return None;
    }
    let divisions = groups
        .into_values()
        .filter(|group| (2..=LARGEST_DIVISION).contains(&group.len()))
        .scan((0u8, 0u8), |(allies, enemies), group| {
            let count = if group[0].friendly { allies } else { enemies };
            *count += 1;
            Some((*count, group))
        })
        .flat_map(|(number, group)| {
            group
                .into_iter()
                .map(move |seat| (seat.name.to_string(), number))
        })
        .collect();
    Some(divisions)
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
    use std::time::Duration;

    use super::*;
    use crate::model::{ArenaVehicle, Relation};

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
            .map(|index| (index, format!("I64({})", 1000 + index)))
            .chain([
                (name_at, format!("String(Text({name:?}))")),
                (name_at + 1, "Object(Dict([]))".to_string()),
                (name_at + ON_START_AFTER_NAME, format!("I64({on_start})")),
                (name_at + PREBATTLE_AFTER_NAME, format!("I64({prebattle})")),
            ])
            .collect()
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
    fn the_newest_battle_folder_is_read_first() {
        let root = tempfile::tempdir().unwrap();
        let older = root.path().join("15.9.0.0");
        let newer = root.path().join("15.10.0.0");
        let written = |folder: &Path, age: u64| {
            fs::create_dir(folder).unwrap();
            let file = fs::File::create(folder.join(REPLAY_FILE_NAME)).unwrap();
            file.set_modified(SystemTime::now() - Duration::from_secs(age))
                .unwrap();
        };
        written(&older, 600);
        written(&newer, 5);
        fs::create_dir(root.path().join("no battle")).unwrap();
        assert_eq!(replay_folders(root.path()), vec![newer, older]);
    }

    #[test]
    fn bots_are_left_out_of_the_match() {
        let battle = arena(&[
            ("Rook", Relation::Me),
            ("Bishop", Relation::Ally),
            (":IJN_bot:", Relation::Enemy),
        ]);
        let bot: RawPlayer = [(3, "String(Text(\":IJN_bot:\"))".to_string())]
            .into_iter()
            .collect();
        let players = [raw("Rook", 42), raw("Bishop", 42), bot];
        assert_eq!(
            read(&players, &battle),
            expect(&[("Rook", 1), ("Bishop", 1)])
        );
    }
}
