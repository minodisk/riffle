import { describe, expect, test } from "vitest";
import {
  FACE_IRIS_CENTERS,
  FACE_IRIS_EDGES,
  FACE_OUTLINE_EDGES,
  meshEdges,
  meshPoints,
} from "./facemesh.js";

function expectWellFormed(edges: readonly number[], count: number, low: number, high: number) {
  expect(edges).toHaveLength(2 * count);
  const seen = new Set<string>();
  for (let i = 0; i < edges.length; i += 2) {
    const a = edges[i];
    const b = edges[i + 1];
    for (const index of [a, b]) {
      expect(Number.isInteger(index)).toBe(true);
      expect(index).toBeGreaterThanOrEqual(low);
      expect(index).toBeLessThan(high);
    }
    expect(a).not.toBe(b);
    const key = `${Math.min(a, b)},${Math.max(a, b)}`;
    expect(seen.has(key)).toBe(false);
    seen.add(key);
  }
}

describe("FACE_OUTLINE_EDGES", () => {
  test("holds the 149 contour and nose edges over the first 468 points", () => {
    expectWellFormed(FACE_OUTLINE_EDGES, 149, 0, 468);
  });
});

describe("FACE_IRIS_EDGES", () => {
  test("holds the 8 iris ring edges over the iris points", () => {
    expectWellFormed(FACE_IRIS_EDGES, 8, 468, 478);
  });

  test("leaves the iris centers 468 and 473 out of every edge", () => {
    expect(FACE_IRIS_CENTERS).toEqual([468, 473]);
    for (const center of FACE_IRIS_CENTERS) {
      expect(FACE_IRIS_EDGES).not.toContain(center);
    }
  });
});

describe("meshEdges", () => {
  test("open eyes get the outline, the iris rings and the iris dots", () => {
    const { edges, dots } = meshEdges("open");
    expect(edges).toEqual([...FACE_OUTLINE_EDGES, ...FACE_IRIS_EDGES]);
    expect(dots).toEqual([468, 473]);
  });

  test("closed eyes get the outline only, no iris and no dot", () => {
    const { edges, dots } = meshEdges("closed");
    expect(edges).toEqual(FACE_OUTLINE_EDGES);
    expect(dots).toEqual([]);
  });
});

describe("meshPoints", () => {
  test("a point at the preview's origin sits at the drawn image's corner", () => {
    expect(meshPoints([[0, 0]], 1600, 1080, 1600, 1080)).toEqual([[-800, -540]]);
  });

  test("scales with the drawn size, not the bitmap's", () => {
    expect(
      meshPoints(
        [
          [800, 540],
          [400, 270],
        ],
        1600,
        1080,
        800,
        540,
      ),
    ).toEqual([
      [0, 0],
      [-200, -135],
    ]);
  });
});
