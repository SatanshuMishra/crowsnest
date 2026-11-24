import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { DownloadEvent } from "@tauri-apps/plugin-updater";
import {
  AUTOMATIC_CHECK_DELAY_MS,
  CHECK_FAILED_MESSAGE,
  IDLE,
  INSTALL_FAILED_MESSAGE,
  RESTART_FAILED_MESSAGE,
  UpdateStore,
  reduceUpdate,
  updateAnnouncement,
  type UpdateState,
} from "./updates.svelte";

const plugins = vi.hoisted(() => ({
  check: vi.fn(),
  relaunch: vi.fn(),
  settings: { checkForUpdates: true },
}));

vi.mock("@tauri-apps/plugin-updater", () => ({ check: plugins.check }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: plugins.relaunch }));
vi.mock("./state.svelte", () => ({ app: { settings: plugins.settings } }));

const available: UpdateState = { status: "available", version: "1.4.0", notes: "Faster tables", error: null };
const downloading: UpdateState = {
  status: "downloading",
  version: "1.4.0",
  notes: "Faster tables",
  received: 0,
  total: null,
  percent: null,
};

function progress(state: UpdateState, ...events: DownloadEvent[]): UpdateState {
  return events.reduce((current, event) => reduceUpdate(current, { type: "downloadProgress", event }), state);
}

describe("reduceUpdate", () => {
  it("starts a check from idle, up to date and error", () => {
    for (const state of [IDLE, { status: "upToDate" }, { status: "error", message: "x" }] as UpdateState[]) {
      expect(reduceUpdate(state, { type: "checkStarted", silent: false })).toEqual({ status: "checking", silent: false });
    }
  });

  it("makes a silent check visible when the user asks while it runs", () => {
    const silent = reduceUpdate(IDLE, { type: "checkStarted", silent: true });
    expect(reduceUpdate(silent, { type: "checkStarted", silent: false })).toEqual({ status: "checking", silent: false });
  });

  it("does not restart a check during a download or once ready", () => {
    const ready: UpdateState = { status: "ready", version: "1.4.0", error: null };
    expect(reduceUpdate(downloading, { type: "checkStarted", silent: false })).toBe(downloading);
    expect(reduceUpdate(ready, { type: "checkStarted", silent: false })).toBe(ready);
  });

  it("offers a found update with its version and notes", () => {
    const checking = reduceUpdate(IDLE, { type: "checkStarted", silent: true });
    const offer = { version: "1.4.0", notes: "Faster tables" };
    expect(reduceUpdate(checking, { type: "checkFinished", offer })).toEqual(available);
  });

  it("reports up to date only for a check the user asked for", () => {
    const asked = reduceUpdate(IDLE, { type: "checkStarted", silent: false });
    const automatic = reduceUpdate(IDLE, { type: "checkStarted", silent: true });
    expect(reduceUpdate(asked, { type: "checkFinished", offer: null })).toEqual({ status: "upToDate" });
    expect(reduceUpdate(automatic, { type: "checkFinished", offer: null })).toEqual(IDLE);
  });

  it("shows a failed check only when the user asked for it", () => {
    const asked = reduceUpdate(IDLE, { type: "checkStarted", silent: false });
    const automatic = reduceUpdate(IDLE, { type: "checkStarted", silent: true });
    const failed = { type: "checkFailed", message: "offline" } as const;
    expect(reduceUpdate(asked, failed)).toEqual({ status: "error", message: "offline" });
    expect(reduceUpdate(automatic, failed)).toEqual(IDLE);
  });

  it("ignores check results that arrive outside a check", () => {
    expect(reduceUpdate(IDLE, { type: "checkFinished", offer: null })).toBe(IDLE);
    expect(reduceUpdate(available, { type: "checkFailed", message: "x" })).toBe(available);
  });

  it("starts downloading only from an available update", () => {
    expect(reduceUpdate(available, { type: "downloadStarted" })).toEqual(downloading);
    expect(reduceUpdate(IDLE, { type: "downloadStarted" })).toBe(IDLE);
  });

  it("computes a determinate percent from the content length and chunks", () => {
    const started = progress(downloading, { event: "Started", data: { contentLength: 200 } });
    expect(started).toMatchObject({ total: 200, received: 0, percent: 0 });
    const quarter = progress(started, { event: "Progress", data: { chunkLength: 50 } });
    expect(quarter).toMatchObject({ received: 50, percent: 25 });
    const most = progress(quarter, { event: "Progress", data: { chunkLength: 149 } });
    expect(most).toMatchObject({ received: 199, percent: 99 });
    const over = progress(most, { event: "Progress", data: { chunkLength: 20 } });
    expect(over).toMatchObject({ percent: 100 });
  });

  it("stays indeterminate when the content length is unknown", () => {
    for (const data of [{}, { contentLength: 0 }]) {
      const state = progress(
        downloading,
        { event: "Started", data },
        { event: "Progress", data: { chunkLength: 4096 } },
        { event: "Finished" },
      );
      expect(state).toMatchObject({ status: "downloading", received: 4096, total: null, percent: null });
    }
  });

  it("completes a known download at 100 percent when it finishes", () => {
    const state = progress(
      downloading,
      { event: "Started", data: { contentLength: 1000 } },
      { event: "Progress", data: { chunkLength: 300 } },
      { event: "Finished" },
    );
    expect(state).toMatchObject({ received: 1000, percent: 100 });
  });

  it("ignores progress outside a download", () => {
    expect(progress(available, { event: "Progress", data: { chunkLength: 1 } })).toBe(available);
  });

  it("becomes ready after installing and returns to available with the error when it fails", () => {
    expect(reduceUpdate(downloading, { type: "installed" })).toEqual({ status: "ready", version: "1.4.0", error: null });
    expect(reduceUpdate(downloading, { type: "installFailed", message: "disk full" })).toEqual({
      ...available,
      error: "disk full",
    });
  });

  it("keeps the ready state and records a failed restart", () => {
    const ready: UpdateState = { status: "ready", version: "1.4.0", error: null };
    expect(reduceUpdate(ready, { type: "restartFailed", message: "denied" })).toEqual({ ...ready, error: "denied" });
    expect(reduceUpdate(IDLE, { type: "restartFailed", message: "denied" })).toBe(IDLE);
  });

  it("dismisses an offer or a result but never work in progress", () => {
    expect(reduceUpdate(available, { type: "dismissed" })).toEqual(IDLE);
    expect(reduceUpdate({ status: "upToDate" }, { type: "dismissed" })).toEqual(IDLE);
    expect(reduceUpdate(downloading, { type: "dismissed" })).toBe(downloading);
  });
});

describe("updateAnnouncement", () => {
  it("announces an offer, the start of a download, and the installed update", () => {
    expect(updateAnnouncement(available)).toBe("Crowsnest 1.4.0 is available.");
    expect(updateAnnouncement(downloading)).toBe("Downloading Crowsnest 1.4.0.");
    expect(updateAnnouncement({ status: "ready", version: "1.4.0", error: null })).toBe(
      "Crowsnest 1.4.0 is installed. Restart to update.",
    );
  });

  it("announces a failed install and a failed restart", () => {
    const failed = reduceUpdate(downloading, { type: "installFailed", message: INSTALL_FAILED_MESSAGE });
    expect(updateAnnouncement(failed)).toBe("The update could not be installed. Try again.");
    const stuck: UpdateState = { status: "ready", version: "1.4.0", error: RESTART_FAILED_MESSAGE };
    expect(updateAnnouncement(stuck)).toBe(RESTART_FAILED_MESSAGE);
  });

  it("stays the same while a download makes progress", () => {
    const started = progress(downloading, { event: "Started", data: { contentLength: 1000 } });
    const halfway = progress(started, { event: "Progress", data: { chunkLength: 500 } });
    const finished = progress(halfway, { event: "Finished" });
    expect(new Set([downloading, started, halfway, finished].map(updateAnnouncement))).toEqual(
      new Set(["Downloading Crowsnest 1.4.0."]),
    );
  });

  it("says nothing outside an offer", () => {
    const quiet: UpdateState[] = [IDLE, { status: "checking", silent: false }, { status: "upToDate" }];
    expect(quiet.map(updateAnnouncement)).toEqual(["", "", ""]);
    expect(updateAnnouncement({ status: "error", message: CHECK_FAILED_MESSAGE })).toBe("");
  });
});

describe("UpdateStore", () => {
  beforeEach(() => {
    plugins.check.mockReset();
    plugins.relaunch.mockReset();
    plugins.settings.checkForUpdates = true;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function fakeUpdate(downloadAndInstall = vi.fn().mockResolvedValue(undefined)) {
    return {
      version: "1.4.0",
      body: "  Faster tables\n",
      downloadAndInstall,
      close: vi.fn().mockResolvedValue(undefined),
    };
  }

  it("reports up to date after a check the user asked for", async () => {
    plugins.check.mockResolvedValue(null);
    const store = new UpdateStore();
    await store.checkNow();
    expect(store.state).toEqual({ status: "upToDate" });
  });

  it("shows a failed check the user asked for", async () => {
    plugins.check.mockRejectedValue(new Error("offline"));
    const store = new UpdateStore();
    await store.checkNow();
    expect(store.state).toEqual({ status: "error", message: CHECK_FAILED_MESSAGE });
  });

  it("checks once, five seconds after start, and keeps its failure silent", async () => {
    vi.useFakeTimers();
    plugins.check.mockRejectedValue(new Error("offline"));
    const store = new UpdateStore();
    store.startAutomaticCheck();
    store.startAutomaticCheck();
    await vi.advanceTimersByTimeAsync(AUTOMATIC_CHECK_DELAY_MS - 1);
    expect(plugins.check).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(AUTOMATIC_CHECK_DELAY_MS);
    expect(plugins.check).toHaveBeenCalledTimes(1);
    expect(store.state).toEqual(IDLE);
  });

  it("skips the automatic check when automatic checks are off", async () => {
    vi.useFakeTimers();
    plugins.settings.checkForUpdates = false;
    const store = new UpdateStore();
    store.startAutomaticCheck();
    await vi.advanceTimersByTimeAsync(AUTOMATIC_CHECK_DELAY_MS * 2);
    expect(plugins.check).not.toHaveBeenCalled();
    expect(store.state).toEqual(IDLE);
  });

  it("lets Check now join a running automatic check and show its result", async () => {
    vi.useFakeTimers();
    let finish: (update: null) => void = () => undefined;
    plugins.check.mockReturnValue(
      new Promise<null>((resolve) => {
        finish = resolve;
      }),
    );
    const store = new UpdateStore();
    store.startAutomaticCheck();
    await vi.advanceTimersByTimeAsync(AUTOMATIC_CHECK_DELAY_MS);
    await store.checkNow();
    expect(store.state).toEqual({ status: "checking", silent: false });
    finish(null);
    await vi.waitFor(() => expect(store.state).toEqual({ status: "upToDate" }));
    expect(plugins.check).toHaveBeenCalledTimes(1);
  });

  it("installs an offered update, then restarts into it", async () => {
    const downloadAndInstall = vi.fn(async (onEvent: (event: DownloadEvent) => void) => {
      onEvent({ event: "Started", data: { contentLength: 100 } });
      onEvent({ event: "Progress", data: { chunkLength: 100 } });
      onEvent({ event: "Finished" });
    });
    plugins.check.mockResolvedValue(fakeUpdate(downloadAndInstall));
    const store = new UpdateStore();
    await store.checkNow();
    expect(store.state).toEqual({ status: "available", version: "1.4.0", notes: "Faster tables", error: null });
    await store.installUpdate();
    expect(downloadAndInstall).toHaveBeenCalledTimes(1);
    expect(store.state).toEqual({ status: "ready", version: "1.4.0", error: null });
    await store.restart();
    expect(plugins.relaunch).toHaveBeenCalledTimes(1);
  });

  it("keeps the offer and explains when installing fails", async () => {
    plugins.check.mockResolvedValue(fakeUpdate(vi.fn().mockRejectedValue(new Error("signature"))));
    const store = new UpdateStore();
    await store.checkNow();
    await store.installUpdate();
    expect(store.state).toMatchObject({ status: "available", version: "1.4.0", error: INSTALL_FAILED_MESSAGE });
  });

  it("explains a failed restart", async () => {
    plugins.check.mockResolvedValue(fakeUpdate());
    plugins.relaunch.mockRejectedValue(new Error("denied"));
    const store = new UpdateStore();
    await store.checkNow();
    await store.installUpdate();
    await store.restart();
    expect(store.state).toEqual({ status: "ready", version: "1.4.0", error: RESTART_FAILED_MESSAGE });
  });

  it("does nothing on install when no update is on offer", async () => {
    const store = new UpdateStore();
    await store.installUpdate();
    expect(store.state).toEqual(IDLE);
  });

  it("releases a dismissed offer", async () => {
    const update = fakeUpdate();
    plugins.check.mockResolvedValue(update);
    const store = new UpdateStore();
    await store.checkNow();
    store.dismiss();
    expect(store.state).toEqual(IDLE);
    expect(update.close).toHaveBeenCalledTimes(1);
  });
});
