import { resolveTheme } from "./appearance";
import { backend } from "./backend";
import type { SetupAudience } from "./intro";
import type { AppInfo, Settings, Snapshot } from "./types";

export type View = "monitor" | "settings";

export type SaveSettings = (settings: Settings) => Promise<Settings>;

export class SettingsStore {
  #current = $state<Settings | null>(null);
  #saveError = $state<string | null>(null);
  #confirmed: Settings | null = null;
  #confirmedRequest = 0;
  #latestRequest = 0;
  #latestFailed = false;
  readonly #save: SaveSettings;

  constructor(save: SaveSettings) {
    this.#save = save;
  }

  get current(): Settings | null {
    return this.#current;
  }

  get saveError(): string | null {
    return this.#saveError;
  }

  load(settings: Settings): void {
    this.#current = settings;
    this.#confirmed = settings;
  }

  async update(change: Partial<Settings>): Promise<boolean> {
    if (!this.#current) {
      return false;
    }
    const request = this.#latestRequest + 1;
    const next: Settings = { ...this.#current, ...change };
    this.#latestRequest = request;
    this.#latestFailed = false;
    this.#current = next;
    try {
      this.#confirm(request, await this.#save(next));
      return true;
    } catch (error) {
      this.#reject(request, String(error));
      return false;
    }
  }

  #confirm(request: number, saved: Settings): void {
    if (request < this.#confirmedRequest) {
      return;
    }
    this.#confirmed = saved;
    this.#confirmedRequest = request;
    if (request === this.#latestRequest) {
      this.#current = saved;
      this.#saveError = null;
    } else if (this.#latestFailed) {
      this.#current = saved;
    }
  }

  #reject(request: number, message: string): void {
    if (request !== this.#latestRequest) {
      return;
    }
    this.#latestFailed = true;
    this.#current = this.#confirmed;
    this.#saveError = message;
  }
}

class AppStore {
  snapshot = $state<Snapshot | null>(null);
  info = $state<AppInfo | null>(null);
  view = $state<View>("monitor");
  systemDark = $state(true);
  setup = $state<SetupAudience | null>(null);
  readonly #settings = new SettingsStore(backend.saveSettings);

  get settings(): Settings | null {
    return this.#settings.current;
  }

  get saveError(): string | null {
    return this.#settings.saveError;
  }

  get theme(): "dark" | "light" {
    return resolveTheme(this.settings?.theme ?? "system", this.systemDark);
  }

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
    this.#settings.load(settings);
    this.info = info;
    return unlisten;
  }

  update(change: Partial<Settings>): Promise<boolean> {
    return this.#settings.update(change);
  }
}

export const app = new AppStore();
