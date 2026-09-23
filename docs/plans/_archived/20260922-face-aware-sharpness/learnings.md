# Learnings: face-aware-sharpness

## Step 1: face/eye detector prototype

- tract 0.23 loads YuNet 2023mar and every op is supported; no fallback to
  the 2022mar variant or `ort` was needed.
- The ONNX file carries `value_info` and output shapes for a 640x640 input.
  With a different input fact tract fails to unify (`Conv_0`: 320 vs 160),
  so the loader sets `with_ignore_value_info(true)` and
  `with_ignore_output_shapes(true)`.
- After loading, the output nodes are named `Sigmoid_85`, `Reshape_132`, ...;
  the ONNX output names (`cls_8`, `obj_8`, `bbox_8`, `kps_8`, ...) are the
  outlet labels (`model.outlet_label`), not the node names.
- tract 0.23 API: `into_runnable()` returns `Arc<TypedRunnableModel>`
  (`run` takes `&Arc<Self>`); an output `TValue` is read with
  `try_as_plain_ram()?.as_slice::<f32>()`.
- Input size chosen: a fixed 320x320 square; the preview is box-averaged so
  its long edge is 320 px and padded bottom/right with zeros. A fixed size
  lets the plan be built once in a `OnceLock`; a 3:2 preview wastes about a
  third of the input on padding. Input is BGR NCHW, raw 0..255 floats, as
  OpenCV's `FaceDetectorYN` feeds it. Thresholds: score 0.6, NMS IoU 0.3
  (OpenCV's defaults are 0.9 / 0.3; 0.6 is kept low for the prototype and
  Step 2 applies its own confidence threshold).
- RGB input was chosen (`detect(rgb, w, h)`); the grayscale-replicated
  variant was not measured here since there are no real previews on this
  machine. Recall on grayscale is still to be compared on the Mac.
- `detect` requires an upright input: YuNet is trained on upright faces, so
  callers (the CLI `faces` and `bench` commands, and Step 2/3) must call
  `decode::apply_orientation` before `faces::detect`, not after drawing.
  Passing a raw, un-rotated preview drops recall sharply on portrait frames
  (orientation 6/8) and mislabels eye left/right as top/bottom.
- `default-features = false` on `tract-onnx` drops `tract-transformers`
  (32.8 MB -> 31.3 MB `riffle-cli`, and less to compile).

### Measurements on the Linux worktree machine (WSL2, 24 threads)

- Release `riffle-cli` size: 1,704,680 B before, 31,339,376 B after
  (+29.6 MB, unstripped; tract-linalg's kernels dominate).
- Cold `cargo build --release -p riffle-cli` (fresh target dir): 10.8 s
  before, 140 s after. Rebuild after touching only riffle-core: ~8 s.
- Synthetic latency (single thread, release, 30 runs after one warm-up; the
  OpenCV sample images `lena.jpg` and `messi5.jpg` upscaled to a 1616 px long
  edge, so this includes the box downscale): 1616x1616 mean 18.2 / median
  17.8 / p95 21.5 ms; 1615x1008 mean 17.1 / median 16.9 / p95 18.7 ms. The
  first call (model build) took 46 ms. Both images gave one face with the
  eyes inside the box. Under the 30 ms target, pending confirmation on the
  Mac.

### Pending on the user's Mac (not done in this step)

- `riffle-cli bench` on the α7 V ARWs and M11-P DNGs: the
  `4. face detection` mean / median / p95.
- `riffle-cli faces` on real frames to check the boxes and eye points by eye.
- exiftool on a real ARW / DNG to record whether the MakerNotes expose
  face/eye-AF positions (Sony `Tag2010` / `Tag9405`).
- Release binary size and build time on macOS.

## Step 2: eye-window scoring

- Signature chosen: `score_preview(preview, focus, faces: &[Face])` rather
  than a `Subject` enum; `scan::extract` passes `trusted_focus(..)` and the
  detector output unchanged, and the routing stays inside `sharpness.rs`.
  `scan::extract` passes `&[]` until Step 3 wires the detector.
- Constants: `FACE_CONFIDENCE = 0.8` (above `faces::SCORE_THRESHOLD` 0.6),
  `EYE_WINDOW_MIN = 128`; the eye window side is the face box's long side
  clamped to `[128, WINDOW = 256]`, centered on the midpoint of the two eyes.
- "Focus inside the face" maps the `FocusLocation` to preview pixels with
  `partial::focus_point` and tests it against the face box (edges inclusive).
- Coordinate spaces: `score` decodes the preview as stored (no orientation
  applied), which is also the space `focus_point` maps into. `faces::detect`
  must run on the upright image, so Step 3 has to map each `Face` (box and
  eyes) back to stored-preview coordinates before calling `score_preview`
  for orientation 6/8 frames. The 4-neighbor Laplacian variance over a
  square window is rotation invariant, so scoring in stored coordinates is
  equivalent.

## Step 3: detector at scan time

- `scan::extract` decodes the preview to RGB (full size), makes it upright,
  runs `faces::detect`, and maps each face back with the new
  `faces::to_stored(face, orientation, stored_w, stored_h)` before
  `score_preview`. A decode/detect error or panic is the no-face path.
- `decode::apply_orientation` only rotates 6/8; for orientation 3 the scan
  reverses the pixel order itself so the detector sees the upright image,
  and `to_stored` handles 3 as a half turn.
- The detector's `OnceLock` plan is shared by every rayon worker through
  `&'static`; nothing is rebuilt per file.
- `SCHEMA_VERSION = 10`; v2 to v9 drop `files` and keep `ratings` / `folders`.

### Pending on the user's Mac (not done in this step)

- `riffle-cli scan` before/after on the a7 V and M11-P folders (1 and 8-12
  threads, warm cache, alternated): per-file mean / p95 and whether the
  30 s / 5000-file target still holds. No real ARW/DNG exists on this Linux
  machine, so no numbers were taken; this Done-when item is unmet.
- The extra full-size RGB decode per file is a likely cost; if the scan is
  too slow, decode at a DCT scale (the detector only needs a 320 px long
  edge) before shrinking the model input.

## App binary size (measured by the main agent during Step 1)

- `cargo build --release -p riffle-app` on Linux WSL2, unstripped, same
  machine: `origin/main` before Step 1 (built in a temporary worktree) was
  26,766,448 B (26.8 MB); the Step 1 branch was 47,829,432 B (47.8 MB),
  +21.1 MB. The app did not call the detector yet; tract and the model came
  in through `riffle-core`. The user accepted this size.
- Step 4's review dropped this row from `docs/performance.md` as unsourced,
  because it had not been written here. It is sourced now and can go back.

## Deferred issues (todo candidates)

- Release binary size grows by ~30 MB with tract (Linux measurement, Step 1,
  `crates/core/Cargo.toml`, `crates/core/src/faces.rs`). If the user vetoes
  it, try `ort` or trimming tract features before Step 3 wires it into the
  app.
- Mapping a `Face` from upright to stored-preview coordinates: done in Step 3
  (`faces::to_stored`).
- Scan before/after measurement on the Mac is still pending (Step 3,
  `crates/core/src/scan.rs`); a scaled RGB decode for detection is the first
  lever if the scan cost is too high.
- New guide `docs/agents/tract-onnx-inference.md` (from wrap-up learnings
  extraction). Change: write a guide for loading and running `tract-onnx`
  models in `crates/core`, covering `with_ignore_value_info` /
  `with_ignore_output_shapes` for fixed-size shape annotations, outputs found
  by outlet label rather than node name, the tract 0.23 `Arc<TypedRunnableModel>`
  / `try_as_plain_ram` API, `default-features = false`, upright input plus
  mapping back to stored coordinates, and a `OnceLock`-shared plan. Why: no
  existing guide covers core inference code. Done when: the guide exists and
  is linked from `CLAUDE.md` or `docs/agents/`.
- Restore the `riffle-app` binary-size row in `docs/performance.md`
  "Face detection cost" (26.8 MB -> 47.8 MB, Linux WSL2, unstripped), now
  sourced in "App binary size" above. Done when: the row is back with its
  environment stated.

## Step 4: documentation

- `docs/performance.md` records only the Linux WSL2 numbers (synthetic
  latency, unstripped binary sizes, cold build time); the real-file latency
  and scan before/after are marked as not yet measured. In `todo.md` the
  MakerNote checkbox stays open and a separate open item tracks the Mac
  measurements.
