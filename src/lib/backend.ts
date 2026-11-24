import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppInfo, GameInstall, Region, Settings, Snapshot } from "./types";

export const SNAPSHOT_EVENT = "snapshot";

export const backend = {
  snapshot: () => invoke<Snapshot>("snapshot"),
  settings: () => invoke<Settings>("settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  detectInstalls: () => invoke<GameInstall[]>("detect_installs"),
  inspectInstall: (path: string) => invoke<GameInstall>("inspect_install", { path }),
  verifyApplicationId: (applicationId: string, region: Region) =>
    invoke<void>("verify_application_id", { applicationId, region }),
  refresh: () => invoke<void>("refresh"),
  appInfo: () => invoke<AppInfo>("app_info"),
  openLogFolder: () => invoke<void>("open_log_folder"),
  onSnapshot: (handler: (snapshot: Snapshot) => void): Promise<UnlistenFn> =>
    listen<Snapshot>(SNAPSHOT_EVENT, (event) => handler(event.payload)),
};
