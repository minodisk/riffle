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

# AF eye focus inside the eyelid contour mask

## Purpose

The AF eye in-focus probability (`crates/core/src/candidate.rs`) is measured
over each mesh eye's **bounding rectangle** grown by `EYE_REGION_MARGIN` =
0.5 of its longer side (`eye_region`), scored per eye with `MESH_LOGIT_*`,
the sharper eye counting, with the window between the eyes as the fallback
when no eye counts (`eye_window`, `LOGIT_*`). That was Option A of the
archived plan
[`../_archived/20261008-mesh-eye-focus/plan.md`](../_archived/20261008-mesh-eye-focus/plan.md)
("Region shape"); its `fit.md` chose margin 0.5 and the 24 px floor as the
highest training AUC of 28 margin x floor cells. A rectangle with a 0.5
margin is about four times the contour's area, so on a turned or rolled face
it lets the brow, the nose bridge, the temple and hair edges in, and those
edges can read sharp when the eye is not.

This plan measures Option B, the inside of the 16-point eyelid contour
polygon (optionally dilated), as the region: the Laplacian variance and the
mean edge width are taken only over the pixels inside the mask, the logistic
is refitted, and the mask replaces the rectangle **only if it beats it** on
the same held-out set under the same rules. The todo item "Mask each eye's
region with the eyelid contour polygon" in `todo.md` (face / eye section) is
closed either way.

### What is known

- Current scores (`riffle-cli candidates`, release, 24 threads; archived
  `learnings.md` Step 3): training 406 labeled faced frames AUC 0.882,
  precision 93.9%, coverage 91.4%; held-out 400 frames AUC 0.800, 88.6%,
  95.9%; window fallback on 249 / 406 and 227 / 400 frames. A control that
  refits the window's own features on the meshed frames reaches 0.865 /
  0.777, so the rectangle itself adds about 0.02 AUC. Among the meshed
  held-out frames only 9 are off, so differences of 0.01-0.02 AUC are within
  noise.
- Labeled sets (XMP `xmpDM:good`, Pick = in focus, Reject = off), under
  `D:\Photos\tests\`: training `2026-06-05-focus-sample`,
  `2026-07-31-focus-sample`, `2026-09-13-a-focus-sample`,
  `2026-09-19-focus-sample`, `2026-09-19-focus-sample-2`; held-out
  `2026-06-14-focus-sample`, `2026-07-18-focus-sample`,
  `2026-08-01-focus-sample`, `2026-08-22-focus-sample`. The reserved folders
  are still unlabeled.
- Saved data of the earlier fit in `D:\Photos\tests\2026-10-08-mesh-eye-focus\`:
  `training-dump.jsonl` / `heldout-dump.jsonl` (one JSON line per faced
  file from the scratch `riffle-cli meshdump`,
  [`../_archived/20261008-mesh-eye-focus/meshdump.patch`](../_archived/20261008-mesh-eye-focus/meshdump.patch)),
  `step3-training.txt` / `step3-heldout.txt` (the Step 3 `candidates`
  lines). The fit script is
  [`../_archived/20261008-mesh-eye-focus/fit.py`](../_archived/20261008-mesh-eye-focus/fit.py)
  (pure Python 3; maximum-likelihood logistic by Newton's method, no
  regularization; the mesh threshold set so the training coverage stays at
  310 of 339 picks; the fallback frames' window logit shifted by `mesh
  threshold - 1.2194` for the pooled AUC; ties 0.5). The method is in
  [`../_archived/20261008-mesh-eye-focus/fit.md`](../_archived/20261008-mesh-eye-focus/fit.md).
- Region sizes: the contour's own bounding box (margin 0) has a longer side
  of p10 / median / p90 = 5 / 11 / 18 px training, 4 / 11 / 24 held-out;
  with margin 0.5, 11 / 21 / 35 and 11 / 22 / 47. Margin 0 sent half the
  training eyes to "no edge width" (a box too thin for `edge_width`) and 99
  under 3 px. A polygon mask holds fewer pixels than its box, so the mask's
  dilation and floor are chosen by the fit, not reused.
- Coordinates: `candidate.rs` works on the luma of the stored (unrotated)
  preview; `eyes::mesh_of` returns the 478 points in stored coordinates
  (`Mesh { points: Vec<[f32; 2]>, pose }`). On a portrait preview
  (orientation 6 / 8) the eye stands on end, so every size rule uses the
  region's longer side (`docs/agents/tract-onnx-inference.md`, "Mesh eye
  regions are boxes in stored coordinates"). Contour indices:
  `eyes::LEFT_EYE_CONTOUR` / `RIGHT_EYE_CONTOUR` (16 each, image side).
- The two measures scan a `Window`: `sharpness::laplacian_variance(gray,
  width, window)` over the window interior (one-pixel border excluded),
  `candidate::edge_width(gray, width, window)` with the Sobel 90th-percentile
  threshold over the interior, seeds at the row / column local maxima and a
  monotonic walk bounded by the window. Neither has a masked form.
- Versioning: a change to the cue is a `FACES_VERSION` bump
  (`crates/app/src/index.rs`, now 6; `docs/agents/tauri-app.md` "The second
  pass has its own version"), not `EXTRACTOR_VERSION`. Both models share
  `CANDIDATE_LOGIT` = 1.2194 by shifting the mesh model's intercept
  (`MESH_LOGIT_INTERCEPT` = fit intercept + (1.2194 - fit threshold)); the
  index derives the state from the stored probability alone.
- Two parallel sessions touch nearby code: **dlg-mesh-face-gate** (skip the
  mesh for faces under 60 px, around `focus_cue_unless` / `scored_face` /
  `mesh_of`) and **dlg-store-eyes-pose** (store the EAR and pose in the
  index; `index.rs`, `candidate.rs`, `eyes_of`). At the 24 px floor the gate
  changes one of the 330 counting labeled frames (a 58.2 px training face),
  so the measurement can start before it merges, but the implementation
  must not.

## Steps

- [x] Step 1: Add the contour mask primitives and dump the masked eye features per file, without changing the cue
  - Done when:
    - `crates/core/src/candidate.rs` gains pure, mesh-free functions, unit
      tested on synthetic data, next to `eye_region` / `edge_width`:
      - `eye_mask(points, indices, width, height, dilation) -> Option<EyeMask>`
        (names free): the pixels inside the closed polygon of the 16 contour
        points (scanline even-odd fill at pixel centers), dilated by
        `dilation` times the contour's longer side (a morphological
        dilation of the raster, or an equivalent grow of the polygon away
        from its centroid; say which in the doc comment and keep it), clamped
        into the image. `EyeMask` holds the bounding `Window` and a row-major
        `Vec<bool>` (or bit set) of that window. `None` when an index is out
        of range, a point is non-finite, or fewer than 3 (a named minimum)
        pixels are inside.
      - `masked_laplacian_variance(gray, width, mask) -> f64`: the Laplacian
        over the masked pixels whose four neighbors lie in the image, the same
        integer formula as `sharpness::laplacian_variance`; 0.0 when no pixel
        qualifies.
      - `masked_edge_width(gray, width, mask) -> Option<f64>`: `edge_width`
        with the Sobel 90th percentile taken over the masked interior pixels
        and the seeds (local maxima) restricted to masked pixels; the walk
        stays bounded by the mask's bounding window as today (see
        Trade-offs). `None` when no seed qualifies.
      - A `masked_eye_measures(gray, width, mask) -> (lap, edge_width,
        edge_width_rel)` with `edge_width_rel` over the mask window's longer
        side, mirroring `eye_measures`.
    - `eye_focus`, `mesh_eye_measures`, `Cue`, `EyeFocus`, the constants and
      `FACES_VERSION` (6) are **unchanged**; the scan, the index and the app
      behave exactly as before.
    - A scratch `riffle-cli maskdump <dir>...` subcommand, written as
      `maskdump.patch` in this plan folder (like the archived
      `meshdump.patch`: applied on this step's commit, built in release, run,
      then reverted; **not** merged), prints one JSON line per faced file at
      full precision: the name, flag, state, p, face long side, AF point, the
      baseline window's lap and edge width, the pose, and per eye the current
      0.5-margin rectangle's window / lap / edge width (to reproduce the
      baseline exactly) and, for each dilation in `[0, 0.1, 0.25, 0.5]` (at
      least; 0.5 is the closest mask analogue of the current rectangle), the
      mask's bounding window, pixel count, masked lap, masked edge width and
      the rectangle edge width of the mask's bounding window (for the
      "mask for Laplacian only" variant).
    - The dump of the 5 training and 4 held-out folders is saved as
      `training-mask.jsonl` / `heldout-mask.jsonl` in
      `D:\Photos\tests\2026-10-08-mesh-eye-mask\` (not committed), together
      with the commit hash and whether the face gate was on main at the time.
    - `learnings.md` records: per dilation, the mask pixel count and bounding
      longer side p10 / median / p90 on both sets, how many eyes have no
      masked edge width, and the per-eye cost of the mask raster + masked
      measures (microseconds; a `std::time::Instant` around the calls in the
      dump, one thread) so Step 4 can state the pass-2 cost without a full
      timing.
    - Tests: polygon fill on a hand-placed quadrilateral and on a concave
      shape (pixel counts, a point inside and outside), dilation grows the
      count and never shrinks it, clamping at an image edge, `None` on a
      non-finite point / out-of-range index / tiny polygon; masked Laplacian
      equals `laplacian_variance` when the mask covers the whole window
      interior and the `ramp` images; masked edge width on the existing
      `two_steps` image with a mask that excludes one step (only the other
      step's width is measured); `mise run ci` passes.
  - Implementation approach:
    - Keep the primitives in `candidate.rs` next to `eye_region` /
      `eye_measures` (the archived plan's layout); do not touch
      `sharpness.rs`'s `laplacian_variance` or `edge_width`, nor
      `eyes.rs` (`mesh_of`, `judge_mesh`, the EAR), `pose.rs` or the crop.
    - Base the dump on the archived `meshdump.patch`; the baseline columns
      must reproduce `fit.md`'s 0.882 / 0.800 when fed to the fit in Step 2.
    - If dlg-mesh-face-gate has merged before the dump runs, rebase onto it
      first; otherwise record that the dump predates it (one training frame
      may differ; Step 2 reports the delta).

- [x] Step 2: Refit the logistic on the masked features and decide whether the mask replaces the rectangle
  - Done when:
    - `fit.py` in this plan folder (copied from the archived one, extended
      with a loader for the mask dump; pure Python 3, not part of the
      workspace), `fit-out.txt` (its full output), and `fit.md` with, on the
      406 training frames and the 400 held-out frames under the archived
      method (same pooling, same fallback rule "window only when no eye
      counts", threshold keeping 310 / 339 training picks, held-out at the
      frozen threshold, sharper eye = higher per-eye logit, both eyes
      carrying the frame's label):
      - (a) baseline: the current rectangle (margin 0.5, floor 24) from the
        dump's rectangle columns, which **must reproduce 0.882 / 93.9% /
        91.4% and 0.800 / 88.6% / 95.9%**;
      - (b) mask for both measures (masked lap + masked edge width), at
        each dilation of the dump x floors `[0, 8, 12, 16, 24]` on the mask
        window's longer side (and, if the pixel count ranks better, a
        pixel-count floor; report which floor rule is used);
      - (c) mask for the Laplacian only (masked lap + the bounding window's
        rectangle edge width), same grid;
      - the control row (window features refitted on the same meshed frames)
        and the "meshed AUC mask / rectangle" columns on the frames that did
        not fall back, as in the archived `fit.md`.
      - A table of the cells, the fallback counts, and the selected cell
        (highest pooled training AUC; ties to the lower floor then the
        smaller dilation, as before). If the face gate merged between the
        dumps, a one-line delta for the affected frame(s).
    - `frozen.json` with the selected variant's coefficients, threshold,
      dilation, floor and measure rule at full precision, and the train /
      held-out numbers, in the archived file's shape plus `"adopt"`.
    - A **Decision** section in this plan.md applying the rule: **adopt only
      if the selected cell's held-out AUC exceeds 0.800 AND its held-out
      precision is not below 88.6% AND its coverage is not below 95.9%**.
      Record the numbers and, if not adopted, the reason (which condition
      failed, by how much) and the caveat on the 9 held-out off frames. If
      adopted, state what Step 3 implements (dilation, floor, which measure
      is masked, the intercept shift onto `CANDIDATE_LOGIT`), superseding
      Step 3's wording where they differ. If not adopted, Step 3 is skipped
      (marked so in Progress) and Step 4 records the finding.
    - `mise run ci` passes (lint, lychee on the new files).
  - Implementation approach:
    - Assumes Step 1 is merged and its dumps exist. No new fitting method:
      the comparison must stay comparable with the archived numbers.
    - Files: this folder's `fit.py`, `fit.md`, `fit-out.txt`, `frozen.json`,
      `plan.md` (Decision).

- [ ] Step 3 (skipped, mask not adopted; conditional, gated): Score the AF eye inside the mask in the scan
  - **Gate:** start only after Step 2's Decision is "adopt" **and**
    dlg-mesh-face-gate is on `main` (check `git log origin/main` for its
    merge; if dlg-store-eyes-pose has merged too, take it in). Rebase onto
    `main` first and adapt to any change in `scored_face` /
    `focus_cue_unless` / `EyeMeasures`.
  - Done when:
    - `candidate.rs`: `EYE_MASK_DILATION` (and the floor constant, renamed
      or re-documented if its meaning moved to the mask window / pixel
      count) and the refitted `MESH_LOGIT_*` from `frozen.json`, the
      intercept shifted onto `CANDIDATE_LOGIT` the archived way, documented
      with the training / held-out numbers; `mesh_eye_measures` measures each
      eye's contour over its mask (`RegionMeasures` gains the mask window and
      pixel count, or a small `EyeMask` summary); `eye_focus`'s eye rule
      (sharper eye, window fallback only when no eye counts) is unchanged;
      the old rectangle path (`eye_region` + `eye_measures`) is removed if
      nothing else uses it (the iris regions still use `eye_region`; keep
      what is used, remove what is not).
    - `crates/app/src/index.rs`: `FACES_VERSION` 7 with its history line
      ("re-runs it after the cue moved inside the eyelid contour mask"); no
      schema change.
    - `crates/cli/src/main.rs`: `mesh_columns` prints the mask's bounding
      window (and pixel count if cheap) in place of the rectangle; the doc
      comment's field count is updated; `riffle-cli candidates` on the
      training and held-out folders reproduces `fit.md`'s selected numbers
      within rounding (recorded in `learnings.md`, the lines saved as
      `step3-training.txt` / `step3-heldout.txt` in the data folder).
    - `docs/agents/tauri-app.md`: the `FACES_VERSION` bullet's list names
      the mask.
    - Tests: the frozen-coefficients test updated for the new constants
      (`assert_eq!` on the shifted intercept literal, as before); the
      existing eye-rule and fallback tests (`the_sharper_eye_scores_the_frame_whatever_the_pose`,
      `an_eye_counts_from_the_minimum_region_side`,
      `an_eye_that_does_not_count_leaves_the_other`,
      `the_window_scores_a_face_with_no_eye_that_counts`) pass on masked
      synthetic eyes; a test that a sharp edge just outside the contour but
      inside its bounding rectangle does not raise the eye's logit; the
      cancel tests unchanged. `mise run ci` passes.
  - Implementation approach:
    - Do not touch `eyes_of` / `read_eyes`, `judge_mesh`, the EAR constants,
      `pose.rs` or the face crop. The on-demand focus-mark drawing
      (`facemesh.ts`) is unaffected: it draws the contour, not the region.

- [x] Step 4: Bring the docs and the todo in line with the outcome
  - Done when (if adopted):
    - `docs/humans/usage.md` **Focus mark** bullet says the probability is
      measured inside each eye's eyelid contour (dilated by the chosen
      amount) rather than "over each eye's eyelid region"; the 24 px / floor
      wording and the precision / coverage sentence carry Step 2's numbers.
      `usage.ja.md` in sync. `README.md` / `README.ja.md` only if the Focus
      mark bullet's wording changes meaning.
    - `docs/humans/performance.md` "Focus candidate pass": one paragraph
      with the per-eye cost of the mask from Step 1's `learnings.md` and one
      before / after `riffle-cli candidates D:\photos\2026\2026-09-19 24`
      pair (the `... total` line, warm cache, two runs each is enough: the
      mask raster is microseconds against the ~100 ms mesh) and the
      candidate count change on that folder; `performance.ja.md` in sync.
    - `CLAUDE.md` Layout: the `src/candidate.rs` sentence says the measures
      are taken inside each eye's eyelid contour mask from the face mesh.
    - `docs/agents/tract-onnx-inference.md`: the "Mesh eye regions are boxes
      in stored coordinates" bullet says the region is now a mask whose size
      rules still use its bounding window's longer side.
    - `todo.md`: the "Mask each eye's region with the eyelid contour polygon"
      item is checked with a one-line result and the plan folder in
      backticks; the face-gate item's numbers are left as they are unless
      the floor moved.
  - Done when (if not adopted):
    - `todo.md`: the item is checked with the measured best cell's numbers
      and the failed condition, the plan folder in backticks; no new item
      unless Step 2 names a concrete follow-up (e.g. re-run on the reserved
      folders once labeled, which the existing item already covers: extend
      it, do not duplicate).
    - `docs/humans/performance.md` "Focus candidate pass" (and `.ja.md`): one
      sentence that the polygon mask was measured and did not beat the
      rectangle, pointing at this plan folder.
    - The Step 1 mask primitives are removed from `candidate.rs` with their
      tests (they have no caller; the patch and the fit in this folder keep
      the measurement reproducible).
    - In both cases `mise run ci` passes.
  - Implementation approach:
    - Grep `eyelid region`, `bounding box`, `margin 0.5`, `EYE_REGION_MARGIN`
      across `README*.md`, `docs/humans`, `docs/agents`, `CLAUDE.md`,
      `todo.md` (not `docs/plans/_archived`). Keep the "Closed-eyes judgment
      on demand" and eye-state survey sections untouched.

## Decision

**Not adopted: the rectangle stays.** Details in [`fit.md`](fit.md), output in
[`fit-out.txt`](fit-out.txt), the selected cell in [`frozen.json`](frozen.json)
(`"adopt": false`).

- Baseline reproduced: the archived dump refits to the archived coefficients
  bit for bit, 0.882 / 93.9% / 91.4% training and 0.800 / 88.6% / 95.9%
  held-out. With the face gate on (the mask dump), one training reject
  (`_DSC2748.ARW`, 58.2 px face) moves to the window fallback and stays off;
  the post-gate baseline is 0.882 / 93.9% / 91.4% and 0.80026 / 328 of 370 /
  328 of 342 held-out, the same at the rule's precision.
- Selected cell (highest pooled training AUC over 110 cells: (b) both measures
  masked and (c) the Laplacian only, dilations 0-1.0, longer-side floors 0-24
  px and pixel-count floors 0-400; the pixel-count rule ranked higher): **(b),
  dilation 0.1, a 50-pixel floor**, training 0.889 / 93.7% / 91.4%, held-out
  **0.797 / 88.6% (325 / 367) / 95.0% (325 / 342)**.
- The rule fails on all three conditions, against both the post-gate baseline
  and the archived numbers (the same outcome, so no stricter one to choose):
  AUC 0.797 is not above 0.800 (-0.003); precision 88.56% is below 88.65%
  (328 / 370); coverage 95.0% is below 95.9% (3 fewer in-focus frames found).
- With the longer-side floor rule instead, the best cell (dilation 0.25, 16
  px) reaches held-out AUC 0.806 but 88.59% precision and 95.3% coverage, so it
  fails too; post hoc, no cell of the 110 passes all three on held-out.
- Caveat: the held-out set's meshed frames hold only 9-11 off frames, so the
  AUC differences of 0.003-0.007 around 0.800 are within noise; the mask did
  rank the training set's meshed frames better (0.974 against the window's
  0.951, where the rectangle reached 0.948), which did not carry over. Re-validating on the reserved folders once labeled is the
  existing todo item's follow-up.
- So Step 3 is skipped (marked so in Progress), and Step 4 takes its
  "if not adopted" branch.

## Trade-offs and risks

- **Where the mask primitives live if not adopted (decided: commit them).**
  They are committed in Step 1 with tests so the measurement is a reviewable
  PR and reproducible, and removed in Step 4 if Step 2 says no. The
  alternative (keep them only inside `maskdump.patch` until Step 3) was
  declined at plan approval.
- **How the edge width is masked.** Default: the seeds (local maxima) are
  restricted to masked pixels and the threshold is the 90th percentile over
  masked pixels, but the monotonic walk stays bounded by the mask's bounding
  window, so a width means the same thing as today and only which edges
  count changes. Alternative: clip the walk at the mask boundary; on an
  11 px eye this truncates widths and biases them down, which the logistic
  cannot tell from sharpness. Step 2 adds the clipped walk as a variant only
  if the dump carries it cheaply; otherwise it is out.
- **Dilation unit.** A fraction of the contour's longer side (planned),
  consistent with `EYE_REGION_MARGIN` and making dilation 0.5 the mask
  analogue of the current rectangle, so the shape effect is isolated at
  equal extent. Whole-pixel dilation would be simpler to rasterize but scale
  differently on the 5 px and 57 px eyes the sets contain.
- **Floor rule.** The bounding window's longer side (as today) or the mask's
  pixel count. Both are in the dump; Step 2 reports which ranks better and
  Step 3 implements that one. Using the longer side keeps the
  `tract-onnx-inference.md` rule intact.
- **The adoption rule is strict and the evidence thin.** Only 9 held-out off
  frames are meshed, so the mask may win or lose by noise. The requirement's
  rule (AUC > 0.800 and precision >= 88.6% and coverage >= 95.9%) is applied
  as written; the Decision records the margin so a marginal adopt is not
  oversold, and the existing todo item on re-validating with the reserved
  folders once labeled covers the follow-up.
- **Parallel sessions.** dlg-mesh-face-gate changes which faces get a mesh
  (one counting labeled frame); dlg-store-eyes-pose may change
  `EyeMeasures` / `scored_face`. Step 1 and 2 may run before either merges,
  with any delta reported; Step 3 is gated on the face gate being on `main`
  and rebases onto whatever else landed. The cost is a rebase, not a design
  conflict, as long as Step 3 does not restructure `focus_cue_unless`.
- **Dead code if iris regions keep `eye_region`.** The iris windows are
  measured and printed by the CLI but not scored. If Step 3 removes
  `eye_region`'s only scoring use, leave the iris path as it is (it is
  pre-existing, not this plan's orphan) and note it in `learnings.md`.

## Progress

- 2026-10-09 Step 1 done: the eyelid contour mask primitives and their tests
  landed in `crates/core/src/candidate.rs`. The mask dump ran on this commit
  with the face gate already on `main`; its data is in
  `D:\Photos\tests\2026-10-08-mesh-eye-mask\`. See `learnings.md` for the size
  and cost tables.
- 2026-10-09 Step 2 done: the eyelid contour mask refit ran on the
  training and held-out sets; the mask is not adopted. The selected cell
  (dilation 0.1, mask pixel-count floor 50, masked Laplacian plus edge width)
  reaches training AUC 0.889 against the baseline's 0.882 but held-out AUC
  0.797 against 0.800, with precision 88.56% and coverage 95.0% failing the
  rule. See `fit.md` and `frozen.json` (`adopt: false`). Step 3 is skipped
  and Step 4 takes its "if not adopted" branch.
- (2026-10-09) Step 3 skipped (mask not adopted)
- (2026-10-09) Step 4 complete
