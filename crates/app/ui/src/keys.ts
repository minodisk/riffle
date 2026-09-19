export type Binding = { action: string; keys: string[] };

type KeyEvent = Pick<KeyboardEvent, "key" | "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">;

const MODIFIER_KEYS = ["Control", "Alt", "Shift", "Meta"];

const CODE_NAMES: Record<string, string> = {
  Minus: "-",
  Equal: "=",
  Space: "space",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Semicolon: ";",
  Quote: "'",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Backquote: "`",
};

// A plain key is `event.key` lower-cased. With a modifier held, the name is
// `ctrl+alt+shift+meta+` (only those held) and the key named from
// `event.code`, as Option and Shift change `event.key`. A lone modifier has
// no name.
export function keyName(event: KeyEvent): string | null {
  if (MODIFIER_KEYS.includes(event.key)) {
    return null;
  }
  const modifiers = [
    event.ctrlKey && "ctrl",
    event.altKey && "alt",
    event.shiftKey && "shift",
    event.metaKey && "meta",
  ].filter((m): m is string => m !== false);
  if (modifiers.length === 0) {
    return event.key === " " ? "space" : event.key.toLowerCase();
  }
  const code = event.code.replace(/^(Digit|Key|Numpad)/, "");
  return `${modifiers.join("+")}+${(CODE_NAMES[code] ?? code).toLowerCase()}`;
}
