import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { app } from "./state.svelte";

export const AUTOMATIC_CHECK_DELAY_MS = 5000;

export const CHECK_FAILED_MESSAGE =
  "Crowsnest could not reach the update server. Check your internet connection, then try again.";
export const INSTALL_FAILED_MESSAGE =
  "The update could not be installed. Try again, or download it from the releases page on GitHub.";
export const RESTART_FAILED_MESSAGE = "Crowsnest could not restart itself. Close it and open it again.";

export interface UpdateOffer {
  version: string;
  notes: string | null;
}

export interface DownloadingState {
  status: "downloading";
  version: string;
  notes: string | null;
  received: number;
  total: number | null;
  percent: number | null;
}

export type UpdateState =
  | { status: "idle" }
  | { status: "checking"; silent: boolean }
  | { status: "upToDate" }
  | { status: "available"; version: string; notes: string | null; error: string | null }
  | DownloadingState
  | { status: "ready"; version: string; error: string | null }
  | { status: "error"; message: string };

export type UpdateAction =
  | { type: "checkStarted"; silent: boolean }
  | { type: "checkFinished"; offer: UpdateOffer | null }
  | { type: "checkFailed"; message: string }
  | { type: "downloadStarted" }
  | { type: "downloadProgress"; event: DownloadEvent }
  | { type: "installed" }
  | { type: "installFailed"; message: string }
  | { type: "restartFailed"; message: string }
  | { type: "dismissed" };

export const IDLE: UpdateState = { status: "idle" };

function percentOf(received: number, total: number | null): number | null {
  if (total === null) {
    return null;
  }
  return Math.min(100, Math.floor((received / total) * 100));
}

function withProgress(state: DownloadingState, event: DownloadEvent): DownloadingState {
  switch (event.event) {
    case "Started": {
      const length = event.data.contentLength;
      const total = length !== undefined && length > 0 ? length : null;
      return { ...state, received: 0, total, percent: percentOf(0, total) };
    }
    case "Progress": {
      const received = state.received + event.data.chunkLength;
      return { ...state, received, percent: percentOf(received, state.total) };
    }
    case "Finished":
      return state.total === null ? state : { ...state, received: state.total, percent: 100 };
  }
}

export function reduceUpdate(state: UpdateState, action: UpdateAction): UpdateState {
  switch (action.type) {
    case "checkStarted":
      if (state.status === "checking") {
        return { status: "checking", silent: state.silent && action.silent };
      }
      if (state.status === "downloading" || state.status === "ready") {
        return state;
      }
      return { status: "checking", silent: action.silent };
    case "checkFinished":
      if (state.status !== "checking") {
        return state;
      }
      if (action.offer) {
        return { status: "available", version: action.offer.version, notes: action.offer.notes, error: null };
      }
      return state.silent ? IDLE : { status: "upToDate" };
    case "checkFailed":
      if (state.status !== "checking") {
        return state;
      }
      return state.silent ? IDLE : { status: "error", message: action.message };
    case "downloadStarted":
      if (state.status !== "available") {
        return state;
      }
      return {
        status: "downloading",
        version: state.version,
        notes: state.notes,
        received: 0,
        total: null,
        percent: null,
      };
    case "downloadProgress":
      return state.status === "downloading" ? withProgress(state, action.event) : state;
    case "installed":
      return state.status === "downloading" ? { status: "ready", version: state.version, error: null } : state;
    case "installFailed":
      if (state.status !== "downloading") {
        return state;
      }
      return { status: "available", version: state.version, notes: state.notes, error: action.message };
    case "restartFailed":
      return state.status === "ready" ? { ...state, error: action.message } : state;
    case "dismissed":
      if (state.status === "checking" || state.status === "downloading" || state.status === "ready") {
        return state;
      }
      return IDLE;
  }
}

export function updateAnnouncement(state: UpdateState): string {
  switch (state.status) {
    case "available":
      return state.error ? "The update could not be installed. Try again." : `Crowsnest ${state.version} is available.`;
    case "downloading":
      return `Downloading Crowsnest ${state.version}.`;
    case "ready":
      return state.error ?? `Crowsnest ${state.version} is installed. Restart to update.`;
    default:
      return "";
  }
}

function offerFrom(update: Update | null): UpdateOffer | null {
  if (!update) {
    return null;
  }
  const notes = update.body?.trim();
  return { version: update.version, notes: notes ? notes : null };
}

export class UpdateStore {
  #state = $state<UpdateState>(IDLE);
  #pending: Update | null = null;
  #automaticCheckScheduled = false;

  get state(): UpdateState {
    return this.#state;
  }

  checkNow(): Promise<void> {
    return this.#check(false);
  }

  startAutomaticCheck(): void {
    if (this.#automaticCheckScheduled) {
      return;
    }
    this.#automaticCheckScheduled = true;
    setTimeout(() => {
      if (app.settings?.checkForUpdates) {
        void this.#check(true);
      }
    }, AUTOMATIC_CHECK_DELAY_MS);
  }

  async installUpdate(): Promise<void> {
    const update = this.#pending;
    if (!update || this.#state.status !== "available") {
      return;
    }
    this.#dispatch({ type: "downloadStarted" });
    try {
      await update.downloadAndInstall((event) => this.#dispatch({ type: "downloadProgress", event }));
      this.#dispatch({ type: "installed" });
    } catch {
      this.#dispatch({ type: "installFailed", message: INSTALL_FAILED_MESSAGE });
    }
  }

  async restart(): Promise<void> {
    try {
      await relaunch();
    } catch {
      this.#dispatch({ type: "restartFailed", message: RESTART_FAILED_MESSAGE });
    }
  }

  dismiss(): void {
    const wasAvailable = this.#state.status === "available";
    this.#dispatch({ type: "dismissed" });
    if (wasAvailable) {
      this.#hold(null);
    }
  }

  async #check(silent: boolean): Promise<void> {
    const alreadyChecking = this.#state.status === "checking";
    this.#dispatch({ type: "checkStarted", silent });
    if (alreadyChecking || this.#state.status !== "checking") {
      return;
    }
    try {
      const update = await check();
      this.#hold(update);
      this.#dispatch({ type: "checkFinished", offer: offerFrom(update) });
    } catch {
      this.#dispatch({ type: "checkFailed", message: CHECK_FAILED_MESSAGE });
    }
  }

  #dispatch(action: UpdateAction): void {
    this.#state = reduceUpdate(this.#state, action);
  }

  #hold(next: Update | null): void {
    const previous = this.#pending;
    this.#pending = next;
    if (previous && previous !== next) {
      previous.close().catch(() => undefined);
    }
  }
}

export const updates = new UpdateStore();
