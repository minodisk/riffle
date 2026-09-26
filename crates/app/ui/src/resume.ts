// The remembered file to resume at when a folder opens: `undefined` (the
// first file) when nothing was remembered or the remembered file is no
// longer listed, otherwise the remembered file. This is only a candidate:
// entries (capture time, ratings, flags, labels) are not loaded yet at open,
// so whether it still passes the strip filter and where its nearest passing
// neighbor sits is decided later, by `firstEntriesAnchor` once the first
// `folder_entries` response lands.
export function resumeTarget(
  remembered: string | null,
  allFiles: readonly string[],
): string | undefined {
  if (remembered === null || !allFiles.includes(remembered)) {
    return undefined;
  }
  return remembered;
}

// The anchor `refilter` should resolve against on the first `folder_entries`
// refresh after a folder opens: the queued resume target when the open
// still has one pending, otherwise the provisional current file `refilter`
// would use on any other refresh.
export function firstEntriesAnchor(
  pending: string | undefined,
  provisional: string | undefined,
): string | undefined {
  return pending ?? provisional;
}

export interface LastViewed {
  dir: string;
  path: string;
}

// Serializes the `set_last_viewed` writes: one in flight at a time, and only
// the latest value pushed meanwhile is sent once it settles, so two writes
// cannot land out of order and a held-down arrow key collapses into a few.
export function lastViewedWriter(
  send: (value: LastViewed) => Promise<unknown>,
): (value: LastViewed) => void {
  let pending: LastViewed | null = null;
  let inFlight = false;
  const next = (): void => {
    if (pending === null) {
      inFlight = false;
      return;
    }
    const value = pending;
    pending = null;
    inFlight = true;
    send(value)
      .catch(() => undefined)
      .finally(next);
  };
  return (value) => {
    pending = value;
    if (!inFlight) {
      next();
    }
  };
}
