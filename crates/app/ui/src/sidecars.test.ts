import { describe, expect, test } from "vitest";
import {
  type DeletePreview,
  type RewritePreview,
  SidecarFlow,
  deleteRows,
  deleteTotalLine,
  formatName,
  rewriteRows,
  rewriteTotalLine,
  rewrittenStatus,
} from "./sidecars.js";

function preview(judged: number, unjudged: number, skipped: number): RewritePreview {
  return { judged, unjudged, skipped, format: "xmp" };
}

describe("rewriteRows", () => {
  test("lists the judged, the unjudged and the skipped files", () => {
    expect(rewriteRows(preview(12, 3, 5))).toEqual([
      "12 files with a judgment",
      "3 files without one (any existing sidecar is cleared)",
      "5 files not in the index (skipped)",
    ]);
  });

  test("leaves a count of zero out and speaks of one file in the singular", () => {
    expect(rewriteRows(preview(1, 1, 0))).toEqual([
      "1 file with a judgment",
      "1 file without one (any existing sidecar is cleared)",
    ]);
    expect(rewriteRows(preview(0, 2, 1))).toEqual([
      "2 files without one (any existing sidecar is cleared)",
      "1 file not in the index (skipped)",
    ]);
  });
});

describe("rewriteTotalLine", () => {
  test("names the format and counts the files written, not the skipped ones", () => {
    expect(rewriteTotalLine(preview(12, 3, 5), "2026-10-05")).toBe(
      "Rewrite the XMP sidecars of 15 files in 2026-10-05?",
    );
    expect(rewriteTotalLine({ ...preview(1, 0, 0), format: "dop" }, "a")).toBe(
      "Rewrite the .dop sidecars of 1 file in a?",
    );
    expect(rewriteTotalLine({ ...preview(2, 0, 0), format: "both" }, "a")).toBe(
      "Rewrite the XMP and .dop sidecars of 2 files in a?",
    );
  });

  test("formatName names each format", () => {
    expect(formatName("xmp")).toBe("XMP");
    expect(formatName("dop")).toBe(".dop");
    expect(formatName("both")).toBe("XMP and .dop");
  });
});

describe("rewrittenStatus", () => {
  test("counts the files written and adds the failures", () => {
    expect(rewrittenStatus({ written: 15, failed: 0, flushed: true }, "a")).toBe(
      "Rewrote the sidecars of 15 files in a",
    );
    expect(rewrittenStatus({ written: 1, failed: 0, flushed: true }, "a")).toBe(
      "Rewrote the sidecars of 1 file in a",
    );
    expect(rewrittenStatus({ written: 13, failed: 2, flushed: true }, "a")).toBe(
      "Rewrote the sidecars of 13 files in a, 2 failed",
    );
    expect(rewrittenStatus({ written: 10, failed: 5, flushed: false }, "a")).toBe(
      "Rewrote the sidecars of 10 files in a, 5 failed",
    );
  });
});

describe("delete text", () => {
  const both: DeletePreview = {
    kinds: [
      { kind: "xmp", count: 12, size_text: "1.2 MB" },
      { kind: "dop", count: 12, size_text: "340.0 KB" },
    ],
    count: 24,
    size_text: "1.5 MB",
  };

  test("deleteRows lists each kind with its size", () => {
    expect(deleteRows(both)).toEqual(["12 XMP sidecars (1.2 MB)", "12 .dop sidecars (340.0 KB)"]);
    expect(
      deleteRows({
        kinds: [{ kind: "dop", count: 1, size_text: "2 KB" }],
        count: 1,
        size_text: "2 KB",
      }),
    ).toEqual(["1 .dop sidecar (2 KB)"]);
  });

  test("deleteTotalLine counts the sidecars and names the folder", () => {
    expect(deleteTotalLine(both, "2026-10-05")).toBe(
      "Move 24 sidecars (1.5 MB) of 2026-10-05 to the Trash?",
    );
    expect(deleteTotalLine({ ...both, count: 1, size_text: "800 B" }, "a")).toBe(
      "Move 1 sidecar (800 B) of a to the Trash?",
    );
  });
});

describe("SidecarFlow", () => {
  const target = { kind: "rewrite" as const, dir: "/photos" };

  test("refuses a second start while one is under way", () => {
    const flow = new SidecarFlow();
    expect(flow.start(target)).toBe(true);
    expect(flow.busy).toBe(true);
    expect(flow.isOpen).toBe(false);
    expect(flow.start({ kind: "delete", dir: "/other" })).toBe(false);
    expect(flow.previewed()).toBe(true);
    expect(flow.isOpen).toBe(true);
    expect(flow.run()).toEqual(target);
    expect(flow.start(target)).toBe(false);
    flow.end();
    expect(flow.busy).toBe(false);
    expect(flow.start(target)).toBe(true);
  });

  test("closes a preview on dismiss but not a running rewrite", () => {
    const flow = new SidecarFlow();
    flow.start(target);
    flow.previewed();
    expect(flow.dismiss()).toBe(true);
    expect(flow.busy).toBe(false);
    expect(flow.run()).toBeNull();
    flow.start(target);
    flow.previewed();
    flow.run();
    expect(flow.dismiss()).toBe(false);
    expect(flow.isOpen).toBe(true);
  });

  test("a failed preview ends the flow before any dialog", () => {
    const flow = new SidecarFlow();
    flow.start(target);
    flow.end();
    expect(flow.previewed()).toBe(false);
    expect(flow.isOpen).toBe(false);
  });
});
