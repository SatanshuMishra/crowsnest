import type { StatKind } from "./format";
import type { Density, Player, PlayerStats, StatLine, TeamAverage } from "./types";

export const WEIGHT_CAP = 40;

export interface PlayerLines {
  overall: StatLine;
  ship: StatLine | null;
}

export function playerLines(stats: PlayerStats, solo: boolean): PlayerLines | null {
  if (stats.kind !== "loaded") {
    return null;
  }
  const overall = (solo ? stats.overallSolo : null) ?? stats.overall;
  const ship = (solo ? stats.shipSolo : null) ?? stats.ship;
  return { overall, ship: ship && ship.battles > 0 ? ship : null };
}

export function lineValue(line: StatLine, kind: StatKind): number | null {
  switch (kind) {
    case "battles":
      return line.battles;
    case "winRate":
      return line.winRate;
    case "personalRating":
      return line.personalRating;
    case "damage":
      return line.avgDamage;
    case "frags":
      return line.avgFrags;
    case "xp":
      return line.avgXp;
  }
}

export type StatScope = "ship" | "account";

export interface StatColumn {
  id: string;
  scope: StatScope;
  kind: StatKind;
  short: string;
  name: string;
}

const SHIP_COLUMNS: readonly StatColumn[] = [
  { id: "shipBattles", scope: "ship", kind: "battles", short: "Battles", name: "Battles in this ship" },
  { id: "shipWinRate", scope: "ship", kind: "winRate", short: "WR", name: "Win rate in this ship" },
  { id: "shipRating", scope: "ship", kind: "personalRating", short: "PR", name: "Personal rating in this ship" },
  { id: "shipDamage", scope: "ship", kind: "damage", short: "Dmg", name: "Average damage in this ship" },
  { id: "shipFrags", scope: "ship", kind: "frags", short: "Frags", name: "Average frags in this ship" },
  { id: "shipXp", scope: "ship", kind: "xp", short: "XP", name: "Average experience in this ship" },
];

const ACCOUNT_COLUMNS: readonly StatColumn[] = [
  { id: "accountBattles", scope: "account", kind: "battles", short: "Battles", name: "Account battles" },
  { id: "accountWinRate", scope: "account", kind: "winRate", short: "WR", name: "Account win rate" },
];

const COMPACT_HIDDEN: ReadonlySet<StatKind> = new Set<StatKind>(["frags", "xp"]);
const MEDIUM_HIDDEN: ReadonlySet<StatKind> = new Set<StatKind>(["xp"]);
const NOTHING_HIDDEN: ReadonlySet<StatKind> = new Set<StatKind>();

export const NARROW_TABLE_WIDTH = 720;
export const MEDIUM_TABLE_WIDTH = 820;

function hiddenKinds(density: Density, tableWidth: number): ReadonlySet<StatKind> {
  if (density === "compact" || tableWidth < NARROW_TABLE_WIDTH) {
    return COMPACT_HIDDEN;
  }
  return tableWidth < MEDIUM_TABLE_WIDTH ? MEDIUM_HIDDEN : NOTHING_HIDDEN;
}

export interface ColumnGroups {
  ship: StatColumn[];
  account: StatColumn[];
}

export function statColumns(density: Density, tableWidth = Number.POSITIVE_INFINITY): ColumnGroups {
  const hidden = hiddenKinds(density, tableWidth);
  const ship = SHIP_COLUMNS.filter((column) => !hidden.has(column.kind));
  return { ship, account: [...ACCOUNT_COLUMNS] };
}

export function columnValue(column: StatColumn, lines: PlayerLines): number | null {
  const line = column.scope === "ship" ? lines.ship : lines.overall;
  return line ? lineValue(line, column.kind) : null;
}

export interface Sample {
  value: number;
  battles: number;
}

function sum(values: readonly number[]): number {
  return values.reduce((total, value) => total + value, 0);
}

export function median(values: readonly number[]): number | null {
  if (values.length === 0) {
    return null;
  }
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  const upper = sorted[middle] ?? 0;
  if (sorted.length % 2 === 1) {
    return upper;
  }
  return ((sorted[middle - 1] ?? upper) + upper) / 2;
}

export function averageOf(samples: readonly Sample[], method: TeamAverage): number | null {
  if (samples.length === 0) {
    return null;
  }
  if (method === "median") {
    return median(samples.map((sample) => sample.value));
  }
  if (method === "weighted") {
    const weighted = samples.map((sample) => ({
      value: sample.value,
      weight: Math.min(Math.max(sample.battles, 0), WEIGHT_CAP),
    }));
    const totalWeight = sum(weighted.map((entry) => entry.weight));
    if (totalWeight === 0) {
      return null;
    }
    return sum(weighted.map((entry) => entry.value * entry.weight)) / totalWeight;
  }
  return sum(samples.map((sample) => sample.value)) / samples.length;
}

export interface TeamSummary {
  personalRating: number | null;
  shipWinRate: number | null;
  accountWinRate: number | null;
  battles: number | null;
  withStats: number;
  total: number;
}

function samplesOf(lines: readonly PlayerLines[], scope: StatScope, kind: StatKind): Sample[] {
  return lines.flatMap((entry) => {
    const line = scope === "ship" ? entry.ship : entry.overall;
    const value = line ? lineValue(line, kind) : null;
    return line && value !== null && Number.isFinite(value) ? [{ value, battles: line.battles }] : [];
  });
}

export function summarizeTeam(players: readonly Player[], method: TeamAverage, solo: boolean): TeamSummary {
  const lines = players.flatMap((player) => {
    const found = playerLines(player.stats, solo);
    return found ? [found] : [];
  });
  return {
    personalRating: averageOf(samplesOf(lines, "ship", "personalRating"), method),
    shipWinRate: averageOf(samplesOf(lines, "ship", "winRate"), method),
    accountWinRate: averageOf(samplesOf(lines, "account", "winRate"), method),
    battles: averageOf(samplesOf(lines, "account", "battles"), method === "weighted" ? "mean" : method),
    withStats: lines.length,
    total: players.length,
  };
}

export function coverageLabel(summary: Pick<TeamSummary, "withStats" | "total">): string {
  return `${summary.withStats} of ${summary.total} with stats`;
}

const AVERAGE_NAMES: Record<TeamAverage, string> = {
  mean: "Mean",
  weighted: "Weighted by battles, capped at 40 per player",
  median: "Median",
};

export function averageName(method: TeamAverage): string {
  return AVERAGE_NAMES[method];
}
