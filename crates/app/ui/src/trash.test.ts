import { describe, expect, test } from "vitest";
import { opensTarget, trashedStatus } from "./trash.js";

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
