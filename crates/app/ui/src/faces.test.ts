import { describe, expect, test } from "vitest";
import { FaceCache, NO_FACES } from "./faces.js";
import type { Faces } from "./focus.js";

const found: Faces = {
  width: 1616,
  height: 1080,
  faces: [
    {
      x: 10,
      y: 20,
      width: 30,
      height: 40,
      score: 0.9,
      left_eye: { x: 18, y: 30 },
      right_eye: { x: 32, y: 30 },
      nose: { x: 25, y: 38 },
      mouth_left: { x: 19, y: 48 },
      mouth_right: { x: 31, y: 48 },
    },
  ],
};

describe("FaceCache", () => {
  test("requests an uncached path once while it is in flight", () => {
    const cache = new FaceCache();
    expect(cache.request("a")).not.toBeNull();
    expect(cache.request("a")).toBeNull();
    expect(cache.request("b")).not.toBeNull();
  });

  test("keeps a response and never requests its path again", () => {
    const cache = new FaceCache();
    const token = cache.request("a") ?? -1;
    expect(cache.settle("a", token, found)).toBe(true);
    expect(cache.get("a")).toBe(found);
    expect(cache.request("a")).toBeNull();
  });

  test("a failure cached as no faces is not retried", () => {
    const cache = new FaceCache();
    const token = cache.request("a") ?? -1;
    cache.settle("a", token, NO_FACES);
    expect(cache.get("a")?.faces).toEqual([]);
    expect(cache.request("a")).toBeNull();
  });

  test("drops a response issued before a clear and keeps the new request in flight", () => {
    const cache = new FaceCache();
    const stale = cache.request("a") ?? -1;
    cache.clear();
    const fresh = cache.request("a") ?? -1;
    expect(cache.settle("a", stale, found)).toBe(false);
    expect(cache.get("a")).toBeUndefined();
    expect(cache.request("a")).toBeNull();
    expect(cache.settle("a", fresh, found)).toBe(true);
    expect(cache.get("a")).toBe(found);
  });

  test("clear forgets what was found", () => {
    const cache = new FaceCache();
    cache.settle("a", cache.request("a") ?? -1, found);
    cache.clear();
    expect(cache.get("a")).toBeUndefined();
    expect(cache.request("a")).not.toBeNull();
  });
});
