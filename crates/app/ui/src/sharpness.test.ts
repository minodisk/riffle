import { describe, expect, test } from "vitest";
import { type RelativeSharpness, type SharpnessFacts, relativeSharpness } from "./sharpness.js";

interface Shot {
  path: string;
  score: number | null;
  burst?: number;
}

// Shots listed in capture order, one second apart.
function facts(shots: Shot[]): Map<string, SharpnessFacts> {
  return new Map(
    shots.map(({ path, score, burst }, at) => [
      path,
      {
        captureTime: `2026:09:23 10:00:${String(at).padStart(2, "0")}`,
        score,
        burst: burst ?? null,
      },
    ]),
  );
}

function run(shots: Shot[], paths: string[] = shots.map(({ path }) => path), radius?: number) {
  const lookup = facts(shots);
  const result = relativeSharpness(paths, (path) => lookup.get(path)!, radius);
  return (path: string): RelativeSharpness | null | undefined => result.get(path);
}

describe("relativeSharpness", () => {
  test("a lone file is the best with ratio 1", () => {
    expect(run([{ path: "a", score: 42 }])("a")).toEqual({ ratio: 1, best: true });
  });

  test("nulls stay null and still take a slot", () => {
    const at = run(
      [
        { path: "a", score: 8 },
        { path: "b", score: null },
        { path: "c", score: null },
        { path: "d", score: 4 },
      ],
      undefined,
      2,
    );
    expect(at("b")).toBeNull();
    expect(at("c")).toBeNull();
    expect(at("d")).toEqual({ ratio: 1, best: true });
    expect(at("a")).toEqual({ ratio: 1, best: true });
  });

  test("the window is clamped at both ends", () => {
    const at = run(
      [4, 1, 1, 1, 1, 1, 8].map((score, index) => ({ path: `p${index}`, score })),
      undefined,
      2,
    );
    expect(at("p0")).toEqual({ ratio: 1, best: true });
    expect(at("p1")).toEqual({ ratio: 0.25, best: false });
    expect(at("p6")).toEqual({ ratio: 1, best: true });
    expect(at("p5")).toEqual({ ratio: 0.125, best: false });
  });

  test("equal scores tie", () => {
    const at = run([
      { path: "a", score: 3 },
      { path: "b", score: 3 },
    ]);
    expect(at("a")).toEqual({ ratio: 1, best: true });
    expect(at("b")).toEqual({ ratio: 1, best: true });
  });

  test("a burst member is compared with its whole burst, hidden members included", () => {
    const shots = [
      { path: "a", score: 20, burst: 1 },
      { path: "b", score: 80, burst: 1 },
      { path: "c", score: 40, burst: 1 },
      { path: "d", score: 100 },
    ];
    const at = run(shots);
    expect(at("a")).toEqual({ ratio: 0.25, best: false });
    expect(at("b")).toEqual({ ratio: 1, best: true });
    expect(at("c")).toEqual({ ratio: 0.5, best: false });
  });

  test("no displayed member is best when the burst's best frame is hidden", () => {
    const at = run([
      { path: "a", score: 20, burst: 1 },
      { path: "b", score: 80, burst: 1 },
      { path: "c", score: 40, burst: 1 },
    ]);
    const displayed = ["a", "c"].map(at);
    expect(displayed).toEqual([
      { ratio: 0.25, best: false },
      { ratio: 0.5, best: false },
    ]);
  });

  test("a single skips the bursts around it", () => {
    const at = run(
      [
        { path: "s1", score: 10 },
        { path: "b1", score: 100, burst: 1 },
        { path: "b2", score: 90, burst: 1 },
        { path: "s2", score: 5 },
        { path: "s3", score: 2 },
        { path: "s4", score: 1 },
      ],
      undefined,
      1,
    );
    expect(at("s1")).toEqual({ ratio: 1, best: true });
    expect(at("s2")).toEqual({ ratio: 0.5, best: false });
    expect(at("s4")).toEqual({ ratio: 0.5, best: false });
    expect(at("b2")).toEqual({ ratio: 0.9, best: false });
  });

  test("the result does not depend on the order of the input paths", () => {
    const shots = [
      { path: "s1", score: 3 },
      { path: "b1", score: 6, burst: 1 },
      { path: "b2", score: 2, burst: 1 },
      { path: "s2", score: 9 },
      { path: "s3", score: null },
      { path: "s4", score: 1 },
    ];
    const paths = shots.map(({ path }) => path);
    const forward = run(shots, paths, 1);
    const backward = run(shots, [...paths].reverse(), 1);
    for (const path of paths) {
      expect(backward(path)).toEqual(forward(path));
    }
    expect(forward("s1")).toEqual({ ratio: 1 / 3, best: false });
  });
});
