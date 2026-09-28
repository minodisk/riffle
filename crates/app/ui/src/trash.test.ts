import { describe, expect, test } from "vitest";
import {
  type TrashPreview,
  TrashFlow,
  canRun,
  emptyFoldersLine,
  failureText,
  folderRows,
  opensTarget,
  shownPath,
  totalLine,
  trashedStatus,
} from "./trash.js";

function preview(counts: number[], sizeText = "23.4 GB"): TrashPreview {
  const folders = counts.map((count, i) => ({ dir: `/photos/${i}`, count, bytes: count * 10 }));
  const total = counts.reduce((a, b) => a + b, 0);
  return {
    folders,
    failed: [],
    total_files: total,
    total_bytes: total * 10,
    size_text: sizeText,
  };
}

describe("trashedStatus", () => {
  test("counts the files it moved", () => {
    expect(trashedStatus({ moved: ["/a.ARW"], failed: [], unread: [] })).toBe(
      "Moved 1 file to the Trash",
    );
    expect(trashedStatus({ moved: ["/a.ARW", "/b.ARW", "/c.ARW"], failed: [], unread: [] })).toBe(
      "Moved 3 files to the Trash",
    );
  });

  test("appends the failures", () => {
    expect(
      trashedStatus({
        moved: ["/b.ARW", "/c.ARW"],
        failed: [{ path: "/a.ARW", message: "denied" }],
        unread: [],
      }),
    ).toBe("Moved 2 files to the Trash, 1 failed");
  });

  test("does not count a folder that could not be read as a failed move", () => {
    expect(
      trashedStatus({
        moved: [],
        failed: [],
        unread: [{ path: "/photos/locked", message: "denied" }],
      }),
    ).toBe("Moved 0 files to the Trash");
  });
});

describe("opensTarget", () => {
  test("holds for a target equal to the open folder, however it is spelled", () => {
    expect(opensTarget("/photos/2026", ["/photos/2026"], false)).toBe(true);
    expect(opensTarget("C:\\Photos\\2026", ["c:/Photos/2026/"], false)).toBe(true);
  });

  test("holds for an open folder under a target when recursive", () => {
    expect(opensTarget("/photos/2026/0101", ["/other", "/photos/2026"], true)).toBe(true);
  });

  test("does not hold for an open folder under a target when not recursive", () => {
    expect(opensTarget("/photos/2026/0101", ["/photos/2026"], false)).toBe(false);
  });

  test("does not hold for an unrelated folder or a sibling sharing a prefix", () => {
    expect(opensTarget("/photos/2025", ["/photos/2026"], true)).toBe(false);
    expect(opensTarget("/photos/2026b", ["/photos/2026"], true)).toBe(false);
    expect(opensTarget("/photos", ["/photos/2026"], true)).toBe(false);
  });
});

describe("the confirmation dialog", () => {
  test("lists one row per folder with rejects, with its count", () => {
    expect(folderRows(preview([1, 0, 147]))).toEqual([
      "/photos/0: 1 rejected file",
      "/photos/2: 147 rejected files",
    ]);
  });

  test("shows a Windows folder without the verbatim prefix", () => {
    expect(shownPath("\\\\?\\C:\\Photos\\2026")).toBe("C:\\Photos\\2026");
    expect(shownPath("\\\\?\\UNC\\nas\\photos")).toBe("\\\\nas\\photos");
    expect(shownPath("/photos/2026")).toBe("/photos/2026");
  });

  test("folds the folders with no rejects into one line", () => {
    expect(emptyFoldersLine(preview([3, 1]))).toBeNull();
    expect(emptyFoldersLine(preview([3, 0]))).toBe("1 more folder with no rejects");
    expect(emptyFoldersLine(preview([0, 3, ...Array<number>(11).fill(0)]))).toBe(
      "12 more folders with no rejects",
    );
  });

  test('drops "more" when no folder row is shown above it', () => {
    expect(emptyFoldersLine(preview([0]))).toBe("1 folder with no rejects");
    expect(emptyFoldersLine(preview([0, 0]))).toBe("2 folders with no rejects");
  });

  test("asks with the total count and size", () => {
    expect(totalLine(preview([100, 48]))).toBe("Move 148 rejected files (23.4 GB) to the Trash?");
    expect(totalLine(preview([1], "24.0 MB"))).toBe("Move 1 rejected file (24.0 MB) to the Trash?");
    expect(totalLine(preview([0]))).toBe("No rejected files to move to the Trash");
  });

  test("lists a folder that could not be read as left out", () => {
    expect(failureText({ path: "/photos/locked", message: "denied" })).toBe(
      "/photos/locked: could not be read, left out: denied",
    );
  });

  test("disables the run when there is nothing to move", () => {
    expect(canRun(preview([0, 0]))).toBe(false);
    expect(canRun(preview([0, 2]))).toBe(true);
  });
});

describe("TrashFlow", () => {
  const target = { dirs: ["/photos"], recursive: true };

  test("refuses a second start while one is under way", () => {
    const flow = new TrashFlow();
    expect(flow.start(target)).toBe(true);
    expect(flow.start(target)).toBe(false);
    expect(flow.isOpen).toBe(false);
    expect(flow.previewed()).toBe(true);
    expect(flow.isOpen).toBe(true);
    expect(flow.run()).toEqual(target);
    expect(flow.start(target)).toBe(false);
    flow.end();
    expect(flow.busy).toBe(false);
    expect(flow.start(target)).toBe(true);
  });

  test("closes a preview on dismiss but not a running move", () => {
    const flow = new TrashFlow();
    flow.start(target);
    flow.previewed();
    expect(flow.dismiss()).toBe(true);
    expect(flow.run()).toBeNull();
    flow.start(target);
    flow.previewed();
    flow.run();
    expect(flow.dismiss()).toBe(false);
    expect(flow.isOpen).toBe(true);
  });
});
