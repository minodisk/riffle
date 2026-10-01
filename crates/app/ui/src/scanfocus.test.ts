import { describe, expect, test } from "vitest";
import { scanFocusPaths } from "./scanfocus.js";

const files = ["a", "b", "c", "d", "e", "f", "g"];

describe("scanFocusPaths", () => {
  test("puts the current file first, then the range outward, earlier side first", () => {
    expect(scanFocusPaths(files, 3, 1, 5)).toEqual(["d", "c", "e", "b", "f"]);
  });

  test("keeps going on the one side left once the other runs out", () => {
    expect(scanFocusPaths(files, 1, 0, 5)).toEqual(["b", "a", "c", "d", "e", "f"]);
  });

  test("is capped at the range", () => {
    expect(scanFocusPaths(files, 3, 2, 4)).toEqual(["d", "c", "e"]);
  });

  test("clamps a range that runs past either end of the strip", () => {
    expect(scanFocusPaths(files, 0, -4, 2)).toEqual(["a", "b", "c"]);
    expect(scanFocusPaths(files, 6, 4, 10)).toEqual(["g", "f", "e"]);
  });

  test("leads with a current file outside the range, then the range nearest to it", () => {
    expect(scanFocusPaths(files, 0, 3, 5)).toEqual(["a", "d", "e", "f"]);
    expect(scanFocusPaths(files, 6, 1, 2)).toEqual(["g", "c", "b"]);
  });

  test("has no duplicates", () => {
    const paths = scanFocusPaths(files, 2, 0, 6);
    expect(new Set(paths).size).toBe(paths.length);
    expect(paths).toHaveLength(files.length);
  });

  test("is empty when there is no current file", () => {
    expect(scanFocusPaths([], 0, 0, 4)).toEqual([]);
    expect(scanFocusPaths(files, 7, 0, 6)).toEqual([]);
  });
});
