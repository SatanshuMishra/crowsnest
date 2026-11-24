import type { ToastRequest } from "./toast.svelte";
import type { Phase, Settings } from "./types";

export const INTRO_VERSION = 1;

export type SetupAudience = "new" | "updating" | "tour";

export interface SetupOutcome {
  changed: boolean;
  finished: boolean;
}

export type SetupFlow = Exclude<SetupAudience, "updating">;

const CHANGE_IT_LATER = "Change it any time in Settings, Appearance.";

export function introAudience(introSeen: Settings["introSeen"]): SetupAudience | null {
  if (introSeen !== null && introSeen >= INTRO_VERSION) {
    return null;
  }
  return introSeen === 0 ? "new" : "updating";
}

export function introCanOpen(phase: Phase, onMonitor: boolean): boolean {
  return onMonitor && phase === "waiting";
}

export function introInterrupted(audience: SetupAudience | null, phase: Phase | undefined): boolean {
  return audience !== null && audience !== "tour" && phase !== undefined && phase !== "waiting";
}

export function setupToast(audience: SetupFlow, outcome: SetupOutcome): ToastRequest | null {
  if (audience === "tour") {
    return outcome.changed ? { title: "Appearance saved", body: "The player table now uses your new look." } : null;
  }
  if (outcome.changed || outcome.finished) {
    return { title: "Appearance saved", body: CHANGE_IT_LATER };
  }
  return { title: "Using the default look", body: CHANGE_IT_LATER };
}
