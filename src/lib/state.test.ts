import { describe, expect, it, vi } from "vitest";
import { SettingsStore, app } from "./state.svelte";
import type { Settings } from "./types";

const backend = vi.hoisted(() => ({
  saveSettings: vi.fn(),
  onSnapshot: vi.fn(),
  snapshot: vi.fn(),
  settings: vi.fn(),
  appInfo: vi.fn(),
}));

vi.mock("./backend", () => ({ backend }));

const initial: Settings = {
  installPath: null,
  replaysDirOverride: null,
  regionOverride: null,
  applicationId: null,
  theme: "system",
  density: "comfortable",
  textSize: "normal",
  palette: "muted",
  coloredValues: true,
  highlight: "edge",
  highlightStat: "personalRating",
  teamAverage: "mean",
  soloStats: false,
  showClanTags: true,
  alwaysOnTop: false,
  closeToTray: false,
  checkForUpdates: true,
  introSeen: 0,
};

interface SaveRequest {
  settings: Settings;
  resolve: (saved: Settings) => void;
  reject: (error: unknown) => void;
}

function mockedBackend() {
  const requests: SaveRequest[] = [];
  const save = vi.fn(
    (settings: Settings) =>
      new Promise<Settings>((resolve, reject) => {
        requests.push({ settings, resolve, reject });
      }),
  );
  const request = (index: number): SaveRequest => {
    const found = requests[index];
    if (!found) {
      throw new Error(`No save request ${index}`);
    }
    return found;
  };
  return { save, request };
}

function loadedStore(save: (settings: Settings) => Promise<Settings>): SettingsStore {
  const store = new SettingsStore(save);
  store.load(initial);
  return store;
}

describe("SettingsStore", () => {
  it("shows a change at once, then what the backend saved", async () => {
    const { save, request } = mockedBackend();
    const store = loadedStore(save);
    const saving = store.update({ applicationId: "ABC" });
    expect(store.current?.applicationId).toBe("ABC");
    request(0).resolve({ ...initial, applicationId: "abc" });
    await expect(saving).resolves.toBe(true);
    expect(store.current?.applicationId).toBe("abc");
    expect(store.saveError).toBeNull();
  });

  it("rolls a failed change back to the last saved settings", async () => {
    const { save, request } = mockedBackend();
    const store = loadedStore(save);
    const dark = { ...initial, theme: "dark" } as const;
    const first = store.update({ theme: "dark" });
    request(0).resolve(dark);
    await first;
    const second = store.update({ density: "compact" });
    request(1).reject("disk full");
    await expect(second).resolves.toBe(false);
    expect(store.current).toEqual(dark);
    expect(store.saveError).toBe("disk full");
  });

  it("rolls back past a change that is still unconfirmed", async () => {
    const { save, request } = mockedBackend();
    const store = loadedStore(save);
    const first = store.update({ theme: "dark" });
    const second = store.update({ density: "compact" });
    expect(request(1).settings).toMatchObject({ theme: "dark", density: "compact" });
    request(1).reject("disk full");
    await second;
    expect(store.current).toEqual(initial);
    expect(store.saveError).toBe("disk full");
    request(0).resolve({ ...initial, theme: "dark" });
    await first;
    expect(store.current).toEqual({ ...initial, theme: "dark" });
    expect(store.saveError).toBe("disk full");
  });

  it("ignores a success that arrives after a newer request", async () => {
    const { save, request } = mockedBackend();
    const store = loadedStore(save);
    const first = store.update({ theme: "dark" });
    const second = store.update({ density: "compact" });
    const newest = { ...initial, theme: "dark", density: "compact" } as const;
    request(1).resolve(newest);
    await second;
    request(0).resolve({ ...initial, theme: "dark" });
    await first;
    expect(store.current).toEqual(newest);
  });

  it("ignores a failure that arrives after a newer request", async () => {
    const { save, request } = mockedBackend();
    const store = loadedStore(save);
    const first = store.update({ theme: "dark" });
    const second = store.update({ density: "compact" });
    const newest = { ...initial, theme: "dark", density: "compact" } as const;
    request(1).resolve(newest);
    await second;
    request(0).reject("disk full");
    await first;
    expect(store.current).toEqual(newest);
    expect(store.saveError).toBeNull();
  });

  it("clears the error once a later change is saved", async () => {
    const { save, request } = mockedBackend();
    const store = loadedStore(save);
    const failed = store.update({ theme: "dark" });
    request(0).reject("disk full");
    await failed;
    const saved = store.update({ theme: "light" });
    request(1).resolve({ ...initial, theme: "light" });
    await saved;
    expect(store.current?.theme).toBe("light");
    expect(store.saveError).toBeNull();
  });

  it("saves nothing before settings load", async () => {
    const { save } = mockedBackend();
    const store = new SettingsStore(save);
    await expect(store.update({ theme: "dark" })).resolves.toBe(false);
    expect(save).not.toHaveBeenCalled();
    expect(store.current).toBeNull();
  });
});

describe("app", () => {
  it("saves settings through the backend after start", async () => {
    backend.onSnapshot.mockResolvedValue(() => undefined);
    backend.snapshot.mockResolvedValue({ phase: "waiting", install: null, battle: null, notice: null });
    backend.settings.mockResolvedValue(initial);
    backend.appInfo.mockResolvedValue({ version: "0.1.0", hasBundledApplicationId: true });
    backend.saveSettings.mockRejectedValue("disk full");
    await app.start();
    await expect(app.update({ theme: "dark" })).resolves.toBe(false);
    expect(backend.saveSettings).toHaveBeenCalledWith({ ...initial, theme: "dark" });
    expect(app.settings).toEqual(initial);
    expect(app.saveError).toBe("disk full");
  });
});
