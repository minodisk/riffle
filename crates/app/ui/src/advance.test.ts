import { describe, expect, test } from "vitest";
import { advancesAfter } from "./advance.js";

describe("advancesAfter", () => {
  test("advances after stars, reject and pick", () => {
    for (const action of ["rate1", "rate2", "rate3", "rate4", "rate5", "reject", "pick"]) {
      expect(advancesAfter(action)).toBe(true);
    }
  });

  test("stays after labels, corrections and navigation", () => {
    for (const action of [
      "red",
      "clearlabel",
      "unflag",
      "clear",
      "clearall",
      "next",
      "previous",
      "zoom",
    ]) {
      expect(advancesAfter(action)).toBe(false);
    }
  });
});
