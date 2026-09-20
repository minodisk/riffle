import { type Binding, displayKey } from "./keys.js";

export type EmptyState = "none" | "no-folder" | "no-files" | "filtered";

export const NO_FILES_TEXT = "This folder has no ARW or DNG files.";
export const FILTERED_TEXT = "No files match the current filter.";

const OPEN_TEXT = "Drop a folder onto the window, or click here to choose one.";

export function emptyState(openDir: string | null, total: number, shown: number): EmptyState {
  if (openDir === null) {
    return "no-folder";
  }
  if (total === 0) {
    return "no-files";
  }
  if (shown === 0) {
    return "filtered";
  }
  return "none";
}

// Before the keymap resolves `bindings` is empty, so the key clause is left
// out rather than showing a placeholder.
export function openHint(bindings: Binding[]): string {
  const keys = bindings.find((b) => b.action === "open")?.keys ?? [];
  if (keys.length === 0) {
    return OPEN_TEXT;
  }
  return `${OPEN_TEXT} Or press ${keys.map(displayKey).join(" or ")}.`;
}
