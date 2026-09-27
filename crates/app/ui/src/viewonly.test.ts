import { describe, expect, test } from "vitest";
import { isViewOnly, sortFor } from "./viewonly.js";

describe("isViewOnly", () => {
  test("is true when every path is a JPEG, whatever the case", () => {
    expect(isViewOnly(["/d/a.jpg", "/d/b.JPG", "/d/c.jpeg", "/d/d.JPEG"])).toBe(true);
  });

  test("is false for an empty list", () => {
    expect(isViewOnly([])).toBe(false);
  });

  test("is false when any path is a RAW", () => {
    expect(isViewOnly(["/d/a.jpg", "/d/b.ARW"])).toBe(false);
    expect(isViewOnly(["/d/a.ARW", "/d/b.dng"])).toBe(false);
  });

  test("looks at the extension only", () => {
    expect(isViewOnly(["/d/a.jpg.ARW"])).toBe(false);
    expect(isViewOnly(["/d.jpg/a.ARW"])).toBe(false);
    expect(isViewOnly(["/d/ajpg"])).toBe(false);
  });
});

describe("sortFor", () => {
  test("forces capture order in view-only mode", () => {
    for (const key of ["name", "capture", "rating"] as const) {
      expect(sortFor(true, key)).toBe("capture");
    }
  });

  test("keeps the user's key otherwise", () => {
    for (const key of ["name", "capture", "rating"] as const) {
      expect(sortFor(false, key)).toBe(key);
    }
  });
});
