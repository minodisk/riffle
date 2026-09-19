// Sticky errors keyed by what they are about: a later error for the same key
// replaces the earlier one in place, keeping its original position.
export class ErrorList {
  private entries = new Map<string, string>();

  add(key: string, message: string): void {
    this.entries.set(key, message);
  }

  dismiss(key: string): void {
    this.entries.delete(key);
  }

  clear(): void {
    this.entries.clear();
  }

  list(): { key: string; message: string }[] {
    return [...this.entries].map(([key, message]) => ({ key, message }));
  }
}
