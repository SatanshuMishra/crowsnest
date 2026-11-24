import { backend } from "./backend";
import type { AppInfo, Settings, Snapshot } from "./types";

export type View = "monitor" | "settings";

class AppStore {
  snapshot = $state<Snapshot | null>(null);
  settings = $state<Settings | null>(null);
  info = $state<AppInfo | null>(null);
  view = $state<View>("monitor");
  saveError = $state<string | null>(null);

  async start(): Promise<() => void> {
    const unlisten = await backend.onSnapshot((next) => {
      this.snapshot = next;
    });
    const [snapshot, settings, info] = await Promise.all([
      backend.snapshot(),
      backend.settings(),
      backend.appInfo(),
    ]);
    this.snapshot = this.snapshot ?? snapshot;
    this.settings = settings;
    this.info = info;
    return unlisten;
  }

  async update(change: Partial<Settings>): Promise<boolean> {
    if (!this.settings) {
      return false;
    }
    const next: Settings = { ...this.settings, ...change };
    const previous = this.settings;
    this.settings = next;
    try {
      this.settings = await backend.saveSettings(next);
      this.saveError = null;
      return true;
    } catch (error) {
      this.settings = previous;
      this.saveError = String(error);
      return false;
    }
  }
}

export const app = new AppStore();
