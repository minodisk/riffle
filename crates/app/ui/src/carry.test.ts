import { describe, expect, test } from "vitest";
import { carriedIndices, carriedOffset } from "./carry.js";

const names = (prefix: string, count: number): string[] =>
  Array.from({ length: count }, (_, at) => `/${prefix}${at}.ARW`);

describe("carriedOffset", () => {
  test("files inserted before a visible focused cell", () => {
    const previous = names("a", 20);
    const next = [...names("n", 3), ...previous];
    expect(carriedOffset(previous, next, 200, 100, 500, 4)).toBe(500);
  });

  test("focused off-screen, files inserted before the view", () => {
    const previous = names("a", 25);
    const next = [...names("n", 100), ...previous];
    expect(carriedOffset(previous, next, 2000, 100, 500, 0)).toBe(12000);
  });

  test("the anchor deleted keeps the raw offset", () => {
    const previous = names("a", 20);
    const next = previous.filter((_, at) => at !== 5);
    expect(carriedOffset(previous, next, 500, 100, 500, 0)).toBe(500);
  });

  test("clamped when the list shrinks", () => {
    const previous = names("a", 20);
    const next = previous.slice(0, 8);
    expect(carriedOffset(previous, next, 1500, 100, 500, 0)).toBe(300);
  });

  test("an empty next", () => {
    expect(carriedOffset(names("a", 20), [], 1500, 100, 500, 0)).toBe(0);
  });
});

describe("carriedIndices", () => {
  test("a reorder", () => {
    expect(carriedIndices(["/a", "/b", "/c"], ["/c", "/a", "/b"], [0, 1, 2])).toEqual(
      new Map([
        [0, 1],
        [1, 2],
        [2, 0],
      ]),
    );
  });

  test("a removal is dropped", () => {
    expect(carriedIndices(["/a", "/b", "/c"], ["/a", "/c"], [0, 1, 2])).toEqual(
      new Map([
        [0, 0],
        [2, 1],
      ]),
    );
  });

  test("an insertion shifts", () => {
    expect(carriedIndices(["/a", "/b"], ["/x", "/a", "/b"], [0, 1])).toEqual(
      new Map([
        [0, 1],
        [1, 2],
      ]),
    );
  });

  test("an untouched list is the identity", () => {
    expect(carriedIndices(["/a", "/b"], ["/a", "/b"], [1])).toEqual(new Map([[1, 1]]));
  });
});
