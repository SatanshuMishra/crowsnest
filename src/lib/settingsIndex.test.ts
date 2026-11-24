import { describe, expect, it } from "vitest";
import { resultsTitle, searchSettings, SECTIONS, SETTING_ENTRIES } from "./settingsIndex";

describe("searchSettings", () => {
  it("finds nothing for a blank query", () => {
    expect(searchSettings("   ")).toEqual([]);
  });

  it("matches the words people use, not only the labels", () => {
    expect(searchSettings("colorblind").map((entry) => entry.key)).toEqual(["palette"]);
    expect(searchSettings("pin").map((entry) => entry.key)).toEqual(["alwaysOnTop"]);
  });

  it("ignores case", () => {
    expect(searchSettings("THEME").map((entry) => entry.key)).toContain("theme");
  });

  it("files every setting under a section that exists", () => {
    const keys = new Set(SECTIONS.map((section) => section.key));
    expect(SETTING_ENTRIES.every((entry) => keys.has(entry.section))).toBe(true);
  });
});

describe("resultsTitle", () => {
  it("counts the matches in words", () => {
    expect(resultsTitle(" tray ", 1)).toBe("1 setting matches “tray”");
    expect(resultsTitle("colour", 3)).toBe("3 settings match “colour”");
    expect(resultsTitle("zzz", 0)).toBe("No setting matches “zzz”");
  });
});
