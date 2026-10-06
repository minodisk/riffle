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

# Face detection: small-face recall and a cheaper detector decode

## Purpose

Two open items of `todo.md` "App: face/eye-aware focus check for culling"
concern the YuNet detector in `crates/core/src/faces.rs`:

1. Recall on small faces is poor: on 36 sampled Leica M11-P DNGs the detector
   found no face in 29, and only 2 of the 7 people in the group frame
   `L1005161.DNG` (`D:\photos\2026\2026-02-01`, 146 DNGs). The cause is the
   geometry of the whole-image path: a 2112 px preview is shrunk to the 320 px
   model input (6.6x), so a face below roughly 70-80 preview px falls under
   YuNet's stride-8 minimum.
2. Detection decodes the full-size RGB preview (`decode::decode_rgb`) and
   then box-averages it down to 320 px; a DCT-scaled mozjpeg decode
   (`Decompress::scale`, as `decode::thumbnail_jpeg_near` already does) would
   skip most of that work.

The two paths of the detector matter for both items (see
[`../../agents/tract-onnx-inference.md`](../../agents/tract-onnx-inference.md)
and `crates/core/src/scan.rs` `extract_analysis_unless`):

- **Cue path** (trusted AF point, `candidate::focus_cue_unless`): one full
  RGB decode, luma for the eye window taken from it (the frozen logistic
  model `LOGIT_*` / `CANDIDATE_LOGIT` was fitted on this native-resolution
  luma), YuNet on a `CATCH_CROP` 480 px native crop shrunk to 320. Faces here
  are 45-60 px or larger by design, so recall is not the problem, and the
  full decode is needed for the luma anyway.
- **Whole-image path** (no trusted AF point: every Leica M file, Sony manual
  focus; `scan::whole_image_faces`, and `faces_of` for the `f` mark): the
  whole upright preview goes to the 320 input. It feeds only the sharpness
  score's eye window (`sharpness::chosen_face`, `FACE_CONFIDENCE` 0.8) and the
  on-demand face boxes, never the focus candidate cue, so the labeled-folder
  AUC cannot move through it. `score_preview` then decodes the preview a
  second time, in grayscale.

Once done, Leica (and other no-AF-point) frames with several or distant
people get their sharpness window on an eye instead of the sharpest tile and
their `f` mark shows the faces, and the second pass spends less time per
file on the decode. The third item, closed-eye detection from the landmarks,
is settled by this plan as not feasible with the current model (YuNet emits
eyes, nose and two mouth corners, no eyelid points) and the todo item is
rewritten and kept open rather than implemented.

The measurements rely on these existing tools: `riffle-cli faces <file>
<out.png>` (boxes drawn on the upright preview; the PNG can be inspected to
count faces by eye), `riffle-cli bench <files>` (row `4. face detection`),
`riffle-cli scan <dir> [threads]` and `riffle-cli candidates <dir>...
[threads]` (per-file cue, AUC, precision, coverage over XMP-labeled folders).
The labeled folders are the nine `D:\Photos\tests\*-focus-sample*` folders
with 100 XMP sidecars each (training set `2026-06-05`, `2026-07-31`,
`2026-09-13-a`, `2026-09-19`, `2026-09-19-focus-sample-2`; held-out
`2026-06-14`, `2026-07-18`, `2026-08-01`, `2026-08-22`), with the reference
numbers in
[`../_archived/20260926-af-eye-in-focus-probability/learnings.md`](../_archived/20260926-af-eye-in-focus-probability/learnings.md)
(training AUC lap 0.816 / combined 0.852, candidates 333, in focus 310,
coverage 91.4%; held-out AUC 0.635 / 0.754, 366 / 326, 89.1% / 95.3%). The
two reserved folders `2026-08-29-focus-sample` and `2026-09-13-b-focus-sample`
still carry no XMP labels (checked 2026-10-05), so they cannot serve as the
check; that todo item stays open.

## User decisions (2026-10-05)

- Cost budget: after Step 3, the whole-image path's per-file detection cost
  (decode + detection) is at most about 2x today's (~16 ms detection on the
  WSL2 numbers; re-measure the baseline on this machine in Step 1). Step 2
  alone may exceed it temporarily.
- `FACES_VERSION` is bumped per PR (Step 2 -> 4, Step 3 -> 5), as the existing
  policy says.
- Closed eyes (item 3): not implemented; the todo item is rewritten to say a
  second model is needed, and kept open.
- Cue-path decode in Step 3: measure the grayscale-full + scaled-RGB-crop
  combination and adopt it only if it is faster and the nine labeled folders'
  per-file states and AUC / precision / coverage are identical; otherwise
  record the measurement and keep the single full RGB decode.
- Recall metric: the agent-counted ground truth on the 36 M11-P DNGs plus
  `L1005161.DNG` (`faces-truth.md` in this folder), no separate user review.

## Steps

- [x] Step 1: Add a folder-level detection report to `riffle-cli` and record the baseline (face counts, ground truth, timings)
  - Done when:
    - `riffle-cli detect <dir|file>... [threads]` runs the same detection the
      second pass runs (`faces::detect_around` with
      `sharpness::trusted_focus`, so a file takes the path the scan gives it)
      on every RAW file given and prints per file the path taken (`crop` /
      `whole`), the face count with each face's box side and score, and the
      decode and detection time; then totals: files, files with at least one
      face, faces, mean / median / p95 of the per-file detection time and of
      the decode time. Unit-tested where the logic is pure (the summary
      stats; the per-file formatting), with the real-file run `#[ignore]`d
      or manual as `riffle-cli` commands are today.
    - A hand-made ground truth `faces-truth.md` in this plan folder lists the
      36 M11-P sample DNGs (the files sampled evenly from
      `D:\photos\2026\2026-02-01`, the way the 2026-09-22 measurement sampled
      them, or the same 36 if they can be identified from the archived
      learnings) plus `L1005161.DNG`, each with the number of visible faces
      counted by eye on the `riffle-cli faces` PNG (reading the PNG with the
      image viewer the agent has), and the approximate face side in preview
      pixels of the smallest face that should count. It also notes how many
      of the 29 "no face" files truly contain no face, which the todo figure
      does not tell apart.
    - Baseline numbers on `main` are recorded in `learnings.md`: the `detect`
      totals on the 36 + 1 DNGs and on about 40 α7 V ARWs taking the crop
      path, recall against the ground truth (files with a face found / files
      with a face; people found in `L1005161.DNG`), `riffle-cli scan` and
      `riffle-cli candidates` wall and per-file times on the 146-DNG folder
      and the 2134-ARW folder, and `riffle-cli candidates` on the five
      training and four held-out folders, matching the reference numbers
      above. Record the machine and OS (the earlier tables are Linux WSL2;
      this one is the Windows 11 machine), release build, warm cache, runs
      alternated as the existing tables do.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/cli/src/main.rs`: add the subcommand next to `faces` /
      `candidates`; reuse `stats` for the timing rows and `is_raw_file` for
      the listing; print one line per file so the output can be diffed between
      runs. Do not change `riffle-core` in this step beyond what the command
      needs (ideally nothing).
    - Do not change detection behaviour in this step: this PR is the "before"
      of Steps 2 and 3.
    - The ground truth is a plan-folder file, not a repository fixture: the
      DNGs stay on disk per the memory rule (samples live in
      `D:\photos\...`), and the file lists names, counts and the smallest
      face side only.

- [x] Step 2: Raise small-face recall on the whole-image path and keep the cue path unchanged
  - Done when:
    - On the ground truth of Step 1, the whole-image path finds a face in
      more of the files that contain one and finds at least 5 of the 7 people
      in `L1005161.DNG` (or the plan records why fewer is the ceiling, e.g.
      occluded or turned heads, with the PNG evidence described), with no new
      false face in the files that contain none at the sharpness score's
      `FACE_CONFIDENCE` 0.8.
    - The per-file detection time on the whole-image path is measured before
      and after with `riffle-cli detect` and `riffle-cli scan` /
      `candidates` on the 146-DNG folder. The chosen method must be able to
      meet the user's budget (at most about 2x today's per-file cost once
      Step 3 recovers the decode); Step 2 alone may exceed it, and the
      expected post-Step-3 figure is recorded.
    - `riffle-cli candidates` on the training and held-out folders prints the
      same per-file states and the same AUC / precision / coverage as the
      Step 1 baseline (the crop path is untouched, so the output should be
      identical, not merely close), and the 2134-ARW `candidates` time does
      not rise.
    - `FACES_VERSION` in `crates/app/src/index.rs` is bumped (to 4) with its
      doc comment extended, because the sharpness score of no-AF files and
      the faces `faces_of` draws change; the migration / re-run tests still
      pass. `EXTRACTOR_VERSION` stays: pass 1 does not detect.
    - Tests: a unit test for whatever new pure logic lands (tile placement
      and merge, or the per-size plan selection and coordinate mapping); the
      `#[ignore]`d real-image test keeps working.
    - `docs/humans/performance.md` "Face detection cost" gets the before /
      after recall and timing tables and the paragraph on recall is updated;
      `docs/humans/performance.ja.md` "顔検出のコスト" is updated in the same
      PR. `todo.md`: the "Improve small-face recall" item is checked with the
      measured numbers.
  - Implementation approach:
    - Decide between, measuring each on the ground truth and timing it:
      (a) a larger model input for the whole-image path only (e.g. 640, the
      size the ONNX file is annotated for), as a second `OnceLock` plan keyed
      by input size, `detect` taking the size and `decode_stride` using
      `size / stride` cells (its grid is already parameterized by `cols`);
      (b) tiling the upright preview into overlapping 320-input tiles (e.g.
      2x2 with an overlap of at least the largest expected face) and merging
      with the existing `nms`; (c) lowering `SCORE_THRESHOLD` for this path
      only. Try (c) first since it is free; it most likely does not reach the
      stride-8 limit, in which case record that and move on. (a) and (b)
      cost about 4x the inference; (a) is simpler (no border handling) and
      one tract run, (b) keeps one plan. Prefer (a) unless its tract build or
      memory cost is bad on this machine, or an intermediate size (e.g. 480)
      reaches the recall target closer to the budget.
    - Keep the cue path at `INPUT` 320 on the 480 crop: the frozen model in
      `candidate.rs` was validated with it, and the plan constraint is no
      change in judgment accuracy. Any threshold change must not touch
      `CATCH_CONFIDENCE` / the crop path.
    - Files: `crates/core/src/faces.rs` (the plan(s), `detect`,
      `detect_around_rgb`), `crates/core/src/scan.rs` (`whole_image_faces`
      only if the signature changes), `crates/app/src/index.rs`
      (`FACES_VERSION`), `crates/cli/src/main.rs` (`bench` row 4 should time
      the path the scan takes, or gain a second row), `docs/agents/tract-onnx-inference.md`
      (add the second plan / tiling rule if one lands),
      `docs/humans/performance.md`, `docs/humans/performance.ja.md`,
      `todo.md`.
    - Measure on the full RGB decode (as today) so the recall change is
      isolated from the decode-resolution change of Step 3.
    - Check the peak memory with 24 threads on the 2134-ARW folder once (a
      640 plan or a tiling buffer adds memory per rayon worker).

- [ ] Step 3: Decode the preview for detection at a DCT-scaled size
  - Done when:
    - On the whole-image path the preview is decoded with
      `mozjpeg::Decompress::scale` at the smallest `n/8` whose long edge is
      not below the model input Step 2 chose (the rule of
      `decode::thumbnail_jpeg_near`), rotated upright at that size, detected,
      and the faces are mapped back to full preview pixel coordinates (the
      `Face` values the sharpness eye window and `faces_of` consume are still
      in stored preview pixels). Full-size RGB is no longer decoded on this
      path.
    - The whole-image path's per-file decode + detection cost is at most
      about 2x the Step 1 baseline (the user's budget), or the step records
      why not and what would close the gap.
    - On the cue path the step measures whether a cheaper combination
      (grayscale full decode for the luma + DCT-scaled RGB decode for the
      crop, against today's single full RGB decode) is faster per file on the
      2134-ARW folder, and adopts it only if it is faster and the nine
      labeled folders' per-file states and AUC / precision / coverage are
      identical; otherwise it records the measurement and the single full RGB
      decode with `candidate::luma` stays.
    - Recall on the Step 1 ground truth does not drop from Step 2's numbers
      (same files with a face found, same count in `L1005161.DNG`), and the
      per-file time on the whole-image path and the `scan` / `candidates`
      wall time on the 146-DNG and 2134-ARW folders are recorded before /
      after.
    - `riffle-cli candidates` on the training and held-out folders prints the
      same states and AUC / precision / coverage as before this step.
    - `FACES_VERSION` is bumped (to 5) since the pixels fed to the model on
      the whole-image path change; its doc comment says so.
    - Tests: `decode.rs` unit tests for the new scaled-decode helper (scale
      choice per source size, output dimensions, orientation mapping with the
      synthetic gradient JPEG the file already builds); `faces.rs` test that
      a face found on a scaled image maps back to stored coordinates within
      the scale's rounding.
    - `docs/humans/performance.md` "Face detection cost" and "Which pass
      carries which cost" are updated (the "Whole-preview face search" row and
      the decode note), `docs/humans/performance.ja.md` in sync; `todo.md`
      "Decode the preview for detection at a DCT-scaled size" is checked with
      the numbers.
  - Implementation approach:
    - Assumes Step 2 is merged (the target size is the one it chose).
    - Put the scaled decode in `crates/core/src/decode.rs` next to
      `thumbnail_jpeg_near` / `preview_jpeg` (e.g. a `decode_rgb_near(jpeg,
      long_edge) -> (rgb, w, h, scale_den)` returning the DCT scale, wrapped in
      the same `catch_unwind` as `decode_rgb`), and keep the coordinate
      mapping in `faces.rs` (`detect_around` is "the one place that decides
      which region is searched"; it becomes the place that decides the decode
      size too). The stored `Detection::width` / `height` must keep reporting
      the full preview size: `faces_of` and the UI scale boxes by it.
    - mozjpeg's `n/8` scaled output dimensions are `ceil(dim * n / 8)`, so map
      coordinates with the actual decoded size against the header size, not
      with `n / 8`.
    - mozjpeg's grayscale output is the JPEG Y channel, not bit-identical to
      `candidate::luma`'s integer BT.601 on the RGB, so the cue-path variant
      will most likely not be identical; measure it anyway and record.
    - Out of scope: the grayscale decode `sharpness::score` does on its own
      (a third decode on the no-AF path). Note it in `learnings.md` as a
      follow-up if the numbers show it matters.

- [ ] Step 4: Rewrite the closed-eyes todo item and settle the section
  - Done when:
    - `todo.md` "App: face/eye-aware focus check for culling": the
      "Optionally, detect closed eyes from the landmarks" item is rewritten
      (and kept open, per the user's decision) to state that YuNet's five
      landmarks (two eyes, nose, two mouth corners) carry no eyelid
      information, so closed eyes need a second model (an eye-crop open /
      closed classifier, or a dense landmark model with eye contours) and the
      added binary / cost it brings. The section's intro paragraph is updated
      to reflect Steps 2 and 3 and links this plan's archived folder.
    - `docs/humans/performance.md` "Face detection cost" reads as one
      consistent section after Steps 2 and 3 (the recall paragraph, the
      tables, the "measured before the move" caveats), `performance.ja.md`
      in sync.
    - `mise run ci` (lint, lychee) passes.
  - Implementation approach:
    - Docs only. If Steps 2 and 3 already left the section consistent, this
      step is just the todo rewrite and can be folded into Step 3's PR; say
      so in Progress.

## Trade-offs and risks

- **Order of items 1 and 2.** Recall first, then the scaled decode: the DCT
  scale depends on the model input size Step 2 picks (a 2/8 decode of a 2112
  px preview is 528 px, enough for 320 but not for 640, which needs 3/8), and
  measuring recall on the full decode isolates the two effects. Step 2 alone
  raises the whole-image path's time (roughly 4x the inference for a 640
  input or 2x2 tiles) until Step 3 lands.
- **Which recall method.** (a) 640 input: one tract run, a second optimized
  plan in memory, simplest coordinate mapping. (b) 2x2 tiling at 320: one
  plan, four runs, faces on tile borders need overlap and the NMS merge.
  (c) threshold lowering: free, but cannot recover faces below the stride-8
  minimum; the sharpness score filters at 0.8 anyway. The step picks on
  measured recall and time; (a) is the default.
- **The 29-of-36 figure has no denominator.** Many sampled M11-P frames may
  contain no face; the recall claim is only meaningful against Step 1's
  hand count. If the count shows most of the 29 have no face, the todo item
  is rewritten to the measured recall rather than "29 of 36".
- **Judgment accuracy check.** The cue path is left untouched in Step 2 by
  design, so `riffle-cli candidates` on the nine labeled folders is expected
  to be identical; a difference is a bug. The two reserved folders are still
  unlabeled, so the open "validate on the reserved folders" todo item is not
  closed by this plan.
- **`FACES_VERSION` bumps.** Two bumps (Steps 2 and 3) mean two full pass-2
  re-runs (~9 s per 2000 files) for users who update between them; accepted.
- **Measurement platform.** Earlier tables are Linux WSL2; this plan measures
  on the Windows 11 machine (`cargo` is available through mise). Record the
  platform with every table and compare before / after on the same machine.

## Progress

- (2026-10-06) Step 1 complete
