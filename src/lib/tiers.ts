import type { StatKind } from "./format";
import type { ExpectedStats, HighlightStat, StatLine } from "./types";

export const TIERS = [
  "bad",
  "belowAverage",
  "average",
  "good",
  "veryGood",
  "great",
  "unicum",
  "superUnicum",
] as const;

export type Tier = (typeof TIERS)[number];

export const PERSONAL_RATING_CUTOFFS: readonly number[] = [750, 1100, 1350, 1550, 1750, 2100, 2450];
export const WIN_RATE_CUTOFFS: readonly number[] = [47, 49, 52, 54, 56, 60, 65];
export const RATIO_CUTOFFS: readonly number[] = [0.6, 0.8, 0.95, 1.1, 1.3, 1.5, 1.8];

const TIER_TOKENS: Record<Tier, string> = {
  bad: "--tier-bad",
  belowAverage: "--tier-below-average",
  average: "--tier-average",
  good: "--tier-good",
  veryGood: "--tier-very-good",
  great: "--tier-great",
  unicum: "--tier-unicum",
  superUnicum: "--tier-super-unicum",
};

const TIER_NAMES: Record<Tier, string> = {
  bad: "Bad",
  belowAverage: "Below average",
  average: "Average",
  good: "Good",
  veryGood: "Very good",
  great: "Great",
  unicum: "Unicum",
  superUnicum: "Super unicum",
};

export const NO_TIER_NAME = "Not rated";

function isPresent(value: number | null | undefined): value is number {
  return value !== null && value !== undefined && Number.isFinite(value);
}

export function tierFor(value: number | null | undefined, cutoffs: readonly number[]): Tier | null {
  if (!isPresent(value)) {
    return null;
  }
  const reached = cutoffs.filter((cutoff) => value >= cutoff).length;
  return TIERS[reached] ?? null;
}

export function ratio(actual: number | null | undefined, expected: number | null | undefined): number | null {
  if (!isPresent(actual) || !isPresent(expected) || expected <= 0) {
    return null;
  }
  return actual / expected;
}

export function personalRatingTier(value: number | null | undefined): Tier | null {
  return tierFor(value, PERSONAL_RATING_CUTOFFS);
}

export function winRateTier(value: number | null | undefined): Tier | null {
  return tierFor(value, WIN_RATE_CUTOFFS);
}

export function damageTier(averageDamage: number | null | undefined, expected: ExpectedStats | null): Tier | null {
  return tierFor(ratio(averageDamage, expected?.damage), RATIO_CUTOFFS);
}

export function fragsTier(averageFrags: number | null | undefined, expected: ExpectedStats | null): Tier | null {
  return tierFor(ratio(averageFrags, expected?.frags), RATIO_CUTOFFS);
}

const TIERED_KINDS: ReadonlySet<StatKind> = new Set<StatKind>(["winRate", "personalRating", "damage", "frags"]);

export function isTiered(kind: StatKind): boolean {
  return TIERED_KINDS.has(kind);
}

export function statTier(kind: StatKind, value: number | null | undefined, expected: ExpectedStats | null): Tier | null {
  switch (kind) {
    case "winRate":
      return winRateTier(value);
    case "personalRating":
      return personalRatingTier(value);
    case "damage":
      return damageTier(value, expected);
    case "frags":
      return fragsTier(value, expected);
    default:
      return null;
  }
}

export interface RatedLines {
  overall: StatLine;
  ship: StatLine | null;
}

export function highlightTier(stat: HighlightStat, lines: RatedLines, expected: ExpectedStats | null): Tier | null {
  switch (stat) {
    case "personalRating":
      return personalRatingTier(lines.ship?.personalRating);
    case "shipWinRate":
      return winRateTier(lines.ship?.winRate);
    case "accountWinRate":
      return winRateTier(lines.overall.winRate);
    case "averageDamage":
      return damageTier(lines.ship?.avgDamage, expected);
  }
}

export function tierToken(tier: Tier): string {
  return TIER_TOKENS[tier];
}

export function tierColor(tier: Tier): string {
  return `var(${TIER_TOKENS[tier]})`;
}

export function tierName(tier: Tier | null): string {
  return tier ? TIER_NAMES[tier] : NO_TIER_NAME;
}
