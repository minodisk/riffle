import { relation } from "./tree.js";

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

// Whether opening `next` while `previous` is open switches to another
// folder, which refits the ranking on the views and judgments left behind.
// The first open after launch does not (the startup refit covers it), nor
// does reopening the same folder.
export function switchesFolder(previous: string | null, next: string, ignoreCase = false): boolean {
  return previous !== null && relation(next, previous, ignoreCase) !== "same";
}

// How long a file's decoded preview has to stay on screen before it counts
// as viewed, so frames flown past while holding an arrow key do not.
export const VIEW_DWELL_MS = 500;

export interface DwellTimer {
  shown(path: string): void;
  cancel(): void;
}

// One dwell clock for the file on screen: `shown` (re)starts it for `path`,
// skipping a path already `viewed`, and `cancel` stops it. `onViewed` fires
// once the path has stayed `VIEW_DWELL_MS` without either being called.
export function dwellTimer(
  viewed: (path: string) => boolean,
  onViewed: (path: string) => void,
): DwellTimer {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const cancel = (): void => {
    clearTimeout(timer);
    timer = undefined;
  };
  return {
    shown: (path) => {
      cancel();
      if (viewed(path)) {
        return;
      }
      timer = setTimeout(() => {
        timer = undefined;
        onViewed(path);
      }, VIEW_DWELL_MS);
    },
    cancel,
  };
}
