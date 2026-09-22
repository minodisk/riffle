// Keep an active comparison frame when it is still displayed. If it has
// disappeared, prefer the filmstrip focus when that is a candidate, then the
// first displayed candidate.
export function reconcileActive(
  candidates: readonly string[],
  active: string | null,
  focused: string | undefined,
): string | null {
  if (active !== null && candidates.includes(active)) return active;
  if (focused !== undefined && candidates.includes(focused)) return focused;
  return candidates[0] ?? null;
}

// Derive the displayed comparison from all of the state that can change it.
// Keeping this calculation together makes scan-time score updates and
// selection-only changes use the same rules as the initial comparison.
export function comparisonCandidates(
  files: readonly string[],
  focused: number,
  selected: ReadonlySet<string>,
  bursts: ReadonlyMap<string, { burst: number }>,
  sharpness: ReadonlyMap<string, number>,
): string[] {
  if (selected.size > 1) {
    return files.filter((path) => selected.has(path)).slice(0, 4);
  }
  const current = files[focused];
  const burst = current === undefined ? undefined : bursts.get(current)?.burst;
  if (burst === undefined) return current === undefined ? [] : [current];
  const ranked = files
    .filter((path) => bursts.get(path)?.burst === burst)
    .sort((a, b) => (sharpness.get(b) ?? -Infinity) - (sharpness.get(a) ?? -Infinity));
  return ranked.length < 2 ? [current] : [current, ranked[0]];
}

// Load the whole comparison as one transaction. If any frame fails, every
// successfully allocated sibling is disposed before the error is propagated.
export async function loadComparisonFrames<T>(
  paths: readonly string[],
  load: (path: string) => Promise<T>,
  dispose: (frame: T) => void,
): Promise<T[]> {
  const settled = await Promise.allSettled(
    paths.map((path) => Promise.resolve().then(() => load(path))),
  );
  const failure = settled.find((result) => result.status === "rejected");
  if (failure !== undefined) {
    for (const result of settled) {
      if (result.status === "fulfilled") dispose(result.value);
    }
    throw failure.reason;
  }
  return settled.map((result) => {
    if (result.status === "rejected") throw result.reason;
    return result.value;
  });
}

// Hit-test the comparison grid, in canvas-local coordinates. Returns the frame
// index under the point, or `null` in the gap between panes.
export function comparePaneAt(
  x: number,
  y: number,
  width: number,
  height: number,
  count: number,
): number | null {
  const columns = count <= 2 ? count : 2;
  const rows = Math.ceil(count / columns);
  const gap = 4;
  const cellWidth = (width - gap * (columns - 1)) / columns;
  const cellHeight = (height - gap * (rows - 1)) / rows;
  const col = Math.floor(x / (cellWidth + gap));
  const row = Math.floor(y / (cellHeight + gap));
  if (x - col * (cellWidth + gap) > cellWidth) return null;
  if (y - row * (cellHeight + gap) > cellHeight) return null;
  const at = row * columns + col;
  return at >= 0 && at < count ? at : null;
}
