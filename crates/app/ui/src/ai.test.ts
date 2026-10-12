import { describe, expect, test } from "vitest";
import { type AiState, aiActive, aiStatus } from "./ai.js";

const base: AiState = {
  enabled: false,
  ready: false,
  downloaded: null,
  total: 378_478_689,
  error: null,
};

describe("aiStatus", () => {
  test("names the download size while off with no model", () => {
    expect(aiStatus(base)).toBe("Off; turning it on downloads the model (378 MB)");
  });

  test("shows Off once the model is in place", () => {
    expect(aiStatus({ ...base, ready: true })).toBe("Off");
  });

  test("shows the download progress", () => {
    expect(aiStatus({ ...base, enabled: true, downloaded: 120_500_000 })).toBe(
      "Downloading the model: 120 / 378 MB",
    );
  });

  test("shows On with the model in place", () => {
    expect(aiStatus({ ...base, enabled: true, ready: true })).toBe("On");
  });

  test("shows a failed download", () => {
    const error = "The model download failed: SHA-256 00, expected 0d";
    expect(aiStatus({ ...base, enabled: true, error })).toBe(error);
  });
});

describe("aiActive", () => {
  test("needs both the setting and the model", () => {
    expect(aiActive({ ...base, enabled: true, ready: true })).toBe(true);
    expect(aiActive({ ...base, enabled: true })).toBe(false);
    expect(aiActive({ ...base, ready: true })).toBe(false);
  });
});
