import { describe, expect, it } from "vitest";
import {
  INTRO_VERSION,
  RELEASES,
  introAudience,
  introCanOpen,
  introInterrupted,
  onboardingPages,
  setupToast,
  type Page,
  type Release,
} from "./intro";

const NEXT_RELEASE: Release = {
  version: 2,
  appearanceSteps: false,
  features: [
    {
      title: "Division marks",
      body: "Players who queued together share a mark.",
      hero: { kind: "glyphs", icons: ["user"] },
      wash: "green",
    },
  ],
};

const LATER = [...(RELEASES as Release[]), NEXT_RELEASE];

const THIS_RELEASE = ["Colour palettes", "Live previews", "Keep on top", "step 1", "step 2", "step 3"];

function titles(pages: Page[]): string[] {
  return pages.map((page) => (page.kind === "feature" ? page.feature.title : `step ${page.number}`));
}

describe("onboardingPages", () => {
  it("shows every player what is new and then the appearance steps, whether they installed or updated", () => {
    expect(titles(onboardingPages("release", null))).toEqual(THIS_RELEASE);
    expect(titles(onboardingPages("release", 0))).toEqual(THIS_RELEASE);
  });

  it("gives a tour from Settings the appearance steps alone", () => {
    expect(titles(onboardingPages("tour", INTRO_VERSION))).toEqual(["step 1", "step 2", "step 3"]);
  });

  it("shows only the features of a later release to someone who already saw this one", () => {
    expect(titles(onboardingPages("release", 1, LATER))).toEqual(["Division marks"]);
  });

  it("keeps the appearance steps for someone who never saw this release", () => {
    expect(titles(onboardingPages("release", null, LATER))).toEqual([
      "Colour palettes",
      "Live previews",
      "Keep on top",
      "Division marks",
      "step 1",
      "step 2",
      "step 3",
    ]);
  });
});

describe("introAudience", () => {
  it("opens the release onboarding for anyone who has not seen the latest release", () => {
    expect(introAudience(null)).toBe("release");
    expect(introAudience(0)).toBe("release");
    expect(introAudience(1, LATER)).toBe("release");
  });

  it("stays closed once the latest release has been seen", () => {
    expect(introAudience(INTRO_VERSION)).toBeNull();
    expect(introAudience(INTRO_VERSION + 1)).toBeNull();
    expect(introAudience(2, LATER)).toBeNull();
  });

  it("stays closed for a release with nothing to show", () => {
    const quiet = [...LATER, { version: 3, appearanceSteps: false, features: [] }];
    expect(introAudience(2, quiet)).toBeNull();
  });
});

describe("introCanOpen", () => {
  it("opens only on the monitor while waiting for a battle", () => {
    expect(introCanOpen("waiting", true)).toBe(true);
    expect(introCanOpen("inBattle", true)).toBe(false);
    expect(introCanOpen("loading", true)).toBe(false);
    expect(introCanOpen("needsInstall", true)).toBe(false);
    expect(introCanOpen("waiting", false)).toBe(false);
  });
});

describe("introInterrupted", () => {
  it("gives way to a battle that starts while the release onboarding is open", () => {
    expect(introInterrupted("release", "loading")).toBe(true);
    expect(introInterrupted("release", "inBattle")).toBe(true);
  });

  it("leaves a tour from Settings and a quiet monitor alone", () => {
    expect(introInterrupted("tour", "loading")).toBe(false);
    expect(introInterrupted("release", "waiting")).toBe(false);
    expect(introInterrupted(null, "loading")).toBe(false);
    expect(introInterrupted("release", undefined)).toBe(false);
  });
});

describe("setupToast", () => {
  it("confirms a changed look", () => {
    expect(setupToast("release", { changed: true })).toEqual({
      title: "Appearance saved",
      body: "Change it any time in Settings, Appearance.",
    });
    expect(setupToast("tour", { changed: true })?.body).toBe("The player table now uses your new look.");
  });

  it("stays quiet when nothing changed", () => {
    expect(setupToast("release", { changed: false })).toBeNull();
    expect(setupToast("tour", { changed: false })).toBeNull();
  });
});
