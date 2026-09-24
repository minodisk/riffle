import { describe, expect, test } from "vitest";
import { SettingsModal, cycleFocus } from "./modal.js";

describe("SettingsModal", () => {
  test("starts closed", () => {
    expect(new SettingsModal().isOpen).toBe(false);
  });

  test("opens once; opening again while open is a no-op", () => {
    const modal = new SettingsModal();
    expect(modal.open()).toBe(true);
    expect(modal.isOpen).toBe(true);
    expect(modal.open()).toBe(false);
    expect(modal.isOpen).toBe(true);
  });

  test("closes, and can open again", () => {
    const modal = new SettingsModal();
    modal.open();
    modal.close();
    expect(modal.isOpen).toBe(false);
    expect(modal.open()).toBe(true);
  });

  test("closing cancels a capture", () => {
    const modal = new SettingsModal();
    modal.open();
    modal.capturing = "next";
    modal.close();
    expect(modal.capturing).toBeNull();
  });

  test("Escape closes, Tab and Shift+Tab move focus", () => {
    const modal = new SettingsModal();
    modal.open();
    expect(modal.key("escape")).toEqual({ kind: "close" });
    expect(modal.key("tab")).toEqual({ kind: "focus", step: 1 });
    expect(modal.key("shift+tab")).toEqual({ kind: "focus", step: -1 });
  });

  test("culling keys are left to the focused control, never mapped to an action", () => {
    const modal = new SettingsModal();
    modal.open();
    for (const key of ["arrowright", "p", "x", "1", "ctrl+alt+1", "meta+z", "space", "enter"]) {
      expect(modal.key(key)).toEqual({ kind: "native" });
    }
    expect(modal.key(null)).toEqual({ kind: "native" });
  });

  test("while capturing, a key is added and Escape cancels instead of closing", () => {
    const modal = new SettingsModal();
    modal.open();
    modal.capturing = "next";
    expect(modal.key("arrowright")).toEqual({ kind: "add", key: "arrowright" });
    expect(modal.key("tab")).toEqual({ kind: "add", key: "tab" });
    expect(modal.key("escape")).toEqual({ kind: "cancel" });
    modal.capturing = null;
    expect(modal.key("escape")).toEqual({ kind: "close" });
  });

  test("while capturing, a lone modifier waits for the rest of the combination", () => {
    const modal = new SettingsModal();
    modal.open();
    modal.capturing = "next";
    expect(modal.key(null)).toEqual({ kind: "native" });
  });
});

describe("cycleFocus", () => {
  test("wraps around both ways", () => {
    expect(cycleFocus(3, 0, 1)).toBe(1);
    expect(cycleFocus(3, 2, 1)).toBe(0);
    expect(cycleFocus(3, 0, -1)).toBe(2);
  });

  test("enters from the first or last element when the focus is outside", () => {
    expect(cycleFocus(3, -1, 1)).toBe(0);
    expect(cycleFocus(3, -1, -1)).toBe(2);
  });
});
