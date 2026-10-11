# Learnings: image-embeddings

## Step 1

- **Python environment.** The system Python 3.13 has no numpy / Pillow /
  onnxruntime. A venv at `%TEMP%\embedvenv` (outside the repository and the
  download directory) with `numpy 2.5.4`, `pillow 12.3.0`, `onnxruntime
  1.31.0`, `onnx` and `huggingface_hub` works; every script runs with
  `python -I` from `%TEMP%` so neither the plan folder nor a download
  directory is on `sys.path`.
- **TinyCLIP has no image-only ONNX.** onnx-community's
  `TinyCLIP-ViT-40M-32-Text-19M-LAION400M-ONNX` ships one combined graph
  (text and vision, `input_ids` / `attention_mask` / `pixel_values`).
  `extract_vision.py` cuts the `pixel_values` -> `image_embeds` subgraph
  with `onnx.utils.extract_model` (159 MB), which tract then loads.
- **PIL-exact resampling pays.** `embed.rs` reimplements PIL's
  `precompute_coeffs` (the kernel widened by the downscale factor, 8-bit
  rounding after each pass), since every candidate's Python processor
  resizes through PIL. With it the FP32 cosine against onnxruntime + PIL is
  0.999997 or better on all 12 reference previews, for every model.
- **onnx-community's int8 / uint8 SigLIP 2 exports are broken, in
  onnxruntime too.** They are dynamic quantizations
  (`DynamicQuantizeLinear` + `MatMulInteger` + `ConvInteger`) whose output
  is far from FP32's even in onnxruntime 1.31: cosine min 0.638 (int8) /
  0.663 (uint8) against the FP32 reference. In tract the int8 file does not
  load (`ConvInteger`: `Impossible to unify U8 with I8`, U8 activations
  against I8 weights), and the uint8 file loads but runs at 8.4 s per image
  (one thread) against 129 ms for FP32: tract's integer matmul path is not
  usable here.
- **onnx-community's fp16 export does not load in tract 0.23.** The
  translation of the attention-pooling head's `Gemm` fails with `Output
  mismatch after rewiring expansion for output #0: expected 1,768,F16 got
  1,768,F32`. onnxruntime runs it at cosine 0.999993 against FP32.
- **Weight-only re-exports load and are accurate, but tract runs them at
  half the FP32 speed.** `weight_only.py` stores the 78 MatMul / Gemm / Conv
  weights as int8 (per-output-channel `DequantizeLinear`, 96 MB) or float16
  (`Cast`, 190 MB) with FP32 compute. Both load with `default-features =
  false`; cosine against the FP32 reference is min 0.99905 (int8) and
  0.999998 (fp16) on the 12 previews. But tract does not fold the widening
  into the packed constant weights: 256-259 ms per image against 129 ms for
  the FP32 file (one thread), with a lower peak working set (332 / 580 MB
  against 1145 MB), so it re-widens the weights on every run.
- **Peak working set of the FP32 file is dominated by loading.** A
  12-file single-thread run of the 378 MB SigLIP 2 file peaks at 1145 MB:
  the file's bytes, the parsed protobuf and the optimized plan's packed
  weights are alive together during `Model::load`.
- **The stored thumbnail is not a usable embedding input, because of its
  q80 re-encode.** On 214 files of `2026-09-19` the thumb (`scan::extract`)
  against the full preview read SigLIP 2 cosine mean 0.961, top-5 agreement
  0.519; OpenCLIP and TinyCLIP 0.91 / 0.60. Suspecting a rotation or
  decoding bug, `thumb_check.py` reproduced it independently (onnxruntime +
  PIL): a 404 px box downscale alone keeps SigLIP 2 at cosine mean 0.9991,
  and a q80 4:2:0 re-encode drops it to 0.9424 (q80 4:4:4: 0.9913; q95
  4:2:0: 0.9852). The gap does not depend on the orientation.
- **The DCT-scaled preview decode is the cheap input that works.**
  `riffle-cli embed --near 404` (`decode::decode_upright_near`, no re-encode)
  reaches SigLIP 2 cosine min 0.993 / mean 0.998, top-5 agreement 0.843
  against the full preview, at 9.0 ms decode against 26.1 ms (24 threads).
  Added to the CLI during the step to give Step 3 a measured input instead of
  just "decode the preview".
- **Top-5 agreement within one shooting day is strict.** SigLIP 2's
  weight-only int8 file, at cosine mean 0.999375 against FP32, agrees on the
  top 5 at 0.890 only: the frames of one day are close together, so small
  perturbations reorder near ties.
- **Memory: the load, not the run, sets the peak.** `embed` prints the
  working set after `Model::load` and the largest one sampled every 10 ms
  during the run, because the process peak (1145 MB for SigLIP 2 FP32) is
  set while loading. The run itself holds 416 MB on one thread and 637 MB
  on 24 threads.
- **`include_bytes!` measured.** A throwaway build that embedded the
  SigLIP 2 FP32 file grew `riffle-cli` by exactly the file's size (+378.5
  MB), and its working set after load was 774 MB against 395 MB from a file.
  The build was reverted before committing.
- **Windows working copy.** The worktree checks files out with CRLF.
  Python edits with `open(..., 'w')` keep it consistent only by accident (the
  text mode writes CRLF), so `newline=''` is the safe way to patch a file in
  place. `thumb_check.py` imports `ref_embed.py` and so sets
  `sys.dont_write_bytecode`, otherwise `-I` still writes a `__pycache__`
  into the plan folder.

## Step 2

- **Pre-registration as its own commit.** `gate.md`'s criterion section was
  committed alone (`ce0ea14`, 2026-10-12T08:12:18+09:00) before `gate.py`
  existed, so the history shows it predates the fit. Its first line says
  "08:15", the planned time; the commit time is the record. Nothing in that
  section changed afterwards.
- **The gate passed narrowly.** Weighted pairwise AUC 0.708 against the
  0.703 bar; Top-1 +15.0 points over the first frame against an SD of 8.7.
  Without the embedding the model reads 0.652, level with the first frame,
  so the embedding is what carries the gain. Per-genre offsets gave nothing
  (0.707), with 1,478 of 1,801 scorable bursts in "sports".
- **Small deviations from the pre-registered text, none touching the
  criterion:**
  - The random baseline uses seed 1, not seed 0. Seed 0 went to the Top-3
    tie-break vector, and reusing it would have made the random ranking and
    the tie-break the same draw. Random is not in the criterion (0.508
    weighted AUC).
  - The inner cross-validation that picks `C` reuses the outer fold's
    imputation, standardization and PCA, which are fit on every training
    folder, instead of refitting them per inner fold. They are unsupervised
    and never see the held-out folder. Refitting would have multiplied the
    PCA fits by 20.
  - Each pair enters once, as `(d, 1)` or `(-d, 0)` alternately, with
    double weight, instead of as both rows. Without an intercept the logistic
    loss of `(-d, 0)` equals that of `(d, 1)`, so the objective is the same
    and the rows are halved.
  - `photoTier` reads the `features` TSV's `ear` (the eyes path's judged
    face, 60 px or more), not pass 2's stored `eyes_ear` of the cue face.
    The app's rule reads the latter. Both are the same face in the common
    case.
  - The `eye_focus` baseline ranks by the model's logit input column, whose
    probability is clipped to [0.001, 0.999], not by the raw `eye_focus`.
    Values at or above 0.999 (or at or below 0.001) therefore tie and count
    0.5, which departs from the pre-registered "`eye_focus` (missing ranks
    last)". The reported 0.564 / Top-1 37.1% carries this; the baseline is
    outside the criterion and was not re-run. The Sharpness baseline's
    `max(ratio, 1e-3)` clip has the same effect, negligibly.
- **Dump cost.** `features` took 1 to 84 s per folder on 24 threads, about
  11 minutes for the 27. `embed --near 404` took 611.6 s for 32,459 files
  (18.8 ms per image of wall time, peak working set 1148 MB, the
  `Model::load` peak again). One `fork: retry: Resource temporarily
  unavailable` from Git Bash after `2026-08-01` did not lose anything: all 27
  TSVs are complete (0 errors) and 27 `.output.txt` listings exist.
- **SigLIP 2 text tower.** `onnx/text_model.onnx` (1.13 GB FP32) takes
  `input_ids` only and outputs `pooler_output`. The tokenizer settings come
  from `tokenizer_config.json`: lowercase, no BOS, EOS appended, right-pad
  with `<pad>` to 64. `exp(logit_scale)` = 111.83 and `logit_bias` =
  -16.77 are not in the ONNX files. They were read from the first bytes
  after the header of `google/siglip2-base-patch32-256`'s
  `model.safetensors` (`HfFileSystem`, seekable mode), without downloading
  the 1.5 GB file. The sigmoid scores are tiny for all but the best label
  (cosines 0.05 to 0.15), which is why the gate's genre feature is a softmax
  over the top-level genres.
- **The provisional prompts call 88% of the frames "sports".** 28,444
  frames go to sports, 3,681 to snap and 334 to everything else. Step 5's
  calibration starts from a heavily skewed set, and these folders hold
  almost no landscape, animal or macro frames to calibrate on.
- **The Bash tool rewrites backslashes in heredocs.** A `python - <<'EOF'`
  body with `D:\\Photos\\tests\\2026...` reached Python as `\t` and `\201`
  escapes and wrote control characters into `gate.md`. Write such a script
  with the Write tool to the scratchpad and run it, or avoid backslashes.
- **Blocking waits stall the agent stream.** A single tool call that waits
  several minutes for the dump tripped the 600 s stream watchdog. Long
  steps run in the background, polled by short bounded loops.

## Deferred issues (todo candidates)

- **Lower `Model::load`'s peak.** It holds about 3x the model file at once
  (the bytes, the parsed protobuf, the optimized plan: 1145 MB for the 378 MB
  SigLIP 2 file). Loading through `model_for_path` or dropping the bytes
  before optimizing may lower it, so measure it in Step 3. Source: Step 1
  `riffle-cli embed` working-set lines; `crates/core/src/embed.rs`
  `Model::load`.
- **A 190 MB download at FP32 speed is untested.** tract runs the
  weight-only fp16 re-export (`weight_only.py`) at 2-5x FP32's cost because
  it re-widens the weights on every run. Widening the float16 initializers
  in the ONNX protobuf (`tract_onnx::onnx().proto_model_for_read`, then
  `model_for_proto_model`) before tract builds the plan could keep the
  smaller download at FP32 speed. Not needed for Step 3. Source: Step 1
  measurements in `docs/plans/20261012-image-embeddings/models.md`;
  `docs/plans/20261012-image-embeddings/weight_only.py`.

## User decisions

- (2026-10-12) The user confirmed Decision 1's distribution: SigLIP 2 FP32
  is downloaded on the first enable (pinned revision, SHA-256 checked), not
  shipped as a resource next to the binary.
