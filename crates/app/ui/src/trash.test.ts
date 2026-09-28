import { describe, expect, test } from "vitest";
import { trashedStatus } from "./trash.js";

describe("trashedStatus", () => {
  test("counts the files it moved", () => {
    expect(trashedStatus({ moved: ["/a.ARW"], failed: [] })).toBe("Moved 1 file to the Trash");
    expect(trashedStatus({ moved: ["/a.ARW", "/b.ARW", "/c.ARW"], failed: [] })).toBe(
      "Moved 3 files to the Trash",
    );
  });

  test("appends the failures", () => {
    expect(
      trashedStatus({
        moved: ["/b.ARW", "/c.ARW"],
        failed: [{ path: "/a.ARW", message: "denied" }],
      }),
    ).toBe("Moved 2 files to the Trash, 1 failed");
  });

  test("counts a folder that could not be read as a failure", () => {
    expect(
      trashedStatus({ moved: [], failed: [{ path: "/photos/locked", message: "denied" }] }),
    ).toBe("Moved 0 files to the Trash, 1 failed");
  });
});
