// The performance graph's data: the CPU samples (`perf-sample`) and the
// per-switch preview latencies of the last `WINDOW_MS`, all on the
// `performance.now()` clock. No DOM access, so the drawing takes `view()`'s
// output.

export const WINDOW_MS = 60_000;
// The latency axis never shrinks below this, so a quiet graph does not
// stretch 20 ms bars to full height.
export const LATENCY_FLOOR_MS = 500;

export interface CpuSample {
  at: number;
  process: number;
  system: number;
}

export interface LatencyPoint {
  at: number;
  ms: number;
}

export interface PerfView {
  cpu: readonly CpuSample[];
  latency: readonly LatencyPoint[];
  latencyMax: number;
}

// A switch's latency runs from the key that turned the page, else from the
// selection change, so a click or a request queued behind an in-flight one
// counts its wait too.
export function latencyStart(keypressAt: number | null, switchAt: number): number {
  return keypressAt ?? switchAt;
}

function prune<T extends { at: number }>(points: T[], now: number): void {
  const from = now - WINDOW_MS;
  let drop = 0;
  while (drop < points.length && points[drop].at < from) {
    drop += 1;
  }
  points.splice(0, drop);
}

// Records only between `start()` and `stop()`, so nothing accumulates while
// the overlay is off; `stop()` drops what was recorded.
export class PerfModel {
  private recording = false;
  private readonly cpu: CpuSample[] = [];
  private readonly latency: LatencyPoint[] = [];

  get on(): boolean {
    return this.recording;
  }

  start(): void {
    this.recording = true;
  }

  stop(): void {
    this.recording = false;
    this.cpu.length = 0;
    this.latency.length = 0;
  }

  addCpu(sample: CpuSample): void {
    if (!this.recording) return;
    this.cpu.push(sample);
    this.prune(sample.at);
  }

  addLatency(point: LatencyPoint): void {
    if (!this.recording) return;
    this.latency.push(point);
    this.prune(point.at);
  }

  view(now: number): PerfView {
    this.prune(now);
    const latencyMax = this.latency.reduce(
      (max, point) => Math.max(max, point.ms),
      LATENCY_FLOOR_MS,
    );
    return { cpu: this.cpu, latency: this.latency, latencyMax };
  }

  private prune(now: number): void {
    prune(this.cpu, now);
    prune(this.latency, now);
  }
}
