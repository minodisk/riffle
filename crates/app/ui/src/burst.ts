// Burst grouping: consecutive frames whose capture times lie within a gap of
// each other. Free of DOM and Tauri so it is tested without mocks.

import { orderFiles, type SortFacts } from "./sort.js";

// The largest gap between two frames of one burst. Leica files carry whole
// seconds only, so anything below 1 s would split every Leica burst.
export const BURST_GAP_MS = 1000;

export interface BurstMember {
  burst: number;
  position: number;
  size: number;
}

// What a strip cell needs to draw its part of a burst's bracket: whether the
// displayed cell above and below belong to the same burst.
export interface BurstMark {
  first: boolean;
  last: boolean;
}

function captureMs(facts: SortFacts): number | null {
  const match = /^(\d{4}):(\d{2}):(\d{2}) (\d{2}):(\d{2}):(\d{2})$/.exec(facts.captureTime ?? "");
  if (match === null) {
    return null;
  }
  const [year, month, day, hour, minute, second] = match.slice(1).map(Number);
  const ms = Date.UTC(year, month - 1, day, hour, minute, second);
  if (Number.isNaN(ms)) {
    return null;
  }
  const subsec = (facts.subsec ?? "").slice(0, 3).padEnd(3, "0");
  return ms + (/^\d{3}$/.test(subsec) ? Number(subsec) : 0);
}

// Every path's burst id (in capture order), its 0-based position within the
// burst and the burst's size. A file without a usable capture time is a burst
// of its own.
export function groupBursts(
  paths: readonly string[],
  lookup: (path: string) => SortFacts,
  gapMs: number = BURST_GAP_MS,
): Map<string, BurstMember> {
  const groups: string[][] = [];
  let previous: number | null = null;
  for (const path of orderFiles("capture", paths, lookup)) {
    const ms = captureMs(lookup(path));
    if (ms === null || previous === null || ms - previous > gapMs) {
      groups.push([path]);
    } else {
      groups[groups.length - 1].push(path);
    }
    previous = ms;
  }
  const members = new Map<string, BurstMember>();
  groups.forEach((group, burst) => {
    group.forEach((path, position) => {
      members.set(path, { burst, position, size: group.length });
    });
  });
  return members;
}

// Per displayed path, `null` unless it is in a burst of two or more.
export function burstMarks(
  files: readonly string[],
  members: ReadonlyMap<string, BurstMember>,
): (BurstMark | null)[] {
  const burstAt = (at: number): number | undefined => {
    const member = files[at] === undefined ? undefined : members.get(files[at]);
    return member !== undefined && member.size > 1 ? member.burst : undefined;
  };
  return files.map((_, at) => {
    const burst = burstAt(at);
    if (burst === undefined) {
      return null;
    }
    return { first: burstAt(at - 1) !== burst, last: burstAt(at + 1) !== burst };
  });
}

// The index `burstNext` / `burstPrevious` move to over the displayed files'
// burst ids: the first displayed file of the next burst, or the first of the
// current burst unless already there, else the first of the previous one.
// Clamps at the ends.
export function burstStep(ids: readonly number[], current: number, direction: -1 | 1): number {
  const start = (at: number): number => {
    while (at > 0 && ids[at - 1] === ids[at]) {
      at -= 1;
    }
    return at;
  };
  if (direction === 1) {
    let at = current;
    while (at < ids.length - 1 && ids[at + 1] === ids[current]) {
      at += 1;
    }
    return at < ids.length - 1 ? at + 1 : current;
  }
  const first = start(current);
  if (first !== current) {
    return first;
  }
  return first > 0 ? start(first - 1) : current;
}

// The index `burstFrameNext` / `burstFramePrevious` move to over the displayed
// files' burst ids: the neighbouring displayed file when it is in the same
// burst run, else `current`.
export function burstFrameStep(ids: readonly number[], current: number, direction: -1 | 1): number {
  const next = current + direction;
  return next >= 0 && next < ids.length && ids[next] === ids[current] ? next : current;
}
