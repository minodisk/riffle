import { describe, expect, test } from "vitest";
import { orderFiles, type SortFacts } from "./sort.js";

function lookupFrom(facts: Record<string, SortFacts>): (path: string) => SortFacts {
  return (path) => facts[path] ?? {};
}

describe("orderFiles", () => {
  test("capture walks a counter rollover in time order", () => {
    const paths = ["/f/DSC00001.ARW", "/f/DSC09998.ARW", "/f/DSC09999.ARW"];
    const lookup = lookupFrom({
      "/f/DSC09998.ARW": { captureTime: "2026:09:19 10:00:00" },
      "/f/DSC09999.ARW": { captureTime: "2026:09:19 10:00:01" },
      "/f/DSC00001.ARW": { captureTime: "2026:09:19 10:00:02" },
    });
    expect(orderFiles("capture", paths, lookup)).toEqual([
      "/f/DSC09998.ARW",
      "/f/DSC09999.ARW",
      "/f/DSC00001.ARW",
    ]);
  });

  test("capture interleaves two bodies by time", () => {
    const paths = ["/f/A001.ARW", "/f/A002.ARW", "/f/L001.DNG", "/f/L002.DNG"];
    const lookup = lookupFrom({
      "/f/A001.ARW": { captureTime: "2026:09:19 10:00:00" },
      "/f/A002.ARW": { captureTime: "2026:09:19 10:00:02" },
      "/f/L001.DNG": { captureTime: "2026:09:19 10:00:01" },
      "/f/L002.DNG": { captureTime: "2026:09:19 10:00:03" },
    });
    expect(orderFiles("capture", paths, lookup)).toEqual([
      "/f/A001.ARW",
      "/f/L001.DNG",
      "/f/A002.ARW",
      "/f/L002.DNG",
    ]);
  });

  test("capture splits an equal second by subsec as a fraction", () => {
    const paths = ["/f/a.ARW", "/f/b.ARW", "/f/c.ARW"];
    const lookup = lookupFrom({
      "/f/a.ARW": { captureTime: "2026:09:19 10:00:00", subsec: "5" },
      "/f/b.ARW": { captureTime: "2026:09:19 10:00:00", subsec: "122" },
      "/f/c.ARW": { captureTime: "2026:09:19 10:00:00" },
    });
    expect(orderFiles("capture", paths, lookup)).toEqual(["/f/c.ARW", "/f/b.ARW", "/f/a.ARW"]);
  });

  test("capture puts files without a time last in name order", () => {
    const paths = ["/f/a.ARW", "/f/b.ARW", "/f/c.ARW", "/f/d.ARW"];
    const lookup = lookupFrom({
      "/f/c.ARW": { captureTime: "2026:09:19 10:00:01" },
      "/f/d.ARW": { captureTime: "2026:09:19 10:00:00" },
    });
    expect(orderFiles("capture", paths, lookup)).toEqual([
      "/f/d.ARW",
      "/f/c.ARW",
      "/f/a.ARW",
      "/f/b.ARW",
    ]);
  });

  test("rating orders stars descending, unrated, then rejected, ties by name", () => {
    const paths = ["/f/a.ARW", "/f/b.ARW", "/f/c.ARW", "/f/d.ARW", "/f/e.ARW", "/f/f.ARW"];
    const lookup = lookupFrom({
      "/f/a.ARW": { rating: -1 },
      "/f/b.ARW": { rating: 1 },
      "/f/d.ARW": { rating: 5 },
      "/f/e.ARW": { rating: 3 },
      "/f/f.ARW": { rating: 5 },
    });
    expect(orderFiles("rating", paths, lookup)).toEqual([
      "/f/d.ARW",
      "/f/f.ARW",
      "/f/e.ARW",
      "/f/b.ARW",
      "/f/c.ARW",
      "/f/a.ARW",
    ]);
  });

  test("name keeps an already name-sorted list as is", () => {
    const paths = ["/f/DSC00001.ARW", "/f/DSC09998.ARW", "/f/DSC09999.ARW", "/f/L1000001.DNG"];
    expect(orderFiles("name", paths, lookupFrom({}))).toEqual(paths);
  });

  test("does not mutate its input", () => {
    const paths = ["/f/b.ARW", "/f/a.ARW"];
    const copy = [...paths];
    expect(orderFiles("name", paths, lookupFrom({}))).toEqual(["/f/a.ARW", "/f/b.ARW"]);
    expect(paths).toEqual(copy);
  });
});
