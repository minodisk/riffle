import { describe, expect, test } from "vitest";
import {
  EYES_CLOSED_EAR,
  EYES_WIDE_OPEN_EAR,
  FOCUS_MARK_COLORS,
  GOOD_EYE_EAR,
  GOOD_EYE_FOCUS,
  GOOD_MAX_PITCH,
  GOOD_MAX_YAW,
  MAX_EYE_OFFSET,
  MIN_EDGE_GAP,
  type MarkFocus,
  applyFaceReady,
  applySharpnessReady,
  eyesOpenness,
  faceMarks,
  focusMark,
  photoTier,
} from "./focus.js";

const noEyes = {
  eyes_ear: null,
  eyes: "unknown",
  eyes_closed: null,
  pose: null,
  eye_offset: null,
  edge_gap: null,
} as const;

const point: MarkFocus = {
  sensor_w: 7008,
  sensor_h: 4672,
  x: 1752,
  y: 1168,
  frame: null,
  manual_focus: false,
  candidate: "unknown",
  eye_focus: null,
  ...noEyes,
};

const good: MarkFocus = {
  ...point,
  candidate: "candidate",
  eye_focus: 1,
  eyes_ear: 0.35,
  eyes: "open",
  eyes_closed: 0.001,
  pose: { yaw: 10, pitch: -5, roll: 3 },
  eye_offset: 0.03,
  edge_gap: 2,
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
      state: "unknown",
    });
  });

  test("a frame is scaled onto the preview and centered on the point", () => {
    const mark = focusMark({ ...point, frame: { width: 876, height: 584 } }, 700, 468);
    expect(mark).toEqual({
      x: -175,
      y: -117,
      rect: { x: -218.75, y: -146.25, width: 87.5, height: 58.5 },
      state: "unknown",
    });
  });

  test.each(["not_candidate", "unknown"] as const)(
    "carries the %s focus candidate state",
    (state) => {
      expect(focusMark({ ...point, candidate: state }, 700, 468)?.state).toBe(state);
    },
  );

  test("a focus candidate carries its photoTier, else candidate_only", () => {
    expect(focusMark(good, 700, 468)?.state).toBe("good");
    expect(focusMark({ ...good, eyes_ear: GOOD_EYE_EAR }, 700, 468)?.state).toBe("good");
    expect(focusMark({ ...good, eyes_ear: 0.1 }, 700, 468)?.state).toBe("candidate_only");
    expect(focusMark({ ...point, candidate: "candidate" }, 700, 468)?.state).toBe("candidate_only");
  });

  test("a manual-focus or missing point stays null whatever the state", () => {
    expect(
      focusMark({ ...point, manual_focus: true, candidate: "candidate" }, 700, 468),
    ).toBeNull();
    expect(focusMark(null, 700, 468)).toBeNull();
  });
});

describe("FOCUS_MARK_COLORS", () => {
  test("bright green for good, dim green for a candidate only, orange, white", () => {
    expect(FOCUS_MARK_COLORS).toEqual({
      good: "#3f3",
      candidate_only: "#8b8",
      not_candidate: "#f93",
      unknown: "#fff",
    });
  });
});

describe("eyesOpenness", () => {
  test("is 0 at the closed EAR and 100 at the wide open one", () => {
    expect(eyesOpenness(EYES_CLOSED_EAR)).toBe(0);
    expect(eyesOpenness(EYES_WIDE_OPEN_EAR)).toBe(100);
  });

  test("clamps below the closed EAR and above the wide open one", () => {
    expect(eyesOpenness(0.05)).toBe(0);
    expect(eyesOpenness(0)).toBe(0);
    expect(eyesOpenness(0.6)).toBe(100);
  });

  test("is linear in the EAR between", () => {
    expect(eyesOpenness((EYES_CLOSED_EAR + EYES_WIDE_OPEN_EAR) / 2)).toBeCloseTo(50, 10);
    expect(eyesOpenness(0.25)).toBeCloseTo(41.39, 2);
  });

  test("is null without an EAR", () => {
    expect(eyesOpenness(null)).toBeNull();
  });
});

describe("photoTier: good", () => {
  const goodPhoto = (focus: MarkFocus | null | undefined) => photoTier(focus) === "good";

  test("passes a candidate whose eyes and pose clear every cut", () => {
    expect(goodPhoto(good)).toBe(true);
  });

  test("is false without a focus", () => {
    expect(goodPhoto(null)).toBe(false);
    expect(goodPhoto(undefined)).toBe(false);
  });

  test.each(["not_candidate", "unknown"] as const)(
    "is false for a %s frame even with good eyes and pose",
    (candidate) => {
      expect(goodPhoto({ ...good, candidate })).toBe(false);
    },
  );

  test("is false when any value is missing", () => {
    expect(goodPhoto({ ...good, eye_focus: null })).toBe(false);
    expect(goodPhoto({ ...good, eyes_ear: null })).toBe(false);
    expect(goodPhoto({ ...good, pose: null })).toBe(false);
    expect(goodPhoto({ ...good, eye_offset: null })).toBe(false);
    expect(goodPhoto({ ...good, edge_gap: null })).toBe(false);
  });

  test("the eye offset passes at and below its cut, not above", () => {
    expect(goodPhoto({ ...good, eye_offset: MAX_EYE_OFFSET })).toBe(true);
    expect(goodPhoto({ ...good, eye_offset: MAX_EYE_OFFSET - 0.001 })).toBe(true);
    expect(goodPhoto({ ...good, eye_offset: MAX_EYE_OFFSET + 0.001 })).toBe(false);
  });

  test("the edge gap passes at and above its cut, not below or outside", () => {
    expect(goodPhoto({ ...good, edge_gap: MIN_EDGE_GAP })).toBe(true);
    expect(goodPhoto({ ...good, edge_gap: MIN_EDGE_GAP + 0.001 })).toBe(true);
    expect(goodPhoto({ ...good, edge_gap: MIN_EDGE_GAP - 0.001 })).toBe(false);
    expect(goodPhoto({ ...good, edge_gap: -0.1 })).toBe(false);
  });

  test("the three frames the user named are not good", () => {
    // `_DSC2638`: the mesh off a rotated face.
    expect(
      photoTier({
        ...good,
        eye_focus: 0.999,
        eyes_ear: 0.3601,
        pose: { yaw: 3, pitch: -11.8, roll: 11.8 },
        eye_offset: 0.1247,
        edge_gap: 0.2241,
      }),
    ).toBeNull();
    // `_DSC2827`: turned, the EAR inflated.
    expect(
      photoTier({
        ...good,
        eye_focus: 0.9984,
        eyes_ear: 0.3032,
        pose: { yaw: -48.6, pitch: 5.2, roll: 9.4 },
        eye_offset: 0.0866,
        edge_gap: 1.6918,
      }),
    ).toBeNull();
    // `_DSC3345`: cut by the left edge, its mesh off too.
    expect(
      photoTier({
        ...good,
        eye_focus: 0.9984,
        eyes_ear: 0.314,
        pose: { yaw: -15.3, pitch: 2.7, roll: -3.6 },
        eye_offset: 0.3521,
        edge_gap: 0.0076,
      }),
    ).toBeNull();
  });

  test("the eye_focus cut passes at and above, not below", () => {
    expect(goodPhoto({ ...good, eye_focus: GOOD_EYE_FOCUS })).toBe(true);
    expect(goodPhoto({ ...good, eye_focus: GOOD_EYE_FOCUS + 0.0005 })).toBe(true);
    expect(goodPhoto({ ...good, eye_focus: GOOD_EYE_FOCUS - 0.0005 })).toBe(false);
  });

  test("the EAR cut passes at and above, not below", () => {
    expect(goodPhoto({ ...good, eyes_ear: GOOD_EYE_EAR })).toBe(true);
    expect(goodPhoto({ ...good, eyes_ear: GOOD_EYE_EAR + 0.001 })).toBe(true);
    expect(goodPhoto({ ...good, eyes_ear: GOOD_EYE_EAR - 0.001 })).toBe(false);
  });

  test.each([1, -1])("the yaw cut passes at and within, not beyond, sign %d", (sign) => {
    const at = (yaw: number) =>
      goodPhoto({ ...good, pose: { yaw: sign * yaw, pitch: 0, roll: 0 } });
    expect(at(GOOD_MAX_YAW)).toBe(true);
    expect(at(GOOD_MAX_YAW - 0.1)).toBe(true);
    expect(at(GOOD_MAX_YAW + 0.1)).toBe(false);
  });

  test.each([1, -1])("the pitch cut passes at and within, not beyond, sign %d", (sign) => {
    const at = (pitch: number) =>
      goodPhoto({ ...good, pose: { yaw: 0, pitch: sign * pitch, roll: 0 } });
    expect(at(GOOD_MAX_PITCH)).toBe(true);
    expect(at(GOOD_MAX_PITCH - 0.1)).toBe(true);
    expect(at(GOOD_MAX_PITCH + 0.1)).toBe(false);
  });

  test("the roll is not a cut", () => {
    expect(goodPhoto({ ...good, pose: { yaw: 0, pitch: 0, roll: 90 } })).toBe(true);
  });
});

describe("photoTier: the folded tier", () => {
  test("is good or null, never another tier", () => {
    expect(photoTier(good)).toBe("good");
    expect(photoTier({ ...good, eyes_ear: 0.1 })).toBeNull();
  });

  test("the cuts are the Step 5 fair cuts", () => {
    expect(GOOD_EYE_FOCUS).toBe(0.99);
    expect(GOOD_EYE_EAR).toBe(0.25);
    expect(GOOD_MAX_YAW).toBe(30);
    expect(GOOD_MAX_PITCH).toBe(45);
    expect(MAX_EYE_OFFSET).toBe(0.1);
    expect(MIN_EDGE_GAP).toBe(0.02);
  });

  test("a frame below the Step 5 good cuts that cleared the fair ones is good", () => {
    expect(photoTier({ ...good, eye_focus: 0.995 })).toBe("good");
    expect(photoTier({ ...good, eyes_ear: 0.27 })).toBe("good");
    expect(photoTier({ ...good, eye_focus: 0.99, eyes_ear: 0.25 })).toBe("good");
  });

  test("the exclusions still apply at the folded cuts", () => {
    const edge = { ...good, eye_focus: GOOD_EYE_FOCUS, eyes_ear: GOOD_EYE_EAR };
    expect(photoTier(edge)).toBe("good");
    expect(photoTier({ ...edge, eye_offset: MAX_EYE_OFFSET + 0.001 })).toBeNull();
    expect(photoTier({ ...edge, edge_gap: MIN_EDGE_GAP - 0.001 })).toBeNull();
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
        { path: "/d/a.ARW", eye_focus: 0.9, candidate: "candidate", sharpness: null, ...noEyes },
        {
          path: "/d/b.ARW",
          eye_focus: 0.3,
          candidate: "not_candidate",
          sharpness: null,
          ...noEyes,
        },
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

  test("patches the stored eye state and head pose", () => {
    const entries = rows();
    const pose = { yaw: 12, pitch: -5, roll: 3 };
    applyFaceReady(
      entries,
      [
        {
          path: "/d/a.ARW",
          eye_focus: 0.9,
          candidate: "candidate",
          sharpness: null,
          eyes_ear: 0.08,
          eyes: "closed",
          eyes_closed: 0.9,
          pose,
          eye_offset: 0.05,
          edge_gap: -0.01,
        },
      ],
      undefined,
    );
    expect(entries.get("/d/a.ARW")?.focus).toMatchObject({
      eyes_ear: 0.08,
      eyes: "closed",
      eyes_closed: 0.9,
      pose,
      eye_offset: 0.05,
      edge_gap: -0.01,
    });
  });

  test("a current file outside the batch is not touched", () => {
    const entries = rows();
    expect(
      applyFaceReady(
        entries,
        [{ path: "/d/a.ARW", eye_focus: 0.9, candidate: "candidate", sharpness: null, ...noEyes }],
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
          { path: "/d/c.ARW", eye_focus: null, candidate: "unknown", sharpness: null, ...noEyes },
          {
            path: "/d/missing.ARW",
            eye_focus: 0.85,
            candidate: "candidate",
            sharpness: null,
            ...noEyes,
          },
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
    ...noEyes,
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
