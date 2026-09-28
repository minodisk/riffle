// What the strip carries from one list to the next when a rescan replaces it:
// the scroll offset, anchored on a file still under the viewport, and the
// cells whose file is still listed.

// The new `scrollLeft` after `previous` becomes `next`. The anchor is the
// focused file when its cell overlapped the viewport, else the first cell
// that did; the view moves by the anchor's index delta, or stays put when the
// anchor is gone, clamped to the new list's width either way.
export function carriedOffset(
  previous: readonly string[],
  next: readonly string[],
  offset: number,
  cellWidth: number,
  viewportWidth: number,
  current: number,
): number {
  const overlaps = (at: number): boolean =>
    at * cellWidth < offset + viewportWidth && (at + 1) * cellWidth > offset;
  const first = Math.max(0, Math.floor(offset / cellWidth));
  const anchorAt = current < previous.length && overlaps(current) ? current : first;
  let carried = offset;
  if (anchorAt < previous.length && overlaps(anchorAt)) {
    const nextAt = next.indexOf(previous[anchorAt]);
    if (nextAt !== -1) {
      carried = offset + (nextAt - anchorAt) * cellWidth;
    }
  }
  return Math.max(0, Math.min(carried, next.length * cellWidth - viewportWidth));
}

// Old index -> new index for each of `indices` whose path is still in `next`.
export function carriedIndices(
  previous: readonly string[],
  next: readonly string[],
  indices: Iterable<number>,
): Map<number, number> {
  const nextIndex = new Map(next.map((path, at) => [path, at]));
  const carried = new Map<number, number>();
  for (const at of indices) {
    const to = nextIndex.get(previous[at]);
    if (to !== undefined) {
      carried.set(at, to);
    }
  }
  return carried;
}
