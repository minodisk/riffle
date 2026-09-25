import { describe, expect, test } from "vitest";
import { refreshOnFacesDone, refreshOnProgress, refreshTimingLine } from "./refresh.js";

describe("refreshOnProgress", () => {
  test("no current file", () => {
    expect(refreshOnProgress(undefined, false, [], null)).toBe(false);
  });

  test("the current file already has its row", () => {
    expect(refreshOnProgress("/a.ARW", true, ["/a.ARW"], null)).toBe(false);
  });

  test("the first tick after an open", () => {
    expect(refreshOnProgress("/a.ARW", false, [], null)).toBe(true);
  });

  test("waits while the row has not landed", () => {
    expect(refreshOnProgress("/a.ARW", false, ["/b.ARW"], "/a.ARW")).toBe(false);
  });

  test("the tick that commits the current file", () => {
    expect(refreshOnProgress("/a.ARW", false, ["/b.ARW", "/a.ARW"], "/a.ARW")).toBe(true);
  });

  test("a file paged to after its row landed", () => {
    expect(refreshOnProgress("/c.ARW", false, [], "/a.ARW")).toBe(true);
  });
});

describe("refreshOnFacesDone", () => {
  test("neither pass wrote anything", () => {
    expect(refreshOnFacesDone({ scanId: 3, total: 0 }, 3, 0)).toBe(false);
  });

  test("the faces pass wrote rows", () => {
    expect(refreshOnFacesDone({ scanId: 3, total: 0 }, 3, 2)).toBe(true);
  });

  test("the scan pass wrote rows", () => {
    expect(refreshOnFacesDone({ scanId: 3, total: 5 }, 3, 0)).toBe(true);
  });

  test("the scan-done belongs to another scan", () => {
    expect(refreshOnFacesDone({ scanId: 2, total: 0 }, 3, 0)).toBe(true);
  });

  test("no scan-done seen", () => {
    expect(refreshOnFacesDone(null, 3, 0)).toBe(true);
  });
});

describe("refreshTimingLine", () => {
  test("every phase", () => {
    const line = refreshTimingLine({
      rows: 3000,
      invoke: 41.25,
      entries: 3.5,
      bursts: 1,
      exif: 12,
      meta: 0.4,
      draw: 0.2,
      sharpness: 2,
      applyBursts: 1.5,
      candidates: 0.9,
      refilter: 7,
      setFiles: false,
      total: 70.04,
    });
    expect(line).toBe(
      "refresh entries: rows=3000 invoke=41.3ms entries=3.5ms bursts=1.0ms exif=12.0ms meta=0.4ms draw=0.2ms sharpness=2.0ms apply_bursts=1.5ms candidates=0.9ms refilter=7.0ms set_files=false total=70.0ms",
    );
    expect(line.length).toBeLessThan(220);
  });
});
