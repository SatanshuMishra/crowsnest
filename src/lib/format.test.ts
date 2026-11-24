import { describe, expect, it } from "vitest";
import {
  clientVersionLabel,
  divisionLetter,
  MISSING,
  THIN_SPACE,
  displayedValue,
  formatDamage,
  formatFrags,
  formatInteger,
  formatRating,
  formatStat,
  formatWinRate,
  formatXp,
  humanizeIdentifier,
  mapName,
  matchType,
  placeholderFor,
  regionLabel,
  regionName,
  romanNumeral,
  shipClassBadge,
} from "./format";

const grouped = (text: string) => text.replaceAll(" ", THIN_SPACE);

describe("formatInteger", () => {
  it("groups thousands with a thin space", () => {
    expect(formatInteger(12345)).toBe(grouped("12 345"));
    expect(formatInteger(1234567)).toBe(grouped("1 234 567"));
    expect(formatInteger(999)).toBe("999");
    expect(formatInteger(0)).toBe("0");
  });

  it("rounds to the nearest integer", () => {
    expect(formatInteger(1499.5)).toBe(grouped("1 500"));
    expect(formatInteger(-1234.4)).toBe(grouped("-1 234"));
  });

  it("shows an en dash for missing values", () => {
    expect(MISSING).toBe("–");
    expect(formatInteger(null)).toBe(MISSING);
    expect(formatInteger(undefined)).toBe(MISSING);
    expect(formatInteger(Number.NaN)).toBe(MISSING);
  });
});

describe("stat formatters", () => {
  it("formats win rate with one decimal and a percent sign", () => {
    expect(formatWinRate(52.345)).toBe("52.3%");
    expect(formatWinRate(50)).toBe("50.0%");
    expect(formatWinRate(null)).toBe(MISSING);
  });

  it("formats rating, damage and experience as grouped integers", () => {
    expect(formatRating(1549.6)).toBe(grouped("1 550"));
    expect(formatDamage(45678.2)).toBe(grouped("45 678"));
    expect(formatXp(1234.4)).toBe(grouped("1 234"));
  });

  it("formats frags with two decimals", () => {
    expect(formatFrags(0.8567)).toBe("0.86");
    expect(formatFrags(1)).toBe("1.00");
    expect(formatFrags(null)).toBe(MISSING);
  });

  it("dispatches by kind", () => {
    expect(formatStat("battles", 2500)).toBe(grouped("2 500"));
    expect(formatStat("winRate", 55.55)).toBe("55.5%");
    expect(formatStat("frags", 0.5)).toBe("0.50");
    expect(formatStat("damage", null)).toBe(MISSING);
  });

  it("offers a placeholder as wide as a typical value", () => {
    expect(placeholderFor("damage")).toBe(grouped("45 678"));
    expect(placeholderFor("winRate")).toBe("52.3%");
    expect(placeholderFor("frags")).toBe("0.85");
  });
});

describe("displayedValue", () => {
  it("rounds each kind to the precision its formatter shows", () => {
    expect(displayedValue("winRate", 46.96)).toBe(47);
    expect(displayedValue("winRate", 46.94)).toBe(46.9);
    expect(displayedValue("personalRating", 1099.6)).toBe(1100);
    expect(displayedValue("damage", 43999.5)).toBe(44000);
    expect(displayedValue("frags", 1.096)).toBe(1.1);
    expect(displayedValue("frags", 1.094)).toBe(1.09);
    expect(displayedValue("battles", 12.4)).toBe(12);
    expect(displayedValue("xp", 1234.5)).toBe(1235);
  });

  it("has no value for a missing one", () => {
    expect(displayedValue("winRate", null)).toBeNull();
    expect(displayedValue("frags", undefined)).toBeNull();
    expect(displayedValue("damage", Number.NaN)).toBeNull();
  });
});

describe("romanNumeral", () => {
  it("covers every ship tier", () => {
    const tiers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(romanNumeral);
    expect(tiers).toEqual(["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI"]);
  });

  it("returns an en dash outside the supported range", () => {
    expect(romanNumeral(0)).toBe(MISSING);
    expect(romanNumeral(-3)).toBe(MISSING);
    expect(romanNumeral(2.5)).toBe(MISSING);
  });
});

describe("mapName", () => {
  it("strips the number prefix and turns underscores into spaces", () => {
    expect(mapName("38_Canada")).toBe("Canada");
    expect(mapName("17_NA_fault_line")).toBe("NA fault line");
    expect(mapName("50_Gold_harbor")).toBe("Gold harbor");
  });

  it("falls back to the map path when the display name is empty", () => {
    expect(mapName("", "spaces/38_Canada")).toBe("Canada");
    expect(mapName("  ", "")).toBe(MISSING);
  });
});

describe("matchType", () => {
  it("names the documented match groups", () => {
    expect(matchType("RandomBattle", "pvp")).toBe("Random battle");
    expect(matchType("RankedBattle", "ranked")).toBe("Ranked battle");
    expect(matchType("ClanBattle", "clan")).toBe("Clan battle");
    expect(matchType("", "cooperative")).toBe("Co-op battle");
  });

  it("humanises an unknown game type, then an unknown match group", () => {
    expect(matchType("AirshipEscortBattle", "airship")).toBe("Airship escort battle");
    expect(matchType("", "space_battle")).toBe("Space battle");
    expect(matchType("", "")).toBe(MISSING);
  });

  it("keeps acronyms intact", () => {
    expect(humanizeIdentifier("PVE")).toBe("PVE");
    expect(humanizeIdentifier("event_PVE_mode")).toBe("Event PVE mode");
  });
});

describe("labels", () => {
  it("abbreviates ship classes and names them", () => {
    expect(shipClassBadge("destroyer")).toEqual({ short: "DD", name: "Destroyer" });
    expect(shipClassBadge("cruiser")?.short).toBe("CA");
    expect(shipClassBadge("battleship")?.short).toBe("BB");
    expect(shipClassBadge("airCarrier")?.short).toBe("CV");
    expect(shipClassBadge("submarine")?.short).toBe("SS");
    expect(shipClassBadge("unknown")).toBeNull();
  });

  it("labels regions", () => {
    expect(regionLabel("eu")).toBe("EU");
    expect(regionLabel("na")).toBe("NA");
    expect(regionName("asia")).toBe("Asia");
  });
});

describe("clientVersionLabel", () => {
  it("shows the comma form the game writes with dots", () => {
    expect(clientVersionLabel("15,10,0,13533378")).toBe("15.10.0.13533378");
  });

  it("leaves a dotted version as it is", () => {
    expect(clientVersionLabel(" 15.10.0.0.13533378 ")).toBe("15.10.0.0.13533378");
  });
});

describe("divisionLetter", () => {
  it("letters a team's divisions in order from A", () => {
    expect([1, 2, 3, 6].map(divisionLetter)).toEqual(["A", "B", "C", "F"]);
  });
});
