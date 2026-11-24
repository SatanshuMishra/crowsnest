import { describe, expect, it } from "vitest";
import { REFRESH_COOLDOWN_MS, canRefresh, cooldownSeconds, refreshLabel } from "./refresh";

describe("cooldownSeconds", () => {
  it("is zero before the first refresh", () => {
    expect(cooldownSeconds(null, 5_000)).toBe(0);
  });

  it("counts the whole cooldown right after a refresh", () => {
    expect(cooldownSeconds(1_000, 1_000)).toBe(REFRESH_COOLDOWN_MS / 1000);
  });

  it("rounds a partial second up so the button never unlocks early", () => {
    expect(cooldownSeconds(0, REFRESH_COOLDOWN_MS - 200)).toBe(1);
  });

  it("ends exactly when the cooldown has passed", () => {
    expect(cooldownSeconds(0, REFRESH_COOLDOWN_MS)).toBe(0);
    expect(cooldownSeconds(0, REFRESH_COOLDOWN_MS + 50_000)).toBe(0);
  });

  it("never waits longer than one cooldown when the clock runs backwards", () => {
    expect(cooldownSeconds(60_000, 0)).toBe(REFRESH_COOLDOWN_MS / 1000);
  });
});

describe("refresh availability", () => {
  it("allows a refresh only with a battle, nothing loading and no cooldown", () => {
    expect(canRefresh({ hasBattle: true, busy: false, cooldown: 0 })).toBe(true);
    expect(canRefresh({ hasBattle: false, busy: false, cooldown: 0 })).toBe(false);
    expect(canRefresh({ hasBattle: true, busy: true, cooldown: 0 })).toBe(false);
    expect(canRefresh({ hasBattle: true, busy: false, cooldown: 3 })).toBe(false);
  });

  it("says why the button is unavailable", () => {
    expect(refreshLabel({ hasBattle: false, busy: false, cooldown: 0 })).toBe(
      "Refresh statistics, available once a battle loads",
    );
    expect(refreshLabel({ hasBattle: true, busy: true, cooldown: 8 })).toBe("Refreshing statistics");
    expect(refreshLabel({ hasBattle: true, busy: false, cooldown: 1 })).toBe(
      "Refresh statistics, available again in 1 second",
    );
    expect(refreshLabel({ hasBattle: true, busy: false, cooldown: 4 })).toBe(
      "Refresh statistics, available again in 4 seconds",
    );
    expect(refreshLabel({ hasBattle: true, busy: false, cooldown: 0 })).toBe("Refresh statistics");
  });
});
