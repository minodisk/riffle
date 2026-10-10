import { type Exif, type ExifGroup, exifKey } from "./exif.js";
import type { EyeState } from "./eyes.js";
import type { MarkState } from "./focus.js";
import type { PickFlag } from "./selection.js";

// The filter menu in the strip pane, after PhotoLab's: the checked items of
// one group are OR-ed, the groups AND-ed, and a group with nothing checked
// lets everything through. `0` stars is unrated; a reject keeps its stars,
// independent of its flag. A label is keyed lowercased, or `none`
// when there is none; a label outside the menu's colors matches no item.
// The menu's `AF eye` items put `good`, `not_candidate` (Bad) or `unknown` in
// `candidates`, and they partition the files like the focus mark's states
// (`afEyeState` in `focus.ts`): a file whose state is not known yet counts as
// `unknown`.
// Its `Eyes` items put the AF face's stored eye state, `open`, `closed` or
// `unknown`, in `eyes`; a file without one counts as `unknown` the same way.
export type Flag = "picked" | "untagged" | "rejected";

// One item of the `AF eye` section: an AF eye state.
export type AfEye = MarkState;

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
  candidates: Set<AfEye>;
  eyes: Set<EyeState>;
  exif: Map<ExifGroup, Set<string>>;
}

// The remembered filter, the wire shape of the `filter` / `set_filter`
// commands: every section but the EXIF groups, which stay per session.
export interface StoredFilter {
  flags: Flag[];
  stars: number[];
  labels: string[];
  orientations: Orientation[];
  candidates: AfEye[];
  eyes: EyeState[];
}

export function toStored(state: FilterState): StoredFilter {
  return {
    flags: [...state.flags],
    stars: [...state.stars],
    labels: [...state.labels],
    orientations: [...state.orientations],
    candidates: [...state.candidates],
    eyes: [...state.eyes],
  };
}

// Replace the remembered sections' checks with `stored`, leaving `exif`.
export function applyStored(state: FilterState, stored: StoredFilter): void {
  const sections = [
    [state.flags, stored.flags],
    [state.stars, stored.stars],
    [state.labels, stored.labels],
    [state.orientations, stored.orientations],
    [state.candidates, stored.candidates],
    [state.eyes, stored.eyes],
  ] as [Set<string | number>, (string | number)[]][];
  for (const [set, values] of sections) {
    set.clear();
    for (const value of values) {
      set.add(value);
    }
  }
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
  afEye?: AfEye,
  eyes?: EyeState,
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
    (state.candidates.size === 0 || state.candidates.has(afEye ?? "unknown")) &&
    (state.eyes.size === 0 || state.eyes.has(eyes ?? "unknown")) &&
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
//
// `previous` is the list the user was looking at before `allFiles` was
// re-listed. When given and `anchor` is gone from `allFiles` (deleted from
// outside the app), the anchor's next surviving passing neighbour in
// `previous` wins, else its previous one, whether or not the anchor itself
// still passes.
export function anchorAfterFilter(
  allFiles: readonly string[],
  pass: (path: string) => boolean,
  anchor: string | undefined,
  previous?: readonly string[],
): string | undefined {
  if (anchor === undefined) {
    return undefined;
  }
  if (previous !== undefined) {
    const present = new Set(allFiles);
    if (!present.has(anchor)) {
      const survives = (path: string) => present.has(path) && pass(path);
      const from = previous.indexOf(anchor);
      const after = previous.slice(from + 1).find(survives);
      const before = previous.slice(0, Math.max(from, 0)).reverse().find(survives);
      return after ?? before;
    }
  }
  if (pass(anchor)) {
    return anchor;
  }
  const from = allFiles.indexOf(anchor);
  const after = allFiles.slice(from + 1).find(pass);
  const before = allFiles.slice(0, from).reverse().find(pass);
  return after ?? before;
}

// Whether `refilter` must rebuild `files`/`index`: the freshly filtered
// `next` list differs from the current `files`, or `force` says to rebuild
// regardless. `force` covers the first `folder_entries` refresh after a
// folder opens with a pending resume target: that refresh's list is often
// identical to the pre-entries one (same name order, no judgment filter
// active), yet the resume target still needs to become current.
export function filterListChanged(
  files: readonly string[],
  next: readonly string[],
  force: boolean,
): boolean {
  return force || next.length !== files.length || next.some((path, at) => path !== files[at]);
}
