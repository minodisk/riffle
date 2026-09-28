import { relation } from "./tree.js";

// `File > Move Rejected in This Folder to Trash…` and the folder tree's
// `Move Rejected to Trash…` items: the confirmation dialog's text and flow,
// what the status line says once the run comes back, and whether the open
// folder was among the folders it trashed in.

export type TrashFailure = { path: string; message: string };

// One folder's row of `trash_rejected_preview`: its rejected RAWs and the
// bytes they and their sidecars take.
export type TrashFolder = { dir: string; count: number; bytes: number };

// What `trash_rejected_preview` returns: every folder read (those with no
// rejects too), the files and folders that could not be read, and the totals
// with the size formatted as the platform's file manager would.
export type TrashPreview = {
  folders: TrashFolder[];
  failed: TrashFailure[];
  total_files: number;
  total_bytes: number;
  size_text: string;
};

// The `Summary` that `trash_rejected_run` returns: the RAWs it moved, which the
// frontend prunes its state by, every file whose move to the Trash failed,
// and every file or folder `collect` never got to try because it could not
// be read (an unreadable folder, a sidecar that does not parse, an index
// query error).
export type TrashSummary = {
  moved: string[];
  failed: TrashFailure[];
  unread: TrashFailure[];
};

function files(count: number): string {
  return count === 1 ? "file" : "files";
}

// A folder as the user knows it: without the verbatim `\\?\` prefix the
// backend's canonical spelling carries on Windows.
export function shownPath(dir: string): string {
  if (dir.startsWith("\\\\?\\UNC\\")) {
    return `\\\\${dir.slice(8)}`;
  }
  return dir.startsWith("\\\\?\\") ? dir.slice(4) : dir;
}

// The dialog's rows: one per folder holding rejects, in the preview's order.
export function folderRows(preview: TrashPreview): string[] {
  return preview.folders
    .filter((folder) => folder.count > 0)
    .map((folder) => `${shownPath(folder.dir)}: ${folder.count} rejected ${files(folder.count)}`);
}

// The line standing in for the folders with no rejects, or null when every
// folder has some.
export function emptyFoldersLine(preview: TrashPreview): string | null {
  const empty = preview.folders.filter((folder) => folder.count === 0).length;
  if (empty === 0) {
    return null;
  }
  return `${empty} more ${empty === 1 ? "folder" : "folders"} with no rejects`;
}

export function totalLine(preview: TrashPreview): string {
  const count = preview.total_files;
  if (count === 0) {
    return "No rejected files to move to the Trash";
  }
  return `Move ${count} rejected ${files(count)} (${preview.size_text}) to the Trash?`;
}

export function canRun(preview: TrashPreview): boolean {
  return preview.total_files > 0;
}

export function failureText(failure: TrashFailure): string {
  return `${shownPath(failure.path)}: could not be read, left out: ${failure.message}`;
}

export function trashedStatus(summary: TrashSummary): string {
  const count = summary.moved.length;
  const moved = `Moved ${count} ${files(count)} to the Trash`;
  return summary.failed.length === 0 ? moved : `${moved}, ${summary.failed.length} failed`;
}

// Whether `openDir` is one of `dirs`, or under one of them when the command
// recursed into subfolders.
export function opensTarget(openDir: string, dirs: string[], recursive: boolean): boolean {
  return dirs.some((dir) => {
    const found = relation(openDir, dir);
    return found === "same" || (recursive && found === "under");
  });
}

export type TrashTarget = { dirs: string[]; recursive: boolean };

// The flow of one trashing, from the menu item through the dialog to the
// run's end. A second start while one is under way does nothing. The dialog
// is open while previewed and while running; a running move cannot be
// canceled.
export class TrashFlow {
  phase: "idle" | "previewing" | "previewed" | "running" = "idle";
  target: TrashTarget | null = null;

  get isOpen(): boolean {
    return this.phase === "previewed" || this.phase === "running";
  }

  get busy(): boolean {
    return this.phase !== "idle";
  }

  start(target: TrashTarget): boolean {
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

  // The targets to run on, or null when there is no preview to run.
  run(): TrashTarget | null {
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
