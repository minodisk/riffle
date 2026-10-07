<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Head pose of the judged face

## Purpose

The face mesh that judges closed eyes (`crates/core/src/eyes.rs`, MediaPipe
Face Landmarker v2) returns 478 (x, y, z) points, and Riffle drops z
(`to_full` keeps x and y only). With z, the head's orientation can be
estimated the way MediaPipe does it: its facial transformation matrix is not
a model output but C++ post-processing in `mediapipe/modules/face_geometry`
(a weighted orthogonal Procrustes fit of the estimated points onto a
canonical face model, after an unprojection through an assumed perspective
camera). Porting that gives yaw / pitch / roll in degrees for the face
`eyes_of` judges, computed only when the file is shown, nothing stored in the
index. The meta pane's Analysis group shows it; a culler can tell a frontal
frame from an oblique one in a burst without opening each.

### What is known about MediaPipe's pipeline

Verified 2026-10-07 against `google-ai-edge/mediapipe` `master`; record the
commit used in `learnings.md`.

- **z scale.** `LandmarkProjectionCalculator` maps the model's z the same way
  as x: `new_z = z * rect.width` (the ROI's normalized width), and
  `TensorsToLandmarks` divides the raw output by the 256 px input first. So
  in Riffle's terms z in full-image pixels is `p[2] * kx`, the same factor
  `to_full` applies to x. The geometry pipeline then takes normalized
  landmarks (x / W, y / H, z / W of the whole frame) and multiplies them
  back by the frustum's width, height and width.
- **Camera.** `PerspectiveCameraFrustum`: `height_at_near = 2 * near *
  tan(fov_v / 2)`, `width_at_near = frame_w * height_at_near / frame_h`,
  `left/right = -/+ width/2`, `bottom/top = -/+ height/2`. MediaPipe's
  tasks graph uses origin `TOP_LEFT_CORNER`, vertical FOV 63 deg, near 1,
  far 10000 (confirm in the graph's environment options and record).
- **ScreenToMetricSpaceConverter::Convert** (two-pass scale estimate):
  1. `ProjectXY`: `y = 1 - y` (top-left origin), scale x by
     `right - left`, y by `top - bottom`, z by `right - left`, translate
     x by `left`, y by `bottom`.
  2. `ChangeHandedness`: `z = -z`.
  3. `depth_offset` = plain mean z of the 468 mesh points (not weighted;
     corrected in Step 1 from the source); first
     `EstimateScale`: Procrustes canonical -> landmarks, scale = norm of
     column 0 of the transform.
  4. `MoveAndRescaleZ`: `z = (z - depth_offset + near) / first_scale`;
     `UnprojectXY`: `x *= z / near`, `y *= z / near`; `ChangeHandedness`;
     second `EstimateScale` on these.
  5. From the original projected landmarks again: `MoveAndRescaleZ` with
     `first_scale * second_scale`, `UnprojectXY`, `ChangeHandedness`, then
     the final Procrustes canonical -> metric landmarks gives
     `pose_transform_matrix` (4x4: 3x3 rotation * scale, translation).
- **Weighted orthogonal Procrustes** (`procrustes_solver.cc`): sqrt of
  weights; weighted centering (`sum(w * p) / sum(w)`); design matrix of
  the centered weighted targets times centered weighted sources
  transposed; 3x3 SVD `U S V^T`; `R = U V^T`, flipping the sign of U's
  last column when `det(U) * det(V) < 0`; scale = `sum(R·src .* tgt) /
  sum(src .* src)` on the centered weighted points (both must exceed an
  epsilon, else the solve fails); translation = weighted mean of
  `tgt - scale * R * src`.
- **Data.** `geometry_pipeline_metadata_landmarks.pbtxt` (Apache-2.0):
  33 `procrustes_landmark_basis` entries, ids
  4, 6, 10, 33, 54, 67, 117, 119, 121, 127, 129, 132, 133, 136, 143, 147,
  198, 205 and their mirrors 263, 284, 297, 346, 348, 350, 356, 358, 361,
  362, 365, 372, 376, 420, 425, with weights such as 0.0709 (4), 0.1206
  (129 / 358), 0.0669 (136 / 365), 0.0587 (33 / 263), 0.0533 (133 / 362).
  The canonical mesh (`canonical_face_model.obj`, 468 vertices in cm,
  vertex 0 = `0.000000 -3.406404 5.979507`) supplies the 33 points'
  coordinates; every other point has weight 0 and does not affect the
  pose, so only those 33 need embedding. Fetch both files at a pinned
  commit, record the URL, commit and SHA-256 in `LICENSE-mediapipe` like
  the `FACE_MESH_EDGES` provenance, and copy the numbers verbatim.
- **Caveats to handle.** (a) MediaPipe feeds a roll-corrected ROI (rotated
  so landmarks 33 and 263 are level); Riffle's crop has no roll
  correction and must stay so (the closed-eyes constants are frozen on
  it), so roll comes entirely from the landmarks and near-profile or
  strongly rolled faces may estimate worse: measure in Step 2. (b)
  `faces::crop_rgb` clamps the square at the image edge, so a crop can be
  non-square and `kx != ky`; MediaPipe letterboxes instead. Use `kx` for
  z (as MediaPipe does) and note faces at the frame edge as a risk. (c)
  The embedded preview is not a webcam: its real vertical FOV follows the
  lens, which `Shot` knows (focal length). The default 63 deg is what
  MediaPipe assumes; whether the EXIF FOV improves the angles is measured
  in Step 2.

### Concurrency with another session

`docs/plans/20261007-eyes-open-probability/` (branch
`feature/eyes-open-probability`) is changing the `Eyes` row to an open
probability: it will touch `eyesValue` and the `["Eyes", ...]` line of
`meta.ts`, the `probability` / `state` fields of `Eyes` in `eyes.ts` and
`EyesJudgment` in `commands.rs`, and the `Eyes` paragraph of `usage.md` /
`usage.ja.md`. This plan does not touch `eyesValue`, the `Eyes` row line,
the `state` / `probability` fields or the sentences about them; it only
**adds** a `pose` field next to them, a separate `poseValue` function and a
separate row, and a separate sentence in the docs. Whichever lands second
rebases; the overlap is additive lines only.

### Tools and samples

- `riffle-cli eyes <dir|file>...` prints one line per file (judged face
  side, EAR, probability, state, model ms); `riffle-cli eyecrops <dir|file>...
  <out-dir>` writes face and eye crops with a diffable index.
- Sample folders: `D:\photos\2026\2026-09-19` (2134 α7 V ARWs, AF point on
  every file) and `D:\photos\2026\2026-02-01` (146 M11-P DNGs, no AF
  point). Crops of the closed-eyes truth set are in
  `D:\Photos\tests\2026-10-07-closed-eyes\` (`arw\`, `dng\`, `tiles\`).
  Hand-check outputs go in `D:\Photos\tests\<date>-<topic>\`.
- Guides: [`../../agents/tract-onnx-inference.md`](../../agents/tract-onnx-inference.md),
  the closed-eyes plan
  [`../_archived/20261007-closed-eyes-detection/plan.md`](../_archived/20261007-closed-eyes-detection/plan.md)
  (how a measure was labeled, measured and then adopted).

## Steps

- [x] Step 1: Port MediaPipe's face geometry pipeline to `riffle-core` and expose the pose from the eyes judgment and the CLI
  - Done when:
    - `landmarks_of` / `to_full` keep z: the points become `[f32; 3]` in
      the full upright image's pixels with `z = p[2] * kx` (the existing
      `to_full` test extends to z; x and y are unchanged, so the face mesh
      overlay and the EAR are byte-identical: `riffle-cli eyes` on 20
      labeled ARWs and 20 DNGs prints the same EAR columns before / after).
      Callers (`ears`, `commands.rs` `read_eyes`, `crates/cli`) adapt
      without behavior change.
    - A new module `crates/core/src/pose.rs` (exported from `lib.rs`), with
      `eyes.rs` depending on it and not the other way round:
      - the 33 canonical points and weights as `const` tables with the
        provenance in a module comment and in
        `crates/core/models/LICENSE-mediapipe` (URL, commit, SHA-256 of
        the two source files, "copied verbatim, no change");
      - a weighted orthogonal Procrustes solver and the two-pass
        screen-to-metric conversion above, `pub fn head_pose(points:
        &[[f32; 3]], width: usize, height: usize) -> Option<Pose>` where
        `Pose { yaw: f64, pitch: f64, roll: f64 }` in degrees extracted
        from the final rotation (the Euler convention and the signs are
        documented on the struct: **yaw positive when the face turns toward
        the image's right, pitch positive when it tilts up, roll positive
        when the head tilts clockwise on screen**; derive the mapping from
        MediaPipe's camera space, x right, y up, z toward the camera, and
        pin it with tests); `None` when the solve is degenerate (epsilon
        checks, non-finite input);
      - the camera is the full upright preview with MediaPipe's defaults
        (vertical FOV 63 deg, near 1, far 10000) as named constants, the
        FOV passed in so Step 2 can try the EXIF one.
    - The 3x3 SVD is written in the module (a Jacobi eigen-solve of
      `A^T A` or a polar decomposition, ~80 lines, unit-tested against
      known matrices) unless the implementer finds a dependency already in
      the tree; adding `nalgebra` is a Trade-offs decision, not a default.
    - `eyes::Judged` gains `pose: Option<Pose>`, computed in `judge_mesh`
      from the same points after the EAR (so a face below `EYES_MIN_FACE`
      has no pose either, same floor, nothing else changed in the eyes
      logic; `EYES_CLOSED_EAR` and `EYES_LOGIT_SLOPE` untouched).
    - `riffle-cli eyes` lines gain `yaw`, `pitch`, `roll` columns (one
      decimal, `-` when none) so Step 2 can measure from the repository
      alone; `eyes_line` tests updated.
    - Tests: Procrustes recovers a known rotation / scale / translation of
      the canonical points (including the reflection case); the pipeline on
      the canonical points rendered through the same frustum at a known
      yaw / pitch / roll returns those angles within ~1 deg, and 0 for an
      unrotated face centered and also off-center in the frame (the
      perspective step is what makes the off-center case hold); the sign
      convention tests above; `None` on degenerate input; the
      `#[ignore]`d real-image test prints the pose.
    - `docs/agents/tract-onnx-inference.md` gains a short item on the z
      output of the face mesh (its scale and what it is used for), tagged
      Inferred / Measured as the guide does.
    - `mise run ci` passes.
  - Implementation approach:
    - Files: `crates/core/src/pose.rs` (new), `crates/core/src/eyes.rs`,
      `crates/core/src/lib.rs`, `crates/core/models/LICENSE-mediapipe`,
      `crates/cli/src/main.rs`, `crates/app/src/commands.rs` (type
      adaptation only), `docs/agents/tract-onnx-inference.md`.
    - Document the frozen constants like `candidate.rs` documents its
      `LOGIT_*`. Do not add a dependency for linear algebra without noting
      the binary and build cost in `learnings.md`.

- [ ] Step 2: Label the head pose of local faces and measure the angles against them
  - Done when:
    - `pose-truth.md` in this plan folder: at least ~120 faces sampled from
      the two folders (the judged face of each file, as `eyes-truth.md`
      picked it), stratified so oblique and profile-ish faces are not
      swamped by frontal ones (e.g. draw by the Step 1 |yaw| buckets and at
      random, shuffled so the source is hidden while labeling), each labeled
      from its `eyecrops` face crop with: yaw class `frontal` (|yaw| under
      ~15 deg), `oblique` (~15-50), `profile` (over ~50) and the turn
      direction `left` / `right` (image side the face turns toward), pitch
      class `up` / `level` / `down`, roll class `left` / `level` / `right`,
      or `x` when the face cannot be read. The agent labels; **the user's
      review of the labels is pending until they confirm it** (the
      conversation is in Japanese; ask them to review the sheet once the
      labels exist, the way the closed-eyes labels were handled). Record
      how YuNet's detection limits the profile class (it finds few true
      profiles) so the measured range is honest.
    - `pose-results.md` in this plan folder: with `riffle-cli eyes` on the
      labeled files, per axis the sign agreement with the labeled direction,
      the median and spread of the angle per class, a confusion matrix of
      the class the angle implies against the label at the thresholds above,
      and the per-face solve time (should be microseconds; the model run is
      already paid). Measured for the default 63 deg FOV **and** the EXIF
      FOV of the file (the vertical FOV from the 35 mm-equivalent focal
      length and the preview aspect), and, if cheap, a weak-perspective
      variant (Procrustes on the pixel points without the unprojection); the
      variant kept is decided here and stated in a "Decision" section of
      this plan.md, any constant change made in `pose.rs` in this PR with
      the test values updated.
    - Any face-size or confidence floor the pose needs beyond
      `EYES_MIN_FACE` is measured here (by face side bucket) and, if one is
      needed, added as a documented constant.
    - `mise run ci` (lint, lychee on the new files) passes.
  - Implementation approach:
    - Assumes Step 1 is merged. Scripts that aggregate the CLI output stay
      out of `main` (copy to `D:\Photos\tests\2026-10-07-head-pose\` and
      say so). The sheets / crops the labels were read from go there too.
    - Files: `docs/plans/20261007-head-pose/pose-truth.md`,
      `pose-results.md`, `plan.md` (Decision), possibly `crates/core/src/pose.rs`.

- [ ] Step 3: Return the pose from `eyes_of` and show it in the meta pane
  - Done when:
    - `EyesJudgment` in `crates/app/src/commands.rs` gains `pose: {yaw,
      pitch, roll} | null` (degrees, serialized as numbers, `null` when the
      solve failed); `read_eyes` fills it from `Judged.pose`. No new
      command, no new request, no index change; the superseding and the
      `eyes` timing line are untouched. The `read_eyes` tests cover the
      field.
    - `crates/app/ui/src/eyes.ts`: the `Eyes` interface gains `pose: Pose |
      null` with `Pose` exported; nothing else in the cache changes.
    - `crates/app/ui/src/meta.ts`: a new `poseValue(pose)` function and one
      Analysis row **`Head pose`** after the `Eyes` row, value `yaw 12°,
      pitch -5°, roll 3°` (integers, signed, the convention of Step 1), no
      row when `eyes` is null / undefined or `pose` is null. Do not edit
      `eyesValue` or the `["Eyes", ...]` line. `meta.test.ts` covers the
      row present, absent with no pose, absent with no eyes, and rounding
      of negative zero to `0°`.
    - `main.ts` needs no change beyond what the types force (the row is
      rendered from `metaGroups` with the cached `Eyes`).
    - `docs/humans/usage.md` **Meta pane** Analysis paragraph gains a
      sentence after the `Eyes` one: `Head pose` is the yaw, pitch and roll
      of that same face in degrees, the sign convention, that it comes from
      MediaPipe's face geometry fit on the same face mesh (Apache-2.0,
      provenance in `LICENSE-mediapipe`), that it is absent whenever `Eyes`
      is, and that near-profile faces are rough (Step 2's numbers in one
      clause); `usage.ja.md` in sync. The Features list bullet and
      `README.md` / `README.ja.md` mention it only if they list the `Eyes`
      row (check; `README.md` line ~126 does name it).
    - Manual checks (GUI; list in `learnings.md` as pending if not run):
      a frontal ARW from `2026-09-19` shows `Head pose` with |yaw| small; a
      labeled oblique one shows the sign of the labeled direction; a DNG
      from `2026-02-01` shows it on the no-AF path; a file with no judged
      face shows neither `Eyes` nor `Head pose`; the `eyes total=` line is
      not noticeably longer than before.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged. Rebase on
      `feature/eyes-open-probability` if it has landed; the only overlap is
      the adjacent lines named above.
    - Files: `crates/app/src/commands.rs`, `crates/app/ui/src/eyes.ts`,
      `crates/app/ui/src/meta.ts`, `crates/app/ui/src/meta.test.ts`,
      `docs/humans/usage.md`, `docs/humans/usage.ja.md`, `README.md` /
      `README.ja.md` (if applicable).

- [ ] Step 4: Record the cost and accuracy in the user docs and the todo
  - Done when:
    - `docs/humans/performance.md` "Closed-eyes judgment on demand (Windows
      11)" gains a paragraph on the pose: the per-face solve time, the
      `eyes_of` total before / after on the same files (the
      `times_eyes_of_on_real_files` test), binary size if it changed, and
      the Step 2 accuracy summary (sign agreement, class confusion);
      `performance.ja.md` in sync.
    - `todo.md`: the face/eye section gets a checked item for the head pose
      with the plan folder in backticks, and fully specified follow-ups,
      unchecked: an MCP companion field; a strip filter is out (on-demand
      only); roll-corrected re-crop for the pose only (a second model run
      per face, ~35 ms) if Step 2 shows roll hurts; using the pose to
      flag "looking away" frames once a threshold is labeled.
    - `mise run ci` passes.
  - Implementation approach:
    - Docs only; numbers from Steps 2 and 3's `learnings.md`, re-measured
      once on the merged `main`. If Step 3's PR already completes the
      performance paragraph, fold this into it and say so in Progress.

## Trade-offs and risks

- **Full MediaPipe pipeline vs weak perspective (recommended: port the
  full pipeline, Step 2 keeps it unless the measurement says otherwise).**
  The requirement names MediaPipe's method as first choice. The
  perspective unprojection is what keeps an off-center face from reading
  as turned; a plain Procrustes on pixel coordinates is ~40 fewer lines but
  biased at the frame edges. Step 2 measures both only if the weak variant
  is cheap to add; otherwise the full pipeline alone.
- **Camera FOV (open; measured in Step 2).** MediaPipe assumes a 63 deg
  vertical FOV webcam. A 200 mm telephoto frame has ~7 deg; the
  unprojection then exaggerates perspective that is not there. Option A:
  keep 63 deg (identical to MediaPipe, comparable to its demos). Option B:
  the EXIF FOV from `Shot` (the 35 mm-equivalent focal length; falls back
  to 63 deg when unknown, e.g. a Leica M with an unreported lens). Step 2
  decides on the numbers; if B wins, `head_pose` takes the FOV and
  `read_eyes` / the CLI compute it from the shot (no new Exif parsing is
  expected, `focal_length` is already read; check `focal_length_35mm`
  availability per format).
- **No roll correction of the crop (fixed).** The closed-eyes constants are
  frozen on the current crop, so the mesh is run on an unrotated crop. A
  strongly rolled face may get worse landmarks and so a worse pose. Not
  changed here; a second roll-corrected run for the pose only is a
  follow-up todo (Step 4) if Step 2 shows the need.
- **Where the pose lives (chosen: inside the eyes judgment).** One request,
  one model run, no second decode. The alternative, a separate `pose_of`
  command, would cost another ~90 ms per shown file for nothing. The cost
  is that `Eyes` carries a field the other session's change also touches
  adjacently; kept additive.
- **Display (chosen: one `Head pose` row with three signed integers).**
  Alternative: three rows (`Yaw`, `Pitch`, `Roll`), more scannable but
  three more lines in a pane already long; or words (`turned 12° right`),
  friendlier but harder to keep in sync between languages. The caller may
  prefer another; it is a one-function change in `meta.ts`.
- **Sign convention (chosen: right / up / clockwise positive).** MediaPipe
  itself exposes a matrix, not Euler angles, and its camera space has y up;
  the chosen convention is a Riffle decision documented in `usage.md` and
  pinned by tests. Changing it later is a sign flip but confuses users.
- **Labels are coarse and the user must review them.** There is no
  ground-truth angle for the sample photos; classes and directions are the
  best a human can give from a crop, so the "accuracy" is class agreement
  and sign agreement, not degree error. The agent labels, the user reviews
  (in Japanese); if the user has no time, the plan records the labels as
  unreviewed, as the closed-eyes plan did.
- **Profiles are rare in the data.** YuNet's detection and the AF-nearest
  face selection favor frontal faces; the profile class may have a handful
  of samples. Report counts honestly rather than claiming accuracy there.
- **Linear algebra dependency.** A hand-written 3x3 SVD keeps the tree
  dependency-free (nothing in `crates/core` does linear algebra today);
  `nalgebra` would be safer numerically but adds compile time and size.
  Step 1 writes it by hand with tests unless it proves fragile.
- **Edge-clamped crops.** A face at the frame edge gets a non-square crop
  (`crop_rgb` clamps) and a non-uniform resize; z is scaled by `kx` as
  MediaPipe scales by the ROI width. The pose of such faces may be off;
  Step 2 notes any such faces separately.

## Progress

- (2026-10-07) Step 1 complete
