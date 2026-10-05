// The inline rename editors' DOM-free decisions, shared by the folder tree
// and the strip: what a key does to a live edit, which typed name is
// worth sending, and the slow second click that starts an edit.

import { relation } from "./tree.js";

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

// A confirmed rename `IdleGate` holds until the scan ends: its cell shows
// `name` in the pending style meanwhile.
export interface Pending {
  path: string;
  name: string;
}

export const PENDING_TITLE = "Renames when the scan finishes";

// The name a cell shows: the pending one while a rename of `path` is held.
export function displayName(pending: Pending | null, path: string, real: string): string {
  return pending !== null && pending.path === path ? pending.name : real;
}

export type Outcome = "keep" | "cancel" | { rename: string };

// A confirmed edit on a cell whose rename is held: an empty or unchanged
// (pending) name keeps it, the original name cancels it, any other replaces it.
export function pendingOutcome(real: string, pendingName: string, typed: string): Outcome {
  const name = typed.trim();
  if (name === "" || name === pendingName) {
    return "keep";
  }
  return name === real ? "cancel" : { rename: name };
}

// A confirmed edit's outcome, held rename or not; "keep" leaves things as
// they are.
export function editOutcome(
  pending: Pending | null,
  path: string,
  real: string,
  typed: string,
): Outcome {
  if (pending !== null && pending.path === path) {
    return pendingOutcome(real, pending.name, typed);
  }
  const name = confirmName(real, typed);
  return name === null ? "keep" : { rename: name };
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

// The folders whose `rename_folder` invoke is in flight: until it settles,
// the tree still draws them and everything under them at paths about to
// move, so opening one of those is refused.
export class RenamesInFlight {
  private paths: string[] = [];

  start(path: string): void {
    this.paths.push(path);
  }

  settle(path: string): void {
    const at = this.paths.indexOf(path);
    if (at !== -1) {
      this.paths.splice(at, 1);
    }
  }

  blocks(path: string, ignoreCase: boolean): boolean {
    return this.paths.some((dir) => relation(path, dir, ignoreCase) !== null);
  }
}
