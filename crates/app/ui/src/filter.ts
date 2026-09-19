import { type Exif, type ExifGroup, exifKey } from "./exif.js";

// The filter menu in the strip pane, after PhotoLab's: the checked items of
// one group are OR-ed, the groups AND-ed, and a group with nothing checked
// lets everything through. `0` stars is unrated, which a reject also counts
// as, since it carries no stars here.
export type Flag = "picked" | "untagged" | "rejected";

export interface FilterState {
  flags: Set<Flag>;
  stars: Set<number>;
  exif: Map<ExifGroup, Set<string>>;
}

export interface Judgement {
  rating: number | null;
  pick: boolean;
  label: string | null;
}

export function passes(
  state: FilterState,
  { rating, pick }: Judgement,
  exif: Exif | null | undefined,
): boolean {
  const flag: Flag = pick ? "picked" : rating === -1 ? "rejected" : "untagged";
  const stars = rating === null || rating === -1 ? 0 : rating;
  return (
    (state.flags.size === 0 || state.flags.has(flag)) &&
    (state.stars.size === 0 || state.stars.has(stars)) &&
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
