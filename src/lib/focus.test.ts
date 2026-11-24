import { describe, expect, it, vi } from "vitest";
import { headingFocus } from "./focus";

function heading() {
  const focus = vi.fn();
  return { element: { focus } as unknown as HTMLElement, focus };
}

describe("headingFocus", () => {
  it("leaves focus alone for the first view the app shows", () => {
    const focusHeading = headingFocus();
    const first = heading();
    focusHeading(first.element);
    expect(first.focus).not.toHaveBeenCalled();
  });

  it("moves focus to the heading of every later view or step", () => {
    const focusHeading = headingFocus();
    const views = [heading(), heading(), heading()];
    views.forEach((view) => focusHeading(view.element));
    expect(views.map((view) => view.focus.mock.calls.length)).toEqual([0, 1, 1]);
  });
});
