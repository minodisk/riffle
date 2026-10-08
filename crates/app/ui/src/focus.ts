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

export interface FocusMark {
  x: number;
  y: number;
  rect: { x: number; y: number; width: number; height: number } | null;
  candidate: MarkFocus["candidate"];
}

// The mark's color per focus candidate state: green when the eyes of the face
// nearest the AF point are sharp, orange when they are not, white when Riffle
// does not know.
export const FOCUS_MARK_COLORS = {
  candidate: "#3f3",
  not_candidate: "#f93",
  unknown: "#fff",
} as const satisfies Record<MarkFocus["candidate"], string>;

// `null` for a manual-focus shot, whose recorded point is not trusted. The
// focus candidate state rides along so the mark's color is read from the mark.
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
  if (focus.frame === null) {
    return { x, y, rect: null, candidate: focus.candidate };
  }
  const width = (focus.frame.width * drawWidth) / focus.sensor_w;
  const height = (focus.frame.height * drawHeight) / focus.sensor_h;
  return {
    x,
    y,
    rect: { x: x - width / 2, y: y - height / 2, width, height },
    candidate: focus.candidate,
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
