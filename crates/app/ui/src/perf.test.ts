import { describe, expect, test } from "vitest";
import { LATENCY_FLOOR_MS, PerfModel, WINDOW_MS, latencyStart } from "./perf.js";

function recording(): PerfModel {
  const model = new PerfModel();
  model.start();
  return model;
}

describe("PerfModel", () => {
  test("records nothing until started", () => {
    const model = new PerfModel();
    model.addCpu({ at: 0, process: 10, system: 20 });
    model.addLatency({ at: 0, ms: 30 });
    expect(model.on).toBe(false);
    expect(model.view(0)).toEqual({ cpu: [], latency: [], latencyMax: LATENCY_FLOOR_MS });
  });

  test("stop drops what was recorded and stops recording", () => {
    const model = recording();
    model.addCpu({ at: 0, process: 10, system: 20 });
    model.addLatency({ at: 0, ms: 30 });
    model.stop();
    model.addLatency({ at: 1, ms: 40 });
    expect(model.on).toBe(false);
    expect(model.view(1)).toEqual({ cpu: [], latency: [], latencyMax: LATENCY_FLOOR_MS });
  });

  test("keeps points in insertion order", () => {
    const model = recording();
    model.addLatency({ at: 100, ms: 30 });
    model.addLatency({ at: 200, ms: 10 });
    model.addCpu({ at: 150, process: 1, system: 2 });
    model.addCpu({ at: 650, process: 3, system: 4 });
    const view = model.view(700);
    expect(view.latency.map((point) => point.at)).toEqual([100, 200]);
    expect(view.cpu.map((sample) => sample.at)).toEqual([150, 650]);
  });

  test("prunes points older than the window on insertion", () => {
    const model = recording();
    model.addCpu({ at: 0, process: 1, system: 1 });
    model.addLatency({ at: 10, ms: 900 });
    model.addCpu({ at: WINDOW_MS + 5, process: 2, system: 2 });
    const view = model.view(WINDOW_MS + 5);
    expect(view.cpu.map((sample) => sample.at)).toEqual([WINDOW_MS + 5]);
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
});

describe("latencyStart", () => {
  test("starts at the keypress when the switch came from a key", () => {
    expect(latencyStart(100, 105)).toBe(100);
  });

  test("starts at the selection change otherwise", () => {
    expect(latencyStart(null, 105)).toBe(105);
  });
});
