export type Binding = { action: string; keys: string[] };

// Letter keys are matched lower-cased, so Shift+J pages like j does. Ctrl+Alt
// keys are named from `event.code`, as Option changes `event.key` on macOS.
export function keyName(event: KeyboardEvent): string {
  if (event.ctrlKey && event.altKey && !event.metaKey) {
    const code = event.code.replace(/^(Digit|Key|Numpad)/, "");
    const named: Record<string, string> = { Minus: "-", Equal: "=", Space: "space" };
    return `ctrl+alt+${(named[code] ?? code).toLowerCase()}`;
  }
  return event.key === " " ? "space" : event.key.toLowerCase();
}

// Ctrl+Alt is the only modified form bound; Ctrl-only, Alt-only and Meta
// combinations are left to the system.
export function isUnboundModifier(event: KeyboardEvent): boolean {
  return event.metaKey || event.ctrlKey !== event.altKey;
}
