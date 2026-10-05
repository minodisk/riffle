// The folder tree's `Rewrite Sidecars from Index…`: the confirmation
// dialog's text and the flow from the menu item through the dialog to the
// run's end. See `crates/app/src/foldersidecars.rs` for the payloads.

export type SidecarFormatName = "xmp" | "dop" | "both";

// What `rewrite_sidecars_preview` returns: the index rows with a judgment,
// those without one, the listed RAWs with no row (skipped), and the format
// the run writes.
export type RewritePreview = {
  judged: number;
  unjudged: number;
  skipped: number;
  format: SidecarFormatName;
};

// What `rewrite_sidecars_run` returns: the files whose sidecars were written,
// those still waiting for a write after the drain (a failure, or the drain
// timed out: `flushed` is false), which the next open of the folder retries.
export type RewriteSummary = { written: number; failed: number; flushed: boolean };

export const REWRITE_RUNNING_NOTE = "Writing the sidecars; this closes once every file is written.";

function files(count: number): string {
  return `${count} ${count === 1 ? "file" : "files"}`;
}

export function formatName(format: SidecarFormatName): string {
  switch (format) {
    case "xmp":
      return "XMP";
    case "dop":
      return ".dop";
    case "both":
      return "XMP and .dop";
  }
}

// The dialog's rows, leaving out a count of zero.
export function rewriteRows(preview: RewritePreview): string[] {
  const rows: string[] = [];
  if (preview.judged > 0) {
    rows.push(`${files(preview.judged)} with a judgment`);
  }
  if (preview.unjudged > 0) {
    const cleared = "any existing sidecar is cleared";
    rows.push(`${files(preview.unjudged)} without one (${cleared})`);
  }
  if (preview.skipped > 0) {
    rows.push(`${files(preview.skipped)} not in the index (skipped)`);
  }
  return rows;
}

export function rewriteTotalLine(preview: RewritePreview, name: string): string {
  const count = preview.judged + preview.unjudged;
  return `Rewrite the ${formatName(preview.format)} sidecars of ${files(count)} in ${name}?`;
}

export function rewrittenStatus(summary: RewriteSummary, name: string): string {
  const head = `Rewrote the sidecars of ${files(summary.written)} in ${name}`;
  return summary.failed === 0 ? head : `${head}, ${summary.failed} failed`;
}

export type SidecarKind = "rewrite" | "delete";

export type SidecarTarget = { kind: SidecarKind; dir: string };

// The flow of one rewrite (or delete) of a folder's sidecars, from the menu
// item through the dialog to the run's end. A second start while one is
// under way does nothing. The dialog is open while previewed and while
// running; a running rewrite cannot be canceled.
export class SidecarFlow {
  phase: "idle" | "previewing" | "previewed" | "running" = "idle";
  target: SidecarTarget | null = null;

  get isOpen(): boolean {
    return this.phase === "previewed" || this.phase === "running";
  }

  get busy(): boolean {
    return this.phase !== "idle";
  }

  start(target: SidecarTarget): boolean {
    if (this.phase !== "idle") {
      return false;
    }
    this.phase = "previewing";
    this.target = target;
    return true;
  }

  previewed(): boolean {
    if (this.phase !== "previewing") {
      return false;
    }
    this.phase = "previewed";
    return true;
  }

  // The target to run on, or null when there is no preview to run.
  run(): SidecarTarget | null {
    if (this.phase !== "previewed") {
      return null;
    }
    this.phase = "running";
    return this.target;
  }

  // True when the dialog should close: Cancel or Escape on a preview.
  dismiss(): boolean {
    if (this.phase !== "previewed") {
      return false;
    }
    this.end();
    return true;
  }

  // The preview failed or the run came back, either way.
  end(): void {
    this.phase = "idle";
    this.target = null;
  }
}
