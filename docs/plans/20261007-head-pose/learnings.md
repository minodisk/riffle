# Learnings: head-pose

## Step 1

- MediaPipe commit used: `212f110c65818db7f2d08c42a9539ecb2b6c0e1d` (the one
  `FACE_MESH_EDGES` was taken from). `master` was
  `739ee8cef99b891f0f985b1cab87b1ec31af3820` on 2026-10-07; the two data files
  (`geometry_pipeline_metadata_landmarks.pbtxt`, SHA-256 `55ca7c4c...359d`,
  and `canonical_face_model.obj`, `8bac8044...e618`) are byte-identical at
  both, `geometry_pipeline.cc` and `procrustes_solver.cc` differ (not
  inspected further; the algorithm below is the one at the pinned commit).
- Camera confirmed in
  `mediapipe/tasks/cc/vision/face_geometry/face_geometry_from_landmarks_graph.cc`
  (`ConfigureFaceGeometryEnvGeneratorCalculator`): `TOP_LEFT_CORNER`,
  vertical FOV 63 deg, near 1, far 10000. The same graph splits the first
  468 landmarks (no iris) before the pipeline.
- Plan correction: `depth_offset` is `screen_landmarks.row(2).mean()`, the
  **plain** mean z over all 468 points, not a weighted mean. `head_pose`
  follows the source (plan.md fixed). Only the 33 basis points enter the
  Procrustes solves (the rest have weight 0), but all 468 enter the offset.
- The basis coordinates were taken from the pbtxt's `canonical_mesh`
  (5 floats per vertex, x y z u v), which is what the pipeline reads; they
  match `canonical_face_model.obj` line for line for the 33 ids.
- MediaPipe's pipeline also drops faces whose normalized points lie within
  1e-3 of their mean (`IsScreenLandmarkListTooCompact`); ported as
  `too_compact`, a `None`.
- No linear algebra dependency: the 3x3 rotation is the polar factor from a
  cyclic Jacobi eigen-solve of `A^T A` (proper V, `u3 = u1 x u2`), which is
  exactly MediaPipe's reflection flip. About 90 lines with the helpers.
- Synthetic check: the canonical basis rendered through the default
  frustum at a known pose, centered and off-center, comes back within
  0.05 deg on every axis (tests allow 0.5 / 1 deg).
- Sign convention checked on real crops: `L1005199.DNG` (face turned to
  the image right) yaw +41.5, `L1005194.DNG` (judged face turned to the
  left) yaw -34.0, `_DSC1884.ARW` (child looking up) pitch +48.2, and the
  ARW closed-eyes faces that are children looking down read pitch -12 to
  -33. `riffle-cli eyes` on 20 labeled ARWs and 20 DNGs: the EAR /
  probability / state columns are byte-identical before and after (model
  time stripped).
- `riffle-cli eyes` prints the pose below `EYES_MIN_FACE` too, like the EAR,
  while `Judged.pose` (the app) has none there. Step 2 can bucket by face
  side with that.
- Writing files with Python's text mode on Windows turned LF into CRLF;
  normalized back with `sed -i 's/\r$//'` before committing.
- CI attempt 1 failed on clippy `needless_range_loop` (two `for k in 0..3`
  loops indexing arrays in `pose.rs`); rewritten with `map` / `zip`.
  Run `cargo clippy --all-targets -- -D warnings` before `mise run ci`.
