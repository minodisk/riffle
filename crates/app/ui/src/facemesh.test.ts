import { describe, expect, test } from "vitest";
import { FACE_MESH_EDGES, meshPoints } from "./facemesh.js";

describe("FACE_MESH_EDGES", () => {
  test("holds the 1322 tessellation edges over the first 468 points", () => {
    expect(FACE_MESH_EDGES).toHaveLength(2 * 1322);
    for (const index of FACE_MESH_EDGES) {
      expect(Number.isInteger(index)).toBe(true);
      expect(index).toBeGreaterThanOrEqual(0);
      expect(index).toBeLessThan(468);
    }
  });

  test("has no self-loop and no edge twice in either direction", () => {
    const seen = new Set<string>();
    for (let i = 0; i < FACE_MESH_EDGES.length; i += 2) {
      const a = FACE_MESH_EDGES[i];
      const b = FACE_MESH_EDGES[i + 1];
      expect(a).not.toBe(b);
      const key = `${Math.min(a, b)},${Math.max(a, b)}`;
      expect(seen.has(key)).toBe(false);
      seen.add(key);
    }
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
