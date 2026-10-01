// The paths `set_scan_focus` hands the running scan, so it produces the rows
// of the files the user is looking at before the rest of the folder.

// The current file first, then the files of the strip's range `[first, last]`
// nearest to it first, the earlier side before the later at the same distance.
// The range caps the list; the current file leads it even when it is outside.
export function scanFocusPaths(
  files: readonly string[],
  current: number,
  first: number,
  last: number,
): string[] {
  const head = files[current];
  if (head === undefined) {
    return [];
  }
  const from = Math.max(0, first);
  const to = Math.min(files.length - 1, last);
  const paths = [head];
  const reach = Math.max(current - from, to - current);
  for (let distance = 1; distance <= reach; distance += 1) {
    for (const at of [current - distance, current + distance]) {
      if (at >= from && at <= to) {
        paths.push(files[at]);
      }
    }
  }
  return paths;
}
