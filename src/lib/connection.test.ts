import { describe, expect, it } from "vitest";
import { connectionStatus } from "./connection";

describe("connectionStatus", () => {
  it("says Connected while Crowsnest watches the game, with what it is doing", () => {
    expect(connectionStatus({ phase: "waiting", connected: true })).toEqual({
      connected: true,
      label: "Connected",
      detail: "Waiting for battle",
      activity: "listening",
    });
    expect(connectionStatus({ phase: "loading", connected: true }).detail).toBe("Loading stats");
    expect(connectionStatus({ phase: "inBattle", connected: true }).activity).toBe("steady");
  });

  it("says Not connected with the reason when nothing is being watched", () => {
    const reasons = (["needsInstall", "needsApplicationId", "replaysDisabled", "waiting"] as const).map(
      (phase) => connectionStatus({ phase, connected: false }),
    );
    expect(reasons.map((status) => status.label)).toEqual(Array(4).fill("Not connected"));
    expect(reasons.map((status) => status.detail)).toEqual([
      "Game not found",
      "Setup needed",
      "Replays off",
      "Replays folder unavailable",
    ]);
  });
});
