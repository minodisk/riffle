// The face parts the `Eyes` judgment was taken on, as the focus mark overlay
// draws them. Free of DOM and Tauri so it is tested without mocks.

import type { Eyes } from "./eyes.js";

/*! MediaPipe Face Mesh `FACEMESH_CONTOURS`, `FACEMESH_NOSE` and
 * `FACEMESH_IRISES`, from mediapipe/python/solutions/face_mesh_connections.py
 * at commit 212f110c65818db7f2d08c42a9539ecb2b6c0e1d
 * (https://github.com/google-ai-edge/mediapipe), Apache License 2.0,
 * Copyright 2021 The MediaPipe Authors. Modification: each set's directed
 * pairs reduced to undirected edges, each as (lower, higher) index, sorted;
 * the contours (`FACEMESH_LIPS`, `FACEMESH_LEFT_EYE`,
 * `FACEMESH_LEFT_EYEBROW`, `FACEMESH_RIGHT_EYE`, `FACEMESH_RIGHT_EYEBROW`,
 * `FACEMESH_FACE_OVAL`, 124 edges) merged with the nose (25 edges) into one
 * table of 149 edges; the irises (`FACEMESH_LEFT_IRIS` +
 * `FACEMESH_RIGHT_IRIS`) kept as 8 edges. Full notice in
 * `crates/core/models/LICENSE-mediapipe`. */
// Two point indices per edge, all below 468.
export const FACE_OUTLINE_EDGES: readonly number[] = [
  0, 37, 0, 267, 1, 4, 1, 19, 2, 94, 2, 97, 2, 326, 4, 5, 4, 45, 4, 275, 5, 195, 6, 168, 6, 197, 7,
  33, 7, 163, 10, 109, 10, 338, 13, 82, 13, 312, 14, 87, 14, 317, 17, 84, 17, 314, 19, 94, 21, 54,
  21, 162, 33, 246, 37, 39, 39, 40, 40, 185, 45, 220, 46, 53, 48, 64, 48, 115, 52, 53, 52, 65, 54,
  103, 55, 65, 58, 132, 58, 172, 61, 146, 61, 185, 63, 70, 63, 105, 64, 98, 66, 105, 66, 107, 67,
  103, 67, 109, 78, 95, 78, 191, 80, 81, 80, 191, 81, 82, 84, 181, 87, 178, 88, 95, 88, 178, 91,
  146, 91, 181, 93, 132, 93, 234, 97, 98, 115, 220, 127, 162, 127, 234, 133, 155, 133, 173, 136,
  150, 136, 172, 144, 145, 144, 163, 145, 153, 148, 152, 148, 176, 149, 150, 149, 176, 152, 377,
  153, 154, 154, 155, 157, 158, 157, 173, 158, 159, 159, 160, 160, 161, 161, 246, 195, 197, 249,
  263, 249, 390, 251, 284, 251, 389, 263, 466, 267, 269, 269, 270, 270, 409, 275, 440, 276, 283,
  278, 294, 278, 344, 282, 283, 282, 295, 284, 332, 285, 295, 288, 361, 288, 397, 291, 375, 291,
  409, 293, 300, 293, 334, 294, 327, 296, 334, 296, 336, 297, 332, 297, 338, 308, 324, 308, 415,
  310, 311, 310, 415, 311, 312, 314, 405, 317, 402, 318, 324, 318, 402, 321, 375, 321, 405, 323,
  361, 323, 454, 326, 327, 344, 440, 356, 389, 356, 454, 362, 382, 362, 398, 365, 379, 365, 397,
  373, 374, 373, 390, 374, 380, 377, 400, 378, 379, 378, 400, 380, 381, 381, 382, 384, 385, 384,
  398, 385, 386, 386, 387, 387, 388, 388, 466,
];

// Two point indices per edge: the two rings around the iris centers.
export const FACE_IRIS_EDGES: readonly number[] = [
  469, 470, 469, 472, 470, 471, 471, 472, 474, 475, 474, 477, 475, 476, 476, 477,
];

// The iris centers, in no edge, so they are drawn as dots.
export const FACE_IRIS_CENTERS: readonly number[] = [468, 473];

// What to draw for a judgment: the parts outline always, the irises only for
// open eyes, since the model places iris points on a closed eye too.
export function meshEdges(state: Eyes["state"]): {
  edges: readonly number[];
  dots: readonly number[];
} {
  if (state === "closed") {
    return { edges: FACE_OUTLINE_EDGES, dots: [] };
  }
  return { edges: [...FACE_OUTLINE_EDGES, ...FACE_IRIS_EDGES], dots: FACE_IRIS_CENTERS };
}

// The points scaled onto the unrotated preview drawn `drawWidth` x
// `drawHeight` and centered on the origin, the same rule as `faceMarks`.
export function meshPoints(
  points: readonly (readonly [number, number])[],
  previewWidth: number,
  previewHeight: number,
  drawWidth: number,
  drawHeight: number,
): [number, number][] {
  const sx = drawWidth / previewWidth;
  const sy = drawHeight / previewHeight;
  return points.map(([x, y]) => [x * sx - drawWidth / 2, y * sy - drawHeight / 2]);
}
