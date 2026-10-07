# Running ONNX models with `tract-onnx` in `crates/core`

Read this before touching `crates/core/src/faces.rs` (the YuNet face / eye
detector), `crates/core/src/eyes.rs` (the MediaPipe Face Landmarker v2 face
mesh that judges closed eyes) or adding another ONNX model to `crates/core`. It lists what loading
and running a model through `tract-onnx` 0.23 took here, each with the reason.

The tags follow [`tauri-app.md`](./tauri-app.md): **Hit** broke something
here, **Measured** steered a design decision, **Inferred** comes from sources
or docs only.

The measurements behind these items (binary size and build time with tract,
per-call latency, the model build time) are in the
[face-aware-sharpness learnings](../plans/_archived/20260922-face-aware-sharpness/learnings.md),
Step 1 and "App binary size"; they are not repeated here.

## Loading

### Ignore the model's fixed-size shape annotations and pin the input (Hit)

The YuNet ONNX file carries `value_info` and output shapes for a 640 px input.
Riffle feeds it 320x320 (`INPUT`) and 640x448 / 448x640 (`WHOLE_INPUT`), and
with a different input fact tract fails to unify the annotated shapes against
the inferred ones. `build` therefore loads with
`tract_onnx::onnx().with_ignore_value_info(true)
.with_ignore_output_shapes(true)` and then pins the input with
`with_input_fact(0, f32::fact([1, 3, height, width]).into())`.

- Rule: when a model's annotations are for another input size than the one
  you feed, drop the annotations rather than resizing to match them, and
  always pin the input fact so the plan is fully typed before optimizing.

### Find outputs by outlet label, not by node name (Hit)

After loading, tract names the output nodes after their ops (`Sigmoid_85`,
`Reshape_132`, ...). The ONNX output names (`cls_8`, `obj_8`, `bbox_8`,
`kps_8`, then `_16` and `_32`) survive only as outlet labels. `build` maps
each `{kind}_{stride}` name to its output index through
`model.outlet_label(outlet)` over `model.output_outlets()` and fails with the
list of labels when one is missing.

- Rule: never index outputs by position or node name; resolve them by outlet
  label once at build time and keep the indices (`Detector::outputs`).

### The tract 0.23 API (Hit)

- `model.into_optimized()?.into_runnable()?` returns
  `Arc<TypedRunnableModel>` (the `Plan` alias in `faces.rs`); `run` takes
  `&Arc<Self>`.
- Run with `plan.run(tvec!(input.into()))?`.
- Read an output `TValue` with `try_as_plain_ram()?.as_slice::<f32>()`.

### `default-features = false` on `tract-onnx` (Measured)

`crates/core/Cargo.toml` depends on `tract-onnx` with
`default-features = false`, which drops `tract-transformers`: YuNet loads
and runs without it, and the binary and the build are smaller. Check a
new model's ops load with this setting before turning the defaults back on.

## Input and output

### Input layout: BGR NCHW, raw 0..255 floats (Inferred)

`input_tensor` box-averages the image by the scale that fits it inside the
model input (`fit`) and writes it into the top-left of a zeroed tensor of
shape `[1, 3, height, width]`, channels in BGR order, values as raw 0..255 floats
(not normalized), as OpenCV's `FaceDetectorYN` feeds YuNet. `detect` divides
the decoded boxes and eye points by the same scale.

- Why fixed sizes: the input fact is pinned, so each optimized plan serves
  every file whatever its aspect ratio. The `CATCH_CROP` crop is square, so
  its 320x320 input wastes nothing. The whole-image search uses a 640x448
  input and its transpose for portrait frames instead of a 640 square: a
  3:2 preview would leave a third of the square as padding, and the
  inference cost follows the input area (Measured: 59.6ms against 85-87ms
  per run, same recall; see "Whole-image recall (Windows 11)" in
  [`performance.md`](../humans/performance.md)).
- The grid of each stride is `width / stride` cells per row
  (`decode_stride`'s `cols`), so a non-square input decodes with the input's
  width, not its height.

### Feed an upright image, map detections back to stored coordinates (Hit)

YuNet is trained on upright faces: an unrotated portrait preview (orientation
6 / 8) loses most of its recall and swaps left / right eyes for top / bottom.
The preview is decoded as stored, so `detect_around_rgb` rotates it with
`upright_rgb` before `detect`, maps the AF point into it with `to_upright`,
and maps every face back with `to_stored`, because the sharpness score and the
AF point (`partial::focus_point`) live in stored coordinates.

- `decode::apply_orientation` only rotates 6 and 8, so `upright_rgb` reverses
  the pixel order itself for orientation 3, and `to_stored` / `to_upright`
  treat 3 as a half turn. A new orientation-aware caller must do the same or
  reuse these helpers.
- The whole-image search decodes at a DCT scale instead (`decode_whole`,
  `decode::decode_upright_near`, already upright) and `scaled_to_stored`
  scales each face by the decoded size against the stored one before
  `to_stored`. mozjpeg rounds a scaled side up (`ceil(dim * n / 8)`), so map
  with the actual sizes, not `n / 8`, and keep `Detection::width` / `height`
  at the stored size: `faces_of` and the UI scale the boxes by it.
- The AF point these coordinates meet comes from the MakerNote parsing
  described in [`raw-metadata-parsing.md`](./raw-metadata-parsing.md).

### Do not swap the cue path's full RGB decode for a cheaper grayscale + scaled decode (Measured)

The cue path (`detect_around_rgb` plus `candidate::luma`) decodes the preview
once at full size, which looks wasteful next to the whole-image search's DCT
scaled decode. A variant that took a grayscale decode for the luma and a 6/8
RGB decode for the face search was faster (11.6 ms against 13.6 ms per file)
but changed the results: 16 of the training set's per-file candidate states
changed and 6 of the held-out set's, with AUC 0.816 -> 0.819 and 0.635 ->
0.646. mozjpeg's grayscale is the Y channel, not the BT.601 integer luma of
the RGB, and the scaled crop's pixels differ.

- Rule: the cue path keeps its single full RGB decode. Only a change that
  passes an identity check of `candidates` on the labeled folders (identical
  per-file lines) may replace it, otherwise it needs a `FACES_VERSION` bump
  and a re-evaluation of the cue's threshold.
- Source: [face-detection-recall-cost learnings, Step 3 "Cue-path variant"](../plans/_archived/20261005-face-detection-recall-cost/learnings.md).

### The face mesh: RGB NHWC 0..1, points by `Identity` (Measured)

`eyes.rs` runs the second model, MediaPipe Face Landmarker v2
(`face_landmarks_detector.onnx`), and it is the first NHWC model here: its
input fact is `[1, 256, 256, 3]` (channels last), RGB, values 0..1, against
YuNet's `[1, 3, height, width]` BGR raw 0..255. `input_tensor` resizes the
face crop bilinearly to 256x256 and builds the `Array4` with the shape
`(1, INPUT, INPUT, 3)`. The layout and range come from MediaPipe's pipeline
config (`face_landmark_cpu.pbtxt`: `output_tensor_float_range 0..1`), not
from the ONNX file; a wrong channel order or range is not an error, it only
shows in the accuracy, so check a new model's input against its pipeline
and a labeled set, not only whether it loads.

- Outputs by outlet label: `Identity` is the 478 points, (x, y, z) each, x
  and y in input pixels (map them back by the crop window's position and
  per-axis scale, since a clamped crop near the image edge is not square);
  `Identity_1` is the face presence logit and `Identity_2` is unused. Only
  `Identity` is resolved (`Model::points`).
- The file is a tf2onnx 1.17.0 conversion of Google's TFLite (provenance in
  `crates/core/models/LICENSE-mediapipe`); tf2onnx widens the float16
  weights to float32, so the ONNX is about twice the TFLite (4.9 MB).
  tflite2onnx 0.4.1 failed on the face meshes (an `IndexError` in its layout
  propagation) (Hit).
- It loads and optimizes with `default-features = false` and the same
  `with_ignore_value_info(true)` / `with_ignore_output_shapes(true)` as
  YuNet (Measured).
- The first call builds the plan: 156-169 ms for the first call against
  ~35 ms for a later one on the Windows 11 machine, so ~120-135 ms of plan
  build, three to four times YuNet's (Measured). A caller that times
  per-face runs warms it first, as `riffle-cli eyes` and `bench` do.
- Per face, crop and resize included: 34.5-35.5 ms mean over the 504
  labeled faces, one thread (Measured). The Step 1 survey measured 49-51 ms
  for the same function while it ran three other models on each face in
  between; treat the lower figure as the model's own cost.
- Source: [closed-eyes-detection learnings](../plans/_archived/20261007-closed-eyes-detection/learnings.md),
  Steps 1 and 2.

### An embedded model only weighs on the binaries that call it (Measured)

`include_bytes!` puts the model in a `static`; the linker drops it from a
binary that never reaches the code reading it. Adding `eyes.rs` grew the
release `riffle-cli` (whose `eyes` and `bench` call it) by 4.97 MB and the
release `riffle-app` (which did not call it yet) by 8 KB. Measure a model's
binary cost on a build where the app actually calls it.

Once `eyes_of` called it, the release `riffle-app` grew from 47,499,776 B to
52,494,336 B (+4,994,560 B, +10.5%), about the ONNX file, so a model's full
size lands on every binary that reaches it. Source:
[closed-eyes-detection learnings, Step 3](../plans/_archived/20261007-closed-eyes-detection/learnings.md).

## Sharing

### Build each plan once in a `OnceLock` shared by rayon workers (Inferred)

The scan runs `scan::extract` and `scan::extract_analysis` on rayon workers. `detector(input)` builds the plan
of each model input (`faces::Input`: the square, the landscape and the
portrait whole-image input) once in its own `static OnceLock` and hands every
worker the same `&'static Detector`;
nothing is rebuilt per file. The lock caches a `Result<Detector, String>`
(the error formatted with `{e:#}`), so a model that fails to build fails
every call quickly with the same message instead of being rebuilt per file.

- Rule: keep one plan per model input, built lazily on first use, with its
  failure cached too. A plan is ~35-40ms to build and is not per worker;
  the larger input's per-run buffers are (Measured: the 146-DNG folder's
  peak working set on 24 threads rose from ~510 to ~646 MB with the
  640x448 plans).

Source: [face-aware-sharpness learnings](../plans/_archived/20260922-face-aware-sharpness/learnings.md),
Steps 1 and 3.
