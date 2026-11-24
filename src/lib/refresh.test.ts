import { describe, expect, it } from "vitest";
import {
  REFRESH_COOLDOWN_MS,
  canRefresh,
  cooldownSeconds,
  followRefresh,
  refreshLabel,
  refreshToast,
  type WatchedState,
} from "./refresh";
import { SAMPLE_TEAM } from "./sample";
import type { Player } from "./types";

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

describe("refreshToast", () => {
  const retry = () => undefined;
  const bot: Player = { ...SAMPLE_TEAM[0]!, name: "Bot", stats: { kind: "bot" } };

  it("counts only the players whose statistics loaded", () => {
    const toast = refreshToast(null, [{ players: [...SAMPLE_TEAM] }, { players: [bot] }], retry);
    expect(toast.tone).toBeUndefined();
    expect(toast.title).toBe("Statistics refreshed");
    expect(toast.body).toBe(`Updated ${SAMPLE_TEAM.length} players from Wargaming.`);
  });

  it("uses the singular for one player", () => {
    expect(refreshToast(null, [{ players: [SAMPLE_TEAM[0]!] }], retry).body).toBe("Updated 1 player from Wargaming.");
  });

  it("reports an error notice as a failure that can be retried", () => {
    const toast = refreshToast({ level: "error", message: "Wargaming did not answer." }, [], retry);
    expect(toast.tone).toBe("error");
    expect(toast.body).toBe("Wargaming did not answer.");
    expect(toast.action?.label).toBe("Try again");
  });

  it("treats a warning as a refresh that still finished", () => {
    expect(refreshToast({ level: "warning", message: "Some ships failed." }, [], retry).title).toBe(
      "Statistics refreshed",
    );
  });
});

describe("followRefresh", () => {
  const loaded = [...SAMPLE_TEAM];
  const pending: Player[] = loaded.map((player) => ({ ...player, stats: { kind: "pending" } }));
  const state = (phase: WatchedState["phase"], players: Player[], id = "battle-1"): WatchedState => ({
    phase,
    battle: { id, allies: { players }, enemies: { players: [] } },
  });
  const fresh = { battleId: "battle-1", started: false };

  it("finishes after the phase passes through loading", () => {
    const during = followRefresh(fresh, state("loading", loaded));
    expect(during).toEqual({ kind: "watching", watch: { battleId: "battle-1", started: true } });
    expect(followRefresh({ battleId: "battle-1", started: true }, state("waiting", loaded))).toEqual({
      kind: "finished",
    });
  });

  it("finishes a refresh of a finished battle that only shows pending statistics", () => {
    expect(followRefresh(fresh, state("waiting", pending))).toEqual({
      kind: "watching",
      watch: { battleId: "battle-1", started: true },
    });
    expect(followRefresh({ battleId: "battle-1", started: true }, state("waiting", loaded)).kind).toBe("finished");
  });

  it("keeps waiting while ship statistics are still arriving", () => {
    const arriving: Player[] = loaded.map((player) =>
      player.stats.kind === "loaded" ? { ...player, stats: { ...player.stats, shipState: "pending" } } : player,
    );
    expect(followRefresh({ battleId: "battle-1", started: true }, state("waiting", arriving)).kind).toBe("watching");
  });

  it("waits for the refresh to begin before calling it finished", () => {
    expect(followRefresh(fresh, state("waiting", loaded))).toEqual({ kind: "watching", watch: fresh });
  });

  it("abandons the refresh when another battle arrives, so no toast reports it", () => {
    expect(followRefresh({ battleId: "battle-1", started: true }, state("loading", pending, "battle-2"))).toEqual({
      kind: "abandoned",
    });
  });
});
