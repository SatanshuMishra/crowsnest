use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::AppState;
use crate::game::install;
use crate::model::{GameInstall, InstallSource, Region, Snapshot};
use crate::settings::{Settings, bundled_application_id};
use crate::shell;
use crate::wg::client::WgClient;

#[tauri::command]
pub fn snapshot(state: State<'_, AppState>) -> Snapshot {
    state.monitor.snapshot()
}

#[tauri::command]
pub fn settings(state: State<'_, AppState>) -> Settings {
    state.monitor.settings()
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    state
        .settings_file
        .save(&settings)
        .map_err(|error| error.to_string())?;
    shell::apply_window_preferences(&app, &settings);
    state.monitor.reconfigure(settings.clone());
    Ok(settings)
}

#[tauri::command]
pub async fn detect_installs() -> Vec<GameInstall> {
    tauri::async_runtime::spawn_blocking(install::detect)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn inspect_install(path: String) -> Result<GameInstall, &'static str> {
    let root = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || install::inspect(&root, InstallSource::Manual))
        .await
        .map_err(|_| "unavailable")?
        .map_err(|error| install_problem(&error))
}

fn install_problem(error: &install::InstallError) -> &'static str {
    match error {
        install::InstallError::Missing => "missing",
        install::InstallError::NotAGameFolder => "notAGameFolder",
        install::InstallError::NoBuild => "noBuild",
    }
}

#[tauri::command]
pub async fn verify_application_id(application_id: String, region: Region) -> Result<(), String> {
    let client = WgClient::new(
        crate::monitor::http_client(),
        region,
        application_id.trim().to_owned(),
    );
    client
        .verify_application_id()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn refresh(state: State<'_, AppState>) {
    state.monitor.refresh();
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub has_bundled_application_id: bool,
}

#[tauri::command]
pub fn app_info(app: AppHandle) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        has_bundled_application_id: bundled_application_id().is_some(),
    }
}

#[tauri::command]
pub fn open_log_folder(app: AppHandle) -> Result<(), String> {
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
}
