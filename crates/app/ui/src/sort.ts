export type SortKey = "name" | "capture" | "rating";

export interface SortFacts {
  captureTime?: string;
  subsec?: string;
  rating?: number;
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
      const ta = a.facts.captureTime;
      const tb = b.facts.captureTime;
      if (ta === undefined || tb === undefined) {
        if (ta !== tb) return ta === undefined ? 1 : -1;
      } else {
        const byTime =
          compareStrings(ta, tb) || compareSubsec(a.facts.subsec ?? "", b.facts.subsec ?? "");
        if (byTime !== 0) return byTime;
      }
    } else if (key === "rating") {
      const byRating = ratingRank(a.facts.rating) - ratingRank(b.facts.rating);
      if (byRating !== 0) return byRating;
    }
    return compareStrings(a.name, b.name);
  });
  return rows.map((row) => row.path);
}
