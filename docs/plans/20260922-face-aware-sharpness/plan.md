<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Face/eye-aware sharpness score

## Purpose

`riffle_core::sharpness::score_preview` scores a 256 px window around the
Sony `FocusLocation` or, without a trustworthy one, the sharpest tile of the
embedded preview (`tile_max`). A portrait focused on the background, the ear
or the nose still scores high, and a Leica DNG (no AF position) or a Sony
manual-focus frame scores whatever happens to be sharpest anywhere. The
`files.sharpness` column, the strip cue (`crates/app/ui/src/sharpness.ts`)
and the burst group (`crates/app/ui/src/burst.ts`) inherit that blind spot.

After this work a lightweight, pre-trained face detector runs on the embedded
preview at scan time, fully offline, and when it finds a face and the AF point
is missing, untrusted or off the face, the score is taken around the eyes.
Nothing is trained on the user's photos and no face identity or feature is
stored. Frames without a face (landscapes, animals) keep today's behaviour
exactly. The first step is a measurable prototype behind `riffle-cli`, so the
runtime and model are judged on real numbers (latency, binary size, build
time) before anything touches the app.

Builds on the "App: face/eye-aware focus check for culling" item in
`todo.md`.

## Decisions taken at planning time

- Runtime: `tract-onnx` (pure Rust, MIT/Apache-2.0, static; no per-platform
  ONNX Runtime binary, no build-time download, no macOS signing work). See
  "Trade-offs and risks" for `ort`. Confirmed by the user.
- Model: YuNet `face_detection_yunet_2023mar.onnx` from the OpenCV Zoo
  (MIT, ~230 KB, dynamic input size, outputs a bounding box, a score and 5
  landmarks: both eyes, nose, both mouth corners). Bundled into `riffle-core`
  with `include_bytes!`; no download at run time. The licence text ships
  next to the model file. Confirmed by the user.
- Detection test with a real face: an `#[ignore]` test reading an image path
  from an environment variable; no face image is committed. Confirmed by the
  user.
- The face region is not stored in the SQLite index in this plan; only the
  score changes. Storing it (for a UI mark or a "sharpest eye in the burst"
  suggestion) is a follow-up.

## Steps

- [x] Step 1: Core + CLI: face/eye detector prototype and its benchmark
  - Done when:
    - `crates/core/src/faces.rs` (new) exposes a pure function of the form
      `detect(gray_or_rgb, width, height) -> Result<Vec<Face>>`, where
      `Face` carries the bounding box, the score and the two eye positions in
      **preview pixel coordinates** (not model-input coordinates)
    - The model bytes live under `crates/core/models/` with the upstream
      licence file beside them, embedded with `include_bytes!`; the tract
      model is built once per process (`std::sync::OnceLock` or similar) and
      is `Send + Sync` so `scan::extract_all`'s rayon workers can share it
    - Pre/post-processing is in Rust: downscale the preview to the model
      input size (record the size chosen; start from a 320 px long edge),
      decode the per-stride outputs, threshold, NMS
    - Unit tests: (a) the box/landmark decode and NMS on hand-built tensors;
      (b) a flat or checkerboard image yields no face; (c) an `#[ignore]`
      detection test reading a face image path from an env var (no face
      image committed)
    - `crates/cli/src/main.rs` gains `riffle-cli faces <file> <out.png>`,
      modelled on `focusbox`: draws the face boxes and eye points on the
      preview and prints the count, the boxes and the detection time
    - `riffle-cli bench` prints a new `4. face detection` row via the
      existing `stats()` helper, measured on the preview decode result
    - Measured on the user's Mac with real files (the α7 V ARWs and the
      M11-P DNGs used in `docs/performance.md`): per-image detection latency
      (mean / median / p95), the increase in release binary size of
      `riffle-cli` and the incremental `cargo build --release` time, written
      to `learnings.md`. Target: median ≤ 30 ms per image on one thread; if
      missed, shrink the input size and re-measure before moving on
    - Whether the Sony/Leica MakerNotes expose face/eye-AF positions is
      recorded in `learnings.md` (run exiftool on a real ARW/DNG on the Mac;
      for Sony they sit in the ciphered `Tag2010`/`Tag9405` blocks that
      `arw.rs` does not read). This is a finding only; no parsing is added in
      this plan
    - `mise run ci` passes on all three OSes (the matrix builds tract)
  - Implementation approach:
    - Decide whether to feed the detector a grayscale image replicated to 3
      channels (reuses the grayscale decode `score()` already does) or an RGB
      decode; mozjpeg's DCT scaling (`scale(1/4)` on a 1616 px preview gives
      ~404 px) makes an extra small RGB decode cheap. Measure both if in
      doubt and record the choice
    - YuNet 2023mar outputs `cls_8/16/32`, `obj_8/16/32`, `bbox_8/16/32`,
      `kps_8/16/32` on a grid; decoding follows OpenCV's `FaceDetectorYN`.
      Verify every op in the model is supported by tract first; if it is not,
      the fallback is the 2022mar fixed-320x320 variant (prior boxes needed)
      and, failing that, `ort` (see Trade-offs)
    - No new dependency in `crates/app` yet; `scan::extract` is untouched in
      this step
    - Keep `faces.rs` independent of `sharpness.rs`; Step 2 joins them

- [ ] Step 2: Core: score sharpness on the eyes when a face is found
  - Done when:
    - `crates/core/src/sharpness.rs` accepts the detected faces (extend
      `score_preview`'s input, e.g. `score_preview(preview, focus, faces)`
      or a small `Subject` enum; pick the one that keeps `scan::extract`
      obvious and record it) and routes:
      1. trusted `FocusLocation` inside the chosen face's box -> the AF
         window exactly as today (Eye-AF already put the point on the eye)
      2. a face found and (no trusted focus, or the focus point outside the
         face box) -> a window centred between the two eyes, side clamped to
         `[EYE_WINDOW_MIN, WINDOW]` from the face box size (constants in
         `sharpness.rs`; record the values)
      3. no face -> unchanged (`trusted_focus` window or `tile_max`)
    - The chosen face is the highest-scoring one above a confidence threshold
      (constant; record the value); ties or multiple faces are not otherwise
      handled
    - Unit tests with the existing `jpeg` / `checker` / `blur` helpers and
      hand-built `Face` values (no inference in these tests): an image sharp
      only at the eyes scores higher through the face path than through
      `tile_max` on the blurred copy; a face with the AF point inside its box
      keeps the AF window; a face with the AF point outside it moves the
      window to the eyes; no face reproduces the existing results
    - Module doc comment describes the three paths
    - `mise run ci` passes
  - Implementation approach:
    - Reuse `laplacian_variance`, `Window`, `window_at`; the eye window is
      just another `Window`
    - Because Laplacian variance depends on window size, keep the eye window
      side within the same order as `WINDOW` so a burst mixing face and
      no-face frames stays comparable (the same caveat the tile-max plan
      accepted)

- [ ] Step 3: Core + App: run the detector at scan time and recompute cached scores
  - Done when:
    - `crates/core/src/scan.rs` `extract` runs `faces::detect` on the preview
      inside the same `catch_unwind` discipline and passes the result to the
      Step 2 scoring; a detector error yields the no-face path, not a failed
      file
    - `crates/app/src/index.rs`: `SCHEMA_VERSION = 10`; `prepare` accepts 9
      and drops `files` for `version != 0 && version < 10` (the v9
      precedent), keeping `ratings` and `folders`; existing migration tests
      bumped, plus one building a v9 database and checking `files` is
      dropped and `ratings` / `folders` survive
    - `riffle-cli scan` measured before/after on the Mac (the α7 V symlink
      folder and the M11-P folder used in `docs/performance.md`, 1 and 8-12
      threads, warm cache, runs alternated) and the per-file mean / p95
      written to `learnings.md`; the 30 s / 5000-file scan target still
      holds at the machine's thread count
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged
    - The tract model is shared across rayon workers; make sure tract's
      `TypedRunnableModel` is used through `&` from every worker rather than
      rebuilt per file
    - If the scan cost is unacceptable, the fallback to note here is running
      detection at a smaller input size, not a setting to turn it off

- [ ] Step 4: Documentation
  - Done when:
    - `docs/performance.md` gains a "Face detection cost" subsection next to
      "Sharpness scoring cost" with the Step 1 latency, the binary size delta
      and the Step 3 scan before/after, in the existing table format
    - `README.md` "Sharpness cue" bullet says the score is taken on the
      subject's eyes when a face is found, else around the AF point, else the
      sharpest region; a short "Offline face detection" note names the model
      and its licence
    - `CLAUDE.md` "Layout" mentions `src/faces.rs` and the model directory
    - `todo.md`: the face/eye item's first three checkboxes are ticked with a
      one-line result each, the closed-eye item stays open, and a follow-up
      line for storing the face region / burst suggestion is added
    - `lychee` (in `mise run lint`) accepts any new links
    - `mise run ci` passes

## Trade-offs and risks

- **Runtime: `tract-onnx` vs `ort`.** `tract` is pure Rust, statically
  linked, builds on the CI matrix as-is, and adds no distribution work; it
  costs compile time and is slower per op than ONNX Runtime. `ort` is faster
  but needs ONNX Runtime binaries per OS and macOS signing of that dylib. If
  YuNet has an op `tract` does not support, or latency misses the target
  after shrinking the input, switching to `ort` is the recorded fallback and
  should be raised with the user before Step 2.
- **Model.** YuNet gives eye landmarks directly and is tiny. BlazeFace ships
  as TFLite with third-party ONNX conversions; UltraFace has no landmarks.
- **Model distribution.** Bundled via `include_bytes!`; a downloaded model
  would contradict "runs fully offline".
- **Measurements need the user's Mac.** No sample ARW/DNG and no exiftool are
  present on the Linux worktree machine; every latency/scan number in Steps 1
  and 3 must be taken on the Mac where the files are.
- **Scoring semantics change for portraits.** A frame focused on the
  background with no sharp face will now score low where it scored high
  before; that is the intent. A burst mixing face and no-face frames compares
  an eye window with an AF/tile window; noted, not addressed.
- **Grayscale input** may lower recall; Step 1 measures and decides.
- **Binary size and build time.** `tract` adds several MB and compile time;
  Step 1 records both so the user can veto.
- **Face region not stored.** A later UI mark or burst suggestion will need
  another schema bump. Deliberately deferred.

## Progress

- (2026-09-22) Step 1 complete
