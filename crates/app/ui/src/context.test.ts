import { describe, expect, test } from "vitest";
import { contextMenuGroups, folderMenuGroups, menuPosition } from "./context.js";

const defaults = [
  { action: "selectAll", keys: ["meta+a"] },
  { action: "pick", keys: ["p"] },
  { action: "reject", keys: ["x"] },
  { action: "unflag", keys: ["u"] },
  { action: "rate1", keys: ["1"] },
  { action: "rate2", keys: ["2"] },
  { action: "rate3", keys: ["3"] },
  { action: "rate4", keys: ["4"] },
  { action: "rate5", keys: ["5"] },
  { action: "clear", keys: ["0"] },
  { action: "red", keys: ["ctrl+alt+1"] },
  { action: "orange", keys: ["ctrl+alt+2"] },
  { action: "yellow", keys: ["ctrl+alt+3"] },
  { action: "green", keys: ["ctrl+alt+4"] },
  { action: "blue", keys: ["ctrl+alt+5"] },
  { action: "pink", keys: ["ctrl+alt+6"] },
  { action: "purple", keys: ["ctrl+alt+7"] },
  { action: "clearlabel", keys: ["ctrl+alt+0"] },
];

const unset = { rating: null, flag: "none" as const, label: null };

describe("contextMenuGroups", () => {
  test("lists Select All above the flag, rating and label groups", () => {
    const groups = contextMenuGroups(defaults, unset);
    expect(groups.map((group) => group.map(({ action, label }) => [action, label]))).toEqual([
      [["selectAll", "Select All"]],
      [
        ["pick", "Pick"],
        ["reject", "Reject"],
        ["unflag", "Unflag"],
      ],
      [
        ["rate1", "1 star"],
        ["rate2", "2 stars"],
        ["rate3", "3 stars"],
        ["rate4", "4 stars"],
        ["rate5", "5 stars"],
        ["clear", "No stars"],
      ],
      [
        ["red", "Red"],
        ["orange", "Orange"],
        ["yellow", "Yellow"],
        ["green", "Green"],
        ["blue", "Blue"],
        ["pink", "Pink"],
        ["purple", "Purple"],
        ["clearlabel", "No label"],
      ],
    ]);
  });

  test("shows the default keys", () => {
    const shortcuts = contextMenuGroups(defaults, unset).map((group) =>
      group.map((item) => item.shortcut),
    );
    expect(shortcuts).toEqual([
      ["meta+a"],
      ["p", "x", "u"],
      ["1", "2", "3", "4", "5", "0"],
      [
        "ctrl+alt+1",
        "ctrl+alt+2",
        "ctrl+alt+3",
        "ctrl+alt+4",
        "ctrl+alt+5",
        "ctrl+alt+6",
        "ctrl+alt+7",
        "ctrl+alt+0",
      ],
    ]);
  });

  test("shows an overridden key", () => {
    const bindings = defaults.map((binding) =>
      binding.action === "rate3" ? { action: "rate3", keys: ["space"] } : binding,
    );
    expect(contextMenuGroups(bindings, unset)[2]?.[2]?.shortcut).toBe("Space");
  });

  test("shows an overridden Select All key", () => {
    const bindings = defaults.map((binding) =>
      binding.action === "selectAll" ? { action: "selectAll", keys: ["ctrl+alt+a"] } : binding,
    );
    expect(contextMenuGroups(bindings, unset)[0]?.[0]?.shortcut).toBe("ctrl+alt+a");
  });

  test("gives Select All no checked state", () => {
    expect(contextMenuGroups(defaults, unset)[0]?.[0]?.checked).toBeUndefined();
  });

  test("shows the first key when an action has several", () => {
    const bindings = [{ action: "reject", keys: ["d", "x"] }];
    expect(contextMenuGroups(bindings, unset)[1]?.[1]?.shortcut).toBe("d");
  });

  test("gives an empty shortcut for an unbound action", () => {
    const shortcuts = contextMenuGroups([], unset).flatMap((group) =>
      group.map((item) => item.shortcut),
    );
    expect(shortcuts).toHaveLength(18);
    expect(shortcuts.every((shortcut) => shortcut === "")).toBe(true);
  });

  test("marks the unset items when nothing is set", () => {
    const checked = contextMenuGroups(defaults, unset)
      .flat()
      .filter((item) => item.checked)
      .map((item) => item.action);
    expect(checked).toEqual(["unflag", "clear", "clearlabel"]);
  });

  test("marks the current flag, rating and label", () => {
    const checked = contextMenuGroups(defaults, { rating: 3, flag: "pick", label: "Orange" })
      .flat()
      .filter((item) => item.checked)
      .map((item) => item.action);
    expect(checked).toEqual(["pick", "rate3", "orange"]);
  });
});

describe("folderMenuGroups", () => {
  test("holds the one reveal item, without a shortcut or checked state", () => {
    expect(folderMenuGroups("Reveal in Finder")).toEqual([
      [{ action: "revealFolder", label: "Reveal in Finder", shortcut: "", checked: undefined }],
    ]);
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
