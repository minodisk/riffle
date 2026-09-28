// Holds an operation pressed while `busy` (a scan runs) until `drain` is
// called, instead of refusing it. One slot: a later press replaces the held
// one. `enterInFlight`/`leaveInFlight` are called by the caller from
// dispatch until the operation settles, so a rescan can queue behind it. A
// count rather than a flag, since two deferred operations can overlap (for
// example a strip rename confirmed while a folder rename's reopen chain is
// still pending): the first to settle must not clear the flag out from
// under the other.
export class IdleGate {
  private inFlightCount = 0;
  private readonly busy: () => boolean;
  private held: { label: string; run: () => void } | null = null;

  constructor(busy: () => boolean) {
    this.busy = busy;
  }

  get inFlight(): boolean {
    return this.inFlightCount > 0;
  }

  enterInFlight(): void {
    this.inFlightCount += 1;
  }

  leaveInFlight(): void {
    this.inFlightCount -= 1;
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
