import { describe, expect, it } from "vitest";
import {
  APPEARANCE_STEPS,
  INTRO_VERSION,
  RELEASES,
  introAudience,
  introCanOpen,
  introInterrupted,
  onboardingPages,
  setupToast,
  skipLabel,
  type Page,
  type Release,
} from "./intro";

const NEXT_RELEASE: Release = {
  version: 2,
  appearanceForEveryone: false,
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

function kinds(pages: Page[]): string[] {
  return pages.map((page) =>
    page.kind === "feature" ? page.feature.title : page.kind === "appearance" ? `step ${page.number}` : "welcome",
  );
}

describe("onboardingPages", () => {
  it("welcomes a new player and walks them through every appearance step", () => {
    expect(kinds(onboardingPages("new", 0))).toEqual(["welcome", "step 1", "step 2", "step 3"]);
  });

  it("gives a tour from Settings the appearance steps alone", () => {
    expect(kinds(onboardingPages("tour", INTRO_VERSION))).toEqual(["step 1", "step 2", "step 3"]);
  });

  it("shows everyone updating to this version what is new and then the appearance steps", () => {
    expect(kinds(onboardingPages("updating", null))).toEqual([
      "Colour palettes",
      "Live previews",
      "Keep on top",
      "step 1",
      "step 2",
      "step 3",
    ]);
  });

  it("shows only the features of a later release to someone who already saw this one", () => {
    expect(kinds(onboardingPages("updating", 1, LATER))).toEqual(["Division marks"]);
  });

  it("keeps the appearance steps for someone who skipped this version entirely", () => {
    const pages = kinds(onboardingPages("updating", null, LATER));
    expect(pages.slice(0, 4)).toEqual(["Colour palettes", "Live previews", "Keep on top", "Division marks"]);
    expect(pages.slice(4)).toEqual(APPEARANCE_STEPS.map((_, index) => `step ${index + 1}`));
  });

  it("still welcomes a new player with the appearance steps in a later release", () => {
    expect(kinds(onboardingPages("new", 0, LATER))).toEqual(["welcome", "step 1", "step 2", "step 3"]);
  });
});

describe("introAudience", () => {
  it("welcomes a fresh install", () => {
    expect(introAudience(0)).toBe("new");
  });

  it("shows what is new to someone whose settings predate the intro", () => {
    expect(introAudience(null)).toBe("updating");
  });

  it("shows nothing once the current intro has been seen", () => {
    expect(introAudience(INTRO_VERSION)).toBeNull();
    expect(introAudience(INTRO_VERSION + 1)).toBeNull();
  });

  it("follows the latest release", () => {
    expect(introAudience(1, LATER)).toBe("updating");
    expect(introAudience(2, LATER)).toBeNull();
  });

  it("stays closed for a release with nothing to show an updating player", () => {
    const quiet = [...LATER, { version: 3, appearanceForEveryone: false, features: [] }];
    expect(introAudience(2, quiet)).toBeNull();
    expect(introAudience(0, quiet)).toBe("new");
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
  it("gives way to a battle that starts while the onboarding is open", () => {
    expect(introInterrupted("new", "loading")).toBe(true);
    expect(introInterrupted("updating", "inBattle")).toBe(true);
  });

  it("leaves a tour from Settings and a quiet monitor alone", () => {
    expect(introInterrupted("tour", "loading")).toBe(false);
    expect(introInterrupted("new", "waiting")).toBe(false);
    expect(introInterrupted(null, "loading")).toBe(false);
    expect(introInterrupted("updating", undefined)).toBe(false);
  });
});

describe("skipLabel", () => {
  it("offers the defaults only while a new player has changed nothing", () => {
    expect(skipLabel("new", false)).toBe("Use the defaults");
    expect(skipLabel("new", true)).toBe("Skip");
  });

  it("lets an updating player skip and gives a tour no skip", () => {
    expect(skipLabel("updating", false)).toBe("Skip");
    expect(skipLabel("tour", false)).toBeNull();
  });
});

describe("setupToast", () => {
  const untouched = { changed: false, finished: false };

  it("confirms a saved look whichever way a new player ended the setup", () => {
    expect(setupToast("new", { changed: true, finished: false })?.title).toBe("Appearance saved");
    expect(setupToast("new", { changed: false, finished: true })?.title).toBe("Appearance saved");
    expect(setupToast("tour", { changed: true, finished: true })?.body).toBe("The player table now uses your new look.");
  });

  it("tells a new player who skipped what they kept", () => {
    expect(setupToast("new", untouched)?.title).toBe("Using the default look");
  });

  it("speaks to an updating player only when their look changed", () => {
    expect(setupToast("updating", { changed: true, finished: true })?.title).toBe("Appearance saved");
    expect(setupToast("updating", { changed: false, finished: true })).toBeNull();
    expect(setupToast("updating", untouched)).toBeNull();
  });

  it("stays quiet when a tour from Settings changed nothing", () => {
    expect(setupToast("tour", untouched)).toBeNull();
    expect(setupToast("tour", { changed: false, finished: true })).toBeNull();
  });
});
