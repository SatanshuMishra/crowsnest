import type { IconName } from "../components/ui/Icon.svelte";
import type { AppearanceKey } from "./appearance";
import type { ToastRequest } from "./toast.svelte";
import type { Phase, Settings } from "./types";

export type SetupAudience = "release" | "tour";

export type Wash = "teal" | "violet" | "amber" | "green";

export type Hero = { kind: "palettes" } | { kind: "glyphs"; icons: readonly IconName[] };

export interface Feature {
  title: string;
  body: string;
  hero: Hero;
  wash: Wash;
}

export interface Release {
  version: number;
  features: readonly Feature[];
  appearanceSteps: boolean;
}

export interface AppearanceStep {
  title: string;
  keys: readonly AppearanceKey[];
}

export type Page = { kind: "feature"; feature: Feature } | { kind: "appearance"; step: AppearanceStep; number: number };

export interface SetupOutcome {
  changed: boolean;
}

export const RELEASES: readonly Release[] = [
  {
    version: 1,
    appearanceSteps: true,
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
  { title: "Theme and size", keys: ["theme", "textSize", "density"] },
  { title: "Colour", keys: ["palette", "coloredValues"] },
  { title: "Rows", keys: ["highlight", "highlightStat", "showClanTags"] },
];

export function onboardingPages(
  audience: SetupAudience,
  introSeen: Settings["introSeen"],
  releases: readonly Release[] = RELEASES,
): Page[] {
  const appearance: Page[] = APPEARANCE_STEPS.map((step, index) => ({ kind: "appearance", step, number: index + 1 }));
  if (audience === "tour") {
    return appearance;
  }
  const unseen = releases.filter((release) => release.version > (introSeen ?? 0));
  const features: Page[] = unseen.flatMap((release) =>
    release.features.map((feature): Page => ({ kind: "feature", feature })),
  );
  return unseen.some((release) => release.appearanceSteps) ? [...features, ...appearance] : features;
}

export function introAudience(
  introSeen: Settings["introSeen"],
  releases: readonly Release[] = RELEASES,
): SetupAudience | null {
  return onboardingPages("release", introSeen, releases).length > 0 ? "release" : null;
}

export function introCanOpen(phase: Phase, onMonitor: boolean): boolean {
  return onMonitor && phase === "waiting";
}

export function introInterrupted(audience: SetupAudience | null, phase: Phase | undefined): boolean {
  return audience === "release" && phase !== undefined && phase !== "waiting";
}

export function setupToast(audience: SetupAudience, outcome: SetupOutcome): ToastRequest | null {
  if (!outcome.changed) {
    return null;
  }
  return audience === "tour"
    ? { title: "Appearance saved", body: "The player table now uses your new look." }
    : { title: "Appearance saved", body: "Change it any time in Settings, Appearance." };
}
