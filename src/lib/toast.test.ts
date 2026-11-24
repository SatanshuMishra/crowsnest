import { describe, expect, it } from "vitest";
import { ToastStore } from "./toast.svelte";

describe("ToastStore", () => {
  it("shows a success toast by default", () => {
    const store = new ToastStore();
    const shown = store.show({ title: "Appearance saved", body: "The player table now uses your new look." });
    expect(store.current).toEqual({ ...shown, tone: "success", action: null });
  });

  it("replaces the toast on screen with the newest one", () => {
    const store = new ToastStore();
    const first = store.show({ title: "Statistics refreshed", body: "Updated 12 players." });
    const second = store.show({ tone: "error", title: "Refresh failed", body: "Try again in a minute." });
    expect(second.id).toBeGreaterThan(first.id);
    expect(store.current?.title).toBe("Refresh failed");
  });

  it("ignores a dismissal meant for a toast that was already replaced", () => {
    const store = new ToastStore();
    const first = store.show({ title: "One", body: "" });
    store.show({ title: "Two", body: "" });
    store.dismiss(first.id);
    expect(store.current?.title).toBe("Two");
  });

  it("clears the toast it is asked to dismiss", () => {
    const store = new ToastStore();
    const shown = store.show({ title: "One", body: "" });
    store.dismiss(shown.id);
    expect(store.current).toBeNull();
  });
});
