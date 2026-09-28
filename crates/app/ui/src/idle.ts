// Holds an operation pressed while `busy` (a scan runs) until `drain` is
// called, instead of refusing it. One slot: a later press replaces the held
// one. `inFlight` is set by the caller from dispatch until the operation
// settles, so a rescan can queue behind it.
export class IdleGate {
  inFlight = false;
  private readonly busy: () => boolean;
  private held: { label: string; run: () => void } | null = null;

  constructor(busy: () => boolean) {
    this.busy = busy;
  }

  request(label: string, run: () => void): void {
    if (this.busy()) {
      this.held = { label, run };
      return;
    }
    this.held = null;
    run();
  }

  drain(): void {
    const held = this.held;
    this.held = null;
    held?.run();
  }

  discard(): void {
    this.held = null;
  }

  get waiting(): string | null {
    return this.held?.label ?? null;
  }
}
