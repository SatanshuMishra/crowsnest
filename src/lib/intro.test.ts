import { describe, expect, it } from "vitest";
import { INTRO_VERSION, introAudience, introCanOpen, introInterrupted, setupToast } from "./intro";

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
  it("gives way to a battle that starts while the welcome or the changelog is open", () => {
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

describe("setupToast", () => {
  const untouched = { changed: false, finished: false };

  it("confirms a saved look whichever way the setup ended", () => {
    expect(setupToast("new", { changed: true, finished: false })?.title).toBe("Appearance saved");
    expect(setupToast("new", { changed: false, finished: true })?.title).toBe("Appearance saved");
    expect(setupToast("tour", { changed: true, finished: true })?.body).toBe("The player table now uses your new look.");
  });

  it("tells someone who skipped what they kept", () => {
    expect(setupToast("new", untouched)?.title).toBe("Using the default look");
  });

  it("stays quiet when a tour from Settings changed nothing", () => {
    expect(setupToast("tour", untouched)).toBeNull();
    expect(setupToast("tour", { changed: false, finished: true })).toBeNull();
  });
});
