# Running ONNX models with `tract-onnx` in `crates/core`

Read this before touching `crates/core/src/faces.rs` (the YuNet face / eye
detector) or adding another ONNX model to `crates/core`. It lists what loading
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
Riffle feeds it 320 px (`INPUT`), and with a different input fact tract fails
to unify the annotated shapes against the inferred ones. `build` therefore
loads with `tract_onnx::onnx().with_ignore_value_info(true)
.with_ignore_output_shapes(true)` and then pins the input with
`with_input_fact(0, f32::fact([1, 3, INPUT, INPUT]).into())`.

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

### Input layout: BGR NCHW, raw 0..255 floats, one fixed square (Hit)

`input_tensor` box-averages the image so its long edge is `INPUT` pixels and
writes it into the top-left of a zeroed `INPUT` x `INPUT` tensor of shape
`[1, 3, INPUT, INPUT]`, channels in BGR order, values as raw 0..255 floats
(not normalized), as OpenCV's `FaceDetectorYN` feeds YuNet. `detect` divides
the decoded boxes and eye points by the same scale.

- Why a fixed square: the input fact is pinned, so one optimized plan serves
  every file whatever its aspect ratio; a 3:2 preview wastes about a third of
  the input on padding, which was accepted.

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
- The AF point these coordinates meet comes from the MakerNote parsing
  described in [`raw-metadata-parsing.md`](./raw-metadata-parsing.md).

## Sharing

### Build the plan once in a `OnceLock` shared by rayon workers (Hit)

The scan runs `scan::extract` and `scan::extract_faces` on rayon workers. `detector()` builds the plan once
in a `static OnceLock` and hands every worker the same `&'static Detector`;
nothing is rebuilt per file. The lock caches a `Result<Detector, String>`
(the error formatted with `{e:#}`), so a model that fails to build fails
every call quickly with the same message instead of being rebuilt per file.

- Rule: keep one plan per model, built lazily, with its failure cached too.

Source: [face-aware-sharpness learnings](../plans/_archived/20260922-face-aware-sharpness/learnings.md),
Steps 1 and 3.
