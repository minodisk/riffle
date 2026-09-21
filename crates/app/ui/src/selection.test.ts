import { describe, expect, test } from "vitest";
import { click, extend, prune, single, targets } from "./selection.js";

const files = ["/a", "/b", "/c", "/d", "/e"];
const plain = { toggle: false, range: false };
const toggle = { toggle: true, range: false };
const range = { toggle: false, range: true };

describe("click", () => {
  test("a plain click collapses to the file", () => {
    const multi = { selected: new Set(["/a", "/b", "/c"]), anchor: "/a" };
    expect(click(multi, files, 0, 3, plain)).toEqual(single("/d"));
  });

  test("toggle adds and removes", () => {
    const added = click(single("/a"), files, 0, 2, toggle);
    expect(added).toEqual({ selected: new Set(["/a", "/c"]), anchor: "/c" });
    expect(click(added, files, 0, 2, toggle)).toEqual({ selected: new Set(["/a"]), anchor: "/c" });
  });

  test("toggle cannot remove the focused file", () => {
    const multi = { selected: new Set(["/a", "/c"]), anchor: "/c" };
    expect(click(multi, files, 0, 0, toggle)).toBe(multi);
  });

  test("range after a toggle uses the toggled file as anchor", () => {
    const toggled = click(single("/a"), files, 0, 2, toggle);
    expect(click(toggled, files, 0, 4, range)).toEqual({
      selected: new Set(["/c", "/d", "/e"]),
      anchor: "/c",
    });
  });

  test("range replaces a previous range from the same anchor", () => {
    const wide = click(single("/b"), files, 1, 4, range);
    expect(click(wide, files, 4, 2, range)).toEqual({
      selected: new Set(["/b", "/c"]),
      anchor: "/b",
    });
  });

  test("range without an anchor is a plain click", () => {
    expect(click(single(undefined), files, 0, 2, range)).toEqual(single("/c"));
  });
});

describe("extend", () => {
  test("grows and shrinks through the anchor", () => {
    let state = { selection: single("/c"), index: 2 };
    state = extend(state.selection, files, state.index, 1);
    expect(state).toEqual({
      selection: { selected: new Set(["/c", "/d"]), anchor: "/c" },
      index: 3,
    });
    state = extend(state.selection, files, state.index, -1);
    state = extend(state.selection, files, state.index, -1);
    expect(state).toEqual({
      selection: { selected: new Set(["/b", "/c"]), anchor: "/c" },
      index: 1,
    });
  });

  test("stops at the ends", () => {
    expect(extend(single("/a"), files, 0, -1).index).toBe(0);
  });
});

describe("prune", () => {
  test("drops hidden paths and resets a hidden anchor to the focused file", () => {
    const multi = { selected: new Set(["/a", "/b", "/d"]), anchor: "/a" };
    expect(prune(multi, ["/b", "/c", "/d"], 0)).toEqual({
      selected: new Set(["/b", "/d"]),
      anchor: "/b",
    });
  });

  test("keeps a visible anchor", () => {
    const multi = { selected: new Set(["/a", "/b"]), anchor: "/a" };
    expect(prune(multi, files, 1).anchor).toBe("/a");
  });
});

describe("targets", () => {
  test("lists the selection in files order", () => {
    const multi = { selected: new Set(["/d", "/a", "/c"]), anchor: "/d" };
    expect(targets(multi, files, 2)).toEqual(["/a", "/c", "/d"]);
  });

  test("falls back to the focused file", () => {
    expect(targets(single("/a"), files, 3)).toEqual(["/d"]);
    expect(targets(single(undefined), [], 0)).toEqual([]);
  });
});
