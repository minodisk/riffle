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

# AF eye focus over the face mesh eye regions

## Purpose

The AF eye in-focus probability (`crates/core/src/candidate.rs`, `eye_focus`
/ `eye_window`) is measured over one square window centered between YuNet's
two eye points whose side is the face box's long side (at least
`CANDIDATE_WINDOW_MIN` = 24 px). That window covers the whole face: brows,
hairline, the face outline and some background, so a frame focused on the
hair or an ear but not on the eyes can still score high. The MediaPipe Face
Landmarker v2 face mesh (`crates/core/src/eyes.rs`) already gives the eyelid
contour and the iris of each eye, and `crates/core/src/pose.rs` gives the
head pose from the same points, but only on demand (`eyes_of`), not in the
scan.

This work measures the Laplacian variance and the mean edge width over each
eye's region cut from the mesh's eyelid contour, scores each eye and picks
the eye to trust (the sharper one, or the eye facing the camera by the head
pose's yaw, whichever the fit favors), runs the mesh in the faces scan pass
to do so, falls back to the current window for faces too small for a
meaningful eye region and for mesh failures, refits the logistic on the same
hand-labeled frames as the current model, and adopts the new region only if
it beats the current one on the held-out set. The user sees the same `AF eye
in focus` percentage, focus mark color and `AF eye` filter; what changes is
what the number looks at.

### What is known

- Current model (`candidate.rs`): `logit = LOGIT_INTERCEPT + LOGIT_LAP *
  ln(lap + 1) + LOGIT_EDGE_WIDTH * ln(edge_width / window side)`, fitted on
  406 hand-labeled faced frames from 5 folders (AUC 0.852; the Laplacian
  alone 0.816), threshold `CANDIDATE_LOGIT` = 1.2194 (p about 0.772) chosen
  to keep the earlier coverage: training precision 93.1% / coverage 91.4%;
  held-out 400 frames from 4 folders AUC 0.754 (0.635), precision 89.1%,
  coverage 95.3%. The fit and the validation are described in
  [`../_archived/20260926-af-eye-in-focus-probability/plan.md`](../_archived/20260926-af-eye-in-focus-probability/plan.md)
  and its `learnings.md`; the reference fitting code was a scratch crate
  (`reference-metrics.rs` there), not part of the workspace, with the frozen
  numbers in `frozen.json`.
- Labeled sets (XMP `xmpDM:good`: Pick = in focus, Reject = off), all under
  `D:\Photos\tests\`: training `2026-06-05-focus-sample`,
  `2026-07-31-focus-sample`, `2026-09-13-a-focus-sample`,
  `2026-09-19-focus-sample`, `2026-09-19-focus-sample-2`; held-out
  `2026-06-14-focus-sample`, `2026-07-18-focus-sample`,
  `2026-08-01-focus-sample`, `2026-08-22-focus-sample`. The reserved
  `2026-08-29-focus-sample` and `2026-09-13-b-focus-sample` still carry no
  XMP sidecars (checked 2026-10-07), so they cannot be used.
- `riffle-cli candidates <dir>... [threads]` (`crates/cli/src/main.rs`)
  prints one line per file (state, p, lap, edge, face box, flag) and, pooled
  over the labeled faced frames, candidates / precision / coverage and the
  AUC of lap and of the combined logit (ties 0.5, frames without an edge
  width skipped from the AUC). It reproduces the numbers above exactly.
- Coordinates: `candidate.rs` works in the preview's stored (unrotated)
  pixels on the luma of the one full-size decode; `eyes.rs` runs the mesh on
  the upright image (`faces::upright_rgb`, `faces::face_to_upright`) and
  `commands.rs` `read_eyes` maps the points back with
  `faces::point_to_stored`. On `main` (`#723`) `eyes::landmarks_of(rgb,
  width, height, &face) -> Result<Option<Vec<[f32; 3]>>>` returns x, y in
  upright pixels and z at the scale of x, with no size floor;
  `judge_mesh` floors at `EYES_MIN_FACE` = 60 px and returns `Judged {
  eyes, points, pose: Option<Pose> }`. The crop is the YuNet box squared and
  grown `FACE_CROP` = 1.25x, no roll correction (frozen for the closed-eyes
  constants; do not change it).
- Head pose (`#723`-`#725`): `pose::head_pose(points: &[[f32; 3]], width,
  height) -> Option<Pose>` on the upright image's points; `Pose { yaw,
  pitch, roll }` in degrees, yaw positive when the face turns toward the
  image's right, pitch positive up, roll positive clockwise; `None` on
  fewer than 468 points, a non-finite point, a degenerate solve or a roll
  beyond `MAX_ROLL`. Measured in the head-pose plan's Step 2
  (`docs/plans/20261007-head-pose/`, to be archived under `_archived/`):
  8.2 us per face, the yaw sign right on 94% of 81 labeled turned faces,
  |yaw| median 10 deg frontal / 41 oblique / 65 profile, ~15 of 2024 faces
  fit upside down and get no pose. So the pose comes for free once the mesh
  runs in the scan, and yaw tells which eye faces the camera: on a turned
  face the far eye is foreshortened, and on a profile the mesh places the
  hidden eye over hair, cheek or background, whose edges can read as sharp,
  so "take the sharper eye" can pick a phantom eye.
- Eye contour indices (MediaPipe `FACEMESH_LEFT_EYE` /
  `FACEMESH_RIGHT_EYE`, the same edges `crates/app/ui/src/facemesh.ts`
  draws): image-left eye 33, 7, 163, 144, 145, 153, 154, 155, 133, 173, 157,
  158, 159, 160, 161, 246; image-right eye 263, 249, 390, 373, 374, 380, 381,
  382, 362, 398, 384, 385, 386, 387, 388, 466. Irises: 468 (center) + 469-472,
  473 (center) + 474-477. Provenance is already in
  `crates/core/models/LICENSE-mediapipe`.
- Cost: the mesh is ~35 ms per face single-threaded (p95 41 ms), the plan
  build ~180 ms once per process (`docs/humans/performance.md`, "Closed-eyes
  judgment on demand"); the pose solve is microseconds. Pass 2 on the
  2134-ARW folder `D:\photos\2026\2026-09-19` takes ~10 s with 24 threads
  (`riffle-cli candidates <dir> 24`, ~1950 files with an AF face), so the
  mesh is expected to add roughly 3-4 s of wall time there (+30-40%) and
  ~35 ms per faced file on one core. The earlier model survey rejected the
  mesh for pass 2 on a +25% budget; this plan measures the real figure in
  Step 4 and the user decides with it on the table (see Trade-offs).
- Versioning: a change to the cue is a `FACES_VERSION` bump
  (`crates/app/src/index.rs`, now 5; `docs/agents/tauri-app.md` "The second
  pass has its own version"), not `EXTRACTOR_VERSION`: pass 1 is unchanged.
  The index stores one probability (`files.eye_focus`) and derives the state
  by comparing it with `candidate_probability()` = sigmoid(`CANDIDATE_LOGIT`),
  so every path that writes a probability must share one threshold (see
  Trade-offs, "Two models, one threshold").
- The head-pose plan is on its last step (Step 4, docs and todo only, in
  review on `head-pose-step-4`): it edits `docs/humans/performance.md` (and
  `.ja.md`, the "Closed-eyes judgment on demand" section) and the face
  section of `todo.md`. This plan's Step 4 edits other sections of those
  files ("Focus candidate pass", "Which pass carries which cost") and adds
  todo items next to the head-pose ones; it assumes head-pose's wrap-up has
  merged, which it will long before. This plan only adds to `eyes.rs`
  (contour constants, a mesh helper) and does not touch `pose.rs`,
  `commands.rs` `read_eyes`, `eyes.ts` or `meta.ts`.

## Steps

- [x] Step 1: Measure per-eye focus features over the mesh eye regions and print them, with the head pose, from `riffle-cli candidates`, without changing the cue
  - Done when:
    - `crates/core/src/eyes.rs` exports the two eyelid contour index tables
      and the two iris index tables (as `const`s next to `LEFT_EYE` /
      `RIGHT_EYE`, with the MediaPipe provenance line), and a
      `pub fn mesh_of(rgb, width, height, orientation, face: &Face) ->
      Option<Mesh>` (name free) that takes the stored-coordinate RGB and
      face the cue has, runs the mesh (`landmarks_of` on the upright image
      via `face_to_upright` / `upright_rgb`, or a crop rotated on its own if
      cheaper; measure which) under `catch_unwind` like `judge_mesh`,
      computes `pose::head_pose` on the upright points (the same call
      `judge_mesh` makes), and returns the points mapped to stored
      coordinates (`point_to_stored`, x and y only) together with the
      `Option<Pose>`, with **no** size floor (the floor is chosen in Step 2
      from the data). `judge_mesh`, `EYES_MIN_FACE`, the crop and the EAR
      are untouched.
    - `crates/core/src/candidate.rs` gains pure, mesh-free functions, unit
      tested on synthetic points: `eye_region(points, contour, width,
      height) -> Option<Window>`, the axis-aligned bounding box of the 16
      contour points grown by a margin (a named constant, a fraction of the
      eye width, so the lid edges and lashes lie inside; the value is a
      Step 2 measurement, start at 0.25) and clamped into the image, `None`
      when any point is non-finite or the box is under `3` px on a side;
      an iris region the same way from the 5 iris points; and
      `eye_measures(gray, width, window) -> (lap, edge_width,
      edge_width_rel)` with `edge_width_rel = edge_width / window.width`
      (the eye width, not the face), reusing `laplacian_variance` and
      `edge_width` unchanged.
    - A `pub struct EyeMeasures` (or similar) holds, per eye, the contour
      and iris windows and their measures, plus the pose; `focus_cue` /
      `eye_focus` and `Cue` are **unchanged**, so the scan, the index and
      the app behave exactly as before and `FACES_VERSION` stays 5.
    - `riffle-cli candidates` prints, after the existing columns, the face's
      long side, the mesh time in ms, `yaw` / `pitch` / `roll` (one decimal,
      `-` when the pose is `None`), and per eye (`L` / `R`, image side): the
      contour window side, lap, edge width, and the iris window side and lap
      (`-` when the mesh failed or a region is `None`); the summary lines
      are unchanged. The lines must stay parseable by a script (one
      space-separated record per file; say the format in a doc comment).
    - The report's per-file lines on the training and held-out folders are
      saved to `D:\Photos\tests\2026-10-08-mesh-eye-focus\` (not committed)
      for Step 2, and `learnings.md` records how many faced frames got a
      mesh, how many got a pose, how many eye regions were under 24 px, the
      |yaw| distribution of the labeled frames, and the mesh time
      distribution.
    - Tests: `eye_region` on hand-placed points (box, margin, clamping at
      an image edge, non-finite point, too small); `eye_measures` on the
      `ramp` images already in `candidate.rs`; the existing tests still
      pass; `mise run ci` passes.
  - Implementation approach:
    - Files: `crates/core/src/eyes.rs`, `crates/core/src/candidate.rs`,
      `crates/cli/src/main.rs`, this plan's `learnings.md`.
    - Keep the per-eye measurement separate from the scoring so Step 3 can
      swap the scoring without touching the measurement, and so tests do
      not need the model. Do not add any new ONNX or dependency; `pose.rs`
      is used, not changed.
    - The CLI already decodes the preview a second time for the report
      (`measures` in `candidates`); add the mesh there, not in
      `extract_analysis`, so this step leaves the scan untouched.

- [x] Step 2: Refit the logistic on the mesh eye features and decide whether it replaces the window, and which eye counts
  - Done when:
    - `fit.md` in this plan folder: on the 406 training frames, with the
      same pooling rules as the reference (labeled faced frames; frames with
      no edge width skipped from the AUC), the AUC and the precision /
      coverage at a threshold chosen to keep the 91.4% training coverage,
      for at least these variants, each also run on the 400 held-out frames
      with the frozen fit: (a) the current window (baseline; must reproduce
      0.852 / 0.754), (b) contour region, sharper eye = the higher per-eye
      logit, fitted on both eyes carrying the frame's label, (c) contour
      region, the eye nearer the AF point only, (d) contour region with the
      margin at 0 / 0.25 / 0.5, (e) iris lap added as a third feature if the
      iris windows are large enough on most faces, (f) the near eye by the
      yaw sign (the eye facing the camera: yaw positive, the face turns
      toward the image's right, so the image-right eye is nearer) when
      |yaw| exceeds a cut, else both eyes / the sharper as in (b), (g) both
      eyes but the far eye dropped above a |yaw| cut, the cut for (f) and
      (g) chosen from the data (try 30 and 45 deg at least, report the
      counts per bucket). A frame whose pose is `None` behaves in (f) and
      (g) as the variant without the pose. Each variant's fallback for
      frames with no mesh region (face below the floor or mesh failure) is
      the baseline model's probability, so the comparison is over the same
      frames. For (f) / (g), also report how often the pose-chosen eye
      differs from the sharper eye, and the label split of those frames
      (that is where a phantom eye would show).
    - The eye-region size floor below which the window fallback is used is
      chosen here from the training data by contour window side (e.g. the
      AUC by bucket), recorded with the fraction of faces it sends to the
      fallback on both sets.
    - `frozen.json` in this plan folder with the chosen variant's
      coefficients, threshold, margin, floor, eye rule and yaw cut at full
      precision, and a copy of the fitting script in this folder (as the
      earlier plan kept `reference-metrics.rs`), fed from the saved Step 1
      lines; the script is not part of the workspace.
    - A **Decision** section in this plan.md: adopt the new region if its
      held-out AUC beats 0.754 and its held-out precision and coverage are
      not both below 89.1% / 95.3%, else keep the current window; and which
      eye rule and yaw cut Step 3 implements. If kept, Step 3 is skipped
      (marked so in Progress) and Step 4 records the finding; the Step 1
      measurement code stays as a CLI diagnostic.
    - `mise run ci` (lint, lychee on the new files) passes.
  - Implementation approach:
    - Assumes Step 1 is merged. Fit with plain maximum-likelihood logistic
      regression (the earlier fit's method); no regularization, no
      cross-validation beyond the held-out set, so the numbers stay
      comparable. Python is fine for the scratch script; it reads only the
      saved CLI output.
    - Files: `docs/plans/20261008-mesh-eye-focus/fit.md`, `frozen.json`,
      the fitting script, `plan.md` (Decision).

- [ ] Step 3: Score the AF eye over the mesh eye regions in the scan, with the pose-based eye choice and the window fallback, and re-run the second pass
  - Done when:
    - `candidate.rs`: new constants for the mesh model (`MESH_LOGIT_*`, the
      threshold, `EYE_REGION_MARGIN`, `EYE_REGION_MIN`, and, if Step 2 chose
      (f) or (g), `EYE_YAW_CUT` in degrees) copied from `frozen.json`,
      documented like the current `LOGIT_*` with the training and held-out
      numbers; `eye_focus` takes the mesh (points and `Option<Pose>`) and
      scores each eye's contour region, the frame's logit being the chosen
      eye's by Step 2's rule (with no pose, the no-pose behavior of that
      rule), with `EyeFocus` carrying both eyes' measures, the pose, and
      which eye won and why; when the mesh is `None`, either region is
      `None` or under `EYE_REGION_MIN`, it scores the current `eye_window`
      with the current coefficients (the fallback), marked as such in
      `EyeFocus`. `Unknown` stays what it is today.
    - One threshold: the state is still `logit >= CANDIDATE_LOGIT` and the
      index still derives it from the stored probability, so both paths'
      logits are on the same threshold (see Trade-offs; the chosen
      resolution is recorded in the constants' doc comments).
    - `focus_cue_unless` runs the mesh between the detection and the scoring
      with a cancel point before it; `scan::extract_analysis` is otherwise
      unchanged. The mesh plan builds once per process (`OnceLock`), so the
      first file of a scan pays ~180 ms; note it in the doc comment.
    - `crates/app/src/index.rs`: `FACES_VERSION` 6 with its history line
      ("re-runs it after the cue moved to the mesh eye regions"); no schema
      change.
    - `riffle-cli candidates` prints the chosen eye, the reason (sharper /
      near by yaw / only one region) and whether the window fallback was
      used, and on the training and held-out folders reproduces `fit.md`'s
      numbers within rounding (recorded in `learnings.md`); `riffle-cli
      faces` keeps printing the state and probability.
    - Tests: eye selection on synthetic eyes of different blur: the sharper
      wins with no pose or below the yaw cut; above the cut the eye the yaw
      sign names wins (both signs) even when the far eye measures sharper;
      a yaw at the cut and just below it (the boundary rule pinned);
      fallback when the mesh is `None`, when a region is too small, and
      when a point is non-finite; the two paths agree on the state at the
      threshold (the ulp round-trip test extended to the mesh model); the
      frozen coefficients test for the new constants; the existing
      `focus_cue` cancel test gains the new cancel point. `mise run ci`
      passes.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged and Step 2 decided to adopt.
    - Files: `crates/core/src/candidate.rs`, `crates/core/src/eyes.rs` (if
      the helper needs a floor parameter), `crates/core/src/scan.rs` (doc
      only), `crates/app/src/index.rs`, `crates/cli/src/main.rs`,
      `docs/agents/tauri-app.md` (the `FACES_VERSION` bullet names the mesh
      and the pose next to the eye window and the detector).
    - The scan's mesh runs on many threads at once: `Arc<TypedRunnableModel>`
      `run` is per-call state, as `faces.rs` already relies on; check the
      peak working set on 24 threads in Step 4.
    - Do not touch `eyes_of` / `read_eyes`, `pose.rs`, the closed-eyes
      constants or the crop; the mesh points, EAR and pose are not stored
      in the index (out of scope, listed as a follow-up in Step 4).

- [ ] Step 4: Measure the pass-2 cost and bring the docs in line
  - Done when:
    - `docs/humans/performance.md` "Focus candidate pass" gains a paragraph:
      `riffle-cli candidates D:\photos\2026\2026-09-19 24` before (the
      commit before Step 3) and after, alternated, four runs each, warm
      cache, Windows 11; per-file mesh time; peak working set if it moved;
      the "Which pass carries which cost" table's closed-eyes row says the
      mesh (and the pose solve) now also runs in the second pass for the
      cue, while the judgment shown in the meta pane stays on demand; the
      app's `scan faces` line on that folder if the GUI is run (else say
      so). `performance.ja.md` in sync. The "Closed-eyes judgment on
      demand" section, which head-pose's Step 4 edited, is left as it is.
    - `docs/humans/usage.md` **Focus mark** bullet: the probability is
      measured over each eye's eyelid region from the face mesh, which eye
      counts (the sharper one, or the eye facing the camera when the head is
      turned more than the cut, per Step 2), small faces (under the floor)
      fall back to the window between the eyes; the training / held-out
      precision and coverage sentence carries Step 2's numbers.
      `usage.ja.md` in sync. `README.md` / `README.ja.md`: the Focus mark
      bullet's "combining their sharpness and edge width" clause mentions
      the eye regions if the wording changes meaning; otherwise untouched.
    - `CLAUDE.md` Layout: `src/candidate.rs` says the measures are taken
      over the mesh eye regions (window fallback) and how the eye is chosen;
      `src/eyes.rs` no longer says "not called by the scan"; the
      `run_faces_scan` sentence names the mesh.
    - `docs/agents/tract-onnx-inference.md`: a short Measured item on
      running the face mesh from the rayon pool (per-file time at 24
      threads, memory).
    - `todo.md`, face/eye section, next to the head-pose items: a checked
      item for this work with the plan folder in backticks; the existing
      "Mark closed eyes in the strip" item is **updated, not duplicated**:
      its "unless the judgment moves into the scan (~35 ms per face ...,
      too costly for the second pass)" clause now says the mesh already
      runs in the second pass for the cue (Step 3), so the remaining cost of
      a strip mark is storing the judgment, not the model run; and fully
      specified unchecked follow-ups: one item for storing the scan's
      mesh-derived EAR (as the open probability) and pose (yaw / pitch /
      roll) in the index (`SCHEMA_VERSION` bump, columns next to
      `eye_focus`, `FaceReady` / `Focus` fields, `FACES_VERSION` bump) so
      the strip can filter closed eyes and looking-away frames without the
      on-demand `eyes_of`, with the files (`crates/app/src/index.rs`,
      `crates/core/src/scan.rs`, `crates/core/src/candidate.rs`,
      `crates/app/ui/src/filter.ts`, `index.html`) and the open questions
      (the on-demand row and the stored value must agree: same crop, same
      floor; the thresholds for "looking away" are unlabeled); a polygon
      mask instead of the contour bounding box if the box lets hair / brow
      edges in; reusing the scan's mesh for `eyes_of` once the points or the
      EAR are stored; the reserved folders validation once labeled.
    - If Step 2 kept the current window: `performance.md` and `todo.md`
      record the measured variants and why none was adopted, the "Mark
      closed eyes" item is left as it is, and `CLAUDE.md` names the CLI's
      per-eye columns only.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 3 (or Step 2's keep decision) is merged, and that the
      head-pose plan's Step 4 and wrap-up have merged (they will long
      before; if not, rebase onto them and keep their `performance.md` /
      `todo.md` edits). A pre-change CLI can be built from a temporary
      `git worktree` under the scratchpad with `CARGO_TARGET_DIR` pointed at
      this worktree's `target` (the earlier plan's learnings). Grep
      `between the eyes`, `eye window`, `not called by the scan`, `Focus
      candidate pass`, `too costly for the second pass` across
      `README*.md`, `docs/humans`, `docs/agents`, `CLAUDE.md`, `todo.md`
      (not `docs/plans/_archived`).

## Decision

Step 2 ([fit.md](fit.md), [frozen.json](frozen.json)): **adopt the mesh eye
regions**.

The chosen variant is (b), the sharper eye: each eye is scored over its
eyelid contour bounding box grown by a margin of **0.5** times the box's
longer side, and the frame takes the higher of the two eyes' logits. The
margin and the floor were chosen on the training set alone. They are the
highest pooled training AUC of the 28 margin x floor cells. On held-out, the
pose rules (f) and (g) do not beat it at any yaw cut (best 0.796 against
0.800). So **no yaw cut**, and Step 3 does not use the pose.

| | train | held-out |
| --- | --- | --- |
| AUC (window: 0.852 / 0.754) | 0.882 | 0.800 |
| precision (window: 93.1% / 89.1%) | 93.9% | 88.6% |
| coverage (window: 91.4% / 95.3%) | 91.4% | 95.9% |

The adopt rule holds. The held-out AUC of 0.800 beats 0.754. Precision
(88.6%) is below 89.1%, but coverage (95.9%) is not below 95.3%, so the two
are not both below.

What Step 3 implements, which supersedes the wording in Step 3 where they
differ:

- `EYE_REGION_MARGIN` = 0.5 and `EYE_REGION_MIN` = 24 px, measured on the
  region's longer side.
- An eye counts when its contour region exists, has an edge width and
  reaches `EYE_REGION_MIN`. The frame's logit is the higher logit of the eyes
  that count.
- The window fallback is used only when **no** eye counts, not when either
  region is missing or small. This is the rule `fit.md` measured. It sends
  61% of the training frames and 57% of the held-out frames to the window.
- Mesh model: `c` = -8.158737592019197, `k1` = 1.9417143647766388,
  `k2` = -1.3161251379398846, threshold 0.8343419969086643. The fallback
  keeps the window coefficients and its 1.2194 state boundary. fit.md's
  pooled AUC already puts both paths on one threshold the option-A way, so
  Step 3 follows Trade-off option A.
- There is no `EYE_YAW_CUT`, and Step 3's yaw-cut tests (the eye the yaw sign
  names, a yaw at the cut) do not apply. The other tests stand.

Caveats behind the adopt (details in fit.md):

- Part of the gain comes from giving the larger-eyed frames a model of their
  own. A control that refits the window's features on the same frames reaches
  0.865 / 0.777, so the region itself adds about 0.02 AUC.
- On the frames that do not fall back, the region and the window rank about
  equally on training (0.951 against 0.949). On held-out the region ranks
  better (0.876 against 0.848), but over only 9 off frames.
- No frame whose face box is under 58 px reaches the floor. Skipping the mesh
  below `EYES_MIN_FACE` (60 px) would change 1 of 330 meshed frames and save
  about a quarter of the mesh runs. That is not measured as a variant and is
  left to Step 3 / Step 4 and the cost decision.

## Trade-offs and risks

- **Region shape (open; Step 2 measures, the caller may narrow it).** Option
  A, the axis-aligned bounding box of the 16 contour points with a margin:
  reuses `laplacian_variance` and `edge_width` unchanged, ~30 lines; a
  turned or rolled face lets some brow, nose bridge or temple into the box.
  Option B, a polygon mask inside the contour: cleaner region but
  `edge_width` walks along rows and columns and has no masked form, so the
  mask would apply to the Laplacian only or need a masked walk (~80 more
  lines). The plan does A and lists B as a follow-up unless A fails to beat
  the baseline in Step 2, in which case the caller decides whether to try B
  before giving up.
- **Which eye (open; Step 2 measures).** The requirement says the sharper
  eye. Fitting a "sharper eye" rule needs per-eye features carrying the
  frame's label, which is wrong for the far eye of an oblique face when only
  the near eye is in focus; worse, on a profile the mesh still places the
  hidden eye, over hair, cheek or background, whose edges can read sharp, so
  the max logit can pick a phantom eye. The head pose from the same points
  (free once the mesh runs) names the eye facing the camera: variants (f)
  (near eye by yaw above a cut) and (g) (drop the far eye above a cut) use
  it; the AF-nearest eye (c) is a pose-free alternative that ignores a
  mislocated AF point. All are measured; the plan recommends (f) or (g) if
  they beat (b) on the held-out set, else the sharper eye. Risks of the
  pose rule: the yaw sign is wrong on ~6% of turned faces (the labeled
  measurement), and ~0.7% of faces get no pose (then the no-pose behavior
  applies); near the cut a small yaw error flips the eye, so the cut should
  sit where few labeled frames are, which `fit.md` reports.
- **Two models, one threshold (decision needed in Step 3; recommended: A).**
  The index derives the state from one stored probability against
  `candidate_probability()`. With a mesh model and a window fallback that
  have different fitted thresholds, a fallback frame's probability would
  read back into the wrong state. Option A: keep one `CANDIDATE_LOGIT`
  (the mesh model's), and shift the fallback model's intercept by the
  difference between its old threshold and the new one, so its state
  boundary is unchanged while its displayed probability moves a little (a
  one-constant change, documented). Option B: store the state in its own
  column (`SCHEMA_VERSION` bump, `FaceReady` / `Focus` gain a field) so each
  path keeps its own threshold; more honest probabilities, a wider change.
  Option C: refit the fallback on the frames that fall back; too few frames
  (~10-15% of faces are under 60 px) to fit three coefficients.
- **Scan cost (user decision after Step 4's numbers).** The mesh adds ~35 ms
  CPU per faced file (the pose solve is negligible), roughly +30-40% on
  pass 2 of a 2134-file folder at 24 threads, more on a laptop with fewer
  cores, and the faces pass already runs at the lowest priority. The
  earlier survey rejected this for a +25% budget; the requirement asks to
  measure, not to cap. If the measured cost is unacceptable, options are:
  run the mesh only for faces above a larger floor (fewer, larger faces
  cost the same per face, so little saving), a smaller mesh model (face
  mesh v1 at 192 px, ~15 ms, less accurate, and no validated pose), or
  reverting to the window. The plan does not pre-decide.
- **Accuracy may not improve.** The held-out AUC of the current cue is
  0.754 on a small, label-noisy set (58 off frames of 400); a region change
  could land within noise. Step 2's adopt rule is explicit, and "keep
  current" is a valid outcome that still leaves the CLI diagnostic and the
  measurements.
- **Reserved folders are still unlabeled**, so the only independent check is
  the same 400 held-out frames the current model was already compared on;
  the comparison is fair but not fresh. Noted in `fit.md`.
- **The mesh on a non-upright crop.** `landmarks_of` crops the upright
  image; for orientation 6 / 8 previews this costs a full-image rotation per
  file unless the implementation rotates only the crop. Step 1 measures;
  either is correct. The pose must be solved on the upright points (as
  `judge_mesh` does), before the mapping to stored coordinates.
- **Memory on 24 threads.** The face mesh is 4.9 MB with per-call state;
  the whole-image YuNet search already raised the peak working set to ~646
  MB at 24 threads. Step 4 reports the new peak; if it is large, the
  fallback is to run the mesh on a smaller pool (a Trade-off for the
  caller, not planned).
- **Overlap with the head-pose plan.** Its Step 4 (docs / todo) is in
  review; this plan's Step 1-3 touch none of its files except additive
  constants in `eyes.rs`, and Step 4 assumes it has merged. The cost is a
  rebase, not a design conflict.
- **Storing the EAR and pose in the index is out of scope** here even
  though the scan now has them; recorded as a Step 4 todo so the strip
  filters can follow as their own plan.

## Progress

- (2026-10-08) Step 1 complete. Deviation (see learnings.md): the margin and `edge_width_rel` use the box's longer side, not `window.width`, so for an upright eye (wider than tall) `edge_width_rel` is a fraction of the eye width as the Done-when text says.
