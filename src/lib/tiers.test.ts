import { describe, expect, it } from "vitest";
import type { StatLine } from "./types";
import {
  NO_TIER_NAME,
  TIERS,
  damageTier,
  fragsTier,
  highlightTier,
  isTiered,
  personalRatingTier,
  ratio,
  statTier,
  tierColor,
  tierName,
  tierToken,
  winRateTier,
} from "./tiers";

const expected = { damage: 40000, frags: 1, winRate: 50 };

function line(change: Partial<StatLine>): StatLine {
  return {
    battles: 100,
    wins: 50,
    winRate: 50,
    avgDamage: 40000,
    avgFrags: 1,
    avgXp: 1200,
    survivalRate: 35,
    personalRating: 1200,
    ...change,
  };
}

describe("personalRatingTier", () => {
  it.each([
    [0, "bad"],
    [749.9, "bad"],
    [750, "belowAverage"],
    [1099, "belowAverage"],
    [1100, "average"],
    [1350, "good"],
    [1550, "veryGood"],
    [1750, "great"],
    [2100, "unicum"],
    [2449, "unicum"],
    [2450, "superUnicum"],
    [4000, "superUnicum"],
  ] as const)("rates %s as %s", (value, tier) => {
    expect(personalRatingTier(value)).toBe(tier);
  });

  it("has no tier for a missing value", () => {
    expect(personalRatingTier(null)).toBeNull();
    expect(personalRatingTier(undefined)).toBeNull();
    expect(personalRatingTier(Number.NaN)).toBeNull();
  });
});

describe("winRateTier", () => {
  it.each([
    [46.9, "bad"],
    [47, "belowAverage"],
    [49, "average"],
    [51.9, "average"],
    [52, "good"],
    [54, "veryGood"],
    [56, "great"],
    [60, "unicum"],
    [65, "superUnicum"],
  ] as const)("rates %s%% as %s", (value, tier) => {
    expect(winRateTier(value)).toBe(tier);
  });
});

describe("ratio tiers", () => {
  it("divides by the expected value and rejects a missing or zero expectation", () => {
    expect(ratio(50, 40)).toBe(1.25);
    expect(ratio(50, 0)).toBeNull();
    expect(ratio(50, null)).toBeNull();
    expect(ratio(null, 40)).toBeNull();
  });

  it.each([
    [0.59, "bad"],
    [0.6, "belowAverage"],
    [0.8, "average"],
    [0.95, "good"],
    [1.1, "veryGood"],
    [1.3, "great"],
    [1.5, "unicum"],
    [1.8, "superUnicum"],
  ] as const)("rates a damage ratio of %s as %s", (factor, tier) => {
    expect(damageTier(factor * 1000, { damage: 1000, frags: 1, winRate: 50 })).toBe(tier);
  });

  it("rates frags against the expected frags", () => {
    expect(fragsTier(1.5, expected)).toBe("unicum");
    expect(fragsTier(0.5, expected)).toBe("bad");
  });

  it("has no tier without expected values", () => {
    expect(damageTier(50000, null)).toBeNull();
    expect(fragsTier(1, null)).toBeNull();
  });
});

describe("statTier", () => {
  it("rates only the tiered kinds", () => {
    expect(statTier("winRate", 55, expected)).toBe("veryGood");
    expect(statTier("personalRating", 2200, expected)).toBe("unicum");
    expect(statTier("damage", 48000, expected)).toBe("veryGood");
    expect(statTier("frags", 0.9, expected)).toBe("average");
    expect(statTier("battles", 5000, expected)).toBeNull();
    expect(statTier("xp", 2000, expected)).toBeNull();
    expect(isTiered("battles")).toBe(false);
    expect(isTiered("xp")).toBe(false);
    expect(isTiered("damage")).toBe(true);
  });
});

describe("highlightTier", () => {
  const lines = {
    overall: line({ winRate: 61, personalRating: 900 }),
    ship: line({ winRate: 48, personalRating: 1800, avgDamage: 20000 }),
  };

  it("follows the chosen statistic", () => {
    expect(highlightTier("personalRating", lines, expected)).toBe("great");
    expect(highlightTier("shipWinRate", lines, expected)).toBe("belowAverage");
    expect(highlightTier("accountWinRate", lines, expected)).toBe("unicum");
    expect(highlightTier("averageDamage", lines, expected)).toBe("bad");
  });

  it("has no ship tier without a ship line", () => {
    const accountOnly = { overall: lines.overall, ship: null };
    expect(highlightTier("personalRating", accountOnly, expected)).toBeNull();
    expect(highlightTier("shipWinRate", accountOnly, expected)).toBeNull();
    expect(highlightTier("averageDamage", accountOnly, expected)).toBeNull();
    expect(highlightTier("accountWinRate", accountOnly, expected)).toBe("unicum");
  });
});

describe("tier presentation", () => {
  it("maps every tier to a token and a name", () => {
    expect(TIERS.map(tierToken)).toEqual([
      "--tier-bad",
      "--tier-below-average",
      "--tier-average",
      "--tier-good",
      "--tier-very-good",
      "--tier-great",
      "--tier-unicum",
      "--tier-super-unicum",
    ]);
    expect(TIERS.map(tierName)).toEqual([
      "Bad",
      "Below average",
      "Average",
      "Good",
      "Very good",
      "Great",
      "Unicum",
      "Super unicum",
    ]);
    expect(tierColor("good")).toBe("var(--tier-good)");
    expect(tierName(null)).toBe(NO_TIER_NAME);
  });
});
