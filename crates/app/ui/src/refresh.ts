// Whether a `scan-progress` tick should re-read the folder's rows. Only while
// the current file's row is missing, and then only when this tick committed
// it, or when the current file is not the one the last progress-triggered
// refresh was for (a file paged to after its row landed still gets it).
export function refreshOnProgress(
  current: string | undefined,
  hasRow: boolean,
  ready: readonly string[],
  lastRefreshedFor: string | null,
): boolean {
  if (current === undefined || hasRow) {
    return false;
  }
  return ready.includes(current) || current !== lastRefreshedFor;
}

// The rows `scan_folder`'s reconcile changed for one scan, kept for its
// `scan-done`.
export interface ScanStarted {
  scanId: number;
  changed: number;
}

// Whether `scan-done` should re-read the folder's rows. Skipped only when
// neither the reconcile nor the scan pass of the same scan wrote anything.
// Assumes a `scan-done` total of zero means the scan pass wrote no row, and
// `changed` of zero means the reconcile deleted no `files` row and wrote or
// cleared no `ratings` row. Both are needed because the open-time
// `refreshEntries` read is not ordered against `scan_folder`'s reconcile; if
// either ever writes rows without counting them, this skip would hide them.
// `changed` also counts 1 when `scan_folder` joined a previous scan of the
// same folder, since that scan's last in-flight batch can land after the
// open-time read and would otherwise go unseen here.
export function refreshOnScanDone(
  started: ScanStarted | null,
  scanId: number,
  total: number,
): boolean {
  return !(started?.scanId === scanId && started.changed === 0 && total === 0);
}

// The `scan-done` total of one scan, kept for its `faces-done`.
export interface ScanDone {
  scanId: number;
  total: number;
}

// Whether `faces-done` should re-read the folder's rows. Skipped only when
// neither pass of the same scan wrote anything. Assumes a `faces-done` total
// of zero means the faces pass made no `write_faces`; if it ever writes rows
// without counting them, this skip would hide them.
export function refreshOnFacesDone(
  scanDone: ScanDone | null,
  scanId: number,
  facesTotal: number,
): boolean {
  return !(scanDone?.scanId === scanId && scanDone.total === 0 && facesTotal === 0);
}

export interface RefreshTiming {
  rows: number;
  invoke: number;
  entries: number;
  bursts: number;
  exif: number;
  meta: number;
  draw: number;
  sharpness: number;
  applyBursts: number;
  candidates: number;
  refilter: number;
  setFiles: boolean;
  total: number;
}

// One `Riffle.log` line per `refreshEntries` run, times in milliseconds.
export function refreshTimingLine(t: RefreshTiming): string {
  const ms = (value: number): string => `${value.toFixed(1)}ms`;
  return [
    "refresh entries:",
    `rows=${t.rows}`,
    `invoke=${ms(t.invoke)}`,
    `entries=${ms(t.entries)}`,
    `bursts=${ms(t.bursts)}`,
    `exif=${ms(t.exif)}`,
    `meta=${ms(t.meta)}`,
    `draw=${ms(t.draw)}`,
    `sharpness=${ms(t.sharpness)}`,
    `apply_bursts=${ms(t.applyBursts)}`,
    `candidates=${ms(t.candidates)}`,
    `refilter=${ms(t.refilter)}`,
    `set_files=${t.setFiles}`,
    `total=${ms(t.total)}`,
  ].join(" ");
}
