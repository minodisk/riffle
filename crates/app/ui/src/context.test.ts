import { describe, expect, test } from "vitest";
import { flagMenuItems, menuPosition } from "./context.js";

const defaults = [
  { action: "pick", keys: ["p"] },
  { action: "reject", keys: ["x"] },
  { action: "unflag", keys: ["u"] },
];

describe("flagMenuItems", () => {
  test("shows the default keys", () => {
    expect(flagMenuItems(defaults, "dop")).toEqual([
      { action: "pick", label: "Pick", shortcut: "p" },
      { action: "reject", label: "Reject", shortcut: "x" },
      { action: "unflag", label: "Unflag", shortcut: "u" },
    ]);
  });

  test("shows an overridden key", () => {
    const bindings = [...defaults.slice(0, 2), { action: "unflag", keys: ["space"] }];
    expect(flagMenuItems(bindings, "dop")[2]?.shortcut).toBe("Space");
  });

  test("shows the first key when an action has several", () => {
    const bindings = [{ action: "reject", keys: ["d", "x"] }, ...defaults.slice(2)];
    expect(flagMenuItems(bindings, "xmp")[0]?.shortcut).toBe("d");
  });

  test("drops pick under XMP", () => {
    expect(flagMenuItems(defaults, "xmp").map((item) => item.action)).toEqual(["reject", "unflag"]);
  });

  test("gives an empty shortcut for an unbound action", () => {
    expect(flagMenuItems([], "dop").map((item) => item.shortcut)).toEqual(["", "", ""]);
  });
});

describe("menuPosition", () => {
  test("leaves an interior point alone", () => {
    expect(menuPosition(100, 100, 50, 40, 800, 600)).toEqual({ left: 100, top: 100 });
  });

  test("flips left near the right edge", () => {
    expect(menuPosition(780, 100, 50, 40, 800, 600)).toEqual({ left: 730, top: 100 });
  });

  test("flips up near the bottom edge", () => {
    expect(menuPosition(100, 590, 50, 40, 800, 600)).toEqual({ left: 100, top: 550 });
  });

  test("shifts to the edge when flipping would leave the viewport", () => {
    expect(menuPosition(30, 20, 50, 40, 60, 50)).toEqual({ left: 0, top: 0 });
  });
});
