import { describe, expect, test } from "vitest";
import { labelNamesPayload } from "./labels.js";

describe("labelNamesPayload", () => {
  test("trims every field and keeps blanks for the backend's fallback", () => {
    const values: Record<string, string> = {
      red: " レッド ",
      yellow: "イエロー",
      green: "",
      blue: "  ",
      purple: "Purple",
    };
    expect(labelNamesPayload((color) => values[color])).toEqual({
      red: "レッド",
      yellow: "イエロー",
      green: "",
      blue: "",
      purple: "Purple",
    });
  });
});
