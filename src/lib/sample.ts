import type { ExpectedStats, Player, Ship, StatLine } from "./types";

function line(battles: number, winRate: number, avgDamage: number, personalRating: number | null): StatLine {
  return {
    battles,
    wins: Math.round((battles * winRate) / 100),
    winRate,
    avgDamage,
    avgFrags: 0.9,
    avgXp: 1450,
    survivalRate: 38,
    personalRating,
  };
}

interface SampleSpec {
  name: string;
  relation: Player["relation"];
  clanTag: string | null;
  division: number | null;
  ship: Ship;
  expectedDamage: number;
  shipLine: StatLine;
  accountLine: StatLine;
}

function player(spec: SampleSpec, accountId: number): Player {
  const expected: ExpectedStats = { damage: spec.expectedDamage, frags: 0.9, winRate: 50 };
  return {
    name: spec.name,
    relation: spec.relation,
    clanTag: spec.clanTag,
    ship: spec.ship,
    shipId: spec.ship.id,
    expected,
    division: spec.division,
    stats: {
      kind: "loaded",
      accountId,
      overall: spec.accountLine,
      overallSolo: null,
      overallDiv2: null,
      overallDiv3: null,
      ship: spec.shipLine,
      shipSolo: null,
      shipDiv2: null,
      shipDiv3: null,
      shipState: "ready",
      createdAt: null,
    },
  };
}

function ship(id: number, name: string, shipClass: Ship["class"], nation: string): Ship {
  return { id, name, tier: 10, class: shipClass, nation, isPremium: false, isSpecial: false };
}

const SPECS: readonly SampleSpec[] = [
  {
    name: "Kestrel_7",
    relation: "me",
    clanTag: "RAIN",
    division: 1,
    ship: ship(1, "Des Moines", "cruiser", "usa"),
    expectedDamage: 60000,
    shipLine: line(412, 48.2, 98400, 1980),
    accountLine: line(8240, 55.3, 61200, null),
  },
  {
    name: "BlueHeron",
    relation: "ally",
    clanTag: "NAVY",
    division: null,
    ship: ship(2, "Shimakaze", "destroyer", "japan"),
    expectedDamage: 45000,
    shipLine: line(230, 61, 52700, 1180),
    accountLine: line(5120, 46.1, 38900, null),
  },
  {
    name: "Tidewalker",
    relation: "ally",
    clanTag: null,
    division: null,
    ship: ship(3, "Yamato", "battleship", "japan"),
    expectedDamage: 95000,
    shipLine: line(640, 53, 79800, 2610),
    accountLine: line(12480, 61.4, 70300, null),
  },
  {
    name: "salt_and_iron",
    relation: "ally",
    clanTag: "ANCR",
    division: null,
    ship: ship(4, "Hindenburg", "cruiser", "germany"),
    expectedDamage: 72000,
    shipLine: line(188, 58.1, 88100, 640),
    accountLine: line(3960, 50.6, 47800, null),
  },
  {
    name: "Corvid",
    relation: "ally",
    clanTag: "RAIN",
    division: 1,
    ship: ship(5, "Gearing", "destroyer", "usa"),
    expectedDamage: 52000,
    shipLine: line(355, 44, 39000, 1620),
    accountLine: line(6710, 52.6, 41500, null),
  },
];

export const SAMPLE_TEAM: readonly Player[] = SPECS.map((spec, index) => player(spec, index + 1));
