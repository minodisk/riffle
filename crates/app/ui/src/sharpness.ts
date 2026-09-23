// The strip's sharpness cue: each file's score relative to its burst, or to
// the singles shot around it. Free of DOM and Tauri so it is tested without
// mocks.

import { orderFiles, type SortFacts } from "./sort.js";

// How many singles on each side a single is compared with (up to 5 frames).
export const SHARPNESS_RADIUS = 2;

export interface RelativeSharpness {
  ratio: number;
  best: boolean;
}

// What the cue needs of a file: its capture time (for the capture order), its
// score, and its burst id, `null` unless it is in a burst of two or more.
export interface SharpnessFacts extends SortFacts {
  score: number | null;
  burst: number | null;
}

// Per path over all files, whatever the filter or sort: `null` when the file
// has no score; otherwise its score over the maximum non-null score it is
// compared with, and whether it holds that maximum (every tied file does). A
// burst member is compared with every member of its burst, so when the burst's
// best frame is filtered out no displayed member is `best`. A single is
// compared with up to `radius` singles on each side in capture order, bursts
// skipped; a single without a score still takes a slot.
export function relativeSharpness(
  paths: readonly string[],
  lookup: (path: string) => SharpnessFacts,
  radius: number = SHARPNESS_RADIUS,
): Map<string, RelativeSharpness | null> {
  const burstMax = new Map<number, number>();
  const singles: (number | null)[] = [];
  const singlePaths: string[] = [];
  for (const path of orderFiles("capture", paths, lookup)) {
    const { score, burst } = lookup(path);
    if (burst === null) {
      singles.push(score);
      singlePaths.push(path);
    } else if (score !== null && score > (burstMax.get(burst) ?? -Infinity)) {
      burstMax.set(burst, score);
    }
  }
  const result = new Map<string, RelativeSharpness | null>();
  const relative = (score: number, max: number): RelativeSharpness => ({
    ratio: max > 0 ? score / max : 1,
    best: score === max,
  });
  singles.forEach((score, at) => {
    if (score === null) {
      result.set(singlePaths[at], null);
      return;
    }
    let max = score;
    const last = Math.min(singles.length - 1, at + radius);
    for (let other = Math.max(0, at - radius); other <= last; other += 1) {
      const value = singles[other];
      if (value !== null && value > max) {
        max = value;
      }
    }
    result.set(singlePaths[at], relative(score, max));
  });
  for (const path of paths) {
    const { score, burst } = lookup(path);
    if (burst !== null) {
      result.set(path, score === null ? null : relative(score, burstMax.get(burst) ?? score));
    }
  }
  return result;
}
