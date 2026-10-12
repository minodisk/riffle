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
  - "Leave-one-folder-out over the 27 folders" ran 26 folds: `2026-07-23`
    (18 frames, 0 picks) has no scorable burst, so it is never held out and
    each fold fits on 25 folders with pairs. The learning curve's "26" row
    trains on 26 folders, 25 with pairs, and its 5 / 10 / 20 draws may hold
    one folder fewer with pairs than labeled.
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

## Step 3

- **Loading from the path lowers the load peak by a third.** The deferred
  "lower `Model::load`'s peak" item, measured with a throwaway switch in
  `riffle-cli embed` (12 reference previews, 1 thread, `--near 404`, run
  twice each): `Model::load` on the bytes peaked at 1145-1161 MB,
  `Model::load_path` (`model_for_path`, which maps the file) at 773 MB. The
  plan build time (538 ms) and the 395 MB after the load were the same.
  `model_for_read` copies the caller's bytes into a `Vec` of its own before
  decoding, so three copies were alive at once. The app loads through
  `load_path`; the CLI switch was reverted.
- **Pass 3 on `D:\photos\2026\2026-09-19` (2134 ARW), measured without the
  GUI.** The ignored test `embed_pass_on_a_real_folder` (`commands.rs`) runs
  the app's own `run_scan`, `run_faces_pass` and `run_embed_pass` into a
  fresh index at the app's priorities. It samples the process's working set
  every 20 ms and times a preview read + full decode on a normal-priority
  thread every 250 ms, a stand-in for the viewer's paging. Release build,
  Windows 11, i7-13700 (24 logical cores), warm page cache after the first
  run:

  | Pass 3 threads | Pass 3 wall time | ms/file | Peak working set | Preview read + decode during pass 3 (median / p95) |
  |---|---|---|---|---|
  | 22 (first run, cold) | 47.2 s | 22.1 | 812 MB | 32.0 / 39.1 ms |
  | 22 | 41.3 s | 19.4 | 745 MB | 32.8 / 37.8 ms |
  | 11 | 56.1 s | 26.3 | 800 MB | 22.1 / 28.4 ms |
  | 6 | 70.3 s | 33.0 | 796 MB | 14.6 / 23.9 ms |

  The same probe read a median of 9.9-10.2 ms (p95 12.1-15.6 ms) idle, and
  27.0-34.1 ms (p95 30.5-38.3 ms) during pass 2 on 22 threads (16.5-19.7 s,
  peak 338-356 MB). Pass 1 took 32.6 s cold. After each run
  `embeddings_missing` was empty, so a rescan embeds 0 files (the test
  asserts it). 0 errors.
- **Pass 3 runs on a quarter of the cores, not on pass 2's count.** The plan
  said pass 2's count unless memory said fewer, and "drop the thread count
  before anything else" if the latency showed interference. Memory does not
  set the count: the load sets the peak at any thread count. The latency
  does: at 22 threads the probe tripled, as during pass 2, but pass 3 runs
  for 40+ s while the user culls, after `faces-done`. `embed_threads()` =
  cores / 4 (6 here) keeps the probe at 1.5x idle for 70 s instead of 41 s.
  `Priority::Lowest` alone does not protect the viewer's thread: SMT
  siblings and the all-core clock ignore thread priority.
- **The download works end to end.** The ignored test
  `the_pinned_model_downloads_and_matches_its_hash` (`ai.rs`) fetched
  `MODEL_URL` through `reqwest::blocking` + rustls (`ring` provider, platform
  verifier) and installed it after the size and SHA-256 checks in 9.8 s.
  Hugging Face redirects the `resolve` URL to its CDN, and reqwest follows
  it. reqwest 0.13, rustls 0.23 and sha2 0.10 were already in `Cargo.lock`
  through the updater and Tauri, so the lock only gained edges.
- **Design choices beyond the plan's text:**
  - `embeddings` has a `dir` column, so `evict_folder`, `rename_dir` and
    `reconcile` handle it like `files` / `ratings`.
  - A failed embedding is stored with `vec` `NULL` and is not retried until
    the file's stat or `EMBED_VERSION` moves, as pass 2 stores unreadable
    files. Otherwise one corrupt preview would reload the 378 MB model on
    every rescan.
  - `faces-done` keeps its time, so the frontend's `scanRunning` clears
    before pass 3. Pass 3 therefore does not count as scanning
    (`ScansState.embedding`). Otherwise trash, renames and Clear Cache,
    released at `faces-done`, would be refused with "a scan is running" for
    a minute. `clear_index`, the renames and `set_ai_enabled(false)` cancel
    it (`stop_embedding`). A late batch is swept by
    `drop_stray_embeddings` (evict / clear) or by `reconcile`.
  - A switch-off between `embed_work` and marking the pass is caught by
    re-checking `aiEnabled` under the `Scans` lock, which `set_ai_enabled`
    takes after switching off.
  - A launch with `aiEnabled` on and the model missing starts the download
    again, as a later enable would.
  - Error rows (a failed first pass) are never listed: their preview is
    unreadable.
- **No unit test runs `run_embed_scan` itself.** It needs the 378 MB model.
  The ignored harness covers it on real data, and the unit tests cover the
  table, the staleness rule, the `ScansState` phase and the download's
  `install` with a local reader.
- **Heredocs with `'%.jpg'` broke the Bash tool.** A `python - <<'EOF'`
  script containing SQL `LIKE '%.jpg'` failed with `unexpected EOF while
  looking for matching '`. The Edit tool, or a script file, avoids it.

## Deferred issues (todo candidates)

- **A 190 MB download at FP32 speed is untested.** tract runs the
  weight-only fp16 re-export (`weight_only.py`) at 2-5x FP32's cost because
  it re-widens the weights on every run. Widening the float16 initializers
  in the ONNX protobuf (`tract_onnx::onnx().proto_model_for_read`, then
  `model_for_proto_model`) before tract builds the plan could keep the
  smaller download at FP32 speed. Not needed for Step 3. Source: Step 1
  measurements in `docs/plans/20261012-image-embeddings/models.md`;
  `docs/plans/20261012-image-embeddings/weight_only.py`.

- **Pending manual check: pass 3 in the real app on Windows.** Step 3's
  checkbox was ticked on the automated criteria: the headless harness
  numbers above, the download test and the unit tests. What only the GUI
  shows is unchecked. On the Windows 11 machine, with a build of this
  branch:
  1. Settings > Culling: tick "AI analysis of what each photo shows". The
     status line should count `Downloading the model: N / 378 MB` up to
     `On`. The model should then be at
     `%LOCALAPPDATA%\com.minodisk.riffle\models\siglip2-base-patch32-256\vision_model.onnx`
     (378,478,689 bytes).
  2. With `D:\photos\2026\2026-09-19` open, the folder should rescan by
     itself once the status reads `On`. `Riffle.log` should get an
     `embed model ready` and a `scan embed: dir=... files=2134 ... errors=0
     threads=6` line.
  3. While that pass runs, Task Manager should show the app's working set
     (expected about 800 MB above its usual footprint). Page through the
     strip with Timing logs on, or with the performance graph overlay. The
     page latency should sit close to its level with AI off; the harness
     proxy read 15 ms against 10 ms idle.
  4. Reload the folder (`File > Reload Folder`). No new `scan embed` line
     should appear (0 files to embed).
  5. Untick the checkbox during a pass. `Riffle.log` should show the pass
     ending with `canceled=true`. Ticking it again should finish the
     remaining files only.
  Source: Step 3 Done-when "Measured ... page latency while it runs";
  `crates/app/src/commands.rs` `run_embed_pass`, `crates/app/src/ai.rs`.
- **Pending manual check: the model download on macOS and Linux.** It was
  verified on Windows only (`the_pinned_model_downloads_and_matches_its_hash`,
  9.8 s). rustls' platform verifier uses each OS's trust store. On a Mac
  and on Linux, run `cargo test --release -p riffle-app
  the_pinned_model_downloads_and_matches_its_hash -- --ignored --nocapture`,
  or tick the setting in the app. Expected: the download completes and
  passes the hash check. Source: `crates/app/src/ai.rs` `fetch`.
- **The embedding plan stays resident after pass 3.** `embed::shared` keeps
  the SigLIP 2 plan (about 395 MB of working set) in a `OnceLock` for the
  rest of the process once one pass has run, even with `aiEnabled` turned
  off. Dropping it after a pass would cost the ~0.5 s build and the 773 MB
  load peak again on the next folder. Revisit if memory becomes a
  complaint. Source: Step 3; `crates/core/src/embed.rs` `shared`.
- **Turning AI off keeps the 378 MB model file on disk.** Nothing removes
  `models/siglip2-base-patch32-256/vision_model.onnx` from the local data
  dir; a "remove the model" action could go next to the checkbox. A cut
  download restarts from zero (no HTTP range resume). Source: Step 3;
  `crates/app/src/ai.rs`.
- **No progress for pass 3 in the UI.** `embed-progress` / `embed-done` are
  emitted but nothing listens yet. The status line shows nothing while the
  pass runs, which is fine while no feature reads the vectors (Step 4 may
  want an `embedding N / M` status). Source: Step 3;
  `crates/app/src/commands.rs` `start_scan`.

## User decisions

- (2026-10-12) The user confirmed Decision 1's distribution: SigLIP 2 FP32
  is downloaded on the first enable (pinned revision, SHA-256 checked), not
  shipped as a resource next to the binary.
