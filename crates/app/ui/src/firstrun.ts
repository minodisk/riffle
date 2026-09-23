// Folder opens wait on the first-launch sidecar format choice. The gate starts
// closed; `open` lets folders open from then on and runs the one action that
// was deferred while it was closed (the startup restore of the last folder).
export class FormatGate {
  private opened = false;
  private deferred: (() => void) | null = null;

  get isOpen(): boolean {
    return this.opened;
  }

  whenOpen(action: () => void): void {
    if (this.opened) {
      action();
      return;
    }
    this.deferred = action;
  }

  open(): void {
    if (this.opened) {
      return;
    }
    this.opened = true;
    const action = this.deferred;
    this.deferred = null;
    action?.();
  }
}
