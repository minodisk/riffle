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

# Closed-eyes detection on the faces YuNet finds

## Purpose

A blink is one of the commonest reasons to reject a frame, and Riffle cannot
see it: YuNet (`crates/core/src/faces.rs`) emits five landmarks (two eye
centers, nose, two mouth corners) and none on the eyelids, so an open and a
closed eye land on the same point (settled in
`docs/plans/_archived/20261005-face-detection-recall-cost/`, Step 4, and the
open `todo.md` item "Optionally, detect closed eyes" under "App: face/eye-aware
focus check for culling"). Telling the user that the eyes of the subject are
closed needs a second model run on each face YuNet already finds, either an
open / closed classifier on an eye crop or a dense landmark model whose eyelid
points give an eye aspect ratio (EAR).

That second model costs an ONNX file and its license in
`crates/core/models/`, binary size, and per-face inference in the second pass
(`run_faces_scan` in `crates/app/src/index.rs`, through
`scan::extract_analysis_unless`). The todo item says to pick a model only
after measuring its size and per-face cost against the pass-2 budget, so this
plan's first step is that measurement, with a hand-labeled open / closed set,
and every later step is conditional on its outcome. If no candidate fits, the
plan ends after Step 1 with the numbers recorded and the todo item rewritten.

Once adopted, the second pass stores a per-file closed-eyes probability next
to `eye_focus` and `sharpness`, the meta pane's Analysis group shows it, and
the filter menu can narrow the strip to frames whose subject has closed eyes.

### Where the model would run (current state)

- **Cue path** (trusted AF point; every Sony frame with AF):
  `candidate::focus_cue_unless` decodes the full-size RGB preview once (ARW
  1616x1080, DNG 2112x1408), runs YuNet on a `CATCH_CROP` 480 px crop and
  scores the eyes of the face nearest the AF point (`nearest_face`). The
  full-size upright RGB and the chosen `Face` (with `left_eye` /
  `right_eye`) are in hand, so an eye or face crop costs no extra decode.
  Baseline on the Windows 11 machine (`riffle-cli detect`, one thread): decode
  ~10 ms + detection ~24 ms per ARW; `riffle-cli candidates` on the 2134-ARW
  folder `D:\photos\2026\2026-09-19` 12-15 s at 24 threads; 1952 of 2134 files
  have a face, 3787 faces (~1.8 per file).
- **Whole-image path** (no AF point; Leica M, manual focus):
  `scan::whole_image_faces` decodes at a 3/8 DCT scale (792 px long edge) for
  the 640x448 search; there is no full-size RGB, and the faces only pick the
  sharpness window (`sharpness::chosen_face`, score >= 0.8). ~72 ms per DNG.
  A face of 100 preview px is ~37 px at 3/8, its eye ~8 px: an eye-state
  model would most likely need another decode here.
- What the index stores per file (`FACES_VERSION` 5, schema v16):
  `eye_focus`, `sharpness`, `faces_extractor`. `FaceReady` (the
  `faces-progress` payload) and `Focus` (`indexed_file`) carry `eye_focus` to
  the UI; `meta.ts` shows `AF eye in focus`; `filter.ts` + `index.html`
  (`data-candidate`) + `main.ts` form the `AF eye` filter section.

### Tools and samples

- `riffle-cli detect <dir|file>... [threads]` (per-file path, faces, times),
  `riffle-cli faces <file> <out.png>` (boxes drawn), `riffle-cli bench`,
  `riffle-cli candidates <dir>... [threads]` (per-file cue, AUC on the
  XMP-labeled focus folders). `crates/cli` already depends on `image` with
  PNG output.
- Sample RAWs: `D:\photos\samples\<FORMAT>\`; real folders
  `D:\photos\2026\2026-09-19` (2134 α7 V ARWs, AF point on every file) and
  `D:\photos\2026\2026-02-01` (146 M11-P DNGs, no AF point). The nine
  `D:\Photos\tests\*-focus-sample*` folders carry focus labels, not eye
  labels; `2026-08-29-focus-sample` and `2026-09-13-b-focus-sample` are
  unlabeled. Hand-check copies and exported crops go in
  `D:\Photos\tests\<date>-<topic>\`.
- Guide: [`../../agents/tract-onnx-inference.md`](../../agents/tract-onnx-inference.md)
  (model loading with `default-features = false`, outlet labels, `OnceLock`
  plans, BGR NCHW input conventions).

## Steps

- [x] Step 1: Survey at least two candidate eye-state models on a hand-labeled set and decide
  - Done when:
    - `riffle-cli eyecrops <dir|file>... <out-dir>` exists: it runs the
      scan's detection (`faces::detect_around` with
      `sharpness::trusted_focus`, so a file takes the path the scan gives it),
      and for every face writes PNG crops to `out-dir` (the face box with a
      margin, and each eye as a square derived from the inter-ocular
      distance, both from the full-size upright RGB) plus one index line per
      face (file, face number, box side in preview px, score, eye positions)
      in a text file that can be diffed. Pure helpers (crop geometry from a
      `Face`, the index line) are unit-tested; the real-file run is manual.
    - A hand-labeled set `eyes-truth.md` in this plan folder: at least ~150
      faces sampled from the 2134-ARW folder (and the 146-DNG folder if
      enough faces are found) with each face labeled `open` / `closed` /
      `unsure` per eye by reading the crops with the image viewer, including
      every closed-eyes face that can be found (blinks are rare, so sample
      with a bias toward them: bursts, the `Soft` candidates, and any folder
      the user names). It records how many faces were too small or too
      blurred to label, by face side in preview px: this is the floor of what
      any model can do on the embedded preview.
    - Each candidate model is checked for: license (must permit
      redistribution in a desktop app; Apache-2.0 / MIT / BSD acceptable, a
      non-commercial clause excludes it), ONNX file size, whether
      `tract-onnx` 0.23 with `default-features = false` loads and optimizes
      it (`into_optimized` with the input fact pinned) and where its outputs
      are found (outlet labels), per-face latency on the Windows 11 machine
      (one thread, release, mean / median / p95 over the labeled faces,
      including the crop and resize, as `detect` times its own pre-work),
      and accuracy on the labeled set (AUC of its closed probability or EAR,
      and accuracy / precision / recall of `closed` at the best threshold,
      per eye and per face as the max of the two eyes). At least two
      candidates, one of each kind, are measured end to end; others may be
      dropped early with the reason recorded (license, unsupported op, size).
      Candidates to start from (verify each during the survey, none is
      pre-approved): an eye-crop open / closed classifier (e.g. OpenVINO Open
      Model Zoo `open-closed-eye-0001`, Apache-2.0, 32x32 input, if an ONNX
      conversion is obtainable and its provenance can be stated), and a dense
      landmark model with eyelid points (e.g. MediaPipe Face Mesh / Face
      Landmarker, Apache-2.0, 192x192 or 256x256, in an ONNX conversion whose
      ops tract supports, or a PFLD 98-point WFLW model, 112x112, if its
      license is permissive). InsightFace's `2d106det` is non-commercial and
      is excluded up front; OpenCV Zoo has no eyelid model.
    - The measurement code (model loading and the per-face run for each
      candidate) is kept out of `main`: do it on a scratch branch or in the
      scratchpad against `riffle-core`'s `faces` helpers, and record the
      exact ONNX files (name, SHA-256, source URL, license) in
      `model-survey.md` in this plan folder, so Step 2 adds the chosen one
      without re-finding it. The raw outputs go to
      `D:\Photos\tests\2026-10-xx-closed-eyes\`.
    - A "Decision" section is added to this plan.md: the chosen model (or
      "none"), its per-face cost against the budget (see Trade-offs), and
      whether the whole-image path is in scope (whether the 3/8 decode's eye
      crops were labelable / scorable, or a full decode would be needed and
      what it costs). If the decision is "none", the same PR rewrites the
      `todo.md` item with the measured numbers (what was tried, size, cost,
      accuracy, the preview-resolution floor) and keeps it open, and Steps
      2-5 are struck through with a one-line reason each.
    - `docs/humans/performance.md` "Face detection cost" gains a subsection
      "Eye-state model survey (Windows 11)" with the comparison table (model,
      license, size, per-face latency, AUC / accuracy, labeled-set size and
      the unlabelable fraction); `docs/humans/performance.ja.md` in sync.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/cli/src/main.rs`: add the subcommand next to `detect` /
      `faces`; reuse `faces::upright_rgb`, `faces::to_upright`,
      `faces::crop_rgb` (already public) and `image` for the PNGs. Do not
      change `riffle-core` in this step beyond a public helper for the eye
      crop geometry if it is needed by both the CLI and the later core
      module (then put it in `faces.rs` next to `crop_rgb`, unit-tested).
    - Crop geometry to fix here so the survey and Step 2 measure the same
      pixels: eye square side as a fraction of the inter-ocular distance
      (e.g. 0.6-0.8x, clamped to a minimum of ~16 px), centered on YuNet's
      eye point; face crop as the box expanded by a margin (e.g. 1.25x)
      and squared. Try without roll correction first; record whether tilted
      heads hurt.
    - Timing: build the plan before timing (as `detect` does for the three
      YuNet plans); time per face on one thread; the labeled faces are the
      timing set so size and accuracy come from one run.
    - Also measure the current pass-2 figures on the same day as the
      reference (the `detect` crop-path numbers on 40 ARWs and `candidates`
      on the 2134-ARW folder, three alternated runs), since the machine's
      speed drifted ~11% between days in the recall plan.
    - Expect the preview resolution to be the limiting factor: note the
      smallest face side at which a human can call the eye state from the
      crop, and treat faces below it as `Unknown` in Step 2.

- [ ] ~~Step 2: Add the chosen model to `riffle-core` and return a closed-eyes probability per scored face~~
  - Not done: Step 1 chose no model (see "Decision").
  - Done when:
    - The chosen ONNX file and its license text are in `crates/core/models/`
      (license file named so it is distinguishable from YuNet's, e.g.
      `LICENSE-<model>`, and the existing `LICENSE` renamed or referenced
      accordingly if the README's license text lists model licenses).
    - A new module `crates/core/src/eyes.rs` loads the model once in a
      `OnceLock` (the rule of `faces::detector`: cached `Result`, pinned
      input fact, outputs resolved by outlet label at build time) and exposes
      a function taking the upright RGB, its size and a `Face` and returning
      an `Eyes` value: the per-eye and per-face closed probability (or the
      EAR mapped to a probability by the threshold Step 1 chose) and a state
      (`Open` / `Closed` / `Unknown`, `Unknown` when the face is below the
      size floor Step 1 found or the crop leaves the image). The constant
      threshold and size floor are documented with the Step 1 numbers, as
      `candidate.rs` documents `LOGIT_*` / `CANDIDATE_LOGIT`.
    - `candidate::Cue` carries `eyes: Option<Eyes>` (or `eyes_closed:
      Option<f64>`, matching what the index will store) for the face nearest
      the AF point, computed in `focus_cue_unless` on the full-size upright
      RGB after the eye-focus measure, behind the same `cancel` check; the
      existing `eye_focus`, `state` and the labeled-folder AUC are untouched
      (`riffle-cli candidates` prints identical states, AUC, precision and
      coverage before / after on the nine labeled folders). Whole-image path:
      per the Step 1 decision, either scored on the sharpness window's face
      (with the decode it needs) or left `None` with the reason in the doc
      comment.
    - `scan::Analysis` carries the value (through `Cue`), and `riffle-cli
      candidates` and `detect` print it per file so the per-file lines stay
      diffable; `riffle-cli bench` gains a row for the eye-state model per
      face.
    - A failure or panic in the eye model is `None` for the file, not a
      failed file (the `catch_unwind` pattern of `scan::cue`).
    - Tests: unit tests for the crop geometry and the probability-to-state
      mapping; an `#[ignore]`d real-image test like `faces.rs`'s that runs
      the model on a crop and checks the output shape and range.
    - `docs/agents/tract-onnx-inference.md` gains the second model's
      specifics (input layout and normalization, output outlets, anything
      that broke).
    - Per-file pass-2 cost before / after on the 2134-ARW folder (`detect`
      on the 40-ARW sample, `candidates` on the folder, alternated runs) and
      the release `riffle-cli` / `riffle-app` binary sizes before / after are
      recorded in `learnings.md` (the final doc tables are Step 5's).
    - `mise run ci` passes. No `FACES_VERSION` bump here: the stored columns
      do not change until Step 3 (the app ignores the new field until then).
  - Implementation approach:
    - Assumes Step 1's decision. Follow `faces.rs` for the input tensor
      (check whether the model wants RGB or BGR, 0..1 or 0..255,
      mean / std normalization; the survey records it) and `candidate.rs`
      for how a frozen measure and its threshold are documented.
    - Reuse `crop_rgb` / `window_at` for the crops and a box-average or
      bilinear resize to the model input (YuNet's `input_tensor` box-averages
      down; an eye crop may need upscaling, so a small bilinear helper may be
      needed, unit-tested).
    - Keep `faces.rs` focused on detection; the new module depends on it,
      not the other way round.
    - Files: `crates/core/models/`, `crates/core/src/eyes.rs`,
      `crates/core/src/lib.rs`, `crates/core/src/candidate.rs`,
      `crates/core/src/scan.rs`, `crates/cli/src/main.rs`,
      `docs/agents/tract-onnx-inference.md`, `README.md` / `README.ja.md`
      (the bundled-model / license sentence next to YuNet's).

- [ ] ~~Step 3: Store the closed-eyes probability in the index and stream it with the second pass~~
  - Not done: there is no closed-eyes value to store without a model.
  - Done when:
    - `files` gains `eyes_closed REAL` (`NULL` = unknown / not computed) in a
      new schema version (v16 -> v17, the `ALTER TABLE ... ADD COLUMN` path
      the v10-v14 -> v15 migration uses, with the older-database tests
      extended); `FACES_VERSION` 5 -> 6 with its doc comment extended, so
      every folder's second pass re-runs once; `EXTRACTOR_VERSION` stays
      (pass 1 does not change).
    - `write_faces` writes it with `eye_focus` and `sharpness`;
      `reset_faces` (the `UPDATE ... SET eye_focus = NULL, sharpness = NULL`)
      clears it too; `FaceReady` and `Focus` carry `eyes_closed:
      Option<f64>`, read in `indexed_file` (the `INDEXED_FILE` column list)
      and serialized to the UI; `run_faces_scan`'s `flush` passes it
      through.
    - Tests: the round-trip test pattern of `a_sharpness_score_round_trips`
      for the new column; the migration tests for a v16 database; the
      `run_faces_scan` test that checks `FaceReady` fields.
    - The MCP companion: if `src/mcp.rs` / `companion.ts` expose the focus
      state of the current file, add the field there in the same shape;
      otherwise leave it and say so in Progress.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 2 is merged. The `Focus` struct is `None` for a file
      without an AF point, so if Step 1 decided the whole-image path is out
      of scope, the value lives naturally on `Focus`; if it is in scope,
      the value must live on `IndexedFile` itself (next to `sharpness`),
      since no-AF files have no `Focus`. Decide by the Step 1 decision and
      say which in the PR.
    - Files: `crates/app/src/index.rs`, `crates/app/src/commands.rs` (only
      if a command's return type changes), `crates/app/src/mcp.rs`,
      `crates/app/ui/src/companion.ts` (if applicable).

- [ ] ~~Step 4: Show the eye state in the meta pane and the filter menu~~
  - Not done: there is no eye state to show without a model.
  - Done when:
    - `meta.ts` `FocusCue` (or the entry) carries `eyes_closed`, and the
      Analysis group gets an `Eyes` row: `Closed (NN%)` at or above the
      threshold, `Open (NN%)` below it, no row when unknown, following the
      `AF eye in focus` row's style; `meta.test.ts` covers the three cases.
    - The filter menu gains an `Eyes` section with `Closed` / `Open` /
      `Unknown` items modeled on the `AF eye` section (`index.html`
      `data-eyes` items, `filter.ts` `FilterState` + `passes`, `main.ts`
      wiring, `faces-progress` refilling as the pass runs, the active-filter
      indicator); `filter.test.ts` covers it. The threshold the UI uses is
      the one the backend state encodes (prefer a serialized state string
      next to the probability, as `candidate` is derived from `eye_focus` by
      `serialize_candidate`, so the frontend never re-derives the threshold).
    - No new strip mark and no change to the focus mark's colors (kept
      minimal; see Trade-offs).
    - `docs/humans/usage.md` (meta pane Analysis paragraph, filter menu
      paragraph, the feature list) and `usage.ja.md`, `README.md` /
      `README.ja.md` feature bullet, in the same PR.
    - `mise run ci` passes (`pnpm exec vp check`, `fmt`, `test`).
    - Manual checks (GUI, cannot be driven by an agent; list them in
      `learnings.md` as pending): open `D:\photos\2026\2026-09-19` after
      updating, let the second pass re-run (`FACES_VERSION` 6), select a
      frame Step 1 labeled `closed`: the meta pane shows `Eyes: Closed`;
      the filter `Eyes > Closed` lists the labeled frames and refills while
      the pass runs; a Leica DNG shows no `Eyes` row (or the row, if the
      whole path is in scope).
  - Implementation approach:
    - Assumes Step 3 is merged. Reuse the `AF eye` section's DOM and state
      pattern exactly (`data-candidate` -> `data-eyes`; `shownCandidates` ->
      a parallel set); the `applyFaceReady` patch path in `focus.ts` is
      where the new field lands on the entry.
    - Files: `crates/app/ui/index.html`, `crates/app/ui/src/meta.ts`,
      `meta.test.ts`, `filter.ts`, `filter.test.ts`, `focus.ts`,
      `focus.test.ts`, `main.ts`, `docs/humans/usage.md`, `usage.ja.md`,
      `README.md`, `README.ja.md`.

- [ ] ~~Step 5: Record the final cost and size and close the todo item~~
  - Not done: Step 1 recorded the costs and rewrote the todo item, which stays open.
  - Done when:
    - `docs/humans/performance.md` "Face detection cost" gets a subsection
      "Closed-eyes model (Windows 11)" with: binary size before / after
      (release `riffle-cli` and `riffle-app`, the table format of the
      existing size table), per-face model latency, the cue path's per-file
      `detect` numbers before / after, `candidates` wall time on the
      2134-ARW folder before / after (three alternated runs each), and the
      accuracy on the labeled set, and "Which pass carries which cost" lists
      the new work under pass 2; `performance.ja.md` in sync.
    - `todo.md`: the "Optionally, detect closed eyes" item is checked with
      the chosen model, the measured size / cost / accuracy, and a link in
      backticks to this plan folder; the section intro sentence about closed
      eyes is updated.
    - `mise run ci` (lint, lychee) passes.
  - Implementation approach:
    - Docs only; the measurements come from Steps 1 and 2's `learnings.md`
      re-run once on the merged `main` so before / after are on the same
      day. If Step 2's PR already left the performance section complete,
      this step is the todo update only and can be folded into Step 4's PR;
      say so in Progress.

## Decision

Recorded by Step 1 on 2026-10-07; the numbers are in
[`model-survey.md`](model-survey.md) and the labels in
[`eyes-truth.md`](eyes-truth.md).

- **Chosen model: none.** Four candidates were measured end to end on 504
  hand-labeled faces (432 from the 2134-ARW folder, 72 from the 146-DNG
  folder), one thread on the Windows 11 machine, crop and resize included:

  | Candidate | Size | Per face | Per-file pass-2 cost | Face AUC |
  |-----------|------|----------|----------------------|----------|
  | OMZ `open-closed-eye-0001` (eye-crop classifier) | 46 KB | 0.5 ms | +1% | 0.72 |
  | MediaPipe face mesh v1 (EAR) | 2.4 MB | 15.4 ms | +33-35% | 0.89 |
  | MediaPipe Face Landmarker v2 (EAR) | 4.9 MB | 49-51 ms | +105-109% | 0.97 |
  | MediaPipe Iris (eyelid contour) | 2.6 MB | 18.6 ms | +40% | 0.86 |

  The budget (+25% of the cue path's per-file `detect` time, one face per
  file) was about +11.5 ms on the day, against decode 13.5 ms + detection
  32.4 ms per ARW. The classifier is the only one inside it and is close to
  useless on these previews (precision 0.35 at a recall of 0.70 at its
  0.5 threshold; no crop variant got its face AUC above 0.78). Face mesh v2
  is the only one good enough for a filter (precision 0.94 at a recall of
  0.86) and costs about 4x the budget.
- **Preview-resolution floor.** A human can call the eye state of about one
  face in five below a 60 px face side on the 1616 px ARW preview and about
  two in three from 60 px up. 40% of the randomly drawn faces could not be
  labeled: 25% had no eye to judge at all (turned away, occluded, false
  detections) and 15% only eyes too small, blurred or dark to call.
- **Whole-image path: out of scope.** On the 3/8 decode the whole-image
  search uses, a fifth of the labelable DNG faces were no longer labelable
  and the classifier's eye AUC fell from 0.78 to 0.59 on the same faces; a
  landmark model there would need its own full-size decode (about 14 ms per
  DNG) on top of the model.
- **What could revive it**: an on-demand eye state for the current file
  only (as `faces_of` detects faces on demand), where ~50 ms for face
  mesh v2 is acceptable but the filter menu gets nothing; a larger pass-2
  budget; or a faster runtime for the same model. These are in the rewritten
  `todo.md` item.

## Trade-offs and risks

The user approved this plan with the defaults below on 2026-10-07.

- **The pass-2 budget.** Default (approved): per-face cost that keeps the cue
  path's per-file `detect` time within ~+25% (about +8 ms per ARW, ~1.8
  faces, so roughly <= 4-5 ms per face including the crop). The alternative,
  an absolute cap on `candidates` wall time on the 2134-ARW folder (e.g.
  <= 15 s at 24 threads), was not chosen. No binary size cap was set; record
  the size and prefer the smaller binary when both kinds fit (the YuNet model
  is 233 KB; a Face Mesh ONNX is a few MB, a 32x32 classifier tens of KB).
- **Classifier vs dense landmarks.** An eye-crop classifier is tiny and fast
  (two runs per face at ~32 px input) and directly answers the question, but
  it is trained on its own crop convention, so the eye square derived from
  YuNet's eye point must match it, and tilted heads are not corrected. A
  landmark model gives an EAR that is interpretable and roll-invariant and
  also enables later features (gaze, mouth), but costs one run per face at
  a 112-256 px input (likely 5-20 ms on this CPU through tract, the same
  order as YuNet's 320 run at ~20 ms), several MB of binary, and an EAR
  threshold that depends on head pose. Step 1 decides on the measured
  numbers; if both fit the budget, prefer the smaller binary.
- **Preview resolution may be the real ceiling.** A 100 px face on a 1616 px
  ARW preview has eyes of ~20 px; below ~60 px faces a human cannot call the
  eye state either. Step 1 records the labelable fraction by face side; the
  feature may end up applying only to the subject-sized faces the cue path
  already scores, which is also where the culling value is. If the
  labelable fraction is small even there, stopping after Step 1 is the right
  outcome.
- **Whole-image path out of scope (approved default).** No full-size decode
  exists there (3/8 scale), so eye state would need another decode (~14 ms
  per DNG) or an upscale from the 3/8 image. The value is `None` there and
  lives on `Focus`; Step 1's decision records it.
- **Which face (approved default).** Only the face nearest the AF point (the
  one the cue scores) is evaluated, not every face: it keeps the cost at one
  or two runs per file, matches the `AF eye` semantics, and a group's
  bystander blink is not what the user culls on. Evaluating all faces would
  be per-face cost x ~1.8 and needs a per-face store the index does not have.
- **Who labels the truth set (approved default).** The agent labels the
  exported crops with an `unsure` class, and the user reviews only the
  `closed` ones. Open / closed at 20 px is a finer call than face / no face
  and blinks are rare (expect a few percent of frames), so the user may point
  at folders with known blinks.
- **UI scope (approved default).** A meta pane row and a filter section, no
  strip icon and no focus-mark color change. A strip mark would make blinks
  visible while paging, but it competes with the candidate face icon and the
  focus-mark colors already encode the AF-eye state; adding "closed" as a
  fourth color conflates two judgments. A strip mark can be a follow-up todo.
- **ONNX provenance.** Converted models (TFLite -> ONNX, OpenVINO IR ->
  ONNX) carry the original license but their conversion source must be
  stated in the license file next to the model, and tract may reject ops
  the conversion emits; Step 1 checks loading with `default-features =
  false` before anything else, and turning tract's default features back
  on is a measured decision (binary size), not a default.
- **`FACES_VERSION` bump.** One bump (Step 3) re-runs pass 2 on every folder
  once (~9 s per 2000 files on 24 threads); accepted. Step 2 adds the
  per-face cost to pass 2 without storing the value until Step 3 lands; if
  the two PRs are far apart the cost is paid for nothing in between, so
  merge them close together or fold Step 2 and 3 into one PR if the review
  size allows.

## Progress

- (none yet)
