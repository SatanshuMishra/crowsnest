import { describe, expect, it } from "vitest";
import { placePopover } from "./popover";

const viewport = { width: 1280, height: 760, top: 52, margin: 16 };
const size = { width: 580, height: 320 };

describe("placePopover", () => {
  it("opens below a row with room underneath, right-aligned to it", () => {
    const placement = placePopover({ top: 200, bottom: 240, left: 300, right: 1000 }, viewport, size);
    expect(placement).toEqual({ side: "below", top: 240, bottom: null, left: 420, width: 580, maxHeight: 504 });
  });

  it("opens above a row near the bottom of the window", () => {
    const placement = placePopover({ top: 600, bottom: 640, left: 300, right: 1000 }, viewport, size);
    expect(placement.side).toBe("above");
    expect(placement.bottom).toBe(160);
    expect(placement.top).toBeNull();
    expect(placement.maxHeight).toBe(532);
  });

  it("stays below and scrolls when neither side has the full height but below has more", () => {
    const placement = placePopover({ top: 100, bottom: 140, left: 0, right: 600 }, { ...viewport, height: 400 }, size);
    expect(placement.side).toBe("below");
    expect(placement.maxHeight).toBe(244);
  });

  it("keeps the popover inside a narrow window", () => {
    const placement = placePopover({ top: 200, bottom: 240, left: 0, right: 300 }, { ...viewport, width: 480 }, size);
    expect(placement.width).toBe(448);
    expect(placement.left).toBe(16);
  });

  it("never runs past the right edge", () => {
    const placement = placePopover({ top: 200, bottom: 240, left: 900, right: 1300 }, viewport, size);
    expect(placement.left + placement.width).toBe(1264);
  });
});
