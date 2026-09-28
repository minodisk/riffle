import { relation } from "./tree.js";

// `File > Move Rejected in This Folder to Trash…` and the folder tree's
// `Move Rejected to Trash…` items: what the status line says once the command
// comes back, and whether the open folder was among the folders it trashed in.

// The `Summary` that `trash_rejected` returns: the RAWs it moved, which the
// frontend prunes its state by, every file whose move to the Trash failed,
// and every file or folder `collect` never got to try because it could not
// be read (an unreadable folder, a sidecar that does not parse, an index
// query error).
export type TrashSummary = {
  moved: string[];
  failed: { path: string; message: string }[];
  unread: { path: string; message: string }[];
};

export function trashedStatus(summary: TrashSummary): string {
  const count = summary.moved.length;
  const moved = `Moved ${count} ${count === 1 ? "file" : "files"} to the Trash`;
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
