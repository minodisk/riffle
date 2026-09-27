// A folder that holds only JPEGs opens view-only: the backend lists JPEGs
// only when there is no RAW, so the listed paths alone decide the mode. No
// judgment, sidecar or face detection applies, and the strip keeps capture
// order whatever sort the user chose for RAW folders.

import type { SortKey } from "./sort.js";

export const VIEW_ONLY_NOTE = "JPEG folder: view only";

export const VIEW_ONLY_REFUSAL = "culling does not apply to a JPEG folder";

const JPEG = /\.jpe?g$/i;

export function isViewOnly(paths: readonly string[]): boolean {
  return paths.length > 0 && paths.every((path) => JPEG.test(path));
}

// The strip's sort key: capture order in view-only mode, else the user's.
export function sortFor(viewOnly: boolean, key: SortKey): SortKey {
  return viewOnly ? "capture" : key;
}
