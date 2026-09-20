import { type Exif, type ExifGroup, exifKey } from "./exif.js";

// The filter menu in the strip pane, after PhotoLab's: the checked items of
// one group are OR-ed, the groups AND-ed, and a group with nothing checked
// lets everything through. `0` stars is unrated, which a reject also counts
// as, since it carries no stars here. A label is keyed lowercased, or `none`
// when there is none; a label outside the menu's colours matches no item.
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
  exif: Map<ExifGroup, Set<string>>;
}

export interface Judgement {
  rating: number | null;
  pick: boolean;
  label: string | null;
}

export function passes(
  state: FilterState,
  { rating, pick, label }: Judgement,
  exif: Exif | null | undefined,
  orientation: number | undefined,
): boolean {
  const flag: Flag = pick ? "picked" : rating === -1 ? "rejected" : "untagged";
  const stars = rating === null || rating === -1 ? 0 : rating;
  const labelKey = label === null ? "none" : label.toLowerCase();
  return (
    (state.flags.size === 0 || state.flags.has(flag)) &&
    (state.stars.size === 0 || state.stars.has(stars)) &&
    (state.labels.size === 0 || state.labels.has(labelKey)) &&
    (state.orientations.size === 0 ||
      (orientation !== undefined && state.orientations.has(orientationOf(orientation)))) &&
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
