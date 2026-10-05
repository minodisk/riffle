// The width of the scan progress bar's indicator, as a CSS percentage.
export function progressWidth(done: number, total: number): string {
  if (total <= 0) {
    return "0%";
  }
  return `${Math.min(100, Math.round((done / total) * 100))}%`;
}
