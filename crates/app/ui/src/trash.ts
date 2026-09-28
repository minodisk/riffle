// `File > Move Rejected in This Folder to Trash…`: what the status line says
// once the command comes back.

// The `Summary` that `trash_rejected` returns: the RAWs it moved, which the
// frontend prunes its state by, and every file or folder that failed.
export type TrashSummary = {
  moved: string[];
  failed: { path: string; message: string }[];
};

export function trashedStatus(summary: TrashSummary): string {
  const count = summary.moved.length;
  const moved = `Moved ${count} ${count === 1 ? "file" : "files"} to the Trash`;
  return summary.failed.length === 0 ? moved : `${moved}, ${summary.failed.length} failed`;
}
