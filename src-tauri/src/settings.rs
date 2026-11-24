use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::Region;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextSize {
    Small,
    #[default]
    Normal,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Palette {
    #[default]
    Muted,
    Vibrant,
    Friendly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Highlight {
    None,
    #[default]
    Edge,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HighlightStat {
    #[default]
    PersonalRating,
    ShipWinRate,
    AccountWinRate,
    AverageDamage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TeamAverage {
    #[default]
    Mean,
    Weighted,
    Median,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub install_path: Option<PathBuf>,
    pub replays_dir_override: Option<PathBuf>,
    pub region_override: Option<Region>,
    pub application_id: Option<String>,
    pub theme: Theme,
    pub density: Density,
    pub text_size: TextSize,
    pub palette: Palette,
    pub colored_values: bool,
    pub highlight: Highlight,
    pub highlight_stat: HighlightStat,
    pub team_average: TeamAverage,
    pub solo_stats: bool,
    pub show_clan_tags: bool,
    pub always_on_top: bool,
    pub close_to_tray: bool,
    pub check_for_updates: bool,
    #[serde(default)]
    pub intro_seen: Option<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            install_path: None,
            replays_dir_override: None,
            region_override: None,
            application_id: None,
            theme: Theme::System,
            density: Density::Comfortable,
            text_size: TextSize::Normal,
            palette: Palette::Muted,
            colored_values: true,
            highlight: Highlight::Edge,
            highlight_stat: HighlightStat::PersonalRating,
            team_average: TeamAverage::Mean,
            solo_stats: false,
            show_clan_tags: true,
            always_on_top: false,
            close_to_tray: false,
            check_for_updates: true,
            intro_seen: None,
        }
    }
}

impl Settings {
    pub fn effective_application_id(&self) -> Option<String> {
        self.own_application_id()
            .map(str::to_owned)
            .or_else(|| bundled_application_id().map(str::to_owned))
    }

    pub fn has_own_application_id(&self) -> bool {
        self.own_application_id().is_some()
    }

    fn own_application_id(&self) -> Option<&str> {
        self.application_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
    }
}

pub fn bundled_application_id() -> Option<&'static str> {
    option_env!("CROWSNEST_WG_APPLICATION_ID")
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("could not read settings: {0}")]
    Read(#[source] std::io::Error),
    #[error("could not write settings: {0}")]
    Write(#[source] std::io::Error),
    #[error("settings file is not valid JSON: {0}")]
    Parse(#[source] serde_json::Error),
}

pub struct SettingsFile {
    path: PathBuf,
}

impl SettingsFile {
    pub fn in_dir(dir: &Path) -> SettingsFile {
        SettingsFile {
            path: dir.join("settings.json"),
        }
    }

    pub fn load(&self) -> Result<Settings, SettingsError> {
        match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(SettingsError::Parse),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(error) => Err(SettingsError::Read(error)),
        }
    }

    pub fn load_or_default(&self) -> Settings {
        self.load().unwrap_or_else(|error| {
            log::warn!("{error}; starting from default settings");
            Settings::default()
        })
    }

    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        let bytes = serde_json::to_vec_pretty(settings).map_err(SettingsError::Parse)?;
        write_atomically(&self.path, &bytes).map_err(SettingsError::Write)
    }
}

pub fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let staging = path.with_extension("tmp");
    let mut file = fs::File::create(&staging)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&staging, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_loads_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = SettingsFile::in_dir(dir.path()).load().unwrap();
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn saved_settings_load_back_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let file = SettingsFile::in_dir(dir.path());
        let settings = Settings {
            install_path: Some(PathBuf::from("C:/Games/World_of_Warships")),
            region_override: Some(Region::Eu),
            density: Density::Compact,
            ..Settings::default()
        };
        file.save(&settings).unwrap();
        assert_eq!(file.load().unwrap(), settings);
    }

    #[test]
    fn unknown_and_missing_keys_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            br#"{"density":"compact","someFutureKey":1}"#,
        )
        .unwrap();
        let loaded = SettingsFile::in_dir(dir.path()).load().unwrap();
        assert_eq!(loaded.density, Density::Compact);
        assert!(loaded.check_for_updates);
        assert_eq!(loaded.palette, Palette::Muted);
        assert_eq!(loaded.intro_seen, None);
    }

    #[test]
    fn a_fresh_install_has_seen_no_release() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = SettingsFile::in_dir(dir.path()).load().unwrap();
        assert_eq!(loaded.intro_seen, None);
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn the_palette_and_intro_marker_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let file = SettingsFile::in_dir(dir.path());
        let settings = Settings {
            palette: Palette::Friendly,
            intro_seen: Some(1),
            ..Settings::default()
        };
        file.save(&settings).unwrap();
        let text = fs::read_to_string(dir.path().join("settings.json")).unwrap();
        assert!(text.contains("\"palette\": \"friendly\""));
        assert!(text.contains("\"introSeen\": 1"));
        assert_eq!(file.load().unwrap(), settings);
    }

    #[test]
    fn a_blank_user_application_id_is_ignored() {
        let settings = Settings {
            application_id: Some("   ".into()),
            ..Settings::default()
        };
        assert_eq!(
            settings.effective_application_id().as_deref(),
            bundled_application_id()
        );
        assert!(!settings.has_own_application_id());
    }

    #[test]
    fn a_user_application_id_counts_as_their_own() {
        let settings = Settings {
            application_id: Some(" 0123456789abcdef0123456789abcdef ".into()),
            ..Settings::default()
        };
        assert!(settings.has_own_application_id());
        assert_eq!(
            settings.effective_application_id().as_deref(),
            Some("0123456789abcdef0123456789abcdef")
        );
    }
}
