// Where the focus mark goes on the unrotated preview, drawn `drawWidth` x
// `drawHeight` and centered on the origin. Free of DOM and Tauri so it is
// tested without mocks.

import type { StoredEyes } from "./eyes.js";

export interface MarkFocus extends StoredEyes {
  sensor_w: number;
  sensor_h: number;
  x: number;
  y: number;
  frame: { width: number; height: number } | null;
  manual_focus: boolean;
  candidate: "candidate" | "not_candidate" | "unknown";
  eye_focus: number | null;
}

// The state the mark's color is read from: `good` for a frame `goodPhoto`
// passes, `candidate_only` for a focus candidate it does not pass, else the
// focus candidate state.
export type MarkState = "good" | "candidate_only" | "not_candidate" | "unknown";

export interface FocusMark {
  x: number;
  y: number;
  rect: { x: number; y: number; width: number; height: number } | null;
  state: MarkState;
}

// The mark's color per state: bright green for a good photo, a dim green for
// a focus candidate that is not one, orange when the eyes of the face nearest
// the AF point are not sharp, white when Riffle does not know.
export const FOCUS_MARK_COLORS = {
  good: "#3f3",
  candidate_only: "#8b8",
  not_candidate: "#f93",
  unknown: "#fff",
} as const satisfies Record<MarkState, string>;

// The good-photo cuts on the values pass 2 stores for the AF face. All four are
// provisional (2026-10-09, `docs/plans/20261008-burst-keep-score/provisional.md`),
// read from the re-dump of six ARW folders (8915 faced AF frames), and are to
// be tuned in that plan's Step 5.
// The in-focus probability of the AF eyes, at the re-dump's 79th percentile.
export const GOOD_EYE_FOCUS = 0.998;
// The EAR of the more closed eye, at the 54th percentile (open probability
// 0.991): wide open rather than not closed.
export const GOOD_EYE_EAR = 0.3;
// |yaw| in degrees, only extreme turns excluded: 86% of the poses are within.
export const GOOD_MAX_YAW = 60;
// |pitch| in degrees: 96% of the poses are within.
export const GOOD_MAX_PITCH = 45;

// A "good photo": a focus candidate whose AF eyes are in focus, whose eyes are
// open and whose face is toward the camera, all at once. Any missing value
// (no face near the AF point, a face too small for the mesh) is not good.
export function goodPhoto(focus: MarkFocus | null | undefined): boolean {
  if (focus === null || focus === undefined || focus.candidate !== "candidate") {
    return false;
  }
  const { eye_focus, eyes_ear, pose } = focus;
  return (
    eye_focus !== null &&
    eye_focus >= GOOD_EYE_FOCUS &&
    eyes_ear !== null &&
    eyes_ear >= GOOD_EYE_EAR &&
    pose !== null &&
    Math.abs(pose.yaw) <= GOOD_MAX_YAW &&
    Math.abs(pose.pitch) <= GOOD_MAX_PITCH
  );
}

function markState(focus: MarkFocus): MarkState {
  if (focus.candidate !== "candidate") {
    return focus.candidate;
  }
  return goodPhoto(focus) ? "good" : "candidate_only";
}

// `null` for a manual-focus shot, whose recorded point is not trusted. The
// mark state rides along so the mark's color is read from the mark.
export function focusMark(
  focus: MarkFocus | null | undefined,
  drawWidth: number,
  drawHeight: number,
): FocusMark | null {
  if (focus === null || focus === undefined || focus.manual_focus) {
    return null;
  }
  const x = -drawWidth / 2 + (focus.x * drawWidth) / focus.sensor_w;
  const y = -drawHeight / 2 + (focus.y * drawHeight) / focus.sensor_h;
  const state = markState(focus);
  if (focus.frame === null) {
    return { x, y, rect: null, state };
  }
  const width = (focus.frame.width * drawWidth) / focus.sensor_w;
  const height = (focus.frame.height * drawHeight) / focus.sensor_h;
  return {
    x,
    y,
    rect: { x: x - width / 2, y: y - height / 2, width, height },
    state,
  };
}

// One file the second scan pass has written, as `faces-progress` carries it.
export interface FaceReady extends StoredEyes {
  path: string;
  eye_focus: number | null;
  candidate: MarkFocus["candidate"];
  sharpness: number | null;
}

// Patch the focus of each ready file in `entries` in place, so the marks
// update without re-reading the whole folder. A file with no row or no focus
// point is left alone. True when `current` was among the patched files.
export function applyFaceReady<T extends { focus: MarkFocus | null }>(
  entries: Map<string, T>,
  ready: FaceReady[],
  current: string | undefined,
): boolean {
  let touched = false;
  for (const item of ready) {
    const focus = entries.get(item.path)?.focus;
    if (focus === null || focus === undefined) {
      continue;
    }
    focus.eye_focus = item.eye_focus;
    focus.candidate = item.candidate;
    focus.eyes_ear = item.eyes_ear;
    focus.eyes = item.eyes;
    focus.eyes_closed = item.eyes_closed;
    focus.pose = item.pose;
    if (item.path === current) {
      touched = true;
    }
  }
  return touched;
}

// Patch the sharpness score of each ready file into `scores` (set, or deleted
// when the pass found none) and into its row in `entries`, if it has one. True
// when any score changed, so the strip's bars are recomputed only then.
export function applySharpnessReady<T extends { sharpness: number | null }>(
  entries: Map<string, T>,
  scores: Map<string, number>,
  ready: FaceReady[],
): boolean {
  let changed = false;
  for (const item of ready) {
    if ((scores.get(item.path) ?? null) !== item.sharpness) {
      changed = true;
    }
    if (item.sharpness === null) {
      scores.delete(item.path);
    } else {
      scores.set(item.path, item.sharpness);
    }
    const entry = entries.get(item.path);
    if (entry !== undefined) {
      entry.sharpness = item.sharpness;
    }
  }
  return changed;
}

// What the `faces_of` command returns: the faces found on one file's preview,
// in the preview's stored (unrotated) pixel coordinates, `eye` being the
// midpoint between the eyes.
export interface Faces {
  width: number;
  height: number;
  faces: {
    x: number;
    y: number;
    width: number;
    height: number;
    eye: { x: number; y: number };
  }[];
}

export interface FaceMark {
  rect: { x: number; y: number; width: number; height: number };
  eye: { x: number; y: number };
}

// The faces scaled onto the unrotated preview drawn `drawWidth` x
// `drawHeight` and centered on the origin, like `focusMark`, so the rotation
// the image got carries them too. The drawn bitmap may be a downscaled copy of
// the preview, so the scale comes from the preview size, not the bitmap's.
export function faceMarks(
  faces: Faces["faces"],
  previewWidth: number,
  previewHeight: number,
  drawWidth: number,
  drawHeight: number,
): FaceMark[] {
  const sx = drawWidth / previewWidth;
  const sy = drawHeight / previewHeight;
  return faces.map((face) => ({
    rect: {
      x: face.x * sx - drawWidth / 2,
      y: face.y * sy - drawHeight / 2,
      width: face.width * sx,
      height: face.height * sy,
    },
    eye: { x: face.eye.x * sx - drawWidth / 2, y: face.eye.y * sy - drawHeight / 2 },
  }));
}
