use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use crate::model::Region;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaysLocation {
    pub dir: PathBuf,
    pub enabled: bool,
}

const GAME_INFO_FILE: &str = "game_info.xml";
const PREFERENCES_FILE: &str = "preferences.xml";
const ENGINE_CONFIG_FILE: &str = "engine_config.xml";
const DEFAULT_REPLAYS_DIR: &str = "replays";
const PUBLIC_TEST_MARKER: &str = ".PT.";
const SERVER_NAME_PREFIX: &str = "wows ";

static XML_COMMENT: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"(?s)<!--.*?-->").ok());

pub(crate) fn read_text(path: &Path) -> Option<String> {
    fs::read(path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

fn element_pattern(name: &str) -> Option<Regex> {
    let escaped = regex::escape(name);
    Regex::new(&format!(r"(?s)<{escaped}\b[^>/]*>(.*?)</{escaped}\s*>")).ok()
}

pub(crate) fn element_texts(text: &str, name: &str) -> Vec<String> {
    element_pattern(name)
        .map(|pattern| {
            pattern
                .captures_iter(text)
                .filter_map(|captures| captures.get(1))
                .map(|found| found.as_str().trim().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn element_text(text: &str, name: &str) -> Option<String> {
    element_pattern(name)?
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|found| found.as_str().trim().to_owned())
}

fn without_comments(text: &str) -> String {
    XML_COMMENT
        .as_ref()
        .map(|comment| comment.replace_all(text, "").into_owned())
        .unwrap_or_else(|| text.to_owned())
}

fn is_numeric_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn latest_build(root: &Path) -> Option<u32> {
    fs::read_dir(root.join("bin"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .filter(|name| is_numeric_name(name))
                .and_then(|name| name.parse::<u32>().ok())
        })
        .max()
}

fn game_info_text(root: &Path) -> Option<String> {
    read_text(&root.join(GAME_INFO_FILE))
}

fn installed_client_version(root: &Path) -> Option<String> {
    let text = game_info_text(root)?;
    let document = roxmltree::Document::parse(&text).ok()?;
    document
        .descendants()
        .find(|node| node.has_tag_name("version") && node.attribute("name") == Some("client"))
        .and_then(|node| node.attribute("installed"))
        .map(|installed| installed.trim().to_owned())
}

pub fn game_id(root: &Path) -> Option<String> {
    let text = game_info_text(root)?;
    let document = roxmltree::Document::parse(&text).ok()?;
    document
        .descendants()
        .find(|node| node.has_tag_name("id"))
        .and_then(|node| node.text())
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
}

pub fn is_public_test(root: &Path) -> bool {
    game_id(root).is_some_and(|id| id.to_ascii_uppercase().contains(PUBLIC_TEST_MARKER))
}

fn version_numbers(raw: &str) -> Option<Vec<u64>> {
    raw.trim()
        .split([',', '.'])
        .map(|part| part.trim().parse::<u64>().ok())
        .collect()
}

fn joined(numbers: &[u64], separator: &str) -> String {
    numbers
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(separator)
}

fn arena_version(raw: &str) -> Option<String> {
    match version_numbers(raw)?.as_slice() {
        [major, minor, patch, _, .., build] => Some(joined(&[*major, *minor, *patch, *build], ",")),
        exact @ [_, _, _, _] => Some(joined(exact, ",")),
        _ => None,
    }
}

fn release_folder_name(raw: &str) -> Option<String> {
    match version_numbers(raw)?.as_slice() {
        [major, minor, patch, hotfix, _, ..] => {
            Some(joined(&[*major, *minor, *patch, *hotfix], "."))
        }
        [major, minor, patch, _] => Some(joined(&[*major, *minor, *patch, 0], ".")),
        _ => None,
    }
}

fn last_server_version(root: &Path) -> Option<String> {
    read_text(&root.join(PREFERENCES_FILE))
        .and_then(|text| element_text(&text, "last_server_version"))
}

pub fn client_version(root: &Path) -> Option<String> {
    installed_client_version(root)
        .and_then(|raw| arena_version(&raw))
        .or_else(|| last_server_version(root).and_then(|raw| arena_version(&raw)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathBase {
    WorkingDirectory,
    Executable,
}

impl PathBase {
    fn parse(raw: &str) -> Option<PathBase> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "CWD" => Some(PathBase::WorkingDirectory),
            "EXE_PATH" => Some(PathBase::Executable),
            _ => None,
        }
    }
}

fn parse_flag(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ReplaysSettings {
    dir_path: Option<String>,
    path_base: Option<PathBase>,
    versioned: Option<bool>,
    enabled: Option<bool>,
}

impl ReplaysSettings {
    fn parse(text: &str) -> ReplaysSettings {
        element_text(&without_comments(text), "replays")
            .map(|block| ReplaysSettings {
                dir_path: element_text(&block, "dirPath").filter(|path| !path.is_empty()),
                path_base: element_text(&block, "pathBase")
                    .as_deref()
                    .and_then(PathBase::parse),
                versioned: element_text(&block, "versioned")
                    .as_deref()
                    .and_then(parse_flag),
                enabled: element_text(&block, "isEnabled")
                    .as_deref()
                    .and_then(parse_flag),
            })
            .unwrap_or_default()
    }

    fn read(path: &Path) -> ReplaysSettings {
        read_text(path)
            .map(|text| ReplaysSettings::parse(&text))
            .unwrap_or_default()
    }

    fn overlaid_with(self, overrides: ReplaysSettings) -> ReplaysSettings {
        ReplaysSettings {
            dir_path: overrides.dir_path.or(self.dir_path),
            path_base: overrides.path_base.or(self.path_base),
            versioned: overrides.versioned.or(self.versioned),
            enabled: overrides.enabled.or(self.enabled),
        }
    }
}

fn executable_dir(build_dir: &Path) -> PathBuf {
    ["bin64", "bin32"]
        .iter()
        .map(|name| build_dir.join(name))
        .find(|candidate| candidate.is_dir())
        .unwrap_or_else(|| build_dir.join("bin64"))
}

fn versioned_folder_name(root: &Path, client_version: Option<&str>) -> Option<String> {
    installed_client_version(root)
        .and_then(|raw| release_folder_name(&raw))
        .or_else(|| client_version.and_then(release_folder_name))
}

pub fn replays_location(root: &Path, build: u32, client_version: Option<&str>) -> ReplaysLocation {
    let build_dir = root.join("bin").join(build.to_string());
    let settings = ReplaysSettings::read(&build_dir.join("res").join(ENGINE_CONFIG_FILE))
        .overlaid_with(ReplaysSettings::read(
            &build_dir.join("res_mods").join(ENGINE_CONFIG_FILE),
        ));
    let base = match settings.path_base.unwrap_or(PathBase::WorkingDirectory) {
        PathBase::WorkingDirectory => root.to_path_buf(),
        PathBase::Executable => executable_dir(&build_dir),
    };
    let unversioned = base.join(settings.dir_path.as_deref().unwrap_or(DEFAULT_REPLAYS_DIR));
    let release_folder = settings
        .versioned
        .unwrap_or(false)
        .then(|| versioned_folder_name(root, client_version))
        .flatten();
    let dir = match release_folder {
        Some(folder) => unversioned.join(folder),
        None => unversioned,
    };
    ReplaysLocation {
        dir,
        enabled: settings.enabled.unwrap_or(true),
    }
}

fn region_from_server_name(raw: &str) -> Option<Region> {
    let lowered = raw.trim().to_lowercase();
    let name = lowered
        .strip_prefix(SERVER_NAME_PREFIX)
        .unwrap_or(&lowered)
        .trim();
    match name {
        "eu" => Some(Region::Eu),
        "na" => Some(Region::Na),
        "asia" => Some(Region::Asia),
        _ => None,
    }
}

pub fn active_region(root: &Path) -> Option<Region> {
    read_text(&root.join(PREFERENCES_FILE))
        .and_then(|text| element_text(&text, "active_server"))
        .and_then(|server| region_from_server_name(&server))
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    const BUILD: u32 = 13533378;
    const GAME_INFO: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<protocol name="game_info" version="2.16" wgc_publisher_id="wargaming">
    <game>
        <id>WOWS.WW.PRODUCTION</id>
        <part_versions>
            <version name="locale" installed="15.10.0.0.13567617" available="15.10.0.0.13567617"/>
            <version name="client" installed="15.10.0.0.13533378" available="15.10.0.0.13533378"/>
        </part_versions>
    </game>
</protocol>
"#;

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().expect("file has a parent")).expect("create parent");
        fs::write(path, contents).expect("write file");
    }

    fn build_dir(root: &Path) -> PathBuf {
        root.join("bin").join(BUILD.to_string())
    }

    fn engine_config(replays: &str) -> String {
        format!(
            "<engine_config.xml>\n  <renderer>\n    <tripleBuffering>true</tripleBuffering>\n  </renderer>\n  <replays>\n{replays}\n  </replays>\n</engine_config.xml>\n"
        )
    }

    fn write_res_config(root: &Path, replays: &str) {
        write(
            &build_dir(root).join("res").join(ENGINE_CONFIG_FILE),
            &engine_config(replays),
        );
    }

    fn write_mods_config(root: &Path, replays: &str) {
        write(
            &build_dir(root).join("res_mods").join(ENGINE_CONFIG_FILE),
            &engine_config(replays),
        );
    }

    fn write_preferences(root: &Path, body: &str) {
        write(
            &root.join(PREFERENCES_FILE),
            &format!(
                "<preferences.xml>\n\t<scriptsPreferences>\n{body}\n\t</scriptsPreferences>\n</preferences.xml>\n"
            ),
        );
    }

    fn fake_root() -> TempDir {
        let dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(build_dir(dir.path())).expect("create build dir");
        dir
    }

    #[test]
    fn latest_build_is_the_highest_numeric_bin_folder() {
        let dir = fake_root();
        let bin = dir.path().join("bin");
        fs::create_dir_all(bin.join("13357625")).expect("older build");
        fs::create_dir_all(bin.join("clientrunner")).expect("runner");
        fs::create_dir_all(bin.join("99999999999")).expect("out of range");
        write(&bin.join("20000000"), "a file, not a build");

        assert_eq!(latest_build(dir.path()), Some(BUILD));
    }

    #[test]
    fn latest_build_is_none_without_a_bin_folder() {
        let dir = TempDir::new().expect("temp dir");

        assert_eq!(latest_build(dir.path()), None);
    }

    #[test]
    fn client_version_reads_the_installed_client_part_in_arena_form() {
        let dir = fake_root();
        write(&dir.path().join(GAME_INFO_FILE), GAME_INFO);

        assert_eq!(
            client_version(dir.path()).as_deref(),
            Some("15,10,0,13533378")
        );
    }

    #[test]
    fn client_version_falls_back_to_the_last_server_version() {
        let dir = fake_root();
        write_preferences(
            dir.path(),
            "\t\t<last_server_version>\t15,10,0,13533378\t</last_server_version>",
        );

        assert_eq!(
            client_version(dir.path()).as_deref(),
            Some("15,10,0,13533378")
        );
    }

    #[test]
    fn client_version_is_none_when_the_files_are_missing_or_garbled() {
        let dir = fake_root();
        assert_eq!(client_version(dir.path()), None);

        write(&dir.path().join(GAME_INFO_FILE), "<protocol><game>");
        write_preferences(
            dir.path(),
            "<last_server_version>soon</last_server_version>",
        );
        assert_eq!(client_version(dir.path()), None);
    }

    #[test]
    fn replays_default_to_the_root_replays_folder_when_configs_are_missing() {
        let dir = fake_root();

        assert_eq!(
            replays_location(dir.path(), BUILD, None),
            ReplaysLocation {
                dir: dir.path().join("replays"),
                enabled: true,
            }
        );
    }

    #[test]
    fn replays_with_a_cwd_base_live_under_the_install_root() {
        let dir = fake_root();
        write_res_config(
            dir.path(),
            "<dirPath>replays</dirPath><pathBase>CWD</pathBase><versioned>false</versioned><maxReplaysToSave>30</maxReplaysToSave>",
        );

        assert_eq!(
            replays_location(dir.path(), BUILD, Some("15,10,0,13533378")),
            ReplaysLocation {
                dir: dir.path().join("replays"),
                enabled: true,
            }
        );
    }

    #[test]
    fn replays_with_an_exe_path_base_prefer_bin64_then_bin32() {
        let dir = fake_root();
        write_res_config(
            dir.path(),
            "<dirPath>replays</dirPath><pathBase>exe_path</pathBase>",
        );
        fs::create_dir_all(build_dir(dir.path()).join("bin32")).expect("bin32");

        assert_eq!(
            replays_location(dir.path(), BUILD, None).dir,
            build_dir(dir.path()).join("bin32").join("replays")
        );

        fs::create_dir_all(build_dir(dir.path()).join("bin64")).expect("bin64");

        assert_eq!(
            replays_location(dir.path(), BUILD, None).dir,
            build_dir(dir.path()).join("bin64").join("replays")
        );
    }

    #[test]
    fn res_mods_overrides_the_res_config_field_by_field() {
        let dir = fake_root();
        write_res_config(
            dir.path(),
            "<dirPath>replays</dirPath><pathBase>CWD</pathBase><versioned>false</versioned>",
        );
        write_mods_config(dir.path(), "<dirPath>my_replays</dirPath>");

        assert_eq!(
            replays_location(dir.path(), BUILD, None),
            ReplaysLocation {
                dir: dir.path().join("my_replays"),
                enabled: true,
            }
        );

        write_mods_config(dir.path(), "<isEnabled>false</isEnabled>");

        assert_eq!(
            replays_location(dir.path(), BUILD, None),
            ReplaysLocation {
                dir: dir.path().join("replays"),
                enabled: false,
            }
        );
    }

    #[test]
    fn versioned_replays_use_the_release_folder_from_game_info() {
        let dir = fake_root();
        write(&dir.path().join(GAME_INFO_FILE), GAME_INFO);
        write_res_config(
            dir.path(),
            "<dirPath>replays</dirPath><pathBase>CWD</pathBase><versioned>false</versioned>",
        );
        write_mods_config(
            dir.path(),
            "<dirPath>replays</dirPath><pathBase>CWD</pathBase><versioned>true</versioned><isEnabled>true</isEnabled>",
        );

        assert_eq!(
            replays_location(dir.path(), BUILD, Some("15,10,0,13533378")),
            ReplaysLocation {
                dir: dir.path().join("replays").join("15.10.0.0"),
                enabled: true,
            }
        );
    }

    #[test]
    fn versioned_replays_fall_back_to_the_given_client_version() {
        let dir = fake_root();
        write_res_config(dir.path(), "<versioned>TRUE</versioned>");

        assert_eq!(
            replays_location(dir.path(), BUILD, Some("15,10,0,13533378")).dir,
            dir.path().join("replays").join("15.10.0.0")
        );
        assert_eq!(
            replays_location(dir.path(), BUILD, Some("15.9.1.0.13357625")).dir,
            dir.path().join("replays").join("15.9.1.0")
        );
        assert_eq!(
            replays_location(dir.path(), BUILD, None).dir,
            dir.path().join("replays")
        );
    }

    #[test]
    fn disabled_replays_are_reported_as_disabled() {
        let dir = fake_root();
        write_res_config(
            dir.path(),
            "<dirPath>replays</dirPath><pathBase>CWD</pathBase><isEnabled>false</isEnabled>",
        );

        assert_eq!(
            replays_location(dir.path(), BUILD, None),
            ReplaysLocation {
                dir: dir.path().join("replays"),
                enabled: false,
            }
        );
    }

    #[test]
    fn commented_out_replays_settings_are_ignored() {
        let dir = fake_root();
        write(
            &build_dir(dir.path()).join("res").join(ENGINE_CONFIG_FILE),
            "<engine_config.xml>\n<!-- <replays><dirPath>old</dirPath><isEnabled>false</isEnabled></replays> -->\n<replays><dirPath>kept</dirPath></replays>\n</engine_config.xml>",
        );

        assert_eq!(
            replays_location(dir.path(), BUILD, None),
            ReplaysLocation {
                dir: dir.path().join("kept"),
                enabled: true,
            }
        );
    }

    #[test]
    fn active_region_reads_a_tab_padded_server_name() {
        let dir = fake_root();
        write_preferences(
            dir.path(),
            "\t\t<active_server>\tWOWS NA\t</active_server>\n\t\t<broken attr=>&&<<\n",
        );

        assert_eq!(active_region(dir.path()), Some(Region::Na));
    }

    #[test]
    fn active_region_maps_every_supported_server() {
        assert_eq!(region_from_server_name("WOWS EU"), Some(Region::Eu));
        assert_eq!(region_from_server_name(" wows asia "), Some(Region::Asia));
        assert_eq!(region_from_server_name("NA"), Some(Region::Na));
    }

    #[test]
    fn active_region_is_none_for_unsupported_or_missing_servers() {
        let dir = fake_root();
        assert_eq!(active_region(dir.path()), None);

        write_preferences(dir.path(), "<active_server>\tWOWS RU\t</active_server>");
        assert_eq!(active_region(dir.path()), None);

        write_preferences(dir.path(), "<active_server>WOWS CIS</active_server>");
        assert_eq!(active_region(dir.path()), None);

        write_preferences(dir.path(), "<active_server></active_server>");
        assert_eq!(active_region(dir.path()), None);

        write_preferences(dir.path(), "<other_setting>WOWS NA</other_setting>");
        assert_eq!(active_region(dir.path()), None);
    }

    #[test]
    fn public_test_installs_are_recognised_by_their_game_id() {
        let dir = fake_root();
        write(&dir.path().join(GAME_INFO_FILE), GAME_INFO);
        assert!(!is_public_test(dir.path()));

        write(
            &dir.path().join(GAME_INFO_FILE),
            &GAME_INFO.replace("WOWS.WW.PRODUCTION", "WOWS.PT.PRODUCTION"),
        );
        assert!(is_public_test(dir.path()));
    }
}
