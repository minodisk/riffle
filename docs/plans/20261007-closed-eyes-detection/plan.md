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
closed needs a second model run on the face YuNet already finds, either an
open / closed classifier on an eye crop or a dense landmark model whose eyelid
points give an eye aspect ratio (EAR).

Step 1 measured four such models against the second scan pass's budget
(`run_faces_scan` in `crates/app/src/index.rs`, through
`scan::extract_analysis_unless`) and found none that fits it: the only one
accurate enough, MediaPipe Face Landmarker v2, costs about four times the
budget per file. The user then decided (2026-10-07) to adopt that model **on
demand for the file being shown only**, the way the focus mark's faces are
detected today (`faces_of` in `crates/app/src/commands.rs` through the scan's
`detect_around`, without touching the index). ~50 ms once per shown file is
acceptable where ~50 ms per file across a 2000-file folder was not.

Once done, selecting a file runs the model on the face the scan judges for
that file and the meta pane's Analysis group shows `Eyes: Closed (NN%)` or
`Open (NN%)`. Nothing is stored in the index (no column, no `FACES_VERSION`
or schema bump) and the filter menu gets no section: a filter needs every
file scored, which the on-demand design rules out.

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
  model needs a full-size decode here (~14 ms per DNG).
- **On-demand path** (what this plan builds on): `faces_of`
  (`commands.rs` `read_faces`) reads the preview, calls `detect_around` with
  `sharpness::trusted_focus` and returns the faces for the focus mark; it
  runs in `spawn_blocking`, is requested from `drawFaceMarks` in `main.ts`
  through `FaceCache` (`ui/src/faces.ts`: per-path session cache, in-flight
  dedupe, a generation that drops responses issued before the last `clear`),
  and is cleared on folder change. `focus_crop` is the other on-demand
  command with a latency budget (keypress to pixels 39-45 ms, measured in
  `docs/humans/performance.md` "The 1:1 focus check path").
- What the index stores per file (`FACES_VERSION` 5, schema v16):
  `eye_focus`, `sharpness`, `faces_extractor`. `meta.ts` shows `AF eye in
  focus` from `entries.get(path)?.focus`; this plan adds no column.

### Tools and samples

- `riffle-cli detect <dir|file>... [threads]` (per-file path, faces, times),
  `riffle-cli faces <file> <out.png>` (boxes drawn), `riffle-cli bench`,
  `riffle-cli candidates <dir>... [threads]`, `riffle-cli eyecrops
  <dir|file>... <out-dir>` (Step 1: face and eye crops with a diffable
  index). `crates/cli` depends on `image` with PNG output.
- Sample RAWs: `D:\photos\samples\<FORMAT>\`; real folders
  `D:\photos\2026\2026-09-19` (2134 α7 V ARWs, AF point on every file) and
  `D:\photos\2026\2026-02-01` (146 M11-P DNGs, no AF point). Step 1's labels
  are in [`eyes-truth.md`](eyes-truth.md); its crops, tiles, the four ONNX
  files, the scratch survey crate and the raw per-face outputs
  (`survey-arw.tsv`, `survey-dng.tsv`) are in
  `D:\Photos\tests\2026-10-07-closed-eyes\`. Hand-check copies go in
  `D:\Photos\tests\<date>-<topic>\`.
- Guide: [`../../agents/tract-onnx-inference.md`](../../agents/tract-onnx-inference.md)
  (model loading with `default-features = false`, outlet labels, `OnceLock`
  plans, input conventions).

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
      every closed-eyes face that can be found. It records how many faces
      were too small or too blurred to label, by face side in preview px:
      this is the floor of what any model can do on the embedded preview.
    - Each candidate model is checked for: license, ONNX file size, whether
      `tract-onnx` 0.23 with `default-features = false` loads and optimizes
      it and where its outputs are found (outlet labels), per-face latency
      on the Windows 11 machine (one thread, release, mean / median / p95,
      crop and resize included), and accuracy on the labeled set (AUC, and
      accuracy / precision / recall of `closed` at the best threshold, per
      eye and per face). At least two candidates, one of each kind, measured
      end to end.
    - The measurement code is kept out of `main`; the exact ONNX files (name,
      SHA-256, source URL, license, conversion recipe) are in
      `model-survey.md` so Step 2 adds the chosen one without re-finding it.
    - A "Decision" section is added to this plan.md: the chosen model, its
      per-face cost against the pass-2 budget, and whether the whole-image
      path is in scope.
    - `docs/humans/performance.md` "Face detection cost" gains a subsection
      "Eye-state model survey (Windows 11)" with the comparison table;
      `docs/humans/performance.ja.md` in sync.
    - `mise run ci` passes.
  - Implemented on branch `closed-eyes-detection-step-1`. It first recorded
    the decision "none" for pass 2 and struck Steps 2-5; the user replaced
    that with the on-demand adoption on 2026-10-07 (see "Decision"). The
    branch also carries the matching updates to this plan, `todo.md`,
    `performance.md` / `.ja.md` and `learnings.md`. The user's review of the
    `closed` labels (`learnings.md` "Deferred issues") stays pending and
    does not block the PR.

- [ ] Step 2: Add MediaPipe Face Landmarker v2 to `riffle-core` as the `eyes` module
  - Done when:
    - `crates/core/models/face_landmarks_detector.onnx` (4,921,000 B, SHA-256
      `cafab16c...c05804` from `model-survey.md`) and its license text are
      in `crates/core/models/`. The license file is named so it is
      distinguishable from YuNet's `LICENSE` (e.g. `LICENSE-mediapipe`), is
      the Apache-2.0 text plus a provenance note: the `.task` URL and
      SHA-256, the inner `face_landmarks_detector.tflite` SHA-256, and the
      conversion recipe (tf2onnx 1.17.0 on TensorFlow 2.21.0,
      `--tflite ... --opset 13`, float16 weights widened to float32). The
      existing YuNet `LICENSE` is left as is unless its name must change for
      the pair to read clearly; `docs/humans/usage.md` references
      `crates/core/models/LICENSE` by name, so if it is renamed, update the
      reference (and `usage.ja.md`).
    - A new module `crates/core/src/eyes.rs` (exported from `lib.rs`):
      - loads the model once in a `OnceLock` with the rule of
        `faces::detector` (cached `Result`, `with_ignore_value_info(true)`,
        `with_ignore_output_shapes(true)`, input fact pinned to
        `[1, 256, 256, 3]`, the `Identity` outlet resolved by label at build
        time; `Identity_1` / `Identity_2` unused);
      - exposes a function taking the full-size upright RGB, its size and a
        `Face` (in upright coordinates) and returning `Option<Eyes>` /
        `Eyes { probability: f64, state: EyeState }` with
        `EyeState::{Open, Closed}`; `None` (unknown) when the face's longer
        box side is below the floor `EYES_MIN_FACE` = 60 px (the preview
        side at which a human calls about one face in five below and two in
        three above, `eyes-truth.md`) or the crop leaves the image;
      - cuts the face crop exactly as the survey did (the YuNet box squared on
        its longer side and grown `FACE_CROP` = 1.25x, `faces::crop_rgb`
        clamped inside the image, bilinear resize to 256x256, RGB NHWC 0..1,
        no roll correction), so the Step 1 numbers transfer;
      - computes the EAR per eye from the six points
        `33, 160, 158, 133, 153, 144` and `362, 385, 387, 263, 373, 380`,
        takes the more closed (lower) of the two, and maps it to a closed
        probability with a one-dimensional logistic whose constants are
        documented like `candidate.rs`'s `LOGIT_*` / `CANDIDATE_LOGIT`:
        the decision boundary is EAR <= `EYES_CLOSED_EAR` = 0.137 (best F1
        on the labeled set: accuracy 0.957, precision 0.94, recall 0.86), so
        `probability >= 0.5` and `state == Closed` are the same test and the
        meta pane's percentage and word agree. The slope is fit on the saved
        per-face EARs (`survey-arw.tsv` / `survey-dng.tsv` against
        `eyes-truth.md`), recorded in `learnings.md` with the fitting script
        kept out of `main`.
    - `face_square` and `face_to_upright` (and `FACE_CROP`) move from
      `crates/cli/src/main.rs` into `riffle-core` (`faces.rs` next to
      `crop_rgb` / `to_upright`, or `eyes.rs` if only the eyes code uses
      them), with their unit tests; `eyecrops` output on the 146-DNG folder is
      byte-identical before / after. The eye-square helpers stay in the CLI
      (no core consumer).
    - A failure or panic in the model is `None`, not an error, at the
      boundary the app will call (the `catch_unwind` pattern of `scan::cue`),
      so a bad crop never fails the shown file.
    - `riffle-cli eyes <dir|file>...` (or an extension of `detect`'s per-file
      line, whichever keeps the lines diffable) prints per file the judged
      face's side, EAR, probability and state so the Step 1 AUC can be
      reproduced from `riffle-core` itself: run it on the labeled ARW and DNG
      files and record in `learnings.md` that the face AUC on `eyes-truth.md`
      matches the survey (0.97) within noise. `riffle-cli bench` gains a row
      for the model per face.
    - Tests: unit tests for the crop geometry (moved with the helpers), the
      EAR of a synthetic six-point set, the logistic mapping and the state
      boundary (EAR just below / above 0.137), the size floor; an
      `#[ignore]`d real-image test like `faces::detects_a_face_in_a_real_image`
      (env var path) that runs YuNet then the model on a crop and checks 478
      points in 0..256 and a probability in 0..1.
    - `docs/agents/tract-onnx-inference.md` gains the second model's
      specifics: NHWC RGB 0..1 input (the first NHWC model here, against
      YuNet's BGR NCHW), outlet labels `Identity` / `Identity_1`, the
      first-call plan build time, and anything that broke, tagged Hit /
      Measured / Inferred as the guide does.
    - `README.md` "Offline face detection" bullet and `README.ja.md`'s
      mention the second bundled model next to YuNet (MediaPipe Face
      Landmarker, Apache-2.0, converted from Google's TFLite) and what it is
      used for (closed-eyes judgment of the shown file).
    - Recorded in `learnings.md`: release `riffle-cli` / `riffle-app` binary
      sizes before / after (the table format of performance.md "Face detection
      cost"), the per-face latency of the core function on one thread over
      the labeled faces (should match the survey's 49-51 ms), the first-call
      plan build time, and the cold `cargo build --release` time if it
      changed noticeably.
    - `mise run ci` passes. No `FACES_VERSION`, schema or `scan::Analysis`
      change: the scan does not call this module.
  - Implementation approach:
    - Follow `faces.rs` for the loading code and `candidate.rs` for how a
      frozen measure and its threshold are documented. Keep `faces.rs`
      focused on detection; `eyes.rs` depends on it, not the other way round.
    - `faces::crop_rgb` box-averages down (YuNet's `input_tensor`); a face
      crop of 75-300 px must be resized both down and up to 256, so add a
      small bilinear resize helper (unit-tested) or reuse `image`'s if it is
      already a dependency with the needed feature (`image` is in
      `riffle-core` with `jpeg` only; check binary cost before enabling
      more).
    - Files: `crates/core/models/`, `crates/core/src/eyes.rs`,
      `crates/core/src/lib.rs`, `crates/core/src/faces.rs`,
      `crates/cli/src/main.rs`, `docs/agents/tract-onnx-inference.md`,
      `README.md`, `README.ja.md`, `docs/humans/usage.md` / `usage.ja.md`
      (only the license file reference, if renamed).

- [ ] Step 3: Judge the shown file's eyes on demand and show the result in the meta pane
  - Done when:
    - A Tauri command `eyes_of(path)` in `crates/app/src/commands.rs`
      (registered in `lib.rs` next to `faces_of`) returns
      `{ state: "open" | "closed", probability }` or `null` (unknown: no
      face, face below the floor, JPEG file, model failure). It is
      `raw_only`, runs in `spawn_blocking`, and touches neither the index nor
      the stored state. The face it judges is the one the scan judges for
      that file (see Trade-offs "Which face"): with a trusted AF point
      (`sharpness::trusted_focus`), `candidate::nearest_face` of
      `detect_around`'s faces; without one, the largest face (by box area)
      at or above `sharpness::FACE_CONFIDENCE`, as the labeled DNG set
      picked it (not `sharpness::chosen_face`, which takes the
      highest-scoring face; a small shared helper is fine). Without an AF point the crop comes
      from a full-size decode the whole-image path does not otherwise make
      (see Trade-offs "No-AF path"). The AF path must not decode the
      full-size preview twice (detection and crop): decode once and call
      `detect_around_rgb`, or extend `faces` with a variant that returns the
      upright RGB along with the detection; measure that the command's total
      on an ARW is about decode + detection + model (~14 + ~32 + ~50 ms) and
      record it in `learnings.md`.
    - **Supersede on paging**: the command takes a request id; an
      `EyesRequests` state (a `Mutex<u64>` latest id, the pattern of
      `ScansState::latest_id`) is bumped on each call, and the blocking task
      re-checks it before detection and before the model run, returning
      `null` early when a newer request exists. On the frontend, a new
      `crates/app/ui/src/eyes.ts` (DOM- and Tauri-free like `faces.ts`)
      keeps a per-path session cache with in-flight dedupe and a generation
      (the `FaceCache` shape), plus the one-in-flight / re-request-if-stale
      rule `main.ts` uses for `inFlight` so at most one judgment runs at a
      time and fast paging never queues one per passed file; a response for
      a path the user paged away from is still cached (coming back is
      free). `main.ts` requests it from `show()` after `requestPreview()`
      and `requestMetadata()`, re-renders the meta pane when a response for
      the current file settles, and clears the cache where `faceCache` is
      cleared (folder open, `refreshEntries`).
    - `meta.ts`: `metaGroups` takes the eyes result and the Analysis group
      gets an `Eyes` row after `AF eye in focus`: `Closed (NN%)` when the
      state is closed, `Open (NN%)` when open (the percentage is the closed
      probability in the first case and the open one, `100 - NN`, in the
      second, so the number always backs the word), no row when unknown or
      not yet answered; `meta.test.ts` covers the three cases;
      `eyes.test.ts` covers the cache, the dedupe, the stale drop and the
      re-request rule. No strip mark, no focus-mark color change, no filter
      section, no MCP companion field (a follow-up todo if wanted).
    - Latency target (see Trade-offs): on the Windows 11 machine with the
      folder idle (no scan running), the row appears within ~150 ms of the
      preview of an ARW with an AF point, and paging through 20 files with
      the key held shows no added stall against the previous build. Measured
      by hand with `Timing logs` on: add an `eyes` timing line (read, decode,
      detect, model, total) in the shape of `focus_crop`'s header, and record
      the numbers in `learnings.md` (Step 4 puts them in `performance.md`).
    - `docs/humans/usage.md`: the meta pane's **Analysis** paragraph gains
      the `Eyes` row (what it judges, that it is computed when the file is
      shown and not in the scan, that it is absent below ~60 px faces or
      without a face, and that a downcast eye reads as closed, as
      `eyes-truth.md` found), and the Features list's face bullet mentions
      it; `usage.ja.md` in sync. `README.md` / `README.ja.md` feature bullet
      if the feature list there names the AF eye row.
    - Manual checks (GUI; list them in `learnings.md` as pending if not run):
      open `D:\photos\2026\2026-09-19`, select a frame `eyes-truth.md` labels
      closed (the "Faces with a closed eye" list): the meta pane shows
      `Eyes: Closed (NN%)`; an open-eyed frame shows `Open`; a frame with no
      face or a tiny face shows no row; a Leica DNG from
      `D:\photos\2026\2026-02-01` with a face shows the row (no-AF path);
      holding the page key through 30 files leaves no stale row from a
      previous file and the preview keeps pace.
    - `mise run ci` passes (`pnpm exec vp check`, `fmt`, `test`).
  - Implementation approach:
    - Assumes Step 2 is merged. Mirror `faces_of` / `read_faces` for the
      command and `FaceCache` / `requestFaces` for the frontend; keep the
      decision logic in `eyes.ts` so Vitest covers it without mocks.
    - Files: `crates/app/src/commands.rs`, `crates/app/src/lib.rs`,
      `crates/app/ui/src/eyes.ts`, `eyes.test.ts`, `meta.ts`,
      `meta.test.ts`, `main.ts`, `docs/humans/usage.md`, `usage.ja.md`,
      `README.md` / `README.ja.md` (if applicable).

- [ ] Step 4: Record the on-demand cost and size and close the todo item
  - Done when:
    - `docs/humans/performance.md` "Face detection cost" gets a subsection
      "Closed-eyes judgment on demand (Windows 11)" after the survey
      subsection: binary size before / after (release `riffle-cli` and
      `riffle-app`, the existing size table's format), the model's per-face
      latency and first-call plan build, the `eyes_of` breakdown (read,
      decode, detect, model, total) on an ARW with an AF point and on a DNG
      without one, the hand-measured time from preview to row, and the
      accuracy on the labeled set (face AUC, precision / recall at the
      threshold). "Which pass carries which cost" gains a line saying the
      closed-eyes judgment runs in neither pass but on demand for the shown
      file. `performance.ja.md` in sync.
    - `todo.md`: the "Optionally, detect closed eyes" item is checked with
      the chosen model, the measured size / cost / accuracy and a link in
      backticks to this plan folder; the section intro sentence about closed
      eyes is updated. Follow-up items are added, unchecked, each
      fully specified: judging every face at or above the floor (per-face
      rows or a worst-of summary); surfacing the model's face presence
      output (`Identity_1`, a logit; needs a threshold calibrated on labeled
      faces, which `eyes-truth.md`'s `x` labels could seed); an MCP companion
      field; a strip mark for closed eyes; and the user's pending review of
      the `closed` labels if still open.
    - `mise run ci` (lint, lychee) passes.
  - Implementation approach:
    - Docs only; the numbers come from Steps 2 and 3's `learnings.md`,
      re-measured once on the merged `main` so before / after are on the
      same day (the machine drifted ~11% between days in the recall plan).
      If Step 3's PR already left the performance subsection complete, this
      step is the todo update only and can be folded into Step 3's PR; say
      so in Progress.

## Decision

Measured by Step 1 on 2026-10-07 (numbers in
[`model-survey.md`](model-survey.md), labels in
[`eyes-truth.md`](eyes-truth.md)); decided by the user on 2026-10-07,
replacing Step 1's first recording of "none".

- **For the scan's second pass: no model fits.** Four candidates were
  measured end to end on 504 hand-labeled faces (432 from the 2134-ARW
  folder, 72 from the 146-DNG folder), one thread on the Windows 11 machine,
  crop and resize included:

  | Candidate | Size | Per face | Per-file pass-2 cost | Face AUC |
  |-----------|------|----------|----------------------|----------|
  | OMZ `open-closed-eye-0001` (eye-crop classifier) | 46 KB | 0.5 ms | +1% | 0.72 |
  | MediaPipe face mesh v1 (EAR) | 2.4 MB | 15.4 ms | +33-35% | 0.89 |
  | MediaPipe Face Landmarker v2 (EAR) | 4.9 MB | 49-51 ms | +105-109% | 0.97 |
  | MediaPipe Iris (eyelid contour) | 2.6 MB | 18.6 ms | +40% | 0.86 |

  The budget (+25% of the cue path's per-file `detect` time, one face per
  file) was about +11.5 ms on the day, against decode 13.5 ms + detection
  32.4 ms per ARW. The classifier is the only one inside it and is close to
  useless on these previews (no crop variant got its face AUC above 0.78).
  Face mesh v2 is the only one good enough to act on (precision 0.94 at a
  recall of 0.86 at EAR <= 0.137) and costs about 4x the budget. The scan
  stays as it is: no index column, no `FACES_VERSION` or schema bump, no
  filter section.
- **Adopted: MediaPipe Face Landmarker v2 on demand for the shown file.**
  The face-mesh v2 ONNX (4.9 MB, Apache-2.0, converted from Google's TFLite
  as recorded in `model-survey.md`) judges the eye state via EAR of the one
  face the scan judges for the file, run when the file is shown, the way
  `faces_of` detects the focus mark's faces today. ~50 ms once per shown
  file (plus ~14 ms decode and ~32 ms detection, in `spawn_blocking`,
  superseded when the user pages on) is acceptable where the same cost on
  every file of a folder was not. The result goes to the meta pane's
  Analysis group only (`Eyes: Closed (NN%)` / `Open (NN%)`, no row when
  unknown).
- **Threshold and floor.** Closed iff EAR <= 0.137 (best F1 on the labeled
  set); a logistic on the EAR gives the percentage, with 0.5 at that
  boundary. Faces below a 60 px side on the stored preview are not judged:
  a human can call about one face in five there and two in three from 60 px
  up; 40% of the randomly drawn faces could not be labeled. Downcast eyes
  read as closed, which the labels and the model agree on.
- **Whole-image path: in scope on demand.** The 3/8 decode the scan's
  whole-image search uses is not good enough (a fifth of the labelable DNG
  faces lost, the classifier's eye AUC 0.78 -> 0.59), but on demand a
  full-size decode (~14 ms per DNG) is affordable, so a no-AF file is
  judged on its sharpness face from a full-size crop.
- **Out of scope**: the model's face presence output (`Identity_1`; a
  follow-up todo), judging every face, a strip mark, a filter, the MCP
  companion.

## Trade-offs and risks

The user settled the adoption on 2026-10-07 ("Decision") and approved this
revision with the recommended defaults below the same day.

- **Which face is judged (chosen: the one the scan judges).** With an AF
  point, the face nearest it (`candidate::nearest_face`, the `AF eye` face);
  without one, the largest face at or above
  `sharpness::FACE_CONFIDENCE` (not `chosen_face`'s highest-scoring one, so
  that it matches the labeled DNG set). These are exactly the faces `eyes-truth.md` labeled and the 0.97 AUC was measured on, they match
  the `AF eye in focus` row's subject, and they give one row. Alternative:
  every face at or above the 60 px floor (~50 ms x ~1.8 faces on average,
  more in groups): it would catch a bystander's blink, but needs per-face
  rows or a worst-of summary the meta pane has no pattern for, and its
  accuracy on non-subject faces is unmeasured. Left as a follow-up todo
  (Step 4).
- **No-AF path (chosen: in scope, two decodes).** Option A: keep
  `detect_around` as `faces_of` uses it (the 3/8 whole-image search, so the
  judged face is exactly one the focus mark draws) and make a second,
  full-size decode for the crop: ~5 + 14 ms. Option B: one full-size decode
  and `detect_around_rgb` with `focus = None`: one decode fewer, but the
  faces may differ slightly from the drawn ones. A is chosen for consistency
  with what the user sees; the difference is ~5 ms on a ~100 ms call. On the
  AF path one full-size decode serves both detection and crop.
- **Face presence output (out of scope).** Reading `Identity_1` is free, but
  using it needs a threshold, and no labels exist for "is this a face" beyond
  the `x` labels of `eyes-truth.md`. Follow-up todo in Step 4.
- **Latency and cancellation (chosen: ~150 ms to the row, frontend
  one-in-flight plus backend latest-wins).** The call is ~14 ms decode +
  ~32 ms detection + ~50 ms model on one thread, so about 100 ms plus IPC
  on an idle machine. It runs in `spawn_blocking` so it never blocks the
  preview's own decode; the frontend sends at most one request at a time and
  re-requests only for the file that is current when it settles, and the
  backend drops a request a newer one has superseded between its stages.
  Alternatives: fire the request only once the preview bitmap has landed
  (`shown.seq === seq`), or a real `CancellationToken` into tract (not worth
  it at 15-50 ms per stage).
- **Binary size.** `include_bytes!` of the 4.9 MB ONNX grows `riffle-app`
  by about 10%; accepted by the decision, measured in Step 2. A float16 ONNX
  would halve it but tract's float16 support with `default-features = false`
  is unverified; loading the model from a file at run time would complicate
  the Tauri bundle. Neither is taken.
- **Threshold transfer.** The 0.137 EAR boundary and the 0.97 AUC come from
  a crop cut exactly as `eyecrops` cuts it (1.25x squared box, no roll
  correction, bilinear resize). Step 2 keeps that geometry and reproduces the
  AUC from `riffle-core` before Step 3 builds on it.
- **Labels pending review.** The agent labeled the truth set; the user's
  review of the `closed` faces is pending (`learnings.md`). A change there
  moves the AUC and possibly the best threshold; Step 2's constants are
  documented with the labeled-set date so they can be re-fit.
- **Downcast eyes read as closed.** Most ARW `closed` labels are children
  looking down, not blinks; the row says `Closed` for both. `usage.md` says
  so (Step 3).

## Progress

- 2026-10-07 Step 1 done on branch `closed-eyes-detection-step-1`: no
  surveyed eye-state model fits pass 2; MediaPipe Face Landmarker v2 (EAR,
  ~50 ms per face) adopted on demand for the shown file only.
