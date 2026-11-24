import { APPEARANCE, type AppearanceKey } from "./appearance";

export type SectionKey = "game" | "statistics" | "appearance" | "window" | "updates" | "about";

export interface SectionInfo {
  key: SectionKey;
  title: string;
  summary: string;
  description: string;
}

export const SECTIONS: readonly SectionInfo[] = [
  {
    key: "game",
    title: "Game",
    summary: "Game folder and server",
    description: "Where Crowsnest finds the game, its replays and your server.",
  },
  {
    key: "statistics",
    title: "Statistics",
    summary: "Wargaming ID and team averages",
    description: "Crowsnest reads public statistics from the Wargaming API.",
  },
  {
    key: "appearance",
    title: "Appearance",
    summary: "Theme, colours and rows",
    description: "How the player table looks. Point at an option, or focus it, to preview it before you choose.",
  },
  { key: "window", title: "Window", summary: "Always on top and tray", description: "How the Crowsnest window behaves." },
  {
    key: "updates",
    title: "Updates",
    summary: "Version and new releases",
    description: "New releases come from the Crowsnest page on GitHub.",
  },
  { key: "about", title: "About", summary: "Links, logs and licence", description: "" },
];

export interface SettingEntry {
  key: string;
  label: string;
  section: SectionKey;
  description: string;
  terms: string;
}

const APPEARANCE_TERMS: Record<AppearanceKey, string> = {
  theme: "dark light mode",
  textSize: "font zoom bigger smaller",
  density: "compact spacing row height",
  palette: "colour color vibrant muted saturated colour-blind colorblind type",
  coloredValues: "colour color tier numbers",
  highlight: "row tint edge fill mark",
  highlightStat: "personal rating win rate damage",
  showClanTags: "clan tag",
};

const APPEARANCE_ENTRIES: SettingEntry[] = (Object.keys(APPEARANCE_TERMS) as AppearanceKey[]).map((key) => ({
  key,
  label: APPEARANCE[key].label,
  section: "appearance",
  description: APPEARANCE[key].description,
  terms: APPEARANCE_TERMS[key],
}));

export const SETTING_ENTRIES: readonly SettingEntry[] = [
  {
    key: "installPath",
    label: "Game folder",
    section: "game",
    description: "Detect the game again, or choose the folder yourself.",
    terms: "install path replays folder detect",
  },
  {
    key: "region",
    label: "Server",
    section: "game",
    description: "Automatic reads the server from the game.",
    terms: "region eu na asia",
  },
  {
    key: "applicationId",
    label: "Application ID",
    section: "statistics",
    description: "Use your own Wargaming application ID, or the built-in one.",
    terms: "wargaming api key id developer",
  },
  {
    key: "soloStats",
    label: "Solo statistics",
    section: "statistics",
    description: "Count only battles played without a division.",
    terms: "division solo",
  },
  {
    key: "teamAverage",
    label: "Team average",
    section: "statistics",
    description: "How the team summary combines its players.",
    terms: "mean median weighted average summary",
  },
  ...APPEARANCE_ENTRIES,
  {
    key: "tour",
    label: "Appearance tour",
    section: "appearance",
    description: "Walk through the appearance options with a live preview.",
    terms: "tour onboarding guide walkthrough setup",
  },
  {
    key: "alwaysOnTop",
    label: "Always on top",
    section: "window",
    description: "Keep Crowsnest above other windows. The pin in the title bar does the same.",
    terms: "pin top float above overlay",
  },
  {
    key: "closeToTray",
    label: "Close to tray",
    section: "window",
    description: "Closing the window hides Crowsnest in the notification area. Quit it from the tray menu.",
    terms: "tray minimise minimize close background",
  },
  {
    key: "checkForUpdates",
    label: "Check for updates automatically",
    section: "updates",
    description: "Look for a new release a few seconds after Crowsnest starts.",
    terms: "update release version",
  },
  {
    key: "logs",
    label: "Open log folder",
    section: "about",
    description: "Attach the latest log when you report a problem.",
    terms: "log logs problem bug report support",
  },
];

export function searchSettings(query: string): SettingEntry[] {
  const needle = query.trim().toLowerCase();
  if (needle === "") {
    return [];
  }
  return SETTING_ENTRIES.filter((entry) =>
    `${entry.label} ${entry.description} ${entry.terms}`.toLowerCase().includes(needle),
  );
}

export function sectionTitle(key: SectionKey): string {
  return SECTIONS.find((section) => section.key === key)?.title ?? key;
}

export function resultsTitle(query: string, count: number): string {
  const shown = query.trim();
  if (count === 0) {
    return `No setting matches “${shown}”`;
  }
  return count === 1 ? `1 setting matches “${shown}”` : `${count} settings match “${shown}”`;
}
