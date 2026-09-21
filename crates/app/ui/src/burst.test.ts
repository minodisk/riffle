import { describe, expect, test } from "vitest";
import { burstMarks, groupBursts, type BurstMember } from "./burst.js";
import type { SortFacts } from "./sort.js";

function lookupFrom(facts: Record<string, SortFacts>): (path: string) => SortFacts {
  return (path) => facts[path] ?? {};
}

function bursts(members: Map<string, BurstMember>): string[][] {
  const groups: string[][] = [];
  for (const [path, member] of members) {
    (groups[member.burst] ??= [])[member.position] = path;
  }
  return groups;
}

describe("groupBursts", () => {
  test("a gap exactly at the threshold stays, one millisecond over splits", () => {
    const lookup = lookupFrom({
      a: { captureTime: "2026:09:19 10:00:00", subsec: "000" },
      b: { captureTime: "2026:09:19 10:00:01", subsec: "000" },
      c: { captureTime: "2026:09:19 10:00:02", subsec: "001" },
    });
    expect(bursts(groupBursts(["a", "b", "c"], lookup, 1000))).toEqual([["a", "b"], ["c"]]);
  });

  test("a run across second, minute and day boundaries stays together", () => {
    const lookup = lookupFrom({
      a: { captureTime: "2026:09:19 23:59:59", subsec: "5" },
      b: { captureTime: "2026:09:20 00:00:00", subsec: "1" },
      c: { captureTime: "2026:09:20 00:00:00", subsec: "9" },
      d: { captureTime: "2026:09:20 00:01:00", subsec: "0" },
    });
    expect(bursts(groupBursts(["a", "b", "c", "d"], lookup))).toEqual([["a", "b", "c"], ["d"]]);
  });

  test("subsec orders a burst whose names are out of capture order", () => {
    const lookup = lookupFrom({
      z: { captureTime: "2026:09:19 10:00:00", subsec: "5" },
      y: { captureTime: "2026:09:19 10:00:00", subsec: "122" },
      x: { captureTime: "2026:09:19 10:00:01", subsec: "4" },
    });
    expect(bursts(groupBursts(["x", "y", "z"], lookup))).toEqual([["y", "z", "x"]]);
  });

  test("a missing subsec is 0 ms, so whole seconds stay grouped", () => {
    const lookup = lookupFrom({
      a: { captureTime: "2026:09:19 10:00:00" },
      b: { captureTime: "2026:09:19 10:00:01" },
      c: { captureTime: "2026:09:19 10:00:02" },
      d: { captureTime: "2026:09:19 10:00:02", subsec: "500" },
      e: { captureTime: "2026:09:19 10:00:04" },
    });
    expect(bursts(groupBursts(["a", "b", "c", "d", "e"], lookup))).toEqual([
      ["a", "b", "c", "d"],
      ["e"],
    ]);
  });

  test("files without a capture time are singletons at the end", () => {
    const lookup = lookupFrom({
      a: { captureTime: "2026:09:19 10:00:00" },
      c: { captureTime: "2026:09:19 10:00:01" },
      d: { captureTime: "garbage" },
    });
    expect(bursts(groupBursts(["a", "b", "c", "d"], lookup))).toEqual([["a", "c"], ["d"], ["b"]]);
  });

  test("a lone file is a burst of one", () => {
    const members = groupBursts(["a"], lookupFrom({ a: { captureTime: "2026:09:19 10:00:00" } }));
    expect(members.get("a")).toEqual({ burst: 0, position: 0, size: 1 });
  });
});

describe("burstMarks", () => {
  const members = new Map<string, BurstMember>([
    ["a", { burst: 0, position: 0, size: 3 }],
    ["b", { burst: 0, position: 1, size: 3 }],
    ["c", { burst: 0, position: 2, size: 3 }],
    ["d", { burst: 1, position: 0, size: 1 }],
    ["e", { burst: 2, position: 0, size: 2 }],
    ["f", { burst: 2, position: 1, size: 2 }],
  ]);

  test("brackets open and close at burst edges and skip singletons", () => {
    expect(burstMarks(["a", "b", "c", "d", "e", "f"], members)).toEqual([
      { first: true, last: false },
      { first: false, last: false },
      { first: false, last: true },
      null,
      { first: true, last: false },
      { first: false, last: true },
    ]);
  });

  test("members separated by another file close and reopen the bracket", () => {
    expect(burstMarks(["a", "e", "b"], members)).toEqual([
      { first: true, last: true },
      { first: true, last: true },
      { first: true, last: true },
    ]);
  });
});
