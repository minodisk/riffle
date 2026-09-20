import { describe, expect, test } from "vitest";
import { emptyState, openHint } from "./empty.js";

describe("emptyState", () => {
  test("no folder open", () => {
    expect(emptyState(null, 0, 0)).toBe("no-folder");
  });

  test("a folder without ARW or DNG files", () => {
    expect(emptyState("/photos", 0, 0)).toBe("no-files");
  });

  test("every file filtered out", () => {
    expect(emptyState("/photos", 12, 0)).toBe("filtered");
  });

  test("files to show", () => {
    expect(emptyState("/photos", 12, 3)).toBe("none");
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
