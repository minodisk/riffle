import { describe, expect, test } from "vitest";
import { rejectedPaths, trashedStatus } from "./trash.js";

describe("rejectedPaths", () => {
  test("keeps the rejects in the given order", () => {
    const ratings = new Map([
      ["/a.ARW", -1],
      ["/b.ARW", 3],
      ["/c.ARW", -1],
    ]);
    expect(rejectedPaths(["/a.ARW", "/b.ARW", "/c.ARW", "/d.ARW"], ratings)).toEqual([
      "/a.ARW",
      "/c.ARW",
    ]);
  });

  test("is empty without a reject", () => {
    expect(rejectedPaths(["/a.ARW"], new Map([["/a.ARW", 0]]))).toEqual([]);
  });
});

describe("trashedStatus", () => {
  test("counts the files it moved", () => {
    expect(trashedStatus({ trashed: 1, failed: [] })).toBe("Moved 1 file to the Trash");
    expect(trashedStatus({ trashed: 3, failed: [] })).toBe("Moved 3 files to the Trash");
  });

  test("appends the failures", () => {
    expect(trashedStatus({ trashed: 2, failed: [{ path: "/a.ARW", message: "denied" }] })).toBe(
      "Moved 2 files to the Trash, 1 failed",
    );
  });
});
