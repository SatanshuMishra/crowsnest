export type Region = "eu" | "na" | "asia";
export type InstallSource = "gameCenter" | "steam" | "defaultPath" | "manual";
export type Relation = "me" | "ally" | "enemy";
export type ShipClass = "destroyer" | "cruiser" | "battleship" | "airCarrier" | "submarine" | "unknown";

export interface GameInstall {
  root: string;
  source: InstallSource;
  build: number | null;
  clientVersion: string | null;
  replaysDir: string;
  replaysEnabled: boolean;
  region: Region | null;
}

export interface Ship {
  id: number;
  name: string;
  tier: number;
  class: ShipClass;
  nation: string;
  isPremium: boolean;
  isSpecial: boolean;
}

export interface ArenaVehicle {
  playerId: number;
  name: string;
  relation: Relation;
  shipId: number;
}

export interface ArenaInfo {
  startedAt: string;
  mapId: number | null;
  mapName: string;
  mapDisplayName: string;
  matchGroup: string;
  gameType: string;
  scenario: string;
  playersPerTeam: number;
  playerName: string;
  playerVehicle: string;
  clientVersion: string;
  vehicles: ArenaVehicle[];
}

export interface StatLine {
  battles: number;
  wins: number;
  winRate: number | null;
  avgDamage: number | null;
  avgFrags: number | null;
  avgXp: number | null;
  survivalRate: number | null;
  personalRating: number | null;
}

export type PlayerStats =
  | { kind: "pending" }
  | { kind: "bot" }
  | { kind: "hidden"; accountId: number }
  | { kind: "notFound" }
  | { kind: "failed"; reason: string }
  | {
      kind: "loaded";
      accountId: number;
      overall: StatLine;
      overallSolo: StatLine | null;
      ship: StatLine | null;
      shipSolo: StatLine | null;
      createdAt: number | null;
    };

export interface ExpectedStats {
  damage: number;
  frags: number;
  winRate: number;
}

export interface Player {
  name: string;
  relation: Relation;
  clanTag: string | null;
  ship: Ship | null;
  shipId: number;
  expected: ExpectedStats | null;
  division: number | null;
  stats: PlayerStats;
}

export interface Team {
  players: Player[];
}

export interface Battle {
  id: string;
  region: Region | null;
  arena: ArenaInfo;
  allies: Team;
  enemies: Team;
}

export type Phase =
  | "needsInstall"
  | "needsApplicationId"
  | "replaysDisabled"
  | "waiting"
  | "loading"
  | "inBattle";

export type NoticeLevel = "info" | "warning" | "error";

export interface Notice {
  level: NoticeLevel;
  message: string;
}

export interface Snapshot {
  phase: Phase;
  install: GameInstall | null;
  battle: Battle | null;
  notice: Notice | null;
}

export type Theme = "system" | "dark" | "light";
export type Density = "comfortable" | "compact";
export type TextSize = "small" | "normal" | "large";
export type Highlight = "none" | "edge" | "fill";
export type HighlightStat = "personalRating" | "shipWinRate" | "accountWinRate" | "averageDamage";
export type TeamAverage = "mean" | "weighted" | "median";

export interface Settings {
  installPath: string | null;
  replaysDirOverride: string | null;
  regionOverride: Region | null;
  applicationId: string | null;
  theme: Theme;
  density: Density;
  textSize: TextSize;
  coloredValues: boolean;
  highlight: Highlight;
  highlightStat: HighlightStat;
  teamAverage: TeamAverage;
  soloStats: boolean;
  showClanTags: boolean;
  alwaysOnTop: boolean;
  closeToTray: boolean;
  checkForUpdates: boolean;
}

export interface AppInfo {
  version: string;
  hasBundledApplicationId: boolean;
}
