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

## Deferred issues (todo candidates)

- Release binary size grows by ~30 MB with tract (Linux measurement, Step 1,
  `crates/core/Cargo.toml`, `crates/core/src/faces.rs`). If the user vetoes
  it, try `ort` or trimming tract features before Step 3 wires it into the
  app.
