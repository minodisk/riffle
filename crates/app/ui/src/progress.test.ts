import { describe, expect, test } from "vitest";
import { progressWidth } from "./progress.js";

describe("progressWidth", () => {
  test("nothing to scan", () => {
    expect(progressWidth(0, 0)).toBe("0%");
  });

  test("partway", () => {
    expect(progressWidth(223, 500)).toBe("45%");
  });

  test("complete", () => {
    expect(progressWidth(500, 500)).toBe("100%");
  });

  test("done past total is clamped", () => {
    expect(progressWidth(600, 500)).toBe("100%");
  });
});
