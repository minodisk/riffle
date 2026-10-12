// `recommended` keeps the bursts in capture order and orders the frames of
// each by the personal ranking's score, highest first; a frame without a
// score follows the scored ones of its burst in capture order.
export type SortKey = "name" | "capture" | "rating" | "recommended";

export interface SortFacts {
  captureTime?: string;
  subsec?: string;
  rating?: number;
  // The id `groupBursts` gave the file's burst, in capture order.
  burst?: number;
  // The ranking's score within the burst.
  recommended?: number;
}

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

function compareStrings(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}

function compareSubsec(a: string, b: string): number {
  const width = Math.max(a.length, b.length);
  return compareStrings(a.padEnd(width, "0"), b.padEnd(width, "0"));
}

function compareCapture(a: SortFacts, b: SortFacts): number {
  const ta = a.captureTime;
  const tb = b.captureTime;
  if (ta === undefined || tb === undefined) {
    return ta === tb ? 0 : ta === undefined ? 1 : -1;
  }
  return compareStrings(ta, tb) || compareSubsec(a.subsec ?? "", b.subsec ?? "");
}

function ratingRank(rating: number | undefined): number {
  if (rating === undefined || rating === 0) return 6;
  if (rating < 0) return 7;
  return 6 - rating;
}

export function orderFiles(
  key: SortKey,
  paths: readonly string[],
  lookup: (path: string) => SortFacts,
): string[] {
  const rows = paths.map((path) => ({ path, name: baseName(path), facts: lookup(path) }));
  rows.sort((a, b) => {
    if (key === "capture") {
      const byTime = compareCapture(a.facts, b.facts);
      if (byTime !== 0) return byTime;
    } else if (key === "recommended") {
      const byBurst = (a.facts.burst ?? Infinity) - (b.facts.burst ?? Infinity);
      if (byBurst !== 0 && !Number.isNaN(byBurst)) return byBurst;
      const sa = a.facts.recommended;
      const sb = b.facts.recommended;
      if (sa !== sb) {
        if (sa === undefined || sb === undefined) return sa === undefined ? 1 : -1;
        return sb - sa;
      }
      const byTime = compareCapture(a.facts, b.facts);
      if (byTime !== 0) return byTime;
    } else if (key === "rating") {
      const byRating = ratingRank(a.facts.rating) - ratingRank(b.facts.rating);
      if (byRating !== 0) return byRating;
    }
    return compareStrings(a.name, b.name);
  });
  return rows.map((row) => row.path);
}
