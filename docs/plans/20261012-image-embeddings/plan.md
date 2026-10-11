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

# Local image embeddings: similarity search, genre classification, personal ranking

## Purpose

Riffle's analysis today is technical: the AF eye's in-focus probability, the
eyes' openness, the head pose and the sharpness score (pass 2,
`crates/core/src/scan.rs`, `crates/app/src/index.rs` `run_faces_scan`). It
knows nothing about what a photo shows. One local CLIP-style image embedding
per file, computed once in the background and cached in the index, gives
three features that share it:

- **Visual similarity**: "show me the frames that look like this one" across
  a folder, independent of the time-based bursts (`crates/app/ui/src/burst.ts`,
  `BURST_GAP_MS` 1000, untouched), and the visual groups built from it.
- **Genre classification**: multi-label genres (sports, portrait, landscape,
  birds / animals, snap, macro, other, with sub-genres) from precomputed
  prompt text vectors, with a calibrated `Unknown`, as a filter menu
  section.
- **Personal best-shot ranking**: a small linear model learned from the
  user's own picks, with per-genre weights when they help, shown as an
  ignorable recommendation and a sort order, never changing a star or a flag
  by itself.

Constraints the user set, binding on every step:

- Input is the embedded JPEG only, never a RAW develop; everything runs
  locally; RAW files are never modified; recommendations never change a
  judgment.
- Background analysis takes the on-screen files first through the shared
  `ScanFocus` and must not slow browsing or judging; results are cached and
  not recomputed on rescan unless the file's content (size / mtime) or the
  model version changed.
- Only RAW folders are embedded (as pass 2); JPEG-only view-only folders are
  skipped.
- Every AI feature can be turned off by one setting (`aiEnabled`, persisted
  like `mcpEnabled` in `crates/app/src/commands.rs`), and nothing it adds is
  in the way when ignored.
- Staged and verifiable on real data; no abstraction before its second
  user.

### What is known

- **Model.** MobileCLIP is **excluded from shipping**: its weights are under
  the Apple Machine Learning Research Model license (research use only;
  `https://raw.githubusercontent.com/apple/ml-mobileclip/main/LICENSE_MODELS`).
  It may be measured as a reference point only. Primary candidate: SigLIP 2
  ViT-B/32 @256 (Apache-2.0, `onnx-community/siglip2-base-patch32-256-ONNX`,
  768-d, IN-1k zero-shot 74.0, image tower about 378 MB FP32 / 96 MB int8).
  Compared against OpenCLIP ViT-B/32 (MIT; DataComp.XL 72.7 needs an
  `open_clip` export, or the laion2b export
  `immich-app/ViT-B-32__laion2b-s34b-b79k`) and TinyCLIP ViT-40M/32 (MIT,
  59.8, smaller). **Text encoders are not shipped**: the genre prompt
  vectors are precomputed offline (the SigLIP 2 text tower is 1.13 GB with a
  Gemma tokenizer). No free-text search.
- **Inference** stays on `tract-onnx` 0.23 with `default-features = false`
  (`crates/core/Cargo.toml`). In tract's source, LayerNormalization, Gelu,
  Erf, Softmax, MatMul, MatMulInteger and DynamicQuantizeLinear are
  registered without `tract-transformers`; only the ORT fused
  `Attention` / `MultiHeadAttention` ops are gated, so use plain exports.
  Follow [`tract-onnx-inference.md`](../../agents/tract-onnx-inference.md):
  ignore `value_info`, pin the input fact, resolve outputs by outlet label,
  one `OnceLock` plan shared by the rayon workers (`faces.rs`, `eyes.rs`),
  and `include_bytes!` lands on every binary that links the module.
- **Input image.** The index stores `files.thumb`, a 404x270 (long edge
  404) JPEG at `THUMBNAIL_QUALITY` 80, unrotated (`scan.rs`). Embedding it,
  rotated by the row's orientation, avoids rereading the RAW; Step 1
  measures how far it is from embedding the full preview.
- **Index.** `SCHEMA_VERSION` 19, `EXTRACTOR_VERSION` 13, `FACES_VERSION` 9
  (`index.rs`); migrations run on `user_version` with ranged `ALTER` blocks.
  `write_batch` does `INSERT OR REPLACE` of whole `files` rows, so an
  embedding column on `files` would be wiped by every pass-1 rewrite; the
  vectors get their own table. Eviction (`evict_folder`, `clear`,
  `used_bytes`), `reconcile`, `rename_file` and `rename_dir` must carry
  the new table.
- **Scan.** `start_scan` (`commands.rs`) runs `run_scan` then
  `run_faces_pass` and emits `faces-done`; both share `ScanFocus` and the
  cancel flag; priorities are `scan::Priority` (`BelowNormal` for pass 1,
  `Lowest` for pass 2).
- **Labels for the ranking.** Of about 33k labeled frames only 58 are
  explicit rejects ([burst-keep-score data.md](../_archived/20261008-burst-keep-score/data.md)).
  Approved reading: in a folder marked "culling finished", a non-pick is a
  "not chosen" negative, kept distinct from an explicit reject (which, when
  present, counts as a stronger negative, also in unfinished folders); an
  unjudged frame in an unfinished folder is never a negative. A burst whose
  frames are all picked yields no pairs. For the historical set, the 27
  sidecar-labeled ARW folders under `D:\photos\2026\` count as finished.
- **Prior evidence the ranking gate must beat.** burst-keep-score
  Decision 1: within-burst pairwise AUC on picks is 0.653 for "first
  frame", 0.577 sharpness, 0.564 `eye_focus`, 0.542 eyes open, 0.495
  random; picks within a burst are timing-driven. af-eye-good-bad-cascade
  Decision C: sharpness follows the texture under the AF point. The
  embeddings' hypothesis is that they carry composition, expression and
  scene that the technical features do not.
- **Tooling.** `riffle-cli features <dir>... [threads]` (`crates/cli/src/main.rs`)
  dumps every technical feature, the flags and the capture time per file as
  TSV; the burst-keep-score folder holds the Python it was analyzed with
  (`fit.py`, `metrics.py`, `keep.py`). Python 3.13 is on the machine.
  Manual and measurement artifacts go under `D:\Photos\tests\<date>-<topic>\`;
  downloaded models go in their own new directory under it and are
  untrusted data; Python that reads them runs with `python -I` from a
  scripts directory outside the download directory.

### Architecture

- `crates/core/src/embed.rs`: `embed(rgb, width, height, orientation) ->
  Result<Vec<f32>>` (L2-normalized, `D` = the model's width), a `OnceLock`
  plan, model-specific preprocessing (resize, mean / std, channel order),
  pinned input fact. In Step 1 the model bytes come from a path the CLI
  passes; from Step 3 the model and its LICENSE live under
  `crates/core/models/` or are downloaded, per the distribution Decision.
- Index table `embeddings(path TEXT PRIMARY KEY, size INTEGER, mtime_ns
  INTEGER, version INTEGER, vec BLOB)`; a row is valid when `version ==
  EMBED_VERSION` and `size` / `mtime_ns` equal the `files` row's. About 3 KB
  per file as f32.
- Pass 3 `run_embed_pass` in `start_scan` after `run_faces_pass` and before
  the final `faces-done`, sharing `ScanFocus` and cancel, at
  `Priority::Lowest`, possibly fewer threads for memory; `embed-progress` /
  `embed-done` events; skipped when `aiEnabled` is off.
- Consumers derive everything on demand from the stored vectors: genre =
  dot products with the precomputed label vectors (backend), similarity =
  brute-force cosine over the folder (backend), ranking = a small linear
  model over the vector and the pass-2 features, with per-genre weight
  offsets if Decision 2 keeps them. Editing genres or retraining never needs
  a rescan.

## Steps

- [x] Step 1: `riffle-cli embed` and the `embed.rs` prototype: load the candidate models in tract and measure accuracy, speed, memory and the thumb-vs-preview agreement
  - Done when:
    - `crates/core/src/embed.rs` exists with `embed_with(model: &Model,
      rgb, width, height, orientation) -> Result<Vec<f32>>` (name free)
      and a `Model::load(bytes: &[u8], spec: &ModelSpec)` where `ModelSpec`
      carries the input size, the mean / std, the channel order and the
      output outlet label of one of the candidates. No `include_bytes!`
      and no `OnceLock` yet: the CLI passes the model path, so no 100-400
      MB file is committed before the distribution Decision. Loading is
      `tract_onnx::onnx().with_ignore_value_info(true)` + pinned input
      fact + `into_optimized().into_runnable()`, as in `faces.rs`.
    - `riffle-cli embed <model.onnx> <spec> <dir|file>... [threads]`
      prints one line per file with the L2-normalized vector (or writes
      a TSV of `path\tvec...` to a given out file), the plan build time,
      the warm ms/image and the peak working set, and
      `riffle-cli embed-compare <a.tsv> <b.tsv>` prints the per-file
      cosine between two dumps and its min / mean plus the top-k (k = 5)
      retrieval agreement between the two dumps. (Exact subcommand shape
      free; keep the usage string in `main.rs` current.)
    - Measured and recorded in `models.md` in this plan folder, for
      SigLIP 2 B/32 @256 FP32 and its int8 / fp16 exports where the HF
      repo has them, OpenCLIP ViT-B/32 (one export), TinyCLIP ViT-40M/32,
      and MobileCLIP as the reference point only, each explicitly marked
      ship / no-ship by license:
      - Loads with `default-features = false` (which ops failed, if any,
        and whether a plain re-export fixes them).
      - Cosine against a Python reference (`onnxruntime` or the PyTorch
        model) on at least 10 real previews from `D:\photos\2026\*`:
        min >= 0.999 for FP32; the int8 / fp16 minimum recorded.
      - Warm ms/image at 1 thread and at the machine's thread count
        (about 22) on the Windows 11 machine; plan build time separately.
      - Peak working set during a 200-file run; the model file's size and
        its effect on the `riffle-cli` binary if it were `include_bytes!`'d
        (estimate from the file size is acceptable; measure only for the
        chosen model).
      - Thumb vs preview input: for the same 200+ files, embed the stored
        thumbnail-size image (404 long edge, q80 JPEG re-encode, rotated)
        and the full preview; record the per-file cosine (min / mean) and
        the top-5 retrieval agreement. The thumbnail input is confirmed
        when mean cosine >= 0.98 and top-5 agreement >= 0.8; else the plan
        notes that pass 3 must decode the preview.
    - The Python reference scripts live in this plan folder (`ref_embed.py`
      and friends), run with `python -I`, and take the model directory and
      the image list as arguments; the downloads live under
      `D:\Photos\tests\2026-10-12-embed-models\<model>\`.
    - **Decision 1 (model and distribution)** is written below with the
      numbers that drove it: which model ships, FP32 / fp16 / int8, and
      whether it is bundled with `include_bytes!`, bundled as a resource
      next to the binary, or downloaded on the first enable (with the
      download's source, hash check and storage dir named).
    - `cargo test -p riffle-core -p riffle-cli` passes; a unit test covers
      the preprocessing on a synthetic image (size, normalization,
      orientation).
  - Implementation approach (as far as it is known):
    - `rgb` input: reuse `decode::decode_rgb` (the full preview) and, for
      the thumb path, re-encode through `scan::thumbnail_jpeg` and decode
      again so the measured input is exactly what the index stores.
    - Rotation by the Exif orientation before resize, as `faces.rs`
      `detect_around` does with its `orientation` argument.
    - Preprocessing per model: SigLIP 2 resizes to 256x256 (no center
      crop), mean 0.5 / std 0.5; OpenCLIP / TinyCLIP resize the short side
      to 224 then center-crop, CLIP mean / std. Confirm each against the
      HF `preprocessor_config.json` and the Python reference; the >= 0.999
      cosine check is what catches a wrong preprocessing.
    - Threads: run the files through `rayon` with the given thread count
      as `features` does; `tract` plans are `Send + Sync` behind an `Arc`.
    - Do not change `crates/app` in this step.

- [x] Step 2: Offline ranking feasibility gate on the 27 labeled ARW folders, with and without per-genre weights
  - Done when:
    - A dump exists under `D:\Photos\tests\2026-10-12-embed-gate\`: the
      Step 1 vectors of the chosen model for every file of the 27 folders
      joined by path with a fresh `riffle-cli features` TSV (flags,
      capture time, `eye_focus`, sharpness, EAR, pose, cue state).
    - Provisional genre probabilities per file: zero-shot scores against
      an uncalibrated prompt list for the initial genres (the same list
      Step 5 starts from), text vectors computed offline by a script in
      this plan folder (`python -I`, the text tower under the models test
      directory). They are features for the gate only, not shipped.
    - `gate.py` in this plan folder (run with `python -I`, numpy / scikit-
      learn allowed) builds the within-burst pairs (gap 1000 ms, as
      `burst.ts`) of pick vs not-chosen (every non-pick in these folders is
      "not chosen"; explicit rejects are reported separately), weights
      each burst's pairs to a total of 1 so one long burst does not
      dominate, fits a pairwise logistic regression on the embedding
      difference projected to about 32 PCA dimensions (fit on the training
      folders only) plus the technical features and the position in the
      burst, L2-regularized, and evaluates leave-one-folder-out
      (shooting day = folder).
    - A second variant adds per-genre weights: interaction terms of the
      burst's genre probabilities with the technical feature differences
      only (not the embedding dimensions), parameterized as the shared
      weight plus a per-genre offset with a stronger L2 penalty on the
      offsets so a genre with few bursts falls back to the shared weights.
      (The genre is constant within a burst, so it cancels in a difference
      unless it enters as an interaction.)
    - The pre-registered criterion is written in `gate.md` before the
      fit runs: held-out weighted pairwise AUC >= 0.703 (0.653 + 0.05) and
      a Top-1 (the pick is the top-ranked frame of its burst) gain over
      the first-frame baseline larger than the per-folder spread
      (standard deviation across folders) of that baseline. The per-genre
      variant is kept only if it beats the shared model held out by more
      than the per-folder spread. Baselines reported with the same folds:
      first frame, `eye_focus`, sharpness, the good-photo tier (`photoTier`
      rule), random; plus the model without the embedding dimensions, so
      the embedding's own contribution is visible.
    - `gate.md` also reports Top-1 / Top-3, a learning curve (folds
      trained on 5, 10, 20, 26 folders), the number of bursts per
      provisional genre, and the per-genre AUC of both variants.
    - **Decision 2 (ranking ships or not, per-genre weights or not)** is
      written below: pass means Step 7 proceeds (with or without per-genre
      weights); fail means Step 7 is dropped and the plan says so with the
      numbers.
  - Implementation approach (as far as it is known):
    - Assumes Step 1 merged (the `embed` CLI). If the dump needs a column
      `features` lacks, add it to `features_line` and its header in the
      same PR rather than a second dump tool.
    - Keep `fit.py` from burst-keep-score as the style reference for the
      pair construction and the folder folds; do not import it.
    - The 27 folders likely lean toward a few genres; report the per-genre
      burst counts so a thin genre's result is read as such.

- [ ] Step 3: Pass 3 in the app: the `embeddings` table, `run_embed_pass`, the `aiEnabled` setting
  - Done when:
    - `embed.rs` gets its `OnceLock` plan and the model bytes per
      Decision 1 (bundled under `crates/core/models/` with its LICENSE, or
      loaded from the download directory); `EMBED_VERSION` names the model
      and preprocessing.
    - `index.rs`: `SCHEMA_VERSION` 20 adds `embeddings(path PRIMARY KEY,
      size, mtime_ns, version, vec BLOB)`; `embeddings_missing(dir)`
      returns the RAW rows whose embedding is absent, stale by version or
      by size / mtime; `write_embeddings` upserts; `reconcile`,
      `evict_folder`, `clear`, `rename_file`, `rename_dir` carry the
      table, and `used_bytes` already counts its pages (verify). Tests
      cover each of those paths and the staleness rule.
    - `commands.rs`: `run_embed_pass` after `run_faces_pass`, same cancel
      flag and `ScanFocus`, `Priority::Lowest`, emitting `embed-progress`
      (count, total) and `embed-done`; `faces-done` timing unchanged for
      the frontend. Skipped entirely, with an immediate `embed-done`, when
      `aiEnabled` is off or the folder is view-only.
    - Settings: `aiEnabled` (default off until a user-facing feature
      ships in Step 4; the default may flip then) read like `mcpEnabled`
      in the settings tuple and set by `set_ai_enabled`; a checkbox in the
      settings modal with the one-line explanation that analysis runs
      locally in the background. Turning it on triggers pass 3 for the
      open folder; turning it off cancels a running pass 3 and keeps the
      stored vectors.
    - Measured and recorded in `learnings.md` on `D:\photos\2026\2026-09-19`
      (2134 ARW): pass-3 wall time after pass 2, peak working set of the
      app during it, page latency while it runs (the strip's page fetch
      time vs with `aiEnabled` off; the performance overlay's preview
      latencies may serve), and that a rescan of the same folder embeds 0
      files.
    - `docs/agents/tauri-app.md` gets the pass-3 / `embeddings` table
      paragraph; `tract-onnx-inference.md` the embedding model's items
      (its input layout and whatever Step 1 hit). `CLAUDE.md`'s layout
      paragraph mentions `embed.rs`, the table and the setting.
  - Implementation approach (as far as it is known):
    - Input: the thumb failed Step 1's agreement check (Decision 1), so
      pass 3 reads the preview through `reader` as pass 2 does and decodes
      it with `decode::decode_upright_near(jpeg, orientation, 404)`, no
      re-encode.
    - Thread count: pass 2's count unless Step 1's peak working set says
      fewer; write the reason in `learnings.md`.
    - No user-visible feature yet beyond the setting.

- [ ] Step 4: Similarity search: "Show Similar" from the strip's right-click menu
  - Done when:
    - `similar_to(path, limit)` command returns the folder's files ordered
      by cosine to the given file (brute-force over the stored vectors;
      files without a vector are excluded), with the cosine.
    - `context.ts` gets a `Show Similar` item (hidden when `aiEnabled` is
      off or the file has no vector yet); choosing it shows the strip
      filtered to the frames above the threshold, ordered by similarity,
      the source file first, with a visible "Similar to <name>" chip or
      bar and one key / click to return to the previous view; selecting
      2-4 of them feeds the existing compare pane (`compare.ts`) unchanged.
    - The threshold comes from hand-labeled same / different scene pairs
      (at least 100 pairs across 5+ folders, labeled by the user; the
      labeling file and its precision / recall at candidate thresholds in
      `similarity.md`); the chosen threshold keeps different scenes apart
      at >= 0.95 precision on that set.
    - Docs: `docs/humans/usage.md` / `usage.ja.md` and `README.md` /
      `README.ja.md` describe the item and the setting.
  - Implementation approach (as far as it is known):
    - Assumes Step 3 merged. Keep the time-based bursts untouched; this is
      a view, not a grouping.
    - Brute force: 2134 x 768 dot products is well under a millisecond;
      ANN only if a real folder shows otherwise (record the measured time).
    - To be refined after Decision 1 (vector size, model).

- [ ] Step 5: Genre classification: a genre data file, precomputed prompt vectors, a filter menu section
  - Done when:
    - `crates/core/genres/<model>.json` (embedded by `build.rs` like
      `i18n/`) lists each genre and sub-genre with its prompt ensemble,
      its precomputed L2-normalized text vector, its parent, and its
      calibrated threshold; `genre.rs` scores a vector against them
      (mean of the prompt vectors per label, softmax-free) and returns the
      labels over their threshold, `Unknown` when none is.
    - An offline script in this plan folder (`genre_vectors.py`, `python
      -I`, the text tower downloaded to the models test directory) writes
      the JSON from a prompt list; re-running it is the only way the
      vectors change, so the JSON's model field must match `EMBED_VERSION`
      (a unit test checks it).
    - Thresholds and `Unknown` are calibrated on about 300 hand-labeled
      frames across the initial genres (sports, portrait, landscape,
      birds / animals, snap, macro, other; sub-genres basketball, soccer,
      group photo, camping, pets); the labels and the per-label precision /
      recall at the chosen thresholds are in `genres.md`.
    - The filter menu gets a `Genre` section (`filter.ts` `passes()`, the
      `index.html` items, tests) fed by a `genres_of(dir)` command (or the
      genre list on each strip row); a file passes an item when that label
      is on; `Unknown` is an item. Hidden when `aiEnabled` is off.
    - Docs: usage / README pairs.
  - Implementation approach (as far as it is known):
    - Assumes Step 3 merged. Genre scoring is backend-side so the JSON
      and the vectors stay in one place; the frontend only sees label
      names.
    - Starts from Step 2's provisional prompt list.
    - To be refined after Decision 1 (which text tower produces the
      vectors; SigLIP uses a sigmoid-trained space, so thresholds are per
      label, not shared).

- [ ] Step 6: Visual similarity groups, separate from bursts
  - Done when:
    - Groups are built from the Step 4 threshold by connected components
      (or single-linkage agglomerative with the same cut) over the
      folder's vectors, in the backend, on demand; they are not stored.
    - The strip can show a subtle group boundary or a "visual groups"
      navigation (jump to next group) that is off by default and
      independent of the burst markers; the burst code and `BURST_GAP_MS`
      are untouched.
    - Validated on Step 4's labeled pairs: same-scene pairs land in one
      group at >= 0.9, different-scene pairs in one group at <= 0.05;
      numbers in `similarity.md`. If connected components chain scenes
      together on a real folder, the step switches to average linkage and
      records why.
  - Implementation approach (as far as it is known):
    - Assumes Step 4 merged. To be refined after Step 4 shows how the
      threshold behaves on whole folders.

- [ ] Step 7: Personal ranking (only if Decision 2 passes): culling-finished folders, a local model, a recommended sort, a reason display
  - Done when:
    - `folders.culling_finished` (a `SCHEMA_VERSION` bump) is toggled from
      the folder tree's right-click; only finished folders contribute
      "not chosen" negatives; explicit rejects count as stronger negatives
      in any folder; unjudged frames in unfinished folders are never
      negatives.
    - `rank.rs`: the Step 2 model (PCA basis + logistic weights; no
      per-genre offsets, Decision 2 dropped them) fit locally on the
      user's finished folders, shrunk toward the Step 2 prior while the
      data is small, stored in the app data dir with its version; refit on
      demand (a settings button) and after a folder is marked finished;
      never changes a judgment.
    - `sort.ts` gets a `recommended` `SortKey`; the meta pane shows the
      score with its top contributing features as the reason (naming the
      genre when a per-genre weight drove it) and a confidence that
      reflects the training set size; the strip shows no extra icon by
      default.
    - Step 2's evaluation is re-run on the app's stored data for the same
      27 folders; the numbers go in `ranking.md`. The gate's eye and pose
      inputs come from the `features` eyes path's judged face, not pass 2's
      stored `eyes_ear` / `pose_*` of the cue face, so an exact match to
      `gate.md` is not expected. Step 7 re-runs `gate.py` on a dump that
      uses the app's face and records both numbers; the shared model must
      still clear the gate's bar there.
    - Docs: usage / README pairs; `CLAUDE.md` layout.
  - Implementation approach (as far as it is known):
    - Assumes Steps 3 and 4 merged. Decision 2 shipped one shared model with
      no per-genre offsets, so Step 5 is not a dependency.

## Trade-offs and risks

- **Model distribution** (Decision 1, after Step 1). `include_bytes!` of a
  96 MB int8 model is simplest and offline but grows every binary that
  links `riffle-core` (the CLI included) and the installer; a resource file
  next to the binary keeps the CLI small but needs a path lookup; a
  first-enable download keeps the installer small but adds a network step,
  a hash check and a failure mode. int8 may lose accuracy in tract (its
  `MatMulInteger` path) or run slower than FP32; Step 1 measures before
  choosing.
- **Thumb input.** A 256x256 resize from the 404x270 thumbnail should lose
  little; if Step 1 shows it does, pass 3 reads the preview and costs a RAW
  read per file.
- **Ranking may not ship.** The prior evidence says picks within a burst
  are timing-driven (first frame AUC 0.653). The gate is pre-registered so
  a marginal result does not get argued into shipping; if it fails, Step 7
  is dropped and the plan records it.
- **Per-genre weights may not pay.** The labeled folders probably lean to a
  few genres, so most genres' offsets stay near zero; they are kept only if
  they beat the shared model held out. Hand-set per-genre sliders are not
  planned (they would blur what the learned weights mean).
- **Negatives.** Treating every non-pick in a "finished" folder as "not
  chosen" inflates negatives with frames the user never looked at. The
  alternative (explicit rejects only, 58 frames) cannot train anything.
  The approved reading is the former, kept distinct from explicit rejects
  in every report.
- **Genre thresholds.** Per-label thresholds calibrated on about 300
  frames of one photographer's folders will not transfer to other users;
  `Unknown` and the data file keep that cheap to retune. Backend scoring
  means the frontend cannot tune live; accepted for the single source of
  truth.
- **Scan cost.** A third pass on a 2000-file folder may take minutes at
  `Priority::Lowest`; the vectors are cached, so it is paid once per model
  version. If Step 3's page latency measurement shows interference, drop
  the thread count before anything else.
- **Default of `aiEnabled`.** Off until Step 4 ships a visible feature,
  then the caller decides whether to flip it on; either way the setting is
  one checkbox.

## Decision 1 (model and distribution)

Decided in Step 1 from the measurements in [models.md](models.md) (Windows
11, i7-13700, 24 logical processors).

- **Model: SigLIP 2 ViT-B/32 @256** (`google/siglip2-base-patch32-256`,
  Apache-2.0; the image tower `onnx/vision_model.onnx` of
  `onnx-community/siglip2-base-patch32-256-ONNX`), 768-d, spec `siglip2`
  in `embed::SPECS`: 256x256 stretch, PIL bilinear, mean / std 0.5, RGB,
  output `pooler_output`. It loads with `default-features = false` and
  matches onnxruntime at cosine min 0.999997 on 12 real previews. It costs
  132 ms per image on one thread and 16.6-17.6 ms per image of wall time on
  24 threads (decode included). OpenCLIP B/32 costs about the same (116 ms,
  14.4-14.6 ms) for 7.4 fewer ImageNet zero-shot points (66.6 against 74.0).
  TinyCLIP 40M/32 costs half as much (62 ms, 7.7-8.2 ms) for 14.2 fewer
  (59.8). SigLIP 2 is also the only shippable candidate whose cheap input
  clears the agreement bar (below). MobileCLIP-S2 is no-ship by license and
  is the slowest in tract anyway (328 ms).
- **Format: FP32** (378.5 MB). onnx-community's fp16 export does not load
  in tract 0.23 (the head `Gemm`). Its int8 / uint8 exports are dynamic
  quantizations that are wrong in onnxruntime too (cosine min 0.638 / 0.663
  against FP32), and the uint8 one takes 8.4 s per image in tract.
  Weight-only re-exports (`weight_only.py`: int8 96 MB at cosine mean
  0.99938, fp16 190 MB at 0.999998 against FP32) load. But tract re-widens
  their weights on every run: 250-275 ms per image on one thread and
  76-85 ms per image on 24 threads, 2x to 5x FP32's cost.
- **Input: the preview, decoded at the DCT scale nearest 404 px with no
  re-encode** (`decode::decode_upright_near(jpeg, orientation, 404)`), so
  **pass 3 must read the preview**, not `files.thumb`. The stored thumbnail
  fails the bar for every model (SigLIP 2: cosine mean 0.961, top-5
  agreement 0.519 on 214 files). The q80 re-encode causes it, not the size:
  a 404 px box downscale alone keeps cosine mean 0.9991 (`thumb_check.py`).
  The scaled decode reaches cosine min 0.993 / mean 0.998 and top-5
  agreement 0.843 against the full preview, and decodes in 9.0 ms against
  26.1 ms (24 threads).
- **Distribution: downloaded on the first enable.** The user confirmed it
  against the bundled-resource option on 2026-10-12.
  - `include_bytes!` is out. It adds 378,479,104 B to every binary that
    reaches the model, measured on `riffle-cli` (29.8 MB -> 408.2 MB).
    `riffle-app` (about 52 MB) would grow the same for a feature that is
    off by default, and the working set after load doubles (774 against
    395 MB).
  - A resource next to the binary grows every installer by about 350 MB,
    since the FP32 weights gzip only 7% smaller (351.2 MB), for users who
    never turn AI on.
  - The download, on turning `aiEnabled` on, comes from
    `https://huggingface.co/onnx-community/siglip2-base-patch32-256-ONNX/resolve/7efccb86b5b6601bfe9e14326e1c11b40b68d44c/onnx/vision_model.onnx`,
    a pinned revision. It is written to a `.part` file, its SHA-256 is
    checked against
    `0ddb7507cfcb54c2cd503029949add9dc6586de67164bb69c2c09588cdcc9719`
    (both the HF LFS oid and the file measured here), and only then is it
    renamed to `models/siglip2-base-patch32-256/vision_model.onnx` under
    the app's local data dir (Tauri `app_local_data_dir()`). The Apache-2.0
    notice ships with the app under `crates/core/models/`. A failed or
    mismatched download leaves pass 3 idle with the error shown, and a later
    enable retries.
- **Memory and threads for pass 3.** On top of the plan, a 214-file run
  holds 416 MB on one thread and 637 MB on 24. `Model::load` peaks at about
  3x the file (1145 MB), whatever the thread count, while the file's bytes,
  the parsed protobuf and the optimized plan are alive together. Step 3
  sizes the pass-3 thread count against those numbers, and loading from a
  path rather than a `Vec` may lower the load peak.

## Decision 2 (ranking ships or not, per-genre weights or not)

Decided in Step 2 by the criterion pre-registered in [gate.md](gate.md)
(commit `ce0ea14`, before the fit), on the 27 sidecar-labeled ARW folders:
32,459 frames, 4,419 picks, 1,801 scorable bursts at gap 1000 ms, 41,942
pick x not-chosen pairs, leave one folder out.

- **Ranking ships: Step 7 proceeds, with the shared model.** Held out, the
  shared model (SigLIP 2 embedding as 32 whitened PCA dimensions + the
  technical features + the position in the burst, pairwise L2 logistic)
  reads a weighted pairwise AUC of **0.708** against the bar of 0.703
  (first frame 0.650 here, 0.653 in burst-keep-score), and a Top-1 of
  **54.4%** against the first frame's 39.4%: a gain of 15.0 points against
  the first frame's per-folder SD of 8.7. Both checks pass, the AUC one by
  0.005, which is less than the per-folder SD of 0.060: the ranking is a
  modest recommendation, not a reliable best-shot pick, and Step 7 should
  present it that way (an ignorable sort order, as planned). Top-3 is
  87.7% against the first frame's 79.2%.
- **The embedding is what clears the bar.** The same model without the
  embedding reads 0.652 (Top-1 44.5%), level with the first frame; the
  embedding alone reads 0.668 (51.5%).
- **No per-genre weights.** The per-genre variant (shared weights + 10x
  penalized per-genre offsets on the technical feature differences) reads
  0.707 (Top-1 55.2%), level with the shared model against a required gain
  of 0.060 (the shared model's per-folder SD). The provisional genres put
  1,478 of the scorable bursts in sports and 287 in snap, 20 or fewer in
  each other genre, so the data cannot show a per-genre effect; Step 7 drops
  the offsets and does not depend on Step 5.
- **Data size.** The learning curve reads 0.687 (5 training folders), 0.699
  (10), 0.704 (20), 0.708 (26), so Step 7's shrinkage toward the Step 2
  prior matters while a user has few finished folders. Explicit rejects
  (58 frames, 3 scorable bursts) are too few to measure as stronger
  negatives.

## Progress

- (2026-10-12) Plan written; the user added per-genre ranking weights (a Step 2 variant, kept for Step 7 only if it wins held out).
- (2026-10-12) Step 1 complete
- (2026-10-12) Step 2 complete (gate passed narrowly: weighted pairwise AUC 0.708 vs the 0.703 bar; Decision 2: ship a shared model, no per-genre offsets)
