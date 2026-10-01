import { describe, expect, test } from "vitest";
import { emptyState, openHint } from "./empty.js";

describe("emptyState", () => {
  test("no folder open", () => {
    expect(emptyState(null, 0, 0, false)).toBe("no-folder");
  });

  test("a folder without RAW or JPEG files", () => {
    expect(emptyState("/photos", 0, 0, false)).toBe("no-files");
  });

  test("every file filtered out", () => {
    expect(emptyState("/photos", 12, 0, false)).toBe("filtered");
  });

  test("files to show", () => {
    expect(emptyState("/photos", 12, 3, false)).toBe("none");
  });

  test("the current file's preview failed", () => {
    expect(emptyState("/photos", 12, 3, true)).toBe("preview-failed");
  });

  test("the other states win over a failed preview", () => {
    expect(emptyState(null, 0, 0, true)).toBe("no-folder");
    expect(emptyState("/photos", 0, 0, true)).toBe("no-files");
    expect(emptyState("/photos", 12, 0, true)).toBe("filtered");
  });
});

describe("openHint", () => {
  test("the default key", () => {
    expect(openHint([{ action: "open", keys: ["o"] }])).toBe(
      "Drop a folder onto the window, or click here to choose one. Or press o.",
    );
  });

  test("several keys", () => {
    expect(openHint([{ action: "open", keys: ["o", "ctrl+o"] }])).toBe(
      "Drop a folder onto the window, or click here to choose one. Or press o or ctrl+o.",
    );
  });

  test("a rebound key", () => {
    expect(openHint([{ action: "open", keys: ["space"] }])).toBe(
      "Drop a folder onto the window, or click here to choose one. Or press Space.",
    );
  });

  test("no key bound", () => {
    expect(openHint([{ action: "open", keys: [] }])).toBe(
      "Drop a folder onto the window, or click here to choose one.",
    );
  });

  test("the keymap has not resolved yet", () => {
    expect(openHint([])).toBe("Drop a folder onto the window, or click here to choose one.");
  });
});
