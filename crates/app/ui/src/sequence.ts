// `File > Sequence JPEG Timestamps…`: the dialog's text and the flow from
// picking the folder (or right-clicking it in the folder tree) to the run's
// end. See `crates/app/src/sequence.rs` for
// the payloads.

export type SequenceFailure = { path: string; message: string };

export type SequenceRow = { path: string; old: string; new: string; changed: boolean };

// What `sequence_preview` returns.
export type SequencePreview = {
  output_dir: string;
  output_exists: boolean;
  rows: SequenceRow[];
  failed: SequenceFailure[];
};

// The `sequence-done` payload. A folder-level error arrives with `total: 0`
// and one failure keyed by `dir`.
export type SequenceDone = {
  run_id: number;
  dir: string;
  output_dir: string;
  written: number;
  total: number;
  failed: SequenceFailure[];
  canceled: boolean;
};

export const RUNNING_NOTE =
  "Cancel stops before the next file; the files written so far stay in the output folder, and the next run rebuilds it.";

function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? path;
}

export function rowText(row: SequenceRow): string {
  return `${baseName(row.path)}  ${row.old} -> ${row.new}`;
}

// Null when the output folder does not exist yet.
export function rebuildNotice(preview: SequencePreview): string | null {
  return preview.output_exists ? "will be rebuilt: its JPEG files are replaced" : null;
}

export function changedLine(rows: SequenceRow[]): string {
  const changed = rows.filter((row) => row.changed).length;
  return `${changed} of ${rows.length} ${rows.length === 1 ? "file gets" : "files get"} a new time`;
}

export function failureText(failure: SequenceFailure): string {
  return `${baseName(failure.path)}: ${failure.message}`;
}

export function progressStatus(done: number, total: number): string {
  return `sequencing ${done} / ${total}`;
}

export function doneStatus(done: SequenceDone): string {
  const head = done.canceled
    ? `canceled, ${done.written} of ${done.total} written`
    : `Wrote ${done.written} of ${done.total} ${done.total === 1 ? "file" : "files"} to ${done.output_dir}`;
  return done.failed.length === 0 ? head : `${head}, ${done.failed.length} failed`;
}

export type Phase = "idle" | "picking" | "previewing" | "previewed" | "running" | "done";

// The flow of one sequencing, from the menu item to `sequence-done`. The
// dialog is open while previewed and while running.
//
// `sequence_run` returns the run id after the run has started, so its events
// can arrive before the id is known: a `sequence-done` that comes first is
// kept until `started`, and a cancel asked for then is passed on by `started`.
// Progress that comes first is dropped; the next tick catches up.
export class SequenceFlow {
  phase: Phase = "idle";
  dir: string | null = null;
  runId: number | null = null;
  private early: SequenceDone | null = null;
  private cancelAsked = false;

  get isOpen(): boolean {
    return this.phase === "previewed" || this.phase === "running";
  }

  // True from `start()` through `done()`/`fail()`: the flow is under way even
  // while its dialog is not shown yet (picking the folder, previewing).
  get busy(): boolean {
    return this.phase !== "idle" && this.phase !== "done";
  }

  // False while a sequencing is already under way.
  start(): boolean {
    if (this.phase !== "idle" && this.phase !== "done") {
      return false;
    }
    this.phase = "picking";
    this.dir = null;
    this.runId = null;
    this.early = null;
    this.cancelAsked = false;
    return true;
  }

  // True when `dir` should be previewed; a dismissed picker ends the flow.
  picked(dir: string | null): boolean {
    if (this.phase !== "picking") {
      return false;
    }
    if (dir === null) {
      this.phase = "idle";
      return false;
    }
    this.dir = dir;
    this.phase = "previewing";
    return true;
  }

  previewed(): boolean {
    if (this.phase !== "previewing") {
      return false;
    }
    this.phase = "previewed";
    return true;
  }

  // The preview or `sequence_run` failed: the flow ends and the dialog closes.
  fail(): void {
    this.phase = "idle";
  }

  // The folder to run on, or null when there is no preview to run.
  run(): string | null {
    if (this.phase !== "previewed") {
      return null;
    }
    this.phase = "running";
    return this.dir;
  }

  // `sequence_run` returned `runId`: the `sequence-done` that already came,
  // if any, and whether a cancel was asked for in the meantime.
  started(runId: number): { done: SequenceDone | null; cancel: boolean } {
    if (this.phase !== "running") {
      return { done: null, cancel: false };
    }
    this.runId = runId;
    const done = this.early?.run_id === runId ? this.early : null;
    this.early = null;
    return { done, cancel: done === null && this.cancelAsked };
  }

  accepts(runId: number): boolean {
    return this.phase === "running" && this.runId === runId;
  }

  // True when `payload` ends this flow's run; the dialog then closes.
  done(payload: SequenceDone): boolean {
    if (this.phase !== "running") {
      return false;
    }
    if (this.runId === null) {
      this.early = payload;
      return false;
    }
    if (payload.run_id !== this.runId) {
      return false;
    }
    this.phase = "done";
    return true;
  }

  // The dialog's Cancel or Escape: close a preview, or cancel the run, whose
  // `sequence-done` closes the dialog. `cancel` carries the run id, or null
  // when it is not known yet (`started` passes the cancel on).
  dismiss(): { kind: "close" } | { kind: "cancel"; runId: number | null } | { kind: "none" } {
    switch (this.phase) {
      case "previewed":
        this.phase = "idle";
        return { kind: "close" };
      case "running":
        if (this.runId === null) {
          this.cancelAsked = true;
        }
        return { kind: "cancel", runId: this.runId };
      default:
        return { kind: "none" };
    }
  }
}
