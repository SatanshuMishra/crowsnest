mod arena;
mod body;
mod commands;
mod game;
mod model;
mod monitor;
mod rating;
mod settings;
mod shell;
mod wg;

use tauri::Manager;

use crate::monitor::Monitor;
use crate::settings::SettingsFile;

pub struct AppState {
    pub monitor: Monitor,
    pub settings_file: SettingsFile,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            shell::show_main_window(app);
        }))
        .plugin(shell::log_plugin())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(shell::persisted_window_state())
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let cache_dir = app.path().app_cache_dir()?;
            let settings_file = SettingsFile::in_dir(&config_dir);
            let settings = settings_file.load_or_default();
            let monitor = Monitor::new(app.handle().clone(), cache_dir, settings.clone());
            app.manage(AppState {
                monitor: monitor.clone(),
                settings_file,
            });
            shell::install_tray(app.handle())?;
            shell::apply_window_preferences(app.handle(), &settings);
            monitor.start();
            shell::show_main_window(app.handle());
            Ok(())
        })
        .on_window_event(shell::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::settings,
            commands::save_settings,
            commands::detect_installs,
            commands::inspect_install,
            commands::verify_application_id,
            commands::refresh,
            commands::app_info,
            commands::open_log_folder,
        ])
        .run(tauri::generate_context!())
        .expect("Crowsnest failed to start");
}
