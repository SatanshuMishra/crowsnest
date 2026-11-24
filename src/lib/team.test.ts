import { describe, expect, it } from "vitest";
import type { Player, PlayerStats, StatLine } from "./types";
import {
  WEIGHT_CAP,
  averageName,
  averageOf,
  columnValue,
  coverageLabel,
  median,
  playerLines,
  statColumns,
  summarizeTeam,
} from "./team";

function line(change: Partial<StatLine> = {}): StatLine {
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

function loaded(change: Partial<Extract<PlayerStats, { kind: "loaded" }>> = {}): PlayerStats {
  return {
    kind: "loaded",
    accountId: 1,
    overall: line(),
    overallSolo: null,
    ship: line(),
    shipSolo: null,
    shipState: "ready",
    createdAt: null,
    ...change,
  };
}

function player(stats: PlayerStats, name = "Captain"): Player {
  return {
    name,
    relation: "ally",
    clanTag: null,
    ship: null,
    shipId: 1,
    expected: null,
    division: null,
    stats,
  };
}

describe("playerLines", () => {
  it("returns nothing for players without loaded stats", () => {
    expect(playerLines({ kind: "pending" }, false)).toBeNull();
    expect(playerLines({ kind: "bot" }, false)).toBeNull();
    expect(playerLines({ kind: "hidden", accountId: 3 }, false)).toBeNull();
  });

  it("reads the solo lines when asked, falling back to the full lines", () => {
    const overallSolo = line({ winRate: 60 });
    const stats = loaded({ overallSolo, shipSolo: null });
    const lines = playerLines(stats, true);
    expect(lines?.overall).toBe(overallSolo);
    expect(lines?.ship?.winRate).toBe(50);
    expect(playerLines(stats, false)?.overall.winRate).toBe(50);
  });

  it("drops a ship line without battles", () => {
    expect(playerLines(loaded({ ship: line({ battles: 0 }) }), false)?.ship).toBeNull();
    expect(playerLines(loaded({ ship: null }), false)?.ship).toBeNull();
  });
});

describe("statColumns", () => {
  it("lists the ship and account columns", () => {
    const columns = statColumns("comfortable");
    expect(columns.ship.map((column) => column.short)).toEqual(["Battles", "WR", "PR", "Dmg", "Frags", "XP"]);
    expect(columns.account.map((column) => column.short)).toEqual(["Battles", "WR"]);
  });

  it("hides frags and experience when compact", () => {
    expect(statColumns("compact").ship.map((column) => column.kind)).toEqual([
      "battles",
      "winRate",
      "personalRating",
      "damage",
    ]);
  });

  it("reads values from the matching line", () => {
    const lines = { overall: line({ battles: 9000 }), ship: line({ avgDamage: 51234 }) };
    const { ship, account } = statColumns("comfortable");
    const damage = ship.find((column) => column.kind === "damage");
    const battles = account.find((column) => column.kind === "battles");
    expect(damage && columnValue(damage, lines)).toBe(51234);
    expect(battles && columnValue(battles, lines)).toBe(9000);
    expect(damage && columnValue(damage, { ...lines, ship: null })).toBeNull();
  });
});

describe("averages", () => {
  it("computes the mean", () => {
    expect(averageOf([{ value: 50, battles: 10 }, { value: 60, battles: 500 }], "mean")).toBe(55);
  });

  it("weights by battles capped at the limit per player", () => {
    expect(WEIGHT_CAP).toBe(40);
    const samples = [
      { value: 40, battles: 10 },
      { value: 60, battles: 1000 },
    ];
    expect(averageOf(samples, "weighted")).toBe(56);
    expect(averageOf([{ value: 50, battles: 0 }], "weighted")).toBeNull();
  });

  it("takes the median of odd and even counts", () => {
    expect(median([70, 40, 50])).toBe(50);
    expect(median([40, 70, 50, 60])).toBe(55);
    expect(median([])).toBeNull();
  });

  it("has no average without samples", () => {
    expect(averageOf([], "mean")).toBeNull();
    expect(averageOf([], "median")).toBeNull();
  });

  it("names each method", () => {
    expect(averageName("mean")).toBe("Mean");
    expect(averageName("median")).toBe("Median");
  });
});

describe("summarizeTeam", () => {
  const players = [
    player(loaded({ overall: line({ winRate: 50, battles: 1000 }), ship: line({ winRate: 40, battles: 10, personalRating: 1000 }) })),
    player(loaded({ overall: line({ winRate: 60, battles: 3000 }), ship: line({ winRate: 60, battles: 1000, personalRating: 2000 }) })),
    player(loaded({ overall: line({ winRate: 55, battles: 2000 }), ship: null })),
    player({ kind: "bot" }),
    player({ kind: "pending" }),
  ];

  it("averages loaded players by the mean", () => {
    const summary = summarizeTeam(players, "mean", false);
    expect(summary.personalRating).toBe(1500);
    expect(summary.shipWinRate).toBe(50);
    expect(summary.accountWinRate).toBe(55);
    expect(summary.battles).toBe(2000);
    expect(summary.withStats).toBe(3);
    expect(summary.total).toBe(5);
    expect(coverageLabel(summary)).toBe("3 of 5 with stats");
  });

  it("weights win rates and rating but keeps the battle count a mean", () => {
    const summary = summarizeTeam(players, "weighted", false);
    expect(summary.shipWinRate).toBe(56);
    expect(summary.personalRating).toBe(1800);
    expect(summary.accountWinRate).toBe(55);
    expect(summary.battles).toBe(2000);
  });

  it("takes medians", () => {
    const summary = summarizeTeam(players, "median", false);
    expect(summary.accountWinRate).toBe(55);
    expect(summary.shipWinRate).toBe(50);
    expect(summary.battles).toBe(2000);
  });

  it("reads solo lines when solo statistics are on", () => {
    const solo = [player(loaded({ overall: line({ winRate: 50 }), overallSolo: line({ winRate: 70 }) }))];
    expect(summarizeTeam(solo, "mean", true).accountWinRate).toBe(70);
    expect(summarizeTeam(solo, "mean", false).accountWinRate).toBe(50);
  });

  it("is empty for a team without stats", () => {
    const summary = summarizeTeam([player({ kind: "pending" })], "mean", false);
    expect(summary.personalRating).toBeNull();
    expect(summary.shipWinRate).toBeNull();
    expect(summary.accountWinRate).toBeNull();
    expect(summary.battles).toBeNull();
    expect(coverageLabel(summary)).toBe("0 of 1 with stats");
  });
});

describe("statColumns by table width", () => {
  const kinds = (width: number) => statColumns("comfortable", width).ship.map((column) => column.kind);

  it("shows every ship column in a wide table", () => {
    expect(kinds(900)).toEqual(["battles", "winRate", "personalRating", "damage", "frags", "xp"]);
  });

  it("drops experience first as the table narrows", () => {
    expect(kinds(780)).toEqual(["battles", "winRate", "personalRating", "damage", "frags"]);
  });

  it("drops frags too in a narrow table", () => {
    expect(kinds(620)).toEqual(["battles", "winRate", "personalRating", "damage"]);
  });

  it("keeps the account columns at any width", () => {
    expect(statColumns("comfortable", 400).account.map((column) => column.id)).toEqual([
      "accountBattles",
      "accountWinRate",
    ]);
  });
});
