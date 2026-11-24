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
