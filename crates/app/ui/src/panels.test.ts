import { describe, expect, test } from "vitest";
import { toggle, toggleSides } from "./panels.js";

const all = { left: true, strip: true, right: true };

describe("toggle", () => {
  test("flips only the named panel", () => {
    expect(toggle(all, "left")).toEqual({ left: false, strip: true, right: true });
    expect(toggle(all, "strip")).toEqual({ left: true, strip: false, right: true });
    expect(toggle(toggle(all, "right"), "right")).toEqual(all);
  });
});

describe("toggleSides", () => {
  test("hides both side panes when both are shown, and twice restores them", () => {
    const hidden = toggleSides(all);
    expect(hidden).toEqual({ left: false, strip: true, right: false });
    expect(toggleSides(hidden)).toEqual(all);
  });

  test("hides both when only one is shown", () => {
    expect(toggleSides({ left: false, strip: false, right: true })).toEqual({
      left: false,
      strip: false,
      right: false,
    });
  });
});
