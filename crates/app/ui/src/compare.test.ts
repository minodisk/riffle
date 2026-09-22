import { describe, expect, test } from "vitest";
import { reconcileActive } from "./compare.js";

describe("reconcileActive", () => {
  test("keeps an active frame that is still displayed", () => {
    expect(reconcileActive(["/a", "/b"], "/b", "/a")).toBe("/b");
  });

  test("falls back to the focused candidate when the active frame disappears", () => {
    expect(reconcileActive(["/c", "/d"], "/b", "/d")).toBe("/d");
  });

  test("falls back to the first candidate when the focus is not displayed", () => {
    expect(reconcileActive(["/c", "/d"], "/b", "/a")).toBe("/c");
  });

  test("clears the active frame with no candidates", () => {
    expect(reconcileActive([], "/b", undefined)).toBeNull();
  });
});
