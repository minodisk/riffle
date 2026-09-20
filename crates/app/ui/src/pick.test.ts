import { describe, expect, test } from "vitest";
import { effectivePick } from "./pick.js";

describe("effectivePick", () => {
  test("keeps a pick while `.dop` is selected", () => {
    expect(effectivePick(true, "dop")).toBe(true);
  });

  test("drops a pick kept across a switch to XMP", () => {
    expect(effectivePick(true, "xmp")).toBe(false);
  });

  test("leaves an unpicked file alone in either format", () => {
    expect(effectivePick(false, "dop")).toBe(false);
    expect(effectivePick(false, "xmp")).toBe(false);
  });
});
