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

// The tier `photoTier` puts a frame in: a good photo, or a fair one just below.
export type PhotoTier = "good" | "fair";

// The state the mark's color is read from: the frame's `photoTier`,
// `candidate_only` for a focus candidate in neither tier, else the focus
// candidate state.
export type MarkState = PhotoTier | "candidate_only" | "not_candidate" | "unknown";

export interface FocusMark {
  x: number;
  y: number;
  rect: { x: number; y: number; width: number; height: number } | null;
  state: MarkState;
}

// The mark's color per state: bright green for a good photo, azure for a fair
// one, a dim green for a focus candidate in neither tier, orange when the eyes
// of the face nearest the AF point are not sharp, white when Riffle does not
// know.
export const FOCUS_MARK_COLORS = {
  good: "#3f3",
  fair: "#5af",
  candidate_only: "#8b8",
  not_candidate: "#f93",
  unknown: "#fff",
} as const satisfies Record<MarkState, string>;

// The good-photo cuts on the values pass 2 stores for the AF face, read from
// the re-dump of six ARW folders (8915 faced AF frames) on 2026-10-09 and tuned
// in Step 5 from the user's stars on a 60-frame sample
// (`docs/plans/20261008-burst-keep-score/provisional.md`).
// The in-focus probability of the AF eyes, at the re-dump's 79th percentile.
export const GOOD_EYE_FOCUS = 0.998;
// The EAR of the more closed eye, at the 54th percentile (open probability
// 0.991): wide open rather than not closed.
export const GOOD_EYE_EAR = 0.3;
// |yaw| in degrees, "both eyes visible": a turned face foreshortens the eye
// and inflates its EAR. 56% of the poses are within.
export const GOOD_MAX_YAW = 30;
// |pitch| in degrees: 96% of the poses are within.
export const GOOD_MAX_PITCH = 45;

// The fair-tier cuts, looser than the good ones on the eyes (the same
// `provisional.md`); with the Step 5 pose cut and exclusions the two tiers
// together mark 13.7% of the re-dump's faced AF frames (good 6.0%, fair 7.7%).
// At the re-dump's 67th percentile of `eye_focus`.
export const FAIR_EYE_FOCUS = 0.99;
// At the 35th percentile of the EAR (open probability 0.964, openness 41).
export const FAIR_EYE_EAR = 0.25;
// The same pose cut as the good tier.
export const FAIR_MAX_YAW = 30;
export const FAIR_MAX_PITCH = 45;

// Exclusions from both tiers, on how well the mesh sits on the face.
// The mesh's eyes farther than this from YuNet's eye landmarks, in face box
// sides, mean a mesh fitted off the face (an in-plane rotated face): the
// sample's well-fitted tier frames were all under 0.09, the misfit
// `_DSC2638` at 0.125; 7.6% of the frames the cuts alone tier exceed it.
export const MAX_EYE_OFFSET = 0.1;
// The face box or a mesh eye region closer to the preview's edge than this,
// in face box sides, means a face the frame cuts: YuNet's box ends at the
// edge of what is visible (`_DSC3345` at 0.008), so a small margin, not 0.
export const MIN_EDGE_GAP = 0.02;

// The EAR the eyes count as closed at and below, `EYES_CLOSED_EAR` in
// `crates/core/src/eyes.rs`.
export const EYES_CLOSED_EAR = 0.137;
// The EAR the openness reaches 100 at: the re-dump's 90th percentile (0.408)
// over the faced AF frames with an EAR, rounded.
export const EYES_WIDE_OPEN_EAR = 0.41;

// How open the eyes are, 0 at `EYES_CLOSED_EAR` and below to 100 at
// `EYES_WIDE_OPEN_EAR` and above, linear in the EAR between.
export function eyesOpenness(ear: number | null): number | null {
  if (ear === null) {
    return null;
  }
  const t = (ear - EYES_CLOSED_EAR) / (EYES_WIDE_OPEN_EAR - EYES_CLOSED_EAR);
  return Math.min(Math.max(t, 0), 1) * 100;
}

function clears(
  { eye_focus, eyes_ear, pose, eye_offset, edge_gap }: MarkFocus,
  minEyeFocus: number,
  minEar: number,
  maxYaw: number,
  maxPitch: number,
): boolean {
  return (
    eye_focus !== null &&
    eye_focus >= minEyeFocus &&
    eyes_ear !== null &&
    eyes_ear >= minEar &&
    pose !== null &&
    Math.abs(pose.yaw) <= maxYaw &&
    Math.abs(pose.pitch) <= maxPitch &&
    eye_offset !== null &&
    eye_offset <= MAX_EYE_OFFSET &&
    edge_gap !== null &&
    edge_gap >= MIN_EDGE_GAP
  );
}

// A focus candidate whose AF eyes are in focus, whose eyes are open and whose
// face is toward the camera, all at once: `good` when it clears the good
// cuts, else `fair` when it clears the fair ones. A mesh off the face or a
// face the frame's edge cuts is in neither tier, nor is any missing value (no
// face near the AF point, a face too small for the mesh).
export function photoTier(focus: MarkFocus | null | undefined): PhotoTier | null {
  if (focus === null || focus === undefined || focus.candidate !== "candidate") {
    return null;
  }
  if (clears(focus, GOOD_EYE_FOCUS, GOOD_EYE_EAR, GOOD_MAX_YAW, GOOD_MAX_PITCH)) {
    return "good";
  }
  if (clears(focus, FAIR_EYE_FOCUS, FAIR_EYE_EAR, FAIR_MAX_YAW, FAIR_MAX_PITCH)) {
    return "fair";
  }
  return null;
}

function markState(focus: MarkFocus): MarkState {
  if (focus.candidate !== "candidate") {
    return focus.candidate;
  }
  return photoTier(focus) ?? "candidate_only";
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
    focus.eye_offset = item.eye_offset;
    focus.edge_gap = item.edge_gap;
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
