import type { PlayerStats, Region } from "./types";

const SUBDOMAINS: Record<Region, string> = {
  eu: "",
  na: "na.",
  asia: "asia.",
};

function playerUrl(host: string, region: Region, accountId: number, name: string): string {
  return `https://${SUBDOMAINS[region]}${host}/player/${accountId},${encodeURIComponent(name)}/`;
}

export function wowsNumbersUrl(region: Region, accountId: number, name: string): string {
  return playerUrl("wows-numbers.com", region, accountId, name);
}

export function wowsKarmaUrl(region: Region, accountId: number, name: string): string {
  return playerUrl("wows-karma.com", region, accountId, name);
}

export interface PlayerLink {
  id: string;
  label: string;
  url: string;
}

export function accountIdOf(stats: PlayerStats): number | null {
  return stats.kind === "loaded" || stats.kind === "hidden" ? stats.accountId : null;
}

export function playerLinks(region: Region | null, stats: PlayerStats, name: string): PlayerLink[] {
  const accountId = accountIdOf(stats);
  if (region === null || accountId === null) {
    return [];
  }
  return [
    { id: "wows-numbers", label: "wows-numbers.com", url: wowsNumbersUrl(region, accountId, name) },
    { id: "wows-karma", label: "wows-karma.com", url: wowsKarmaUrl(region, accountId, name) },
  ];
}
