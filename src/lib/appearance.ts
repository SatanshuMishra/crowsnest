import type { HighlightStat, Settings } from "./types";

export type AppearanceKey =
  | "theme"
  | "textSize"
  | "density"
  | "palette"
  | "coloredValues"
  | "highlight"
  | "highlightStat"
  | "showClanTags";

export type AppearanceValues = Pick<Settings, AppearanceKey>;

export interface AppearanceOption<V> {
  value: V;
  label: string;
  caption: string;
}

export interface AppearanceField<K extends AppearanceKey> {
  key: K;
  label: string;
  description: string;
  options: AppearanceOption<AppearanceValues[K]>[];
}

export type AppearanceFields = { [K in AppearanceKey]: AppearanceField<K> };

export const APPEARANCE: AppearanceFields = {
  theme: {
    key: "theme",
    label: "Theme",
    description: "System follows your Windows setting.",
    options: [
      { value: "system", label: "System", caption: "Follows your Windows setting." },
      { value: "dark", label: "Dark", caption: "Light text on near-black." },
      { value: "light", label: "Light", caption: "Dark text on white, for a bright room." },
    ],
  },
  textSize: {
    key: "textSize",
    label: "Text size",
    description: "Scales all text in Crowsnest, the player table included.",
    options: [
      { value: "small", label: "Small", caption: "Text at 90%, so more columns fit." },
      { value: "normal", label: "Normal", caption: "The standard size." },
      { value: "large", label: "Large", caption: "Text at 115%, easier to read from a distance." },
    ],
  },
  density: {
    key: "density",
    label: "Density",
    description: "How tall each player row is.",
    options: [
      { value: "comfortable", label: "Comfortable", caption: "Taller rows that are easier to scan." },
      { value: "compact", label: "Compact", caption: "Shorter rows, so more of both teams fits." },
    ],
  },
  palette: {
    key: "palette",
    label: "Colour palette",
    description: "The eight tier colours, from bad to super unicum.",
    options: [
      { value: "muted", label: "Muted", caption: "Soft colours of even strength. The default." },
      { value: "vibrant", label: "Vibrant", caption: "Brighter colours that stand out at a glance." },
      {
        value: "friendly",
        label: "Colour-blind friendly",
        caption: "Orange through blue to pink, kept apart for red-green colour blindness.",
      },
    ],
  },
  coloredValues: {
    key: "coloredValues",
    label: "Coloured values",
    description: "Colour win rates and personal rating by their tier. Every tier name stays in the tooltip either way.",
    options: [
      { value: true, label: "Coloured", caption: "Win rate and personal rating take their tier colour." },
      { value: false, label: "Plain", caption: "Numbers stay in the text colour. Tier names stay in the tooltip." },
    ],
  },
  highlight: {
    key: "highlight",
    label: "Row highlight",
    description: "Mark each player with a small tier-coloured square at the edge, or tint the whole row.",
    options: [
      { value: "none", label: "None", caption: "No row marks. Only the numbers carry tier colour." },
      { value: "edge", label: "Edge", caption: "A small tier-coloured square before each name." },
      { value: "fill", label: "Fill", caption: "A faint tint of the tier colour across the row." },
    ],
  },
  highlightStat: {
    key: "highlightStat",
    label: "Highlight by",
    description: "The statistic whose tier sets each row's highlight.",
    options: [
      { value: "personalRating", label: "Personal rating", caption: "Rows follow personal rating in this ship." },
      { value: "shipWinRate", label: "Ship win rate", caption: "Rows follow win rate in this ship." },
      { value: "accountWinRate", label: "Account win rate", caption: "Rows follow win rate across all ships." },
      { value: "averageDamage", label: "Average damage", caption: "Rows follow average damage in this ship." },
    ],
  },
  showClanTags: {
    key: "showClanTags",
    label: "Show clan tags",
    description: "Show each player's clan tag before their name.",
    options: [
      { value: true, label: "Show", caption: "Clan tags in brackets before each name." },
      { value: false, label: "Hide", caption: "Names only, with more room for long names." },
    ],
  },
};

export const HIGHLIGHT_UNAVAILABLE = "Unavailable while Row highlight is None. Choose Edge or Fill first.";
export const PALETTE_UNAVAILABLE = "Unavailable while Coloured values is off and Row highlight is None. Turn either on first.";

export function isUnavailable(key: AppearanceKey, settings: AppearanceValues): boolean {
  if (key === "highlightStat") {
    return settings.highlight === "none";
  }
  if (key === "palette") {
    return !settings.coloredValues && settings.highlight === "none";
  }
  return false;
}

export function unavailableReason(key: AppearanceKey): string | null {
  if (key === "highlightStat") {
    return HIGHLIGHT_UNAVAILABLE;
  }
  return key === "palette" ? PALETTE_UNAVAILABLE : null;
}

export function optionFor<K extends AppearanceKey>(key: K, value: AppearanceValues[K]): AppearanceOption<AppearanceValues[K]> {
  const field = APPEARANCE[key] as AppearanceField<K>;
  return field.options.find((option) => option.value === value) ?? field.options[0]!;
}

const MARKED_COLUMNS: Record<HighlightStat, string> = {
  personalRating: "shipRating",
  shipWinRate: "shipWinRate",
  accountWinRate: "accountWinRate",
  averageDamage: "shipDamage",
};

export function markedColumn(settings: Pick<Settings, "highlight" | "highlightStat">): string | null {
  return settings.highlight === "none" ? null : MARKED_COLUMNS[settings.highlightStat];
}

export function resolveTheme(theme: Settings["theme"], systemDark: boolean): "dark" | "light" {
  if (theme === "system") {
    return systemDark ? "dark" : "light";
  }
  return theme;
}
