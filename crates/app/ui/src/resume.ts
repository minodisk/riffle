import { anchorAfterFilter } from "./filter.js";

// The file to select when a folder opens: `undefined` (the first file) when
// nothing was remembered or the remembered file is no longer listed,
// otherwise the remembered file, or its nearest neighbor passing the filter
// when the filter hides it, as `refilter` picks when a judgment hides the
// current file.
export function resumeTarget(
  remembered: string | null,
  allFiles: readonly string[],
  order: readonly string[],
  pass: (path: string) => boolean,
): string | undefined {
  if (remembered === null || !allFiles.includes(remembered)) {
    return undefined;
  }
  return anchorAfterFilter(order, pass, remembered);
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
