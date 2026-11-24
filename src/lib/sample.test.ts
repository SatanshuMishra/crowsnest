import { describe, expect, it } from "vitest";
import { SAMPLE_TEAM } from "./sample";
import { playerLines } from "./team";
import { highlightTier } from "./tiers";
import type { HighlightStat } from "./types";

const STATS: readonly HighlightStat[] = ["personalRating", "shipWinRate", "accountWinRate", "averageDamage"];

function tiersBy(stat: HighlightStat) {
  return SAMPLE_TEAM.map((player) => {
    const lines = playerLines(player.stats, false);
    return lines ? highlightTier(stat, lines, player.expected) : null;
  });
}

describe("the sample team", () => {
  it("rates every player on every highlight statistic", () => {
    for (const stat of STATS) {
      expect(tiersBy(stat)).not.toContain(null);
    }
  });

  it("gives every row a different tier for each pair of highlight statistics", () => {
    for (const first of STATS) {
      for (const second of STATS.filter((stat) => stat !== first)) {
        const before = tiersBy(first);
        const after = tiersBy(second);
        expect(before.every((tier, index) => tier !== after[index]), `${first} against ${second}`).toBe(true);
      }
    }
  });
});
