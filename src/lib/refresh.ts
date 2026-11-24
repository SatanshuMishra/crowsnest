import type { ToastRequest } from "./toast.svelte";
import type { Notice, Phase, Team } from "./types";

export const REFRESH_COOLDOWN_MS = 10_000;
export const REFRESH_MIN_SPIN_MS = 700;

export function cooldownSeconds(
  lastRefreshAt: number | null,
  now: number,
  cooldownMs: number = REFRESH_COOLDOWN_MS,
): number {
  if (lastRefreshAt === null) {
    return 0;
  }
  const remaining = Math.min(lastRefreshAt + cooldownMs - now, cooldownMs);
  return remaining > 0 ? Math.ceil(remaining / 1000) : 0;
}

export interface RefreshAvailability {
  hasBattle: boolean;
  busy: boolean;
  cooldown: number;
}

export function refreshLabel({ hasBattle, busy, cooldown }: RefreshAvailability): string {
  if (!hasBattle) {
    return "Refresh statistics, available once a battle loads";
  }
  if (busy) {
    return "Refreshing statistics";
  }
  if (cooldown > 0) {
    return `Refresh statistics, available again in ${cooldown} ${cooldown === 1 ? "second" : "seconds"}`;
  }
  return "Refresh statistics";
}

export function canRefresh(availability: RefreshAvailability): boolean {
  return availability.hasBattle && !availability.busy && availability.cooldown === 0;
}

export function refreshToast(notice: Notice | null, teams: readonly Team[], retry: () => void): ToastRequest {
  if (notice?.level === "error") {
    return {
      tone: "error",
      title: "Refresh failed",
      body: notice.message,
      action: { label: "Try again", run: retry },
    };
  }
  const loaded = teams.flatMap((team) => team.players).filter((player) => player.stats.kind === "loaded").length;
  return {
    title: "Statistics refreshed",
    body: `Updated ${loaded} ${loaded === 1 ? "player" : "players"} from Wargaming.`,
  };
}

export interface RefreshWatch {
  battleId: string;
  started: boolean;
}

export interface WatchedState {
  phase: Phase;
  battle: { id: string; allies: Team; enemies: Team } | null;
}

export type RefreshProgress =
  | { kind: "watching"; watch: RefreshWatch }
  | { kind: "finished" }
  | { kind: "abandoned" };

function statisticsPending(battle: NonNullable<WatchedState["battle"]>): boolean {
  return [...battle.allies.players, ...battle.enemies.players].some(
    ({ stats }) => stats.kind === "pending" || (stats.kind === "loaded" && stats.shipState === "pending"),
  );
}

export function followRefresh(watch: RefreshWatch, state: WatchedState): RefreshProgress {
  if (state.battle?.id !== watch.battleId) {
    return { kind: "abandoned" };
  }
  if (state.phase === "loading" || statisticsPending(state.battle)) {
    return { kind: "watching", watch: { ...watch, started: true } };
  }
  return watch.started ? { kind: "finished" } : { kind: "watching", watch };
}
