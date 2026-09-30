import { describe, expect, test } from "vitest";
import {
  FOCUS_MARK_COLORS,
  type MarkFocus,
  applyFaceReady,
  applySharpnessReady,
  faceMarks,
  focusMark,
} from "./focus.js";

const point: MarkFocus = {
  sensor_w: 7008,
  sensor_h: 4672,
  x: 1752,
  y: 1168,
  frame: null,
  manual_focus: false,
  candidate: "unknown",
  eye_focus: null,
};

describe("focusMark", () => {
  test("draws nothing without a focus point", () => {
    expect(focusMark(null, 700, 467)).toBeNull();
    expect(focusMark(undefined, 700, 467)).toBeNull();
  });

  test("draws nothing for a manual-focus shot", () => {
    const frame = { width: 876, height: 584 };
    expect(focusMark({ ...point, frame, manual_focus: true }, 700, 467)).toBeNull();
  });

  test("a point without a frame is a crosshair only", () => {
    expect(focusMark(point, 700, 468)).toEqual({
      x: -175,
      y: -117,
      rect: null,
      candidate: "unknown",
    });
  });

  test("a frame is scaled onto the preview and centered on the point", () => {
    const mark = focusMark({ ...point, frame: { width: 876, height: 584 } }, 700, 468);
    expect(mark).toEqual({
      x: -175,
      y: -117,
      rect: { x: -218.75, y: -146.25, width: 87.5, height: 58.5 },
      candidate: "unknown",
    });
  });

  test.each(["candidate", "not_candidate", "unknown"] as const)(
    "carries the %s focus candidate state",
    (state) => {
      expect(focusMark({ ...point, candidate: state }, 700, 468)?.candidate).toBe(state);
    },
  );

  test("a manual-focus or missing point stays null whatever the state", () => {
    expect(
      focusMark({ ...point, manual_focus: true, candidate: "candidate" }, 700, 468),
    ).toBeNull();
    expect(focusMark(null, 700, 468)).toBeNull();
  });
});

describe("FOCUS_MARK_COLORS", () => {
  test("green for a candidate, orange for not a candidate, white for unknown", () => {
    expect(FOCUS_MARK_COLORS).toEqual({
      candidate: "#3f3",
      not_candidate: "#f93",
      unknown: "#fff",
    });
  });
});

describe("faceMarks", () => {
  const face = (x: number, y: number, eyeX: number, eyeY: number) => ({
    x,
    y,
    width: 100,
    height: 120,
    eye: { x: eyeX, y: eyeY },
  });

  test("a face at the preview's origin sits at the drawn image's corner", () => {
    expect(faceMarks([face(0, 0, 50, 40)], 1600, 1080, 1600, 1080)).toEqual([
      { rect: { x: -800, y: -540, width: 100, height: 120 }, eye: { x: -750, y: -500 } },
    ]);
  });

  test("a face at the far corner ends at the drawn image's far corner", () => {
    expect(faceMarks([face(1500, 960, 1550, 1000)], 1600, 1080, 1600, 1080)).toEqual([
      { rect: { x: 700, y: 420, width: 100, height: 120 }, eye: { x: 750, y: 460 } },
    ]);
  });

  test("a preview drawn smaller scales the faces by the preview size", () => {
    expect(faceMarks([face(400, 200, 450, 240)], 1600, 1080, 800, 540)).toEqual([
      { rect: { x: -200, y: -170, width: 50, height: 60 }, eye: { x: -175, y: -150 } },
    ]);
  });
});

describe("applyFaceReady", () => {
  const rows = () =>
    new Map<string, { focus: MarkFocus | null }>([
      ["/d/a.ARW", { focus: { ...point } }],
      ["/d/b.ARW", { focus: { ...point } }],
      ["/d/c.ARW", { focus: null }],
    ]);

  test("patches the ready files and reports whether the current one was among them", () => {
    const entries = rows();
    const touched = applyFaceReady(
      entries,
      [
        { path: "/d/a.ARW", eye_focus: 0.9, candidate: "candidate", sharpness: null },
        { path: "/d/b.ARW", eye_focus: 0.3, candidate: "not_candidate", sharpness: null },
      ],
      "/d/b.ARW",
    );
    expect(touched).toBe(true);
    expect(entries.get("/d/a.ARW")?.focus).toMatchObject({
      eye_focus: 0.9,
      candidate: "candidate",
    });
    expect(entries.get("/d/b.ARW")?.focus).toMatchObject({
      eye_focus: 0.3,
      candidate: "not_candidate",
    });
  });

  test("a current file outside the batch is not touched", () => {
    const entries = rows();
    expect(
      applyFaceReady(
        entries,
        [{ path: "/d/a.ARW", eye_focus: 0.9, candidate: "candidate", sharpness: null }],
        "/d/b.ARW",
      ),
    ).toBe(false);
    expect(entries.get("/d/b.ARW")?.focus?.candidate).toBe("unknown");
  });

  test("a file with no row or no focus point is skipped", () => {
    const entries = rows();
    expect(
      applyFaceReady(
        entries,
        [
          { path: "/d/c.ARW", eye_focus: null, candidate: "unknown", sharpness: null },
          { path: "/d/missing.ARW", eye_focus: 0.85, candidate: "candidate", sharpness: null },
        ],
        "/d/c.ARW",
      ),
    ).toBe(false);
    expect(entries.get("/d/c.ARW")?.focus).toBeNull();
    expect(entries.has("/d/missing.ARW")).toBe(false);
  });
});

describe("applySharpnessReady", () => {
  const ready = (path: string, sharpness: number | null) => ({
    path,
    eye_focus: null,
    candidate: "unknown" as const,
    sharpness,
  });

  test("sets a new score in the map and the row, and reports the change", () => {
    const entries = new Map([["/d/a.ARW", { sharpness: null as number | null }]]);
    const scores = new Map<string, number>();
    expect(applySharpnessReady(entries, scores, [ready("/d/a.ARW", 12.5)])).toBe(true);
    expect(scores.get("/d/a.ARW")).toBe(12.5);
    expect(entries.get("/d/a.ARW")?.sharpness).toBe(12.5);
  });

  test("a file the pass found no score for is deleted from the map and nulled in the row", () => {
    const entries = new Map([["/d/a.ARW", { sharpness: 3 as number | null }]]);
    const scores = new Map([["/d/a.ARW", 3]]);
    expect(applySharpnessReady(entries, scores, [ready("/d/a.ARW", null)])).toBe(true);
    expect(scores.has("/d/a.ARW")).toBe(false);
    expect(entries.get("/d/a.ARW")?.sharpness).toBeNull();
  });

  test("unchanged scores report no change", () => {
    const entries = new Map([
      ["/d/a.ARW", { sharpness: 3 as number | null }],
      ["/d/b.ARW", { sharpness: null as number | null }],
    ]);
    const scores = new Map([["/d/a.ARW", 3]]);
    expect(
      applySharpnessReady(entries, scores, [ready("/d/a.ARW", 3), ready("/d/b.ARW", null)]),
    ).toBe(false);
    expect(scores.get("/d/a.ARW")).toBe(3);
    expect(scores.has("/d/b.ARW")).toBe(false);
  });

  test("a file with no row still gets its score in the map", () => {
    const entries = new Map<string, { sharpness: number | null }>();
    const scores = new Map<string, number>();
    expect(applySharpnessReady(entries, scores, [ready("/d/missing.ARW", 7)])).toBe(true);
    expect(scores.get("/d/missing.ARW")).toBe(7);
    expect(entries.has("/d/missing.ARW")).toBe(false);
  });
});
