import { describe, expect, test } from "vitest";
import {
  type TrashPreview,
  TrashFlow,
  canRun,
  emptyFoldersLine,
  failureText,
  folderRows,
  opensTarget,
  restoredInto,
  restoredStatus,
  shownPath,
  stillHeld,
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
    expect(trashedStatus({ moved: ["/a.ARW"], failed: [], unread: [], run_id: null })).toBe(
      "Moved 1 file to the Trash",
    );
    expect(
      trashedStatus({
        moved: ["/a.ARW", "/b.ARW", "/c.ARW"],
        failed: [],
        unread: [],
        run_id: null,
      }),
    ).toBe("Moved 3 files to the Trash");
  });

  test("appends the failures", () => {
    expect(
      trashedStatus({
        moved: ["/b.ARW", "/c.ARW"],
        failed: [{ path: "/a.ARW", message: "denied" }],
        unread: [],
        run_id: null,
      }),
    ).toBe("Moved 2 files to the Trash, 1 failed");
  });

  test("does not count a folder that could not be read as a failed move", () => {
    expect(
      trashedStatus({
        moved: [],
        failed: [],
        unread: [{ path: "/photos/locked", message: "denied" }],
        run_id: null,
      }),
    ).toBe("Moved 0 files to the Trash");
  });
});

describe("sidecar runs", () => {
  test("trashedStatus counts the sidecars moved", () => {
    const summary = { moved: ["/a.xmp", "/b.xmp"], failed: [], unread: [], run_id: 1 };
    expect(trashedStatus(summary, "sidecars")).toBe("Moved 2 sidecars to the Trash");
    expect(trashedStatus({ ...summary, moved: ["/a.xmp"] }, "sidecars")).toBe(
      "Moved 1 sidecar to the Trash",
    );
    expect(
      trashedStatus(
        { ...summary, failed: [{ path: "/c.ARW", message: "c.xmp: denied" }] },
        "sidecars",
      ),
    ).toBe("Moved 2 sidecars to the Trash, 1 failed");
  });

  test("restoredStatus counts the sidecars that did not fail", () => {
    const run = { what: "sidecars" as const, count: 3 };
    expect(restoredStatus({ restored: [], failed: [] }, run)).toBe(
      "Restored 3 sidecars from the Trash",
    );
    expect(
      restoredStatus({ restored: [], failed: [{ path: "/a.xmp", message: "gone" }] }, run),
    ).toBe("Restored 2 sidecars from the Trash, 1 failed");
    expect(restoredStatus({ restored: [], failed: [] }, { ...run, count: 1 })).toBe(
      "Restored 1 sidecar from the Trash",
    );
  });

  test("restoredStatus of a run of RAWs counts the RAWs whatever the count", () => {
    expect(restoredStatus({ restored: ["/a.ARW"], failed: [] }, { count: 5 })).toBe(
      "Restored 1 file from the Trash",
    );
  });
});

describe("restoredStatus", () => {
  test("counts the files that came back", () => {
    expect(restoredStatus({ restored: ["/a.ARW"], failed: [] })).toBe(
      "Restored 1 file from the Trash",
    );
    expect(restoredStatus({ restored: ["/a.ARW", "/b.ARW"], failed: [] })).toBe(
      "Restored 2 files from the Trash",
    );
  });

  test("appends the failures", () => {
    expect(
      restoredStatus({
        restored: ["/b.ARW"],
        failed: [
          { path: "/a.ARW", message: "already exists at the original location" },
          { path: "/a.xmp", message: "left in the Trash: its RAW could not be restored" },
        ],
      }),
    ).toBe("Restored 1 file from the Trash, 2 failed");
  });
});

describe("restoredInto", () => {
  test("keeps the RAWs directly in the open folder that are not listed yet", () => {
    expect(
      restoredInto(
        "/photos/a",
        ["/photos/a/1.ARW", "/photos/a/2.ARW", "/photos/a/sub/3.ARW", "/photos/b/4.ARW"],
        ["/photos/a/2.ARW"],
      ),
    ).toEqual(["/photos/a/1.ARW"]);
  });

  test("compares Windows paths the way the folder tree does", () => {
    expect(restoredInto("C:\\photos", ["c:\\photos\\1.ARW"], [])).toEqual(["c:\\photos\\1.ARW"]);
  });

  test("matches a canonical path against an open folder spelled without the verbatim prefix", () => {
    expect(restoredInto("C:\\photos", ["\\\\?\\C:\\photos\\1.ARW"], [])).toEqual([
      "\\\\?\\C:\\photos\\1.ARW",
    ]);
    expect(
      restoredInto("\\\\?\\C:\\photos", ["\\\\?\\C:\\photos\\1.ARW"], ["\\\\?\\C:\\photos\\1.ARW"]),
    ).toEqual([]);
  });

  test("matches an open folder spelled in another case only when the case is ignored", () => {
    expect(restoredInto("c:\\Photos", ["C:\\photos\\1.ARW"], [], true)).toEqual([
      "C:\\photos\\1.ARW",
    ]);
    expect(restoredInto("c:\\Photos", ["C:\\photos\\1.ARW"], [])).toEqual([]);
  });
});

describe("stillHeld", () => {
  test("keeps a run the backend refused while a scan ran", () => {
    expect(stillHeld("a scan is running; wait for it to finish")).toBe(true);
  });

  test("drops a run the backend no longer holds, to undo or to redo", () => {
    expect(stillHeld("this run can no longer be undone")).toBe(false);
    expect(stillHeld("this run can no longer be redone")).toBe(false);
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

  test("holds for an open folder spelled in another case only when the case is ignored", () => {
    expect(opensTarget("d:\\photos\\2026", ["D:\\Photos\\2026"], false, true)).toBe(true);
    expect(opensTarget("d:\\photos\\2026", ["D:\\Photos\\2026"], false)).toBe(false);
    expect(opensTarget("/Photos/2026/0101", ["/photos/2026"], true, true)).toBe(true);
    expect(opensTarget("/Photos/2026/0101", ["/photos/2026"], true)).toBe(false);
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
