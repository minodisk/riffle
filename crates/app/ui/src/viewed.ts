// Whether every file of the open folder's complete listing has been shown in
// the preview at least once. Paths in `viewed` that are no longer listed do
// not matter; an empty listing is never finished.
export function allViewed(allFiles: readonly string[], viewed: ReadonlySet<string>): boolean {
  return allFiles.length > 0 && allFiles.every((path) => viewed.has(path));
}

export interface ViewedBatch {
  dir: string;
  paths: string[];
}

// Serializes the `mark_viewed` writes: one in flight at a time, and the paths
// pushed meanwhile are collected, per folder, into the next call, so paging
// through a folder sends a few batches rather than one write per frame.
export function viewedWriter(
  send: (batch: ViewedBatch) => Promise<unknown>,
): (dir: string, path: string) => void {
  const pending = new Map<string, Set<string>>();
  let inFlight = false;
  const next = (): void => {
    const first = pending.entries().next();
    if (first.done === true) {
      inFlight = false;
      return;
    }
    const [dir, paths] = first.value;
    pending.delete(dir);
    inFlight = true;
    send({ dir, paths: [...paths] })
      .catch(() => undefined)
      .finally(next);
  };
  return (dir, path) => {
    const paths = pending.get(dir) ?? new Set<string>();
    paths.add(path);
    pending.set(dir, paths);
    if (!inFlight) {
      next();
    }
  };
}
