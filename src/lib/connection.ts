import type { Phase, Snapshot } from "./types";

export type Activity = "listening" | "busy" | "steady";

export interface ConnectionStatus {
  connected: boolean;
  label: string;
  detail: string;
  activity: Activity;
}

const WHILE_CONNECTED: Partial<Record<Phase, { detail: string; activity: Activity }>> = {
  loading: { detail: "Loading stats", activity: "busy" },
  inBattle: { detail: "In battle", activity: "steady" },
};

const WHILE_DISCONNECTED: Partial<Record<Phase, string>> = {
  needsInstall: "Game not found",
  needsApplicationId: "Setup needed",
  replaysDisabled: "Replays off",
};

export function connectionStatus(snapshot: Pick<Snapshot, "phase" | "connected">): ConnectionStatus {
  if (snapshot.connected) {
    const { detail, activity } = WHILE_CONNECTED[snapshot.phase] ?? {
      detail: "Waiting for battle",
      activity: "listening",
    };
    return { connected: true, label: "Connected", detail, activity };
  }
  return {
    connected: false,
    label: "Not connected",
    detail: WHILE_DISCONNECTED[snapshot.phase] ?? "Replays folder unavailable",
    activity: "steady",
  };
}
