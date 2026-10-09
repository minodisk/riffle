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

# Face mesh off the face: measure the misfits and de-rotate the crop by the eye line

## Purpose

The scan's second pass and the on-demand `eyes_of` fit the MediaPipe Face
Landmarker mesh on a square crop of the YuNet face box cut from the upright
preview, **with no roll correction** (`crates/core/src/eyes.rs`, module
comment). A face rotated in the image plane (a baby lying down, a tilted
head) gets a mesh fitted as if upright, with its eye points on a hand or the
forehead; yet every value read from that mesh looks plausible: the AF eye
in-focus probability of `crates/core/src/candidate.rs` (since #733 measured
over the mesh's eye regions), the EAR of the closed-eyes judgment
(`eyes.rs`) and the head pose (`crates/core/src/pose.rs`). The frame
`2026-07-11__DSC2638` is the case the user found: good-tier, 1 star; EAR
0.36, yaw 3, roll 12, AF eye 100%, mesh eyes 0.125 face sides from YuNet's.

The burst-keep-score plan's Step 5 (`docs/plans/20261008-burst-keep-score/`,
merged to `main` as #752 on 2026-10-10) adds `eye_offset`
(the larger distance between a mesh eyelid contour's center and YuNet's eye
landmark of the same face, over the box's longer side) and drops frames over
0.10 from the Good / Fair tiers. But 28.6% of the meshed faced AF frames of
its six-folder re-dump exceed 0.10, and nobody has looked at whether those
meshes are truly off the face or the comparison is loose.

This plan (1) measures how many meshes are off and why, (2) counts the
misjudgments they cause in the three consumers, and (3) measures the
countermeasure the user agreed on, **MediaPipe's own pipeline step: compute
the face's roll from the line joining YuNet's two eye landmarks, rotate the
face upright, crop it, run the mesh on that crop, and rotate the points
back**, against the labeled sets, before and after. If the Decision adopts
it, it is implemented on the one crop path both the scan and `eyes_of`
share, and the shift of the stored values is recorded for the plan that owns
the good-photo thresholds.

### What is known

- **The crop today.** `eyes::face_square` squares the YuNet box on its
  longer side and grows it by `FACE_CROP` = 1.25; `landmarks_of` cuts it
  from the full upright image (`faces::crop_rgb`, clamped inside the
  image), `mesh_of` cuts the same window from the stored image through
  `upright_crop` (EXIF 3 / 6 / 8 handled per pixel); both resize bilinearly
  to `INPUT` = 256 and run `landmarks`, then `to_full` maps the points back
  by the window's scale. `mesh_of` solves `pose::head_pose` on the full
  upright points (x, y, z), then maps x / y to stored coordinates
  (`point_to_stored`). `judge_mesh` (the `eyes_of` path,
  `crates/app/src/commands.rs` `read_eyes`) calls `landmarks_of` on the
  upright RGB. A face under `EYES_MIN_FACE` = 60 px gets no mesh
  (`candidate::meshes_face`, `judge_mesh`).
- **The roll today.** `pose::head_pose` returns `None` when |roll| >
  `MAX_ROLL` = 90 (an upside-down fit). The pose's convention: yaw positive
  toward the image's right, pitch positive up, roll positive clockwise on
  screen (`Pose` doc). The mesh's own roll is **wrong on exactly the frames
  this plan is about** (`_DSC2638` read 12 deg), so it cannot stratify them.
- **YuNet's eye landmarks** (`faces::Face::left_eye` / `right_eye`, image
  left then right, in the coordinates of the image detected on; in stored
  coordinates after `to_stored`) exist on every detected face and are what
  Step 5's `eye_offset` compares against. Their accuracy on turned and tiny
  faces is not measured.
- **The six-folder re-dump of burst-keep-score Step 5**
  (`D:\Photos\tests\2026-10-09-good-mark\step5\dump\*.tsv`, `riffle-cli
  features` of that branch, columns `eye_offset` / `edge_gap` after `roll`):
  9868 rows, 7261 with a mesh, 2079 (28.6%) over 0.10. By the mesh's |roll|:
  26.1% under 15 deg (6734 frames), 56% at 15-30 (450), 83% at 30-60 (24),
  4 of 5 at 60+; all 48 frames with no pose exceed it. By |yaw|: 20.7%
  under 30, 34.7% at 30-60, 44.1% at 60+. By face side: 29.5% at 60-100 px,
  28.2% at 100-200, 22.7% at 200-400, 17.6% at 400+, so large faces misfit
  too. Of the 1321 frames the Step 5 cuts alone would tier, 100 (7.6%) are
  over 0.10. In-plane rotated faces are rare in the dump (479 frames with
  mesh |roll| >= 15), so a roll-stratified sample has to be drawn, not a
  random one.
- **Labeled sets to verify the frozen constants on** (all under
  `D:\Photos\tests\`, out of the repository):
  - AF eye: the XMP-flagged `*-focus-sample` folders (training
    `2026-06-05`, `2026-07-31`, `2026-09-13-a`, `2026-09-19`,
    `2026-09-19-focus-sample-2`; held-out `2026-06-14`, `2026-07-18`,
    `2026-08-01`, `2026-08-22`; 406 / 400 labeled faced frames). The
    mesh model's numbers to hold: held-out AUC 0.800, precision / coverage
    88.6% / 95.9% (`docs/plans/_archived/20261008-mesh-eye-focus/fit.md`);
    `riffle-cli candidates <dir>...` reproduces them from the current code.
  - Closed eyes: `docs/plans/_archived/20261007-closed-eyes-detection/eyes-truth.md`
    (253 faces with an open / closed label, 57 closed, on `2026-09-19` and
    the DNG `2026-02-01`; raw files `D:\Photos\tests\2026-10-07-closed-eyes\`,
    `labeled-paths.txt`, `labels-raw.txt`); the numbers to hold: face AUC
    0.974, accuracy 0.957 at `EYES_CLOSED_EAR` = 0.137. `riffle-cli eyes
    <dir|file>...` prints the EAR and pose per file.
  - Head pose: `docs/plans/_archived/20261007-head-pose/pose-truth.md`
    (159 labeled faces, `D:\Photos\tests\2026-10-07-head-pose\labels.tsv`);
    the numbers to hold: sign right on 94% (yaw), 95% (pitch), 15 of 18
    (roll), yaw class 68% (`pose-results.md`). The labels are unreviewed.
  - The user's stars: `D:\photos\samples\ARW\good-mark-2026-10-09\` (60
    ARWs, 1-5 stars in the `.xmp`), lists `samples-manifest.tsv` /
    `samples-scored.tsv` under `D:\Photos\tests\2026-10-09-good-mark\`.
- **Precedents for the staging**: `20261007-closed-eyes-detection`
  (truth set, stratified blind draw, the agent labels, the user's review is
  final), `20261007-head-pose` (a throwaway `posedump` / `sheet` CLI kept as
  a patch), `20261008-mesh-eye-focus` (measure through a CLI column first
  without changing the cue, fit, Decision, then the scan change).
- **What `burst-keep-score-step-5` changes** (merged to `main` as #752
  on 2026-10-10): `crates/core/src/candidate.rs` (`mesh_eye_offset`,
  `edge_gap`, `grown_bounds`; `mesh_eye_measures` takes the face; `Cue` /
  `EyeMeasures` gain the two fields), `crates/app/src/index.rs`
  (`SCHEMA_VERSION` 19, `FACES_VERSION` 8, columns `eye_offset` /
  `edge_gap`), `crates/cli/src/main.rs` (`features` columns),
  `crates/app/ui/src/focus.ts` (`MAX_EYE_OFFSET` 0.10, `MIN_EDGE_GAP` 0.02,
  yaw 30), `docs/agents/tauri-app.md`. It does **not** touch
  `crates/core/src/eyes.rs`, `pose.rs` or `faces.rs`.
- Scratch outputs of this plan go to `D:\Photos\tests\2026-10-09-mesh-roll\`;
  any new RAW sample to `D:\photos\samples\ARW\`.

## Steps

- [x] Step 1: Add the roll-corrected mesh as an opt-in function and a `riffle-cli meshfit` dump that runs both fits side by side with overlays
  - Done when:
    - `crates/core/src/eyes.rs` gains, **next to and not replacing**
      `mesh_of` / `landmarks_of` (so the scan, `eyes_of` and the Step 5
      branch are untouched):
      - `pub fn eye_line_roll(face: &Face) -> f32` (name free): the angle in
        degrees of the line from `left_eye` to `right_eye` (upright
        coordinates; positive clockwise on screen, so it is the `Pose::roll`
        sign; pinned by a test with both signs and 0).
      - a crop that rotates: for a square window of side `face_square`'s
        side centered on the face center, sample the full upright image by
        the inverse rotation about that center (nearest or bilinear, say
        which; pixels outside the image black or edge-clamped, say which),
        so the eye line is horizontal in the crop; and the inverse map of a
        model point back to the full upright image (rotate about the same
        center). Tests: a synthetic image with two marks rotated by a known
        angle lands them on one row of the crop; a point round-trips within
        a pixel; 0 degrees reproduces `crop_rgb`'s window pixel for pixel
        (so the existing behavior is the angle-0 case).
      - `pub fn landmarks_of_rotated(rgb, width, height, face, roll) ->
        Result<Option<Vec<[f32; 3]>>>` (name free; `roll` passed in, not
        read from the face, so a caller can try a dead band) returning the
        points in full upright image pixels, x / y rotated back, z
        unchanged; and `mesh_of_rotated(rgb, width, height, orientation,
        face, roll) -> Option<Mesh>` mirroring `mesh_of` (stored input,
        `upright_crop`-style sampling without rotating the whole image,
        points to stored coordinates, pose solved on the mapped-back upright
        points as `mesh_of` does, so the pose's roll is the true roll by
        construction). A synthetic test on the existing orientation 6
        fixture (`eyes.rs` tests near `mesh_of`) checks the rotated and the
        unrotated function agree at roll 0 on the same stored image.
    - `riffle-cli meshfit <dir|file>... <out-dir>` (`crates/cli/src/main.rs`,
      a new subcommand at the end of the file, one `match` arm) runs, per
      RAW file, the scan's detection (`faces::detect_around` with
      `sharpness::trusted_focus`), takes the AF face
      (`candidate::nearest_face`, else the `eyes::judged_face` fallback so
      the DNG eyes / pose labels can be used), and when `meshes_face`
      accepts it runs **both** `mesh_of` and `mesh_of_rotated` (the roll
      from `eye_line_roll` of the upright face), and prints one
      tab-separated line: folder, file, the XMP flag, face side, the AF /
      no-AF path, `yunet_roll`, then for `before` and `after` each: the
      cue's state and `eye_focus` (`candidate::eye_focus` on the luma with
      the mesh; its `scored` / `logit` too), the EAR (`more_closed_ear`),
      yaw / pitch / roll, `eye_offset` (`candidate::mesh_eye_offset`, on
      `main` since Step 5 merged; the plan first had the CLI reproduce it
      locally, see `learnings.md`), the mesh time in ms. `-` for a missing value, `err`
      for a failed stage; a unit test pins one line.
    - The same command writes, per file, one PNG into `<out-dir>`: the face
      crop (the `face_square` window, upscaled to a fixed side, e.g. 384)
      with YuNet's box and two eye points, the before mesh's two eyelid
      contours (`LEFT_EYE_CONTOUR` / `RIGHT_EYE_CONTOUR`) in one color and
      the after mesh's in another, drawn with `draw_rect`-style marks (no
      new dependency), and the stem, `yunet_roll` and both `eye_offset`s in
      the file name so a sheet can be read without the TSV.
    - The dump over the six re-dump folders of burst-keep-score Step 5 and
      over the labeled sets above is saved under
      `D:\Photos\tests\2026-10-09-mesh-roll\dump\` (TSV and overlays);
      `learnings.md` records the mesh time before / after and the share of
      faces whose `after` mesh failed.
    - `mise run ci` passes.
  - Implementation approach:
    - Files: `crates/core/src/eyes.rs`, `crates/cli/src/main.rs`, this
      plan's `learnings.md`. **No change** to `candidate.rs`, `pose.rs`,
      `scan.rs`, `index.rs`, `commands.rs`: those wait for the gate before
      Step 3. `main.rs` is also changed by Step 5 (the `features` columns);
      keep the new code at the end of the file so a rebase is trivial.
    - MediaPipe's pipeline rotates so the eye line is horizontal and grows
      its box 1.5x; keep `FACE_CROP` and `face_square` as they are so only
      the rotation differs between before and after.
    - Reuse `resize` / `input_tensor` / `landmarks` / `to_full` where the
      scale map is the same; the rotation is a small sampler, not an image
      crate feature (`image` is built with `jpeg` only).

- [x] Step 2: Label the misfits, count the misjudgments, compare before and after on the labeled sets, and write the Decision
  - Done when:
    - **Misfit truth set** (`misfit-truth.md` in this plan folder, as
      `eyes-truth.md` / `pose-truth.md` were made): about 120-150 frames
      drawn from the six-folder dump, stratified by `before eye_offset`
      (under 0.06, 0.06-0.10, 0.10-0.20, over 0.20) and by `yunet_roll`
      (under 10, 10-25, over 25 deg) plus a random slice, shuffled blind,
      read from the Step 1 overlays on sheets under
      `D:\Photos\tests\2026-10-09-mesh-roll\sheets\`; per frame two labels,
      `before` and `after`: `on` (both eye contours on the eyes), `off`
      with the kind (`rotated` in-plane face, `side` profile / far turn,
      `small`, `occluded` hand / hair / glasses, `cut` by the frame,
      `notface` false detection, `other`) or `x` unreadable; and, for the
      comparison itself, whether YuNet's two eye points are on the eyes
      (`yunet ok` / `yunet off`), so a large `eye_offset` with an `on` mesh
      and `yunet off` counts as a comparison artifact. The agent labels; the
      user's review is final (ask in Japanese; the Decision says whether it
      is pending). Acceptance criterion 1: the per-kind counts and the
      share of `off` per `eye_offset` bucket (so the reading of 0.10 as
      "off the face" is checked), per face-size bucket and per `yunet_roll`
      bucket.
    - **Misjudgments** (`compare.md`): over the labeled sets, restricted to
      the frames whose `before` mesh is `off` (from the truth set) or whose
      `before eye_offset` exceeds 0.10 (over the whole dump), how many the
      three consumers misjudge: AF eye (the state against the XMP flag on
      the focus-sample folders), closed eyes (the EAR class against
      `eyes-truth.md`), head pose (the yaw sign / class and the roll sign
      against `pose-truth.md`), before and after; and over the 60 starred
      sample frames the values before / after with the stars beside them
      (information only; the tiers are not re-cut here). Acceptance
      criterion 2.
    - **The frozen constants, verified after rotation** (`compare.md`,
      `compare.py`, pure Python over the Step 1 TSVs): with the
      constants as they are, (a) AF eye on the training and held-out
      focus-sample folders: AUC, precision / coverage at `CANDIDATE_LOGIT`,
      before (must reproduce 0.800 / 88.6% / 95.9% held out) and after;
      (b) closed eyes on the 253 labeled faces: AUC and accuracy at
      `EYES_CLOSED_EAR`, before (0.974 / 0.957) and after; (c) head pose on
      the 159 labeled faces: sign agreement per axis and yaw class, before
      and after. Each also split by `yunet_roll` bucket (the change should
      be concentrated above ~10 deg) and by |yaw| and face-size bucket (where
      YuNet's landmarks are shakiest). If a number falls below its recorded
      value beyond noise, a re-fit on the training rows (`MESH_LOGIT_*` /
      `CANDIDATE_LOGIT` as `mesh-eye-focus/fit.py` did; `EYES_CLOSED_EAR` as
      the best F1 as before) is run here, its held-out numbers reported and
      the coefficients saved in `frozen.json`, so the Decision can adopt
      "rotate and re-fit" as one unit.
    - **Variants compared** (same tables): always rotate; rotate only when
      |`yunet_roll`| exceeds a dead band (10 and 20 deg at least); skip the
      rotation when the YuNet eye distance is under a fraction of the box
      side (a guard for landmarks that are not on the eyes), if the by-yaw
      split shows the need.
    - **Decision** (below): whether to adopt the rotation, which variant,
      whether the constants are re-fitted, what share of the 28.6% it
      recovers and what remains (for Step 5's `MAX_EYE_OFFSET`, which this
      plan does not move), and the residual kinds (side, small, occluded)
      that no rotation fixes. **The plan stops here until the user approves
      the Decision** (the main agent asks in Japanese); Step 3 does not
      start before that.
    - `mise run ci` passes (lint, lychee on the new files).
  - Implementation approach:
    - Assumes Step 1 is merged. Scripts stay in this plan folder as the
      precedents kept `fit.py` / `tune.py`; the sheets, crops and raw TSVs
      stay in `D:\Photos\tests\2026-10-09-mesh-roll\`.
    - Files: `misfit-truth.md`, `compare.md`, `compare.py`, optionally
      `frozen.json`, `plan.md` (Decision), `learnings.md`.

- **Gate (before Step 3 starts):** the user approved the Decision. The
  other half, `burst-keep-score-step-5` merged to `main`, holds since
  2026-10-10 (#752: `SCHEMA_VERSION` 19 / `FACES_VERSION` 8 in
  `crates/app/src/index.rs` on `main`), so Steps 3 and 4 no longer conflict
  with that branch.

- [ ] Step 3: Rotate the crop by the eye line on the shared mesh path and re-run pass 2
  - Done when:
    - `eyes.rs`: `landmarks_of` and `mesh_of` take the rotated crop of
      Step 1 with the roll from `eye_line_roll` of the face (and the dead
      band / guard the Decision chose, as named, documented constants); the
      opt-in functions of Step 1 fold into them (no two public paths left);
      the module comment no longer says "no roll correction" and says what
      the crop is; `judge_mesh` (so `eyes_of`, the meta pane and the mesh
      overlay) and the scan's `focus_cue_unless` change behavior through
      the same two functions and nothing else. The on-demand `eyes_of`
      points stay in stored preview coordinates, so `facemesh.ts` draws
      them unchanged.
    - The pose's roll is the true roll: solved on the mapped-back upright
      points. A synthetic test rotates the existing orientation fixture's
      face by a known in-plane angle and checks `Mesh::pose.roll` moves by
      that angle with the `Pose` sign; `pose.rs` is only touched if that
      test needs a helper. `MAX_ROLL` stays.
    - If the Decision re-fitted: the new `MESH_LOGIT_*` / `CANDIDATE_LOGIT`
      / `EYES_CLOSED_EAR` in `candidate.rs` / `eyes.rs` from `frozen.json`,
      with their doc comments carrying the new training / held-out numbers
      and the date; the frozen-coefficient tests updated. Otherwise the
      constants are untouched and their doc comments gain one line saying
      they were re-verified after the rotation (the numbers from
      `compare.md`).
    - `crates/app/src/index.rs`: `FACES_VERSION` 9 with its history line
      ("re-runs it after the mesh crop gained the eye-line roll
      correction"); no schema change. `docs/agents/tauri-app.md`'s
      `FACES_VERSION` note names it.
    - `riffle-cli candidates` on the training and held-out folders and
      `riffle-cli eyes` on the labeled faces reproduce `compare.md`'s
      "after" numbers within rounding (recorded in `learnings.md`);
      `riffle-cli meshfit` keeps working as a diagnostic (its `before`
      columns now come from a roll of 0 passed explicitly, say so in its
      doc comment) or is reduced to the overlay writer, whichever is less
      code.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes the gate above. Files: `crates/core/src/eyes.rs`,
      `crates/core/src/candidate.rs` (constants only), `crates/core/src/pose.rs`
      (test helper only, if at all), `crates/app/src/index.rs`,
      `crates/cli/src/main.rs`, `docs/agents/tauri-app.md`, `learnings.md`.
    - The mesh runs on many rayon threads; the rotation sampler allocates
      one crop per call as `upright_crop` does. Measure the per-file mesh
      time before / after on `2026-09-19` at 24 threads (`riffle-cli
      candidates`), and the app's `scan faces` log line on that folder if
      the GUI is run (else say so), for Step 4.

- [ ] Step 4: Record the shift of the stored values and bring the docs and the todo in line
  - Done when:
    - **Shift record.** `riffle-cli features` (from `main` after Step 3)
      over the same six folders as the Step 5 re-dump, saved under
      `D:\Photos\tests\2026-10-09-mesh-roll\redump\`; `shift.md` in this
      plan folder: per column the percentiles before (the Step 5 re-dump)
      and after (EAR, `eye_focus`, `eye_offset`, yaw / pitch / roll), the
      share over `MAX_EYE_OFFSET` 0.10 before and after, the share of
      frames whose `candidate` state or `eyes` state changed, and the Good
      / Fair shares the **unchanged** `focus.ts` cuts would give on the new
      values (information for the owning plan; the cuts are not moved
      here). `learnings.md` says the same in two lines and that the
      good-photo thresholds (`GOOD_*` / `FAIR_*` / `MAX_EYE_OFFSET` /
      `MIN_EDGE_GAP`) are to be re-measured by `20261008-burst-keep-score`
      with its 60-frame scoring, not here.
    - `todo.md`, face / eye section: a checked item for this work with the
      plan folder in backticks, the misfit share before / after and the
      residual kinds; an unchecked, fully specified item for the owning
      plan's re-measurement of the `focus.ts` thresholds on the rotated
      values (the files, the dump to use, the 60-frame sample), and one for
      each residual misfit kind the Decision named worth a follow-up (e.g. a
      YuNet-landmark sanity guard, a side-face rule).
    - `docs/humans/usage.md` (Focus mark paragraph, the closed-eyes / head
      pose sentences, if they describe the crop) and `performance.md`
      ("Focus candidate pass", "Closed-eyes judgment on demand") updated
      with the rotation and the Step 3 timings; `usage.ja.md` /
      `performance.ja.md` in sync. `CLAUDE.md` Layout: `src/eyes.rs` names
      the eye-line roll correction. `docs/agents/tract-onnx-inference.md`:
      a short Measured item on feeding the mesh a de-rotated crop (what
      changed in the outputs). `README*.md` only if a sentence changes
      meaning.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 3 is merged. Docs and one re-dump; no code beyond a
      scratch script kept in this plan folder if `shift.md` needs one
      (`shift.py`).

## Trade-offs and risks

- **Always rotate vs a dead band.** MediaPipe always rotates by the eye
  line. Always rotating is the least code and matches the reference
  pipeline, but shifts every EAR / cue / pose value a little, so the
  frozen constants (fitted on unrotated crops) must hold up on the labeled
  sets, which is what Step 2 checks. A dead band (rotate only above ~10
  deg) keeps most frames byte-identical and the constants safe, at the
  price of one more cut to justify and a discontinuity at the band.
  Decided in Step 2 from the by-roll split; default to always-rotate when
  the labeled metrics hold.
- **The roll source is YuNet's landmarks, which can be off.** On profiles,
  tiny faces and occlusions the two eye points can sit off the eyes, and a
  bogus angle makes a good mesh worse. Step 2 measures by |yaw| and face
  size; if the "after" numbers drop there, a guard (skip when the eye
  distance is under a fraction of the box side, or when YuNet's points are
  outside the box) is a cheap option the Decision can pick. The alternative
  roll source, a first mesh pass (MediaPipe's own tracking loop), doubles
  the model cost and is not planned.
- **Roll of the pose: mapped-back points vs add the angle.** Solving
  `head_pose` on the points mapped back to the full upright image gives the
  true roll with no sign bookkeeping; solving on the de-rotated points and
  adding `yunet_roll` needs the `Pose` sign matched. The plan prefers the
  first and pins it with a test; the pitch / yaw are the same either way
  since the unprojection is on the same camera.
- **Re-fit or keep the constants.** `MESH_LOGIT_*` / `CANDIDATE_LOGIT`
  (806 labeled frames) and `EYES_CLOSED_EAR` (253 faces) were fitted on
  unrotated crops. Re-fitting widens Step 3 and moves the stored
  probabilities for every frame; keeping them risks a biased cue on rotated
  faces only (a small share). Step 2 reports both and the Decision chooses;
  a re-fit needs the user's approval as part of the Decision.
- **Rotation does not fix every misfit.** Side faces, small faces, hands
  and glasses, and cut faces will still put the mesh off; the eye offset
  stays the guard for those, and `MAX_EYE_OFFSET` is the owning plan's
  constant to move. Step 2 says what share of the 28.6% is recovered. If it
  is small (say under a third) the Decision may still adopt the rotation
  for the correctness of the EAR and the pose on tilted heads, since the
  cost is one sampler; or decline and record why.
- **The comparison itself may be loose.** YuNet's eye point and the mean of
  the 16 contour points need not coincide even on a perfect fit (the
  landmark sits on the pupil, the contour center a little below on a
  downcast eye); the truth set's `yunet off` label and the `on` share per
  `eye_offset` bucket are what tell a loose comparison from an off mesh.
  If most frames at 0.10-0.15 are `on`, that is a finding for the owning
  plan, recorded, not acted on here.
- **Ordering against `burst-keep-score-step-5`.** It changed
  `candidate.rs`, `index.rs`, `main.rs`, `focus.ts` and merged to `main`
  (#752) before Step 1 was implemented, so the CLI reads
  `candidate::mesh_eye_offset` directly and the ordering risk is gone;
  Steps 1 and 2 still leave `candidate.rs` / `index.rs` alone, and Steps 3
  and 4 wait only for the Decision.
- **Labels are small and reviewed late.** 120-150 misfit labels give
  shares with about +/-8 points; the eyes and pose labels are the earlier,
  partly unreviewed sets. The Decision states what each number rests on
  and whether the user's review is pending.
- **The index re-runs pass 2 once** (`FACES_VERSION` 9) on every folder;
  thumbnails stay. The on-demand `eyes_of` and the stored values keep the
  same crop, so the meta pane and the index agree, which is the invariant
  the `mesh-eyes-index` work set.

## Decision

**Awaiting the user's approval before Step 3.** The misfit labels it rests
on are the agent's; **the user's review of `misfit-truth.md` is pending**.

**Recommendation: do not adopt the eye-line rotation in any variant; no
re-fit; Steps 3 and 4 are not run as written** (see "If approved" below).
The numbers are in `compare.md` and `misfit-truth.md`.

- **Always rotate (MediaPipe's step) loses.** On the 131 readable truth-set
  frames it turns 7 off meshes on and 15 on meshes off (94 -> 86 `on`). AF
  eye held-out AUC 0.800 -> 0.783, precision / coverage 88.6% / 95.9% ->
  88.3% / 95.3%, with the drop on the tilted frames (|`yunet_roll`| 10-25:
  0.716 -> 0.595); closed eyes accuracy 0.957 -> 0.949 (11 -> 13 wrong),
  AUC 0.974 -> 0.973; head pose +1 face on the yaw sign and the yaw class.
  The share over 0.10 on the six folders stays 28.7% (178 frames fall
  under, 177 rise over).
- **Dead bands do not help** (`band 10` matches `always` on the truth set and
  the pose, ties `before` on closed eyes and has the lowest held-out AF AUC
  of all variants, 0.779; `band 20` 83 `on`); `band 10-25`, the one
  in-sample gain on the truth set (+3 `on`), costs held-out AF AUC (0.787)
  and moves little else (yaw sign 76 -> 77, closed-eyes AUC 0.974 -> 0.976,
  six-folder count 2083 -> 2081).
- **The YuNet eye-distance guard (`band 10 + wide 0.20`) is neutral**: it
  rotates 1.8% of the six-folder frames and almost none of the labeled ones
  (0 / 5 / 3 / 1 of the AF training / held-out, closed-eyes, pose faces),
  changes no truth-set label and no frozen number (yaw class +1 face), and
  recovers nothing (2083 -> 2084 over 0.10). The tilted frontal faces it
  targets already fit (12 of 12 `tilted-wide` truth frames `on` before),
  and it flips the 4-star `2026-10-03__DSC3537` to closed eyes (EAR 0.350
  -> 0.055). One more crop path for no measured gain.
- **Why**: where the rotation changes a fit, YuNet's two eye points are not
  on two eyes. Every truth-set label change sits at a YuNet eye distance of
  0.14 of the box side or less (profiles whose points sit together on one
  eye or the nose), so the eye-line angle is noise there. And the case that
  started the plan, `2026-07-11__DSC2638`, is a baby lying head-down:
  `faces.rs` orders YuNet's eye points by image x, so the eye-line roll is
  folded into -90..+90 deg and a face past 90 deg reads nearly level; no
  eye-line rotation turns it upright.
- **What the 28.6% is** (28.6% is the Step 5 re-dump's index rows, 28.7% the `meshfit` dump's population): Weighted back to the six folders, about 8% of the
  meshed AF frames have a mesh off the face; of the frames over 0.10 about
  27% are off and about 65% are an `on` mesh compared with YuNet eye points
  that are off the eyes. Frames with a YuNet eye distance under 0.20 are
  33.5% of the frames but 66% of those over 0.10. So `MAX_EYE_OFFSET`
  0.10 mostly flags a loose comparison, not a misfit (two `on` frames in
  three in the 0.10-0.20 bucket); this is a finding for
  `20261008-burst-keep-score`, which owns the cut, not acted on here.
- **Residual kinds no rotation fixes**: weighted, `side` 78% of the off
  meshes, then `cut`, `rotated` (past 90 deg or with YuNet's points off the
  eyes), `other` (a face seen from straight above), 6-7% each, and
  `notface` (back of a head, two faces in one box). No frame at or above
  60 px was off for being `small`.
- **Misjudgments** on the frames whose `before` mesh is off or over 0.10:
  AF eye state wrong on 68 of 297 before, 71 always, 68 guarded (23%
  against 7% elsewhere); closed eyes 2 of 47 in every variant; yaw sign 4 /
  40 -> 3, yaw class 22 / 56 -> 20, roll sign 2 / 9 always. The errors do
  concentrate on those frames, but on side faces the rotation cannot fix.
- **What each number rests on**: the frozen numbers reproduce exactly from
  the dump (0.800 / 88.6% / 95.9%, 0.974 / 0.957, 94% / 95% / 15 of 18 /
  68%); the AF-eye flags are the user's; the closed-eyes and pose labels are
  the earlier agent sets (the pose set unreviewed); the misfit labels are
  the agent's, 140 frames, about +/-8 points per share, user review
  pending. The guard threshold could not be chosen on the AF training rows
  (it rotates none of them from 0.20 up) and was read off the eye-distance
  distribution and the truth set; the conclusion does not depend on it.

**If approved**, Steps 3 and 4 are replaced by one wrap-up step: no change
to `eyes.rs`' crop, `FACES_VERSION` or the constants; `todo.md` gains a
checked item for this measurement (`docs/plans/20261010-mesh-roll/`, the
28.7% before and after, the residual kinds) and unchecked items for (1) the
owning plan to make `eye_offset` reliable where YuNet's eye distance is
under 0.20 of the box (skip the cut there, or compare against the mesh's own
geometry), (2) a side-face rule for the mesh values (the dominant misfit),
(3) faces rolled past 90 deg, which need a roll source other than YuNet's
x-ordered eye points (e.g. a mesh pass on rotated crops), not planned; and
the user decides whether the Step 1 opt-in functions
(`eye_line_roll`, `rotated_crop`, `landmarks_of_rotated`, `mesh_of_rotated`)
and `riffle-cli meshfit` stay as a diagnostic or are removed.

## Progress

- (2026-10-09) Plan written
- (2026-10-10) Step 1 complete
