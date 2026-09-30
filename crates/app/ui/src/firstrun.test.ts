import { describe, expect, test, vi } from "vitest";
import { FormatGate, asksLanguage, defaultPreset } from "./firstrun.js";
import type { LabelPreset } from "./labels.js";

describe("FormatGate", () => {
  test("starts closed", () => {
    expect(new FormatGate().isOpen).toBe(false);
  });

  test("defers an action until it opens", () => {
    const gate = new FormatGate();
    const action = vi.fn();
    gate.whenOpen(action);
    expect(action).not.toHaveBeenCalled();
    gate.open();
    expect(gate.isOpen).toBe(true);
    expect(action).toHaveBeenCalledOnce();
  });

  test("runs an action at once when already open", () => {
    const gate = new FormatGate();
    gate.open();
    const action = vi.fn();
    gate.whenOpen(action);
    expect(action).toHaveBeenCalledOnce();
  });

  test("runs the deferred action only once", () => {
    const gate = new FormatGate();
    const action = vi.fn();
    gate.whenOpen(action);
    gate.open();
    gate.open();
    expect(action).toHaveBeenCalledOnce();
  });

  test("opening with nothing deferred", () => {
    const gate = new FormatGate();
    gate.open();
    expect(gate.isOpen).toBe(true);
  });
});

describe("asksLanguage", () => {
  test("asks when the format writes XMP", () => {
    expect(asksLanguage("xmp")).toBe(true);
    expect(asksLanguage("both")).toBe(true);
  });

  test("does not ask for PhotoLab alone", () => {
    expect(asksLanguage("dop")).toBe(false);
  });
});

describe("defaultPreset", () => {
  const names = { red: "", yellow: "", green: "", blue: "", purple: "" };
  const presets: LabelPreset[] = [
    { code: "en", name: "English", names },
    { code: "ja", name: "日本語", names },
  ];

  test("picks the preset of the language's primary subtag", () => {
    expect(defaultPreset(presets, "ja-JP")?.code).toBe("ja");
    expect(defaultPreset(presets, "ja")?.code).toBe("ja");
  });

  test("ignores the case of the subtag", () => {
    expect(defaultPreset(presets, "JA-jp")?.code).toBe("ja");
  });

  test("falls back to the first preset", () => {
    expect(defaultPreset(presets, "fr-FR")?.code).toBe("en");
    expect(defaultPreset(presets, "")?.code).toBe("en");
  });

  test("has nothing to pick without presets", () => {
    expect(defaultPreset([], "ja-JP")).toBeUndefined();
  });
});
