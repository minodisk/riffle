import { describe, expect, test } from "vitest";
import { type MarkFocus, focusMark } from "./focus.js";

const point: MarkFocus = {
  sensor_w: 7008,
  sensor_h: 4672,
  x: 1752,
  y: 1168,
  frame: null,
  manual_focus: false,
  face_catch: "unknown",
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
      faceCatch: "unknown",
    });
  });

  test("a frame is scaled onto the preview and centered on the point", () => {
    const mark = focusMark({ ...point, frame: { width: 876, height: 584 } }, 700, 468);
    expect(mark).toEqual({
      x: -175,
      y: -117,
      rect: { x: -218.75, y: -146.25, width: 87.5, height: 58.5 },
      faceCatch: "unknown",
    });
  });

  test.each(["caught", "missed", "unknown"] as const)(
    "carries the %s face-catch state",
    (state) => {
      expect(focusMark({ ...point, face_catch: state }, 700, 468)?.faceCatch).toBe(state);
    },
  );

  test("a manual-focus or missing point stays null whatever the state", () => {
    expect(focusMark({ ...point, manual_focus: true, face_catch: "caught" }, 700, 468)).toBeNull();
    expect(focusMark(null, 700, 468)).toBeNull();
  });
});
