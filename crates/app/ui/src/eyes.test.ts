import { describe, expect, test } from "vitest";
import { type Eyes, EyesCache, type EyesTicket } from "./eyes.js";

const mesh: Eyes["mesh"] = { width: 1600, height: 1080, points: [[800, 540]] };
const closed: Eyes = { state: "closed", probability: 0.81, ear: 0.08, pose: null, mesh };
const open: Eyes = { state: "open", probability: 0.07, ear: 0.3, pose: null, mesh };

function ticket(cache: EyesCache, path: string): EyesTicket {
  const t = cache.request(path);
  if (t === null) {
    throw new Error(`no ticket for ${path}`);
  }
  return t;
}

describe("EyesCache", () => {
  test("judges an uncached path once while it is in flight", () => {
    const cache = new EyesCache();
    expect(cache.request("a")).not.toBeNull();
    expect(cache.request("a")).toBeNull();
  });

  test("runs one judgment at a time and asks again once it settles", () => {
    const cache = new EyesCache();
    const a = ticket(cache, "a");
    expect(cache.request("b")).toBeNull();
    expect(cache.request("c")).toBeNull();
    expect(cache.settle("a", a, open)).toBe(true);
    const c = ticket(cache, "c");
    expect(c.id).toBeGreaterThan(a.id);
  });

  test("keeps a response for a path paged away from and never judges it again", () => {
    const cache = new EyesCache();
    const a = ticket(cache, "a");
    expect(cache.settle("a", a, closed)).toBe(true);
    expect(cache.get("a")).toBe(closed);
    expect(cache.request("a")).toBeNull();
  });

  test("caches unknown eyes, so they are not judged again", () => {
    const cache = new EyesCache();
    cache.settle("a", ticket(cache, "a"), null);
    expect(cache.get("a")).toBeNull();
    expect(cache.request("a")).toBeNull();
  });

  test("a superseded judgment frees the slot without caching", () => {
    const cache = new EyesCache();
    expect(cache.settle("a", ticket(cache, "a"), undefined)).toBe(false);
    expect(cache.get("a")).toBeUndefined();
    expect(cache.request("a")).not.toBeNull();
  });

  test("drops a response issued before a clear and keeps the new request in flight", () => {
    const cache = new EyesCache();
    const stale = ticket(cache, "a");
    cache.clear();
    const fresh = ticket(cache, "b");
    expect(fresh.id).toBeGreaterThan(stale.id);
    expect(cache.settle("a", stale, closed)).toBe(false);
    expect(cache.get("a")).toBeUndefined();
    expect(cache.request("a")).toBeNull();
    expect(cache.settle("b", fresh, open)).toBe(true);
    expect(cache.get("b")).toBe(open);
  });

  test("clear forgets what was judged", () => {
    const cache = new EyesCache();
    cache.settle("a", ticket(cache, "a"), closed);
    cache.clear();
    expect(cache.get("a")).toBeUndefined();
    expect(cache.request("a")).not.toBeNull();
  });
});
