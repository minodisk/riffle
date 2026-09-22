import { describe, expect, test } from "vitest";
import { comparisonCandidates, loadComparisonFrames, reconcileActive } from "./compare.js";
import { extend } from "./selection.js";

describe("reconcileActive", () => {
  test("keeps an active frame that is still displayed", () => {
    expect(reconcileActive(["/a", "/b"], "/b", "/a")).toBe("/b");
  });

  test("falls back to the focused candidate when the active frame disappears", () => {
    expect(reconcileActive(["/c", "/d"], "/b", "/d")).toBe("/d");
  });

  test("falls back to the first candidate when the focus is not displayed", () => {
    expect(reconcileActive(["/c", "/d"], "/b", "/a")).toBe("/c");
  });

  test("clears the active frame with no candidates", () => {
    expect(reconcileActive([], "/b", undefined)).toBeNull();
  });
});

describe("comparisonCandidates", () => {
  const files = ["/a", "/b", "/c", "/d", "/e"];
  const bursts = new Map(files.map((path) => [path, { burst: 1 }]));

  test("refreshes the best frame when scan-time scores change", () => {
    const selected = new Set(["/a"]);
    expect(comparisonCandidates(files, 0, selected, bursts, new Map([["/b", 2]]))).toEqual([
      "/a",
      "/b",
    ]);
    expect(comparisonCandidates(files, 0, selected, bursts, new Map([["/c", 3]]))).toEqual([
      "/a",
      "/c",
    ]);
  });

  test("refreshes a changed range even when its focus stays at the boundary", () => {
    const before = { selected: new Set(["/a", "/e"]), anchor: "/b" };
    expect(comparisonCandidates(files, 4, before.selected, bursts, new Map())).toEqual([
      "/a",
      "/e",
    ]);
    const extended = extend(before, files, 4, 1);
    expect(extended.index).toBe(4);
    expect(
      comparisonCandidates(files, extended.index, extended.selection.selected, bursts, new Map()),
    ).toEqual(["/b", "/c", "/d", "/e"]);
  });
});

describe("loadComparisonFrames", () => {
  test("returns every frame after all loads succeed", async () => {
    await expect(
      loadComparisonFrames(
        ["/a", "/b"],
        async (path) => ({ path }),
        () => undefined,
      ),
    ).resolves.toEqual([{ path: "/a" }, { path: "/b" }]);
  });

  test("disposes every successful sibling when one load fails", async () => {
    const disposed: string[] = [];
    const load = async (path: string): Promise<{ path: string }> => {
      if (path === "/bad") throw new Error("bad preview");
      return { path };
    };
    await expect(
      loadComparisonFrames(["/a", "/bad", "/c"], load, (frame) => disposed.push(frame.path)),
    ).rejects.toThrow("bad preview");
    expect(disposed).toEqual(["/a", "/c"]);
  });
});
