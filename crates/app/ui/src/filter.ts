import { type Exif, type ExifGroup, exifKey } from "./exif.js";
import type { FocusCandidate } from "./meta.js";
import type { PickFlag } from "./selection.js";

// The filter menu in the strip pane, after PhotoLab's: the checked items of
// one group are OR-ed, the groups AND-ed, and a group with nothing checked
// lets everything through. `0` stars is unrated; a reject keeps its stars,
// independent of its flag. A label is keyed lowercased, or `none`
// when there is none; a label outside the menu's colors matches no item.
// The menu's `AF eye` items put `candidate`, `not_candidate` or `unknown` in
// `candidates`; a file whose state is not known yet counts as `unknown`.
export type Flag = "picked" | "untagged" | "rejected";

// The displayed shape, decided by the EXIF Orientation tag alone: every
// sensor Riffle reads is landscape, so a quarter turn (6 or 8) is portrait.
export type Orientation = "portrait" | "landscape";

export function orientationOf(tag: number): Orientation {
  return tag === 6 || tag === 8 ? "portrait" : "landscape";
}

export interface FilterState {
  flags: Set<Flag>;
  stars: Set<number>;
  labels: Set<string>;
  orientations: Set<Orientation>;
  candidates: Set<FocusCandidate>;
  exif: Map<ExifGroup, Set<string>>;
}

export interface Judgment {
  rating: number | null;
  flag: PickFlag;
  label: string | null;
}

export function passes(
  state: FilterState,
  { rating, flag: pickFlag, label }: Judgment,
  exif: Exif | null | undefined,
  orientation: number | undefined,
  candidate?: FocusCandidate,
): boolean {
  const flag: Flag =
    pickFlag === "pick" ? "picked" : pickFlag === "reject" ? "rejected" : "untagged";
  const stars = rating ?? 0;
  const labelKey = label === null ? "none" : label.toLowerCase();
  return (
    (state.flags.size === 0 || state.flags.has(flag)) &&
    (state.stars.size === 0 || state.stars.has(stars)) &&
    (state.labels.size === 0 || state.labels.has(labelKey)) &&
    (state.orientations.size === 0 ||
      (orientation !== undefined && state.orientations.has(orientationOf(orientation)))) &&
    (state.candidates.size === 0 || state.candidates.has(candidate ?? "unknown")) &&
    [...state.exif].every(([group, set]) => {
      if (set.size === 0) {
        return true;
      }
      const key = exifKey(exif, group);
      return key !== null && set.has(key.label);
    })
  );
}

// The path that becomes current after a refilter: `anchor` if it still
// passes; otherwise the next passing path after it (in `allFiles` order), or
// the last one before it.
export function anchorAfterFilter(
  allFiles: string[],
  pass: (path: string) => boolean,
  anchor: string | undefined,
): string | undefined {
  if (anchor === undefined) {
    return undefined;
  }
  if (pass(anchor)) {
    return anchor;
  }
  const from = allFiles.indexOf(anchor);
  const after = allFiles.slice(from + 1).find(pass);
  const before = allFiles.slice(0, from).reverse().find(pass);
  return after ?? before;
}
