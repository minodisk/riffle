// One `Move Rejected to Trash` run, undone by restoring it from the Trash:
// its backend `run_id`, the RAWs it moved, and the folders it ran on.
export type TrashEntry = {
  kind: "trash";
  runId: number;
  count: number;
  dirs: string[];
  recursive: boolean;
};

// An undo entry: a batch of judgment states, or a trash run.
export type Entry<J> = J[] | TrashEntry;

export function isJudgments<J>(entry: Entry<J>): entry is J[] {
  return Array.isArray(entry);
}

// The redo entry of the undone trash run `entry`: the backend keeps only the
// files that came back, so it counts the `restored` RAWs alone, and there is
// nothing to redo when none came back.
export function undoneTrash(entry: TrashEntry, restored: string[]): TrashEntry | null {
  return restored.length === 0 ? null : { ...entry, count: restored.length };
}

// Rewrite every judgment of a batch through `fn`, leaving a trash entry as is.
export function mapJudgments<J>(fn: (judgment: J) => J): (entry: Entry<J>) => Entry<J> {
  return (entry) => (isJudgments(entry) ? entry.map(fn) : entry);
}

// A bounded stack of undo entries, used for both undo and redo: pushing past
// `limit` drops the oldest.
export class History<T> {
  private entries: T[] = [];

  constructor(private readonly limit: number) {}

  push(entry: T): void {
    this.entries.push(entry);
    if (this.entries.length > this.limit) {
      this.entries.shift();
    }
  }

  pop(): T | undefined {
    return this.entries.pop();
  }

  peek(): T | undefined {
    return this.entries.at(-1);
  }

  // Drop `entry` wherever it sits; later entries may have been pushed since.
  remove(entry: T): void {
    const at = this.entries.lastIndexOf(entry);
    if (at !== -1) {
      this.entries.splice(at, 1);
    }
  }

  // Drop every entry `match` accepts, for files that are gone from the
  // folder: undoing one would judge a path that no longer exists.
  removeWhere(match: (entry: T) => boolean): void {
    this.entries = this.entries.filter((entry) => !match(entry));
  }

  // Rewrite every entry through `fn`, e.g. to move a renamed file's
  // judgments onto its new path so undo still targets the right file.
  map(fn: (entry: T) => T): void {
    this.entries = this.entries.map(fn);
  }

  clear(): void {
    this.entries = [];
  }
}
