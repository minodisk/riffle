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

## Step 2

- Measured with a throwaway `riffle-cli posedump <out> <dir|file>...` (the
  judged face as `eyes` picks it, its 200x200 face crop, the pose at 63 deg,
  at the EXIF FOV and at 0.05 deg, and the solve time over 200 repetitions)
  and `riffle-cli sheet <out.png> <cols> <png>...` (tiles crops into a
  labeling sheet). Not merged; the patch and the Python aggregation are in
  `D:\Photos\tests\2026-10-07-head-pose\` (`posedump-cli.patch`,
  `load.py`, `sample.py`, `sample2.py`, `aggregate.py`). No numpy / PIL /
  ImageMagick on this machine, so the sheets are built by the CLI with the
  `image` crate the CLI already depends on.
- The weak-perspective variant needs no code: `head_pose_fov` at a 0.05 deg
  FOV makes the unprojection a uniform scale.
- `Shot` has no 35 mm-equivalent focal length tag; the EXIF FOV here used
  the focal length as is, which holds for the two full-frame bodies only.
  Moot since the default was kept.
- The first 150 faces (drawn by the Step 1 |yaw| buckets) had only 3
  visible rolls; a second blind draw of 40 enriched by |roll| was added so
  the roll sign has 18 labeled leans. Roll labels were read off the line
  through the eyes (image-right eye lower = clockwise = `right`), matching
  `the_signs_follow_the_image`.
- A first guard dropped |yaw| > 90 as well as |roll| > 90 (36 faces); a look
  at the 21 yaw-only ones showed readable far profiles with the right sign
  in 16 of 18, so only the roll bound was kept (`MAX_ROLL`, 15 faces, none
  of them labeled readable).
- `head_pose` takes 8.2 µs median single-thread (p95 18 µs) per face in a
  release build; under 24 rayon threads the same loop read 10-30 µs.
- No judged face on these folders was edge-clamped (all crops square), so
  the `kx != ky` caveat is untested.

## Step 3

- Built on the merged eyes-open-probability change: the Analysis row is now
  `Eyes open` (not `Eyes`), so `Head pose` follows that row and the docs
  sentence follows the `Eyes open` paragraph; `eyesOpenPercent` and its row
  are untouched.
- A yaw past 90 deg is shown as is, a signed integer (`yaw -110°`), as the
  Step 2 Decision keeps it; no clamp or wording. The usage docs say a yaw can
  read past 90 deg on far profiles.
- No `+ 0` trick is needed for negative zero: a template literal prints
  `Math.round(-0.4)` (`-0`) as `0`, which a test pins.
- No real face is in the `read_eyes` test fixtures (gradient JPEGs), so the
  field is covered by serializing an `EyesJudgment` with and without a pose.
- `eyes.test.ts`'s `Eyes` literals gained `pose: null` (forced by the type).
- `node` is not on the Git Bash PATH here; the frontend runs through `mise`.

## Deferred issues (todo candidates)

- **Pending manual check: the user's review of the head-pose labels.** What:
  the 190 labels in `docs/plans/20261007-head-pose/pose-truth.md` (yaw
  class and direction, pitch, roll, `x`) were made by the agent from the
  face crops. Where: Windows, the sheets
  `D:\Photos\tests\2026-10-07-head-pose\sheets\s00.png`-`s16.png` (4 tiles
  across, row by row in the `#` order of the table; crops in `crops\`).
  Steps: open each sheet, compare the tiles to the table rows, note any
  label to change (ask in Japanese). Expected: the labels stand or the
  corrections are listed, after which `aggregate.py` is re-run and
  `pose-results.md` / the plan's Decision updated if a number moves. The
  Step 2 checkbox was ticked on the automated criteria (the labels exist,
  the measurement and the Decision are written, `mise run ci` passes).
- Yaw past 90 deg is returned for far profiles (21 of 2024 judged faces on
  the sample folders; the sign right on 16 of 18 readable). Step 3 shows it
  as is (`yaw -110°`); whether the meta pane should clamp or word it
  (`profile`) is open. Basis: `pose-results.md` "Where it fails";
  files `crates/core/src/pose.rs`, `crates/app/ui/src/meta.ts`.
- `Shot` lacks the 35 mm-equivalent focal length (Exif `0xA405`), so an
  EXIF FOV for crop-sensor bodies cannot be computed; not needed now that
  63 deg is kept, but any later lens-aware feature would need it. Basis:
  Step 2 measurement; files `crates/core/src/arw.rs` (`Shot`),
  `crates/core/src/exif.rs`.
- **Pending manual check: the `Head pose` row in the app (Step 3).** What:
  the meta pane's Analysis group shows `Head pose` after `Eyes open`.
  Where: Windows, `mise run dev` (or the built app). Steps and expected:
  (1) a frontal ARW from `D:\photos\2026\2026-09-19` shows
  `Head pose  yaw N°, pitch N°, roll N°` with |yaw| small; (2) an ARW
  labeled oblique in `pose-truth.md` shows the yaw sign of its labeled
  direction (right positive); (3) a DNG from `D:\photos\2026\2026-02-01`
  shows the row on the no-AF path; (4) a file with no judged face shows
  neither `Eyes open` nor `Head pose`; (5) the `eyes total=` line in
  `Riffle.log` is not noticeably longer than before (the solve is ~8 us).
  The Step 3 checkbox was ticked on the automated criteria (the field,
  the row, the tests and `mise run ci`).
