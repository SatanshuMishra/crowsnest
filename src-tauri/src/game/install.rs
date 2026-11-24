use std::path::{Path, PathBuf};

use crate::game::config;
use crate::model::{GameInstall, InstallSource};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InstallError {
    #[error("the folder does not exist")]
    Missing,
    #[error("the folder is not a World of Warships installation")]
    NotAGameFolder,
    #[error("no game build was found under bin")]
    NoBuild,
}

const MAX_PARENT_STEPS: usize = 3;
const GAME_INFO_FILE: &str = "game_info.xml";
const LAUNCHER_FILE: &str = "WorldOfWarships.exe";
const STEAM_APP_ID: u32 = 552_990;
const DEFAULT_STEAM_ROOT: &str = r"C:\Program Files (x86)\Steam";
const DEFAULT_GAMES_FOLDER: &str = "Games";
const DEFAULT_FOLDER_NAMES: [&str; 4] = [
    "World_of_Warships",
    "World_of_Warships_NA",
    "World_of_Warships_EU",
    "World_of_Warships_ASIA",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    root: PathBuf,
    source: InstallSource,
}

impl Candidate {
    fn new(root: PathBuf, source: InstallSource) -> Candidate {
        Candidate { root, source }
    }
}

pub fn detect() -> Vec<GameInstall> {
    let candidates = [
        game_center_candidates(),
        registry_candidates(),
        steam_candidates(),
        default_folder_candidates(),
    ]
    .concat();
    installs_from(candidates)
}

pub fn inspect(root: &Path, source: InstallSource) -> Result<GameInstall, InstallError> {
    if !root.exists() {
        return Err(InstallError::Missing);
    }
    let reachable: Vec<&Path> = root.ancestors().take(MAX_PARENT_STEPS + 1).collect();
    reachable
        .iter()
        .find_map(|dir| inspect_root(dir, source).ok())
        .ok_or_else(|| {
            if reachable.iter().any(|dir| has_game_marker(dir)) {
                InstallError::NoBuild
            } else {
                InstallError::NotAGameFolder
            }
        })
}

fn has_game_marker(dir: &Path) -> bool {
    [GAME_INFO_FILE, LAUNCHER_FILE]
        .iter()
        .any(|name| dir.join(name).is_file())
}

fn inspect_root(root: &Path, source: InstallSource) -> Result<GameInstall, InstallError> {
    if !root.is_dir() {
        return Err(InstallError::Missing);
    }
    if !has_game_marker(root) {
        return Err(InstallError::NotAGameFolder);
    }
    let build = config::latest_build(root).ok_or(InstallError::NoBuild)?;
    let client_version = config::client_version(root);
    let replays = config::replays_location(root, build, client_version.as_deref());
    Ok(GameInstall {
        root: root.to_path_buf(),
        source,
        build: Some(build),
        client_version,
        replays_dir: replays.dir,
        replays_enabled: replays.enabled,
        region: config::active_region(root),
    })
}

fn installs_from(candidates: Vec<Candidate>) -> Vec<GameInstall> {
    let (live, public_test): (Vec<GameInstall>, Vec<GameInstall>) = distinct_locations(candidates)
        .into_iter()
        .filter_map(|candidate| inspect_root(&candidate.root, candidate.source).ok())
        .partition(|install| !config::is_public_test(&install.root));
    [live, public_test].concat()
}

fn location_key(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn distinct_locations(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let keyed: Vec<(PathBuf, Candidate)> = candidates
        .into_iter()
        .map(|candidate| (location_key(&candidate.root), candidate))
        .collect();
    keyed
        .iter()
        .enumerate()
        .filter(|(index, (key, _))| !keyed[..*index].iter().any(|(earlier, _)| earlier == key))
        .map(|(_, (_, candidate))| candidate.clone())
        .collect()
}

fn game_center_preferences_path() -> PathBuf {
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
        .join("Wargaming.net")
        .join("GameCenter")
        .join("preferences.xml")
}

fn game_center_candidates() -> Vec<Candidate> {
    config::read_text(&game_center_preferences_path())
        .map(|text| working_dirs(&text))
        .unwrap_or_default()
        .into_iter()
        .map(|root| Candidate::new(root, InstallSource::GameCenter))
        .collect()
}

fn working_dirs(preferences: &str) -> Vec<PathBuf> {
    config::element_texts(preferences, "working_dir")
        .iter()
        .map(|raw| decoded_entities(raw))
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn decoded_entities(raw: &str) -> String {
    raw.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn registry_candidates() -> Vec<Candidate> {
    registry::uninstall_locations()
        .into_iter()
        .map(|root| Candidate::new(root, InstallSource::DefaultPath))
        .collect()
}

fn steam_candidates() -> Vec<Candidate> {
    let steam_roots = [
        registry::steam_roots(),
        vec![PathBuf::from(DEFAULT_STEAM_ROOT)],
    ]
    .concat();
    steam_game_dirs(&steam_roots)
        .into_iter()
        .map(|root| Candidate::new(root, InstallSource::Steam))
        .collect()
}

fn steam_game_dirs(steam_roots: &[PathBuf]) -> Vec<PathBuf> {
    steam_roots
        .iter()
        .flat_map(|steam_root| [vec![steam_root.clone()], library_folders(steam_root)].concat())
        .filter_map(|library| steam_game_dir(&library))
        .collect()
}

fn library_folders(steam_root: &Path) -> Vec<PathBuf> {
    config::read_text(&steam_root.join("steamapps").join("libraryfolders.vdf"))
        .map(|text| library_paths(&vdf::parse(&text)))
        .unwrap_or_default()
}

fn steam_game_dir(library: &Path) -> Option<PathBuf> {
    let steamapps = library.join("steamapps");
    let manifest = config::read_text(&steamapps.join(format!("appmanifest_{STEAM_APP_ID}.acf")))?;
    install_dir_name(&vdf::parse(&manifest)).map(|name| steamapps.join("common").join(name))
}

fn is_library_index(key: &str) -> bool {
    !key.is_empty() && key.bytes().all(|byte| byte.is_ascii_digit())
}

fn library_paths(document: &[(String, vdf::Value)]) -> Vec<PathBuf> {
    vdf::find(document, "libraryfolders")
        .map(|folders| {
            folders
                .entries()
                .iter()
                .filter_map(|(key, entry)| match entry {
                    vdf::Value::Section(_) => entry.get("path").and_then(vdf::Value::text),
                    vdf::Value::Text(path) if is_library_index(key) => Some(path.as_str()),
                    vdf::Value::Text(_) => None,
                })
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

fn install_dir_name(manifest: &[(String, vdf::Value)]) -> Option<String> {
    vdf::find(manifest, "AppState")
        .and_then(|state| state.get("installdir"))
        .and_then(vdf::Value::text)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

fn default_folder_candidates() -> Vec<Candidate> {
    default_folders(&registry::drive_roots())
        .into_iter()
        .map(|root| Candidate::new(root, InstallSource::DefaultPath))
        .collect()
}

fn default_folders(drives: &[PathBuf]) -> Vec<PathBuf> {
    drives
        .iter()
        .flat_map(|drive| {
            DEFAULT_FOLDER_NAMES
                .iter()
                .map(move |name| drive.join(DEFAULT_GAMES_FOLDER).join(name))
        })
        .collect()
}

mod vdf {
    use std::iter::Peekable;
    use std::str::Chars;

    const MAX_DEPTH: usize = 32;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Value {
        Text(String),
        Section(Vec<(String, Value)>),
    }

    impl Value {
        pub fn text(&self) -> Option<&str> {
            match self {
                Value::Text(text) => Some(text),
                Value::Section(_) => None,
            }
        }

        pub fn entries(&self) -> &[(String, Value)] {
            match self {
                Value::Section(entries) => entries,
                Value::Text(_) => &[],
            }
        }

        pub fn get(&self, key: &str) -> Option<&Value> {
            find(self.entries(), key)
        }
    }

    pub fn find<'a>(entries: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
        entries
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Token {
        Text(String),
        Open,
        Close,
    }

    type Cursor<'a> = Peekable<Chars<'a>>;

    pub fn parse(text: &str) -> Vec<(String, Value)> {
        section(&tokenize(text), 0).0
    }

    fn tokenize(text: &str) -> Vec<Token> {
        let mut cursor = text.chars().peekable();
        let mut tokens = Vec::new();
        while let Some(current) = cursor.next() {
            match current {
                '{' => tokens.push(Token::Open),
                '}' => tokens.push(Token::Close),
                '"' => tokens.push(Token::Text(quoted(&mut cursor))),
                '/' if cursor.peek() == Some(&'/') => skip_line(&mut cursor),
                other if other.is_whitespace() => {}
                other => tokens.push(Token::Text(bare(other, &mut cursor))),
            }
        }
        tokens
    }

    fn quoted(cursor: &mut Cursor) -> String {
        let mut value = String::new();
        while let Some(current) = cursor.next() {
            match current {
                '"' => break,
                '\\' => match cursor.next() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some(escaped) => value.push(escaped),
                    None => break,
                },
                other => value.push(other),
            }
        }
        value
    }

    fn ends_bare_word(next: &char) -> bool {
        next.is_whitespace() || matches!(next, '{' | '}' | '"')
    }

    fn bare(first: char, cursor: &mut Cursor) -> String {
        let rest: String =
            std::iter::from_fn(|| cursor.next_if(|next| !ends_bare_word(next))).collect();
        format!("{first}{rest}")
    }

    fn skip_line(cursor: &mut Cursor) {
        cursor
            .by_ref()
            .take_while(|current| *current != '\n')
            .for_each(drop);
    }

    fn section(tokens: &[Token], depth: usize) -> (Vec<(String, Value)>, &[Token]) {
        if depth > MAX_DEPTH {
            return (Vec::new(), &[]);
        }
        let mut entries = Vec::new();
        let mut rest = tokens;
        loop {
            match rest {
                [Token::Text(key), Token::Text(value), tail @ ..] => {
                    entries.push((key.clone(), Value::Text(value.clone())));
                    rest = tail;
                }
                [Token::Text(key), Token::Open, tail @ ..] => {
                    let (children, after) = section(tail, depth + 1);
                    entries.push((key.clone(), Value::Section(children)));
                    rest = match after {
                        [Token::Close, remaining @ ..] => remaining,
                        remaining => remaining,
                    };
                }
                [] | [Token::Close, ..] => return (entries, rest),
                [_, tail @ ..] => rest = tail,
            }
        }
    }
}

#[cfg(windows)]
mod registry {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const UNINSTALL_KEYS: [&str; 2] = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];
    const STEAM_UNINSTALL_PREFIX: &str = "Steam App ";
    const GAME_DISPLAY_NAMES: [&str; 2] = ["world of warships", "world_of_warships"];
    const MOUNTED_DEVICES_KEY: &str = r"SYSTEM\MountedDevices";
    const DOS_DEVICE_PREFIX: &str = r"\DosDevices\";
    const FIRST_FIXED_DRIVE: char = 'C';

    fn hives() -> [RegKey; 2] {
        [
            RegKey::predef(HKEY_CURRENT_USER),
            RegKey::predef(HKEY_LOCAL_MACHINE),
        ]
    }

    fn string_value(key: &RegKey, name: &str) -> Option<String> {
        key.get_value::<String, _>(name)
            .ok()
            .map(|value| value.trim().trim_matches('"').trim().to_owned())
            .filter(|value| !value.is_empty())
    }

    pub(super) fn is_game_display_name(display_name: &str) -> bool {
        let lowered = display_name.to_lowercase();
        GAME_DISPLAY_NAMES.iter().any(|name| lowered.contains(name))
    }

    fn game_locations(uninstall: &RegKey) -> Vec<PathBuf> {
        uninstall
            .enum_keys()
            .filter_map(Result::ok)
            .filter(|name| !name.starts_with(STEAM_UNINSTALL_PREFIX))
            .filter_map(|name| uninstall.open_subkey(name).ok())
            .filter(|entry| {
                string_value(entry, "DisplayName").is_some_and(|name| is_game_display_name(&name))
            })
            .filter_map(|entry| string_value(&entry, "InstallLocation"))
            .map(PathBuf::from)
            .collect()
    }

    pub fn uninstall_locations() -> Vec<PathBuf> {
        hives()
            .iter()
            .flat_map(|hive| {
                UNINSTALL_KEYS
                    .iter()
                    .filter_map(|path| hive.open_subkey(path).ok())
                    .collect::<Vec<_>>()
            })
            .flat_map(|uninstall| game_locations(&uninstall))
            .collect()
    }

    pub fn steam_roots() -> Vec<PathBuf> {
        let [user, machine] = hives();
        [
            (&machine, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath"),
            (&machine, r"SOFTWARE\Valve\Steam", "InstallPath"),
            (&user, r"Software\Valve\Steam", "SteamPath"),
        ]
        .iter()
        .filter_map(|(hive, path, value)| {
            hive.open_subkey(path)
                .ok()
                .and_then(|key| string_value(&key, value))
        })
        .map(PathBuf::from)
        .collect()
    }

    pub(super) fn drive_letter(value_name: &str) -> Option<char> {
        let drive = value_name.strip_prefix(DOS_DEVICE_PREFIX)?;
        match drive.chars().collect::<Vec<_>>().as_slice() {
            [letter, ':'] if letter.is_ascii_alphabetic() => Some(letter.to_ascii_uppercase()),
            _ => None,
        }
    }

    fn mounted_drive_letters() -> Vec<char> {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(MOUNTED_DEVICES_KEY)
            .map(|devices| {
                devices
                    .enum_values()
                    .filter_map(Result::ok)
                    .filter_map(|(name, _)| drive_letter(&name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn drive_roots() -> Vec<PathBuf> {
        let mounted = mounted_drive_letters();
        let letters: BTreeSet<char> = if mounted.is_empty() {
            (FIRST_FIXED_DRIVE..='Z').collect()
        } else {
            std::iter::once(FIRST_FIXED_DRIVE).chain(mounted).collect()
        };
        letters
            .into_iter()
            .filter(|letter| *letter >= FIRST_FIXED_DRIVE)
            .map(|letter| PathBuf::from(format!("{letter}:\\")))
            .filter(|drive| drive.is_dir())
            .collect()
    }
}

#[cfg(not(windows))]
mod registry {
    use std::path::PathBuf;

    pub fn uninstall_locations() -> Vec<PathBuf> {
        Vec::new()
    }

    pub fn steam_roots() -> Vec<PathBuf> {
        Vec::new()
    }

    pub fn drive_roots() -> Vec<PathBuf> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile::TempDir;

    use crate::model::Region;

    const BUILD: u32 = 13533378;
    const LIVE_ID: &str = "WOWS.WW.PRODUCTION";
    const PUBLIC_TEST_ID: &str = "WOWS.PT.PRODUCTION";
    const GAME_CENTER_FIXTURE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<protocol name="preferences" version="3.33">
  <application>
    <games_manager>
      <games>
        <game>
          <working_dir>E:\WoWs\World_of_Warships</working_dir>
          <auto_update>true</auto_update>
        </game>
        <game>
          <working_dir>G:\WoWsPT\World_of_Warships_PT</working_dir>
          <auto_update>false</auto_update>
        </game>
      </games>
    </games_manager>
  </application>
</protocol>
"#;
    const APP_MANIFEST_FIXTURE: &str = "\"AppState\"\n{\n\t\"appid\"\t\t\"552990\"\n\t\"name\"\t\t\"World of Warships\"\n\t\"installdir\"\t\t\"World of Warships\"\n}\n";
    const LIBRARY_FOLDERS_FIXTURE: &str =
        "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"FIXTURE_ROOT\"\n\t}\n}\n";

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().expect("file has a parent")).expect("create parent");
        fs::write(path, contents).expect("write file");
    }

    fn build_dir(root: &Path) -> PathBuf {
        root.join("bin").join(BUILD.to_string())
    }

    fn game_info(id: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<protocol name=\"game_info\" version=\"2.16\">\n  <game>\n    <id>{id}</id>\n    <part_versions>\n      <version name=\"client\" installed=\"15.10.0.0.13533378\" available=\"15.10.0.0.13533378\"/>\n    </part_versions>\n  </game>\n</protocol>\n"
        )
    }

    fn fake_install(root: &Path, id: &str) {
        write(&root.join(GAME_INFO_FILE), &game_info(id));
        write(
            &build_dir(root).join("res").join("engine_config.xml"),
            "<engine_config.xml>\n  <replays>\n    <dirPath>replays</dirPath>\n    <pathBase>CWD</pathBase>\n    <versioned>false</versioned>\n  </replays>\n</engine_config.xml>\n",
        );
        write(
            &root.join("preferences.xml"),
            "<preferences.xml>\n\t<active_server>\tWOWS NA\t</active_server>\n\t<last_server_version>\t15,10,0,13533378\t</last_server_version>\n</preferences.xml>\n",
        );
        fs::create_dir_all(build_dir(root).join("bin64")).expect("bin64");
        fs::create_dir_all(root.join("replays").join("15.10.0.0")).expect("replays");
    }

    fn live_install() -> TempDir {
        let dir = TempDir::new().expect("temp dir");
        fake_install(dir.path(), LIVE_ID);
        dir
    }

    fn escaped_for_vdf(path: &Path) -> String {
        path.display().to_string().replace('\\', "\\\\")
    }

    #[test]
    fn inspect_returns_a_fully_filled_install() {
        let dir = live_install();

        assert_eq!(
            inspect(dir.path(), InstallSource::Manual),
            Ok(GameInstall {
                root: dir.path().to_path_buf(),
                source: InstallSource::Manual,
                build: Some(BUILD),
                client_version: Some("15,10,0,13533378".to_owned()),
                replays_dir: dir.path().join("replays"),
                replays_enabled: true,
                region: Some(Region::Na),
            })
        );
    }

    #[test]
    fn inspect_walks_up_from_a_picked_subfolder_to_the_install_root() {
        let dir = live_install();
        let picks = [
            build_dir(dir.path()),
            build_dir(dir.path()).join("bin64"),
            dir.path().join("replays"),
            dir.path().join("replays").join("15.10.0.0"),
            dir.path().join(GAME_INFO_FILE),
        ];

        let roots: Vec<Option<PathBuf>> = picks
            .iter()
            .map(|pick| {
                inspect(pick, InstallSource::Manual)
                    .ok()
                    .map(|install| install.root)
            })
            .collect();

        assert_eq!(roots, vec![Some(dir.path().to_path_buf()); picks.len()]);
    }

    #[test]
    fn inspect_stops_walking_after_three_parents() {
        let dir = live_install();
        let deep = build_dir(dir.path()).join("bin64").join("deeper");
        fs::create_dir_all(&deep).expect("deep folder");

        assert_eq!(
            inspect(&deep, InstallSource::Manual),
            Err(InstallError::NotAGameFolder)
        );
    }

    #[test]
    fn inspect_reports_a_missing_folder() {
        let dir = TempDir::new().expect("temp dir");

        assert_eq!(
            inspect(&dir.path().join("gone"), InstallSource::Manual),
            Err(InstallError::Missing)
        );
    }

    #[test]
    fn inspect_rejects_a_folder_without_game_files() {
        let dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(build_dir(dir.path())).expect("build dir");

        assert_eq!(
            inspect(dir.path(), InstallSource::Manual),
            Err(InstallError::NotAGameFolder)
        );
    }

    #[test]
    fn inspect_reports_a_game_folder_without_a_numeric_build() {
        let dir = TempDir::new().expect("temp dir");
        write(&dir.path().join(GAME_INFO_FILE), &game_info(LIVE_ID));
        fs::create_dir_all(dir.path().join("bin").join("clientrunner")).expect("runner");

        assert_eq!(
            inspect(dir.path(), InstallSource::Manual),
            Err(InstallError::NoBuild)
        );
    }

    #[test]
    fn inspect_accepts_the_launcher_when_game_info_is_missing() {
        let dir = TempDir::new().expect("temp dir");
        write(&dir.path().join(LAUNCHER_FILE), "launcher");
        fs::create_dir_all(build_dir(dir.path())).expect("build dir");

        assert_eq!(
            inspect(dir.path(), InstallSource::DefaultPath),
            Ok(GameInstall {
                root: dir.path().to_path_buf(),
                source: InstallSource::DefaultPath,
                build: Some(BUILD),
                client_version: None,
                replays_dir: dir.path().join("replays"),
                replays_enabled: true,
                region: None,
            })
        );
    }

    #[test]
    fn game_center_preferences_list_every_working_dir() {
        assert_eq!(
            working_dirs(GAME_CENTER_FIXTURE),
            vec![
                PathBuf::from(r"E:\WoWs\World_of_Warships"),
                PathBuf::from(r"G:\WoWsPT\World_of_Warships_PT"),
            ]
        );
        assert_eq!(
            working_dirs(
                "<working_dir> D:\\Games &amp; More\\WoWs </working_dir><working_dir></working_dir>"
            ),
            vec![PathBuf::from(r"D:\Games & More\WoWs")]
        );
        assert!(working_dirs("<protocol><application>").is_empty());
    }

    #[test]
    fn library_folders_fixture_lists_its_library_path() {
        assert_eq!(
            library_paths(&vdf::parse(LIBRARY_FOLDERS_FIXTURE)),
            vec![PathBuf::from("FIXTURE_ROOT")]
        );
    }

    #[test]
    fn legacy_library_folders_list_numbered_paths() {
        let legacy = "\"LibraryFolders\"\n{\n\t\"TimeNextStatsReport\"\t\t\"1700000000\"\n\t\"ContentStatsID\"\t\t\"-4242\"\n\t\"1\"\t\t\"D:\\\\SteamLibrary\"\n}\n";

        assert_eq!(
            library_paths(&vdf::parse(legacy)),
            vec![PathBuf::from(r"D:\SteamLibrary")]
        );
    }

    #[test]
    fn app_manifest_fixture_names_the_install_dir() {
        assert_eq!(
            install_dir_name(&vdf::parse(APP_MANIFEST_FIXTURE)).as_deref(),
            Some("World of Warships")
        );
        assert_eq!(install_dir_name(&vdf::parse("\"AppState\" { }")), None);
    }

    #[test]
    fn vdf_parsing_tolerates_comments_bare_words_and_broken_braces() {
        let commented = "// header\n\"AppState\"\n{\n\tinstalldir \"World of Warships\" // trailing\n\t\"UserConfig\" { \"language\" \"english\" }\n";

        assert_eq!(
            vdf::parse(commented),
            vec![(
                "AppState".to_owned(),
                vdf::Value::Section(vec![
                    (
                        "installdir".to_owned(),
                        vdf::Value::Text("World of Warships".to_owned())
                    ),
                    (
                        "UserConfig".to_owned(),
                        vdf::Value::Section(vec![(
                            "language".to_owned(),
                            vdf::Value::Text("english".to_owned())
                        )])
                    ),
                ])
            )]
        );
        assert!(vdf::parse("").is_empty());
        assert!(vdf::parse("}}}{{\"").is_empty());
        assert_eq!(vdf::parse(&"\"k\" {".repeat(10_000)).len(), 1);
    }

    #[test]
    fn steam_libraries_lead_to_the_game_folder() {
        let steam = TempDir::new().expect("steam root");
        let library = TempDir::new().expect("library");
        write(
            &steam.path().join("steamapps").join("libraryfolders.vdf"),
            &LIBRARY_FOLDERS_FIXTURE.replace("FIXTURE_ROOT", &escaped_for_vdf(library.path())),
        );
        write(
            &library
                .path()
                .join("steamapps")
                .join("appmanifest_552990.acf"),
            APP_MANIFEST_FIXTURE,
        );

        assert_eq!(
            steam_game_dirs(&[steam.path().to_path_buf()]),
            vec![
                library
                    .path()
                    .join("steamapps")
                    .join("common")
                    .join("World of Warships")
            ]
        );
    }

    #[test]
    fn detected_installs_are_unique_and_public_test_comes_last() {
        let live = live_install();
        let public_test = TempDir::new().expect("public test");
        fake_install(public_test.path(), PUBLIC_TEST_ID);
        let missing = live.path().join("not_here");

        let installs = installs_from(vec![
            Candidate::new(public_test.path().to_path_buf(), InstallSource::GameCenter),
            Candidate::new(missing, InstallSource::DefaultPath),
            Candidate::new(live.path().to_path_buf(), InstallSource::GameCenter),
            Candidate::new(live.path().join("."), InstallSource::DefaultPath),
            Candidate::new(live.path().join("bin"), InstallSource::Steam),
        ]);

        let found: Vec<(PathBuf, InstallSource)> = installs
            .into_iter()
            .map(|install| (install.root, install.source))
            .collect();
        assert_eq!(
            found,
            vec![
                (live.path().to_path_buf(), InstallSource::GameCenter),
                (public_test.path().to_path_buf(), InstallSource::GameCenter),
            ]
        );
    }

    #[test]
    fn default_folders_cover_every_regional_variant() {
        let drive = PathBuf::from("D:\\");

        assert_eq!(
            default_folders(std::slice::from_ref(&drive)),
            DEFAULT_FOLDER_NAMES
                .iter()
                .map(|name| drive.join("Games").join(name))
                .collect::<Vec<_>>()
        );
    }

    #[cfg(windows)]
    #[test]
    fn registry_helpers_recognise_game_entries_and_drive_letters() {
        assert!(registry::is_game_display_name("World of Warships"));
        assert!(registry::is_game_display_name("World_of_Warships_PT"));
        assert!(!registry::is_game_display_name("World of Tanks"));
        assert_eq!(registry::drive_letter(r"\DosDevices\D:"), Some('D'));
        assert_eq!(registry::drive_letter(r"\??\Volume{1234}"), None);
        assert_eq!(registry::drive_letter(r"\DosDevices\COM1"), None);
    }
}
