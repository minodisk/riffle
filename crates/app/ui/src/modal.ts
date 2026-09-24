// What a keydown does while the settings modal is open. None of them reaches
// the culling keymap.
export type ModalKey =
  | { kind: "add"; key: string }
  | { kind: "cancel" }
  | { kind: "close" }
  | { kind: "focus"; step: 1 | -1 }
  | { kind: "native" };

// The settings modal's open state and the shortcut row waiting for a key.
export class SettingsModal {
  private opened = false;
  capturing: string | null = null;

  get isOpen(): boolean {
    return this.opened;
  }

  // False when it is already open, so a second open does nothing.
  open(): boolean {
    if (this.opened) {
      return false;
    }
    this.opened = true;
    return true;
  }

  close(): void {
    this.opened = false;
    this.capturing = null;
  }

  // `key` is `keyName`'s name for the event, null for a lone modifier. While
  // a row captures, Escape cancels the capture and any other key is added;
  // otherwise Escape closes and Tab moves focus within the modal.
  key(key: string | null): ModalKey {
    if (this.capturing !== null) {
      if (key === null) {
        return { kind: "native" };
      }
      return key === "escape" ? { kind: "cancel" } : { kind: "add", key };
    }
    switch (key) {
      case "escape":
        return { kind: "close" };
      case "tab":
        return { kind: "focus", step: 1 };
      case "shift+tab":
        return { kind: "focus", step: -1 };
      default:
        return { kind: "native" };
    }
  }
}

// The index focus moves to among `count` focusable elements, wrapping around;
// `from` is -1 when the focus is on none of them.
export function cycleFocus(count: number, from: number, step: 1 | -1): number {
  if (from < 0) {
    return step === 1 ? 0 : count - 1;
  }
  return (from + step + count) % count;
}
