import { describe, expect, test } from "vitest";
import type { Ranked } from "./meta.js";
import { recommended } from "./recommend.js";

function ranked(place: number, of: number, score = 0): Ranked {
  return { score, place, of, reasons: [] };
}

describe("recommended", () => {
  test("a file without a ranking entry is not marked", () => {
    expect(recommended(undefined)).toBe(false);
  });

  test("a lone scored frame is not marked", () => {
    expect(recommended(ranked(1, 1))).toBe(false);
  });

  test("the first of two is marked", () => {
    expect(recommended(ranked(1, 2, 0.4))).toBe(true);
  });

  test("the second of two is not marked", () => {
    expect(recommended(ranked(2, 2, -0.4))).toBe(false);
  });

  test("two frames tied for first are both marked", () => {
    expect(recommended(ranked(1, 3, 0.2))).toBe(true);
    expect(recommended(ranked(1, 3, 0.2))).toBe(true);
    expect(recommended(ranked(3, 3, -0.4))).toBe(false);
  });
});
