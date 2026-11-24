import type { Region, ShipClass } from "./types";

export const MISSING = "–";
export const THIN_SPACE = String.fromCharCode(0x2009);

export type StatKind = "battles" | "winRate" | "personalRating" | "damage" | "frags" | "xp";

type Formatter = (value: number | null | undefined) => string;

function isPresent(value: number | null | undefined): value is number {
  return value !== null && value !== undefined && Number.isFinite(value);
}

export function formatInteger(value: number | null | undefined): string {
  if (!isPresent(value)) {
    return MISSING;
  }
  const rounded = Math.round(value);
  const grouped = Math.abs(rounded)
    .toString()
    .replace(/\B(?=(\d{3})+(?!\d))/g, THIN_SPACE);
  return rounded < 0 ? `-${grouped}` : grouped;
}

export function formatWinRate(value: number | null | undefined): string {
  return isPresent(value) ? `${value.toFixed(1)}%` : MISSING;
}

export function formatFrags(value: number | null | undefined): string {
  return isPresent(value) ? value.toFixed(2) : MISSING;
}

export const formatRating: Formatter = formatInteger;
export const formatDamage: Formatter = formatInteger;
export const formatXp: Formatter = formatInteger;

const STAT_FORMATTERS: Record<StatKind, Formatter> = {
  battles: formatInteger,
  winRate: formatWinRate,
  personalRating: formatRating,
  damage: formatDamage,
  frags: formatFrags,
  xp: formatXp,
};

export function formatStat(kind: StatKind, value: number | null | undefined): string {
  return STAT_FORMATTERS[kind](value);
}

const TYPICAL_VALUES: Record<StatKind, number> = {
  battles: 1234,
  winRate: 52.3,
  personalRating: 1234,
  damage: 45678,
  frags: 0.85,
  xp: 1234,
};

export function placeholderFor(kind: StatKind): string {
  return formatStat(kind, TYPICAL_VALUES[kind]);
}

const ROMAN_STEPS: ReadonlyArray<readonly [number, string]> = [
  [10, "X"],
  [9, "IX"],
  [5, "V"],
  [4, "IV"],
  [1, "I"],
];

export function romanNumeral(value: number): string {
  if (!Number.isInteger(value) || value < 1 || value > 39) {
    return MISSING;
  }
  const [numerals] = ROMAN_STEPS.reduce<[string, number]>(
    ([written, left], [step, numeral]) => {
      const count = Math.floor(left / step);
      return [written + numeral.repeat(count), left - count * step];
    },
    ["", value],
  );
  return numerals;
}

export function mapName(displayName: string, mapPath = ""): string {
  const source = displayName.trim() || (mapPath.trim().split("/").pop() ?? "");
  const name = source
    .replace(/^\d+_/, "")
    .replace(/_+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  return name || MISSING;
}

export function humanizeIdentifier(value: string): string {
  const words = value
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .split(/[\s_-]+/)
    .filter((word) => word.length > 0);
  return words
    .map((word, index) => {
      const isAcronym = word.length > 1 && word === word.toUpperCase() && /[A-Z]/.test(word);
      if (isAcronym) {
        return word;
      }
      const lower = word.toLowerCase();
      return index === 0 ? lower.charAt(0).toUpperCase() + lower.slice(1) : lower;
    })
    .join(" ");
}

const MATCH_GROUPS: Readonly<Record<string, string>> = {
  pvp: "Random battle",
  ranked: "Ranked battle",
  clan: "Clan battle",
  cooperative: "Co-op battle",
  pve: "Operation",
  event: "Event battle",
  brawl: "Brawl",
};

export function matchType(gameType: string, matchGroup: string): string {
  const known = MATCH_GROUPS[matchGroup.trim().toLowerCase()];
  if (known) {
    return known;
  }
  return humanizeIdentifier(gameType) || humanizeIdentifier(matchGroup) || MISSING;
}

export interface ShipClassBadge {
  short: string;
  name: string;
}

const SHIP_CLASSES: Record<ShipClass, ShipClassBadge | null> = {
  destroyer: { short: "DD", name: "Destroyer" },
  cruiser: { short: "CA", name: "Cruiser" },
  battleship: { short: "BB", name: "Battleship" },
  airCarrier: { short: "CV", name: "Aircraft carrier" },
  submarine: { short: "SS", name: "Submarine" },
  unknown: null,
};

export function shipClassBadge(shipClass: ShipClass): ShipClassBadge | null {
  return SHIP_CLASSES[shipClass];
}

const REGIONS: Record<Region, { short: string; name: string }> = {
  eu: { short: "EU", name: "Europe" },
  na: { short: "NA", name: "North America" },
  asia: { short: "Asia", name: "Asia" },
};

export function regionLabel(region: Region): string {
  return REGIONS[region].short;
}

export function regionName(region: Region): string {
  return REGIONS[region].name;
}
