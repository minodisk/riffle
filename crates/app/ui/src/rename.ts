// The inline rename editors' DOM-free decisions, shared by the folder tree
// and the strip: what a key does to a live edit, which typed name is
// worth sending, and the slow second click that starts an edit.

export type Decision = "confirm" | "cancel";

export interface InlineRename {
  kind: "folder" | "file";
  path: string;
  original: string;
  value: string;
  committed: boolean;
}

export function inlineRename(
  kind: InlineRename["kind"],
  path: string,
  original: string,
): InlineRename {
  return { kind, path, original, value: original, committed: false };
}

// Enter and Escape end the edit; every other key is the input's own.
export function editKey(key: string | null): Decision | "native" {
  if (key === "enter") {
    return "confirm";
  }
  if (key === "escape") {
    return "cancel";
  }
  return "native";
}

// The name to rename to, or `null` when there is nothing to rename: an empty
// name or the current one just ends the edit.
export function confirmName(original: string, value: string): string | null {
  const name = value.trim();
  return name === "" || name === original ? null : name;
}

// How much of a file name the editor preselects: the stem, up to its last
// dot, so typing replaces the name and keeps the extension. A name with no
// dot (or only a leading one) is selected whole.
export function stemLength(name: string): number {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? dot : name.length;
}

// The edit's decision, once: the blur that follows an Enter or an Escape
// (the input is removed, or focus moves on) must not decide a second time.
export function commit(rename: InlineRename, decision: Decision): Decision | null {
  if (rename.committed) {
    return null;
  }
  rename.committed = true;
  return decision;
}

export const SLOW_CLICK_DELAY = 500;

// A click on the name of the item that is already current arms a rename,
// which starts `SLOW_CLICK_DELAY` later unless something disarms it first
// (a double-click, another click, a key).
export class SlowClick {
  private armed: { path: string; at: number } | null = null;

  click(path: string, alreadyCurrent: boolean, now: number): boolean {
    this.armed = alreadyCurrent ? { path, at: now } : null;
    return alreadyCurrent;
  }

  cancel(): void {
    this.armed = null;
  }

  due(path: string, now: number): boolean {
    const armed = this.armed;
    if (armed === null || armed.path !== path) {
      return false;
    }
    if (now - armed.at < SLOW_CLICK_DELAY) {
      return false;
    }
    this.armed = null;
    return true;
  }
}
