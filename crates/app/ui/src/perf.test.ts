import { describe, expect, test } from "vitest";
import { LATENCY_FLOOR_MS, PerfModel, READ_FLOOR_BPS, WINDOW_MS, latencyStart } from "./perf.js";

function recording(): PerfModel {
  const model = new PerfModel();
  model.start();
  return model;
}

const EMPTY = {
  samples: [],
  latency: [],
  latencyMax: LATENCY_FLOOR_MS,
  readMax: READ_FLOOR_BPS,
};

function sample(at: number, read: number, disk: number | null = null) {
  return { at, process: 0, system: 0, read, disk };
}

describe("PerfModel", () => {
  test("records nothing until started", () => {
    const model = new PerfModel();
    model.addSample({ at: 0, process: 10, system: 20, read: 0, disk: null });
    model.addLatency({ at: 0, ms: 30 });
    expect(model.on).toBe(false);
    expect(model.view(0)).toEqual(EMPTY);
  });

  test("stop drops what was recorded and stops recording", () => {
    const model = recording();
    model.addSample({ at: 0, process: 10, system: 20, read: 0, disk: null });
    model.addLatency({ at: 0, ms: 30 });
    model.stop();
    model.addLatency({ at: 1, ms: 40 });
    expect(model.on).toBe(false);
    expect(model.view(1)).toEqual(EMPTY);
  });

  test("keeps points in insertion order", () => {
    const model = recording();
    model.addLatency({ at: 100, ms: 30 });
    model.addLatency({ at: 200, ms: 10 });
    model.addSample({ at: 150, process: 1, system: 2, read: 0, disk: null });
    model.addSample({ at: 650, process: 3, system: 4, read: 0, disk: null });
    const view = model.view(700);
    expect(view.latency.map((point) => point.at)).toEqual([100, 200]);
    expect(view.samples.map((sample) => sample.at)).toEqual([150, 650]);
  });

  test("prunes points older than the window on insertion", () => {
    const model = recording();
    model.addSample({ at: 0, process: 1, system: 1, read: 0, disk: null });
    model.addLatency({ at: 10, ms: 900 });
    model.addSample({ at: WINDOW_MS + 5, process: 2, system: 2, read: 0, disk: null });
    const view = model.view(WINDOW_MS + 5);
    expect(view.samples.map((sample) => sample.at)).toEqual([WINDOW_MS + 5]);
    expect(view.latency.map((point) => point.at)).toEqual([10]);
  });

  test("prunes on a tick with no insertion and keeps the window's edge", () => {
    const model = recording();
    model.addLatency({ at: 1_000, ms: 30 });
    model.addLatency({ at: 2_000, ms: 40 });
    expect(model.view(WINDOW_MS + 1_000).latency.map((point) => point.at)).toEqual([1_000, 2_000]);
    expect(model.view(WINDOW_MS + 1_001).latency.map((point) => point.at)).toEqual([2_000]);
    expect(model.view(WINDOW_MS + 2_001).latency).toEqual([]);
  });

  test("the latency axis maximum has a floor", () => {
    const model = recording();
    model.addLatency({ at: 0, ms: 20 });
    expect(model.view(0).latencyMax).toBe(LATENCY_FLOOR_MS);
  });

  test("the latency axis maximum follows a point above the floor until it leaves the window", () => {
    const model = recording();
    model.addLatency({ at: 0, ms: 1_200 });
    model.addLatency({ at: 10, ms: 30 });
    expect(model.view(10).latencyMax).toBe(1_200);
    expect(model.view(WINDOW_MS + 1).latencyMax).toBe(LATENCY_FLOOR_MS);
  });

  test("the read axis maximum has a floor", () => {
    const model = recording();
    model.addSample(sample(0, 200_000));
    expect(model.view(0).readMax).toBe(READ_FLOOR_BPS);
  });

  test("the read axis maximum follows a sample above the floor until it leaves the window", () => {
    const model = recording();
    model.addSample(sample(0, 80_000_000));
    model.addSample(sample(10, 1_000));
    expect(model.view(10).readMax).toBe(80_000_000);
    expect(model.view(WINDOW_MS + 1).readMax).toBe(READ_FLOOR_BPS);
  });

  test("keeps a sample without a disk busy reading", () => {
    const model = recording();
    model.addSample(sample(0, 0, null));
    model.addSample(sample(500, 0, 42));
    expect(model.view(500).samples.map((point) => point.disk)).toEqual([null, 42]);
  });
});

describe("latencyStart", () => {
  test("starts at the keypress when the switch came from a key", () => {
    expect(latencyStart(100, 105)).toBe(100);
  });

  test("starts at the selection change otherwise", () => {
    expect(latencyStart(null, 105)).toBe(105);
  });
});
