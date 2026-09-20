// `File > Move Rejected to Trash…`: which files the command is given, and
// what the status line says once it comes back.

// The `Summary` that `trash_rejected` returns.
export type TrashSummary = {
  trashed: number;
  failed: { path: string; message: string }[];
};

// The rejects of the folder, in `files` order; `pick` is a separate flag, so
// only the rating decides.
export function rejectedPaths(files: string[], ratings: Map<string, number>): string[] {
  return files.filter((path) => ratings.get(path) === -1);
}

export function trashedStatus(summary: TrashSummary): string {
  const moved = `Moved ${summary.trashed} ${summary.trashed === 1 ? "file" : "files"} to the Trash`;
  return summary.failed.length === 0 ? moved : `${moved}, ${summary.failed.length} failed`;
}
