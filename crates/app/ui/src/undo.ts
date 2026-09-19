// A bounded stack of undo entries: pushing past `limit` drops the oldest.
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

  // Drop `entry` wherever it sits; later entries may have been pushed since.
  remove(entry: T): void {
    const at = this.entries.lastIndexOf(entry);
    if (at !== -1) {
      this.entries.splice(at, 1);
    }
  }

  clear(): void {
    this.entries = [];
  }
}
