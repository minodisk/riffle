import { describe, expect, test } from "vitest";
import { displayKey, isModifierCode, keyName } from "./keys.js";

function press(
  key: string,
  code: string,
  mods: { ctrl?: boolean; alt?: boolean; shift?: boolean; meta?: boolean } = {},
) {
  return keyName({
    key,
    code,
    ctrlKey: mods.ctrl ?? false,
    altKey: mods.alt ?? false,
    shiftKey: mods.shift ?? false,
    metaKey: mods.meta ?? false,
  });
}

describe("keyName", () => {
  test("plain keys keep the lower-cased event.key", () => {
    expect(press("j", "KeyJ")).toBe("j");
    expect(press(" ", "Space")).toBe("space");
    expect(press("ArrowLeft", "ArrowLeft")).toBe("arrowleft");
    expect(press("Escape", "Escape")).toBe("escape");
  });

  test("shift is part of the name", () => {
    expect(press("J", "KeyJ", { shift: true })).toBe("shift+j");
  });

  test("ctrl+alt names stay compatible", () => {
    expect(press("1", "Digit1", { ctrl: true, alt: true })).toBe("ctrl+alt+1");
    expect(press("¡", "Digit1", { ctrl: true, alt: true })).toBe("ctrl+alt+1");
    expect(press("!", "Digit1", { ctrl: true, alt: true, shift: true })).toBe("ctrl+alt+shift+1");
  });

  test("modifiers come in a fixed order", () => {
    expect(press("k", "KeyK", { meta: true })).toBe("meta+k");
    expect(press("K", "KeyK", { ctrl: true, alt: true, shift: true, meta: true })).toBe(
      "ctrl+alt+shift+meta+k",
    );
  });

  test("punctuation is named by its unmodified character", () => {
    expect(press("–", "Minus", { alt: true })).toBe("alt+-");
    expect(press(",", "Comma", { meta: true })).toBe("meta+,");
    expect(press("`", "Backquote", { meta: true })).toBe("meta+`");
    expect(press(" ", "Space", { ctrl: true })).toBe("ctrl+space");
  });

  test("a lone modifier has no name", () => {
    expect(press("Control", "ControlLeft", { ctrl: true })).toBeNull();
    expect(press("Alt", "AltLeft", { ctrl: true, alt: true })).toBeNull();
    expect(press("Shift", "ShiftLeft", { shift: true })).toBeNull();
    expect(press("Meta", "MetaLeft", { meta: true })).toBeNull();
    expect(press("control", "ControlLeft")).toBeNull();
    expect(press("shift", "ShiftLeft")).toBeNull();
    expect(press("alt", "AltLeft")).toBeNull();
    expect(press("meta", "MetaLeft")).toBeNull();
    expect(press("control", "ControlLeft", { ctrl: true })).toBeNull();
    expect(press("shift", "ShiftLeft", { shift: true })).toBeNull();
    expect(press("alt", "AltLeft", { alt: true })).toBeNull();
    expect(press("meta", "MetaLeft", { meta: true })).toBeNull();
    expect(press("Control", "ControlRight", { ctrl: true })).toBeNull();
    expect(press("Shift", "ShiftRight", { shift: true })).toBeNull();
    expect(press("Alt", "AltRight", { alt: true })).toBeNull();
    expect(press("Meta", "MetaRight", { meta: true })).toBeNull();
    expect(press("OS", "OSLeft", { meta: true })).toBeNull();
    expect(press("OS", "OSRight", { meta: true })).toBeNull();
    expect(press("Unidentified", "MetaLeft")).toBeNull();
  });
});

describe("displayKey", () => {
  test("space is shown as Space", () => {
    expect(displayKey("space")).toBe("Space");
  });

  test("other keys pass through unchanged", () => {
    expect(displayKey("j")).toBe("j");
  });
});

describe("isModifierCode", () => {
  test("modifier codes end a hold", () => {
    expect(isModifierCode("ControlLeft")).toBe(true);
    expect(isModifierCode("MetaRight")).toBe(true);
  });

  test("other codes do not", () => {
    expect(isModifierCode("KeyG")).toBe(false);
  });
});
