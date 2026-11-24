import { describe, expect, it } from "vitest";
import { windowGesture } from "./window";

describe("windowGesture", () => {
  it("drags the window from a single primary press on the bar", () => {
    expect(windowGesture(0, 1, false)).toBe("drag");
  });

  it("maximizes or restores the window on a double press", () => {
    expect(windowGesture(0, 2, false)).toBe("toggleMaximize");
  });

  it("leaves presses on controls and other buttons to the page", () => {
    expect(windowGesture(0, 1, true)).toBeNull();
    expect(windowGesture(2, 1, false)).toBeNull();
    expect(windowGesture(1, 2, false)).toBeNull();
  });
});
