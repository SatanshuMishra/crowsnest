import type { IconName } from "../components/ui/Icon.svelte";
import type { AppearanceKey } from "./appearance";
import type { ToastRequest } from "./toast.svelte";
import type { Phase, Settings } from "./types";

export type SetupAudience = "new" | "updating" | "tour";

export type Wash = "teal" | "violet" | "amber" | "green";

export type Hero = { kind: "table" } | { kind: "palettes" } | { kind: "glyphs"; icons: readonly IconName[] };

export interface Feature {
  title: string;
  body: string;
  hero: Hero;
  wash: Wash;
}

export interface Release {
  version: number;
  features: readonly Feature[];
  appearanceForEveryone: boolean;
}

export interface AppearanceStep {
  title: string;
  body: string;
  keys: readonly AppearanceKey[];
  wash: Wash;
}

export type Page =
  | { kind: "welcome" }
  | { kind: "feature"; feature: Feature }
  | { kind: "appearance"; step: AppearanceStep; number: number };

export interface SetupOutcome {
  changed: boolean;
  finished: boolean;
}

export const RELEASES: readonly Release[] = [
  {
    version: 1,
    appearanceForEveryone: true,
    features: [
      {
        title: "Colour palettes",
        body: "Show tiers in Muted, Vibrant or Colour-blind friendly colours.",
        hero: { kind: "palettes" },
        wash: "violet",
      },
      {
        title: "Live previews",
        body: "In Settings, point at an appearance option to see the player table in it before you choose.",
        hero: { kind: "glyphs", icons: ["eye"] },
        wash: "teal",
      },
      {
        title: "Keep on top",
        body: "The pin in the title bar keeps Crowsnest above the game, without a trip to Settings.",
        hero: { kind: "glyphs", icons: ["pin"] },
        wash: "amber",
      },
    ],
  },
];

export const INTRO_VERSION = RELEASES[RELEASES.length - 1]?.version ?? 0;

export const APPEARANCE_STEPS: readonly AppearanceStep[] = [
  {
    title: "Theme and size",
    body: "Choose the theme, the text size and how tall each row is.",
    keys: ["theme", "textSize", "density"],
    wash: "teal",
  },
  {
    title: "Colour",
    body: "Choose the tier palette, and whether win rate and rating take tier colours.",
    keys: ["palette", "coloredValues"],
    wash: "violet",
  },
  {
    title: "Rows",
    body: "Choose how each player is marked by tier, and which statistic sets the mark.",
    keys: ["highlight", "highlightStat", "showClanTags"],
    wash: "green",
  },
];

const CHANGE_IT_LATER = "Change it any time in Settings, Appearance.";

export function onboardingPages(
  audience: SetupAudience,
  introSeen: Settings["introSeen"],
  releases: readonly Release[] = RELEASES,
): Page[] {
  const appearance: Page[] = APPEARANCE_STEPS.map((step, index) => ({ kind: "appearance", step, number: index + 1 }));
  if (audience === "tour") {
    return appearance;
  }
  if (audience === "new") {
    return [{ kind: "welcome" }, ...appearance];
  }
  const unseen = releases.filter((release) => release.version > (introSeen ?? 0));
  const features: Page[] = unseen.flatMap((release) =>
    release.features.map((feature): Page => ({ kind: "feature", feature })),
  );
  return unseen.some((release) => release.appearanceForEveryone) ? [...features, ...appearance] : features;
}

export function introAudience(
  introSeen: Settings["introSeen"],
  releases: readonly Release[] = RELEASES,
): SetupAudience | null {
  const latest = releases[releases.length - 1]?.version ?? 0;
  if (introSeen !== null && introSeen >= latest) {
    return null;
  }
  const audience = introSeen === 0 ? "new" : "updating";
  return onboardingPages(audience, introSeen, releases).length > 0 ? audience : null;
}

export function introCanOpen(phase: Phase, onMonitor: boolean): boolean {
  return onMonitor && phase === "waiting";
}

export function introInterrupted(audience: SetupAudience | null, phase: Phase | undefined): boolean {
  return audience !== null && audience !== "tour" && phase !== undefined && phase !== "waiting";
}

export function skipLabel(audience: SetupAudience, changed: boolean): string | null {
  if (audience === "tour") {
    return null;
  }
  return audience === "new" && !changed ? "Use the defaults" : "Skip";
}

export function setupToast(audience: SetupAudience, outcome: SetupOutcome): ToastRequest | null {
  if (audience === "tour") {
    return outcome.changed ? { title: "Appearance saved", body: "The player table now uses your new look." } : null;
  }
  if (audience === "updating") {
    return outcome.changed ? { title: "Appearance saved", body: CHANGE_IT_LATER } : null;
  }
  if (outcome.changed || outcome.finished) {
    return { title: "Appearance saved", body: CHANGE_IT_LATER };
  }
  return { title: "Using the default look", body: CHANGE_IT_LATER };
}
