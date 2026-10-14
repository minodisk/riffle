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

# Ball-game composition as ranking features: evaluate two candidates, then adopt or drop

## Purpose

The personal best-shot ranking (`crates/core/src/rank.rs`, the `recommended`
sort) scores each frame of a burst from 10 technical values, 2 burst-position
values and the SigLIP 2 embedding compressed to 32 whitened PCA dimensions.
The embedding is what cleared the gate
([image-embeddings gate.md](../_archived/20261012-image-embeddings/gate.md):
0.708 against 0.652 without it), but 32 dimensions of a global embedding
carry the place, the sport and the atmosphere, not whether the ball is in the
frame or whether the carrier is contested. 1,478 of the 1,801 scorable bursts
of the labeled set are sports (basketball), so two composition cues a sports
photographer picks on are worth measuring as explicit features:

- (a) **the ball is in the frame**;
- (b) **the ball carrier and at least one opposing player are in the frame**
  (a contested play).

Two ways to measure them, compared offline on the same data, folds and
metrics as the ranking gate before either touches the app:

- **Candidate 1: SigLIP 2 text prompts.** The cosine between the stored image
  embedding and precomputed text vectors ("a ball is visible", "a ball
  carrier contested by a defender", ...), the mechanism of `genre.rs` /
  `crates/core/genres/siglip2.json`. No new model, no new scan work, derived
  on demand from the stored vector. Risk: a small or blurred ball is a few
  pixels of a 256x256 input and may not register.
- **Candidate 2: an object detector.** Person and `sports ball` boxes from a
  COCO-trained detector; features from the boxes (ball presence, person
  count, the ball-to-person relation as the carrier estimate, a second
  person near the carrier as the contest). More direct, at the cost of a
  model (license, bundling or download, an extra decode and inference per
  file in the AI pass).

A combination, or neither, are outcomes too. The acceptance criteria are
pre-registered below, in this file, before any score is computed; the
adopt / drop decision is recorded with the numbers as Decision 1. **If no
candidate clears the bar, Steps 4-6 are dropped, Decision 1 says so with the
numbers, and the plan is finished.**

### What is known

- **Model and prior.** `rank.rs`: `COLUMNS` = 10 technical + 2 position + 32
  PCA; `REASONS` groups them for the meta pane (`Look`, `Burst position`,
  `Sharpness`, `AF eye focus`, `Eyes open`, `Head pose`, `Good photo`);
  `RANK_VERSION` 1. The prior `crates/core/ranking/siglip2.json` holds the
  imputation means, the PCA mean and whitened basis, the standardization,
  the weights and `C`, written by
  [`rank_prior.py`](../_archived/20261012-image-embeddings/rank_prior.py),
  which imports `gate.py`'s `load` / `build` / `choose_c` / `fit`. Any
  column change bumps `RANK_VERSION`, re-runs `rank_prior.py`, and the app's
  `Stored::parse` falls back to the prior until the next refit.
- **Evaluation harness.**
  [`gate.py`](../_archived/20261012-image-embeddings/gate.py) (leave one
  folder out, weighted pairwise AUC, Top-1 / Top-3, per-folder spread,
  learning curve, the baselines) run on the **app dump**
  `D:\Photos\tests\2026-10-12-rank-app\` (`features/<folder>.tsv`,
  `embed.tsv`, `genre_text.tsv`, `index.sqlite`; 32,459 frames, 1,801
  scorable bursts, 41,942 pairs) reads **0.710 weighted AUC / 54.9% Top-1**
  for the shared model ([ranking.md](../_archived/20261012-image-embeddings/ranking.md)).
  That is the reference every candidate is compared against, re-run in the
  same process. The Step 2 dump `D:\Photos\tests\2026-10-12-embed-gate\`
  (`riffle-cli features` values) is the fallback. Labels are the sidecar
  flags and `Output/` listings (pick vs not chosen; rejects reported
  separately), so the per-file-viewed plan's change to the app's negatives
  does not touch the offline evaluation.
- **Python.** The venv `%TEMP%\embedvenv` (numpy 2.5.4, scikit-learn 1.9.1,
  onnxruntime 1.31.0, Pillow, `tokenizers`) still exists; scripts run with
  `python -I` from outside the plan folder and the download directories.
  The SigLIP 2 text tower and tokenizer are at
  `D:\Photos\tests\2026-10-12-embed-models\siglip2-b32-256-text\`
  ([`genre_text.py`](../_archived/20261012-image-embeddings/genre_text.py)
  has the tokenization: lowercase, no BOS, EOS, pad to 64).
- **Previews for a detector.** `riffle-cli embed ... --previews <dir>`
  writes each file's full preview JPEG and a `list.tsv`, but refuses
  `--near`; the full previews of 32,459 ARW are roughly 15 GB. A DCT-scaled
  export (the `decode_upright_near` path) at 640-800 px would be a few GB.
- **Detector licensing (to verify per model card in Step 3).** Excluded by
  license: Ultralytics YOLOv5 / v8 / v10 / v11 (AGPL-3.0), YOLOv6 / v7 / v9
  (GPL-3.0), YOLO-NAS weights (non-commercial). Shortlist with Apache-2.0
  code and weights: YOLOX (Megvii; Nano 0.9 M params, Tiny 5 M @416, S 9 M
  @640, official ONNX export, plain conv / SiLU ops), RTMDet (OpenMMLab;
  tiny 4.8 M, s 8.9 M, exported through mmdeploy), NanoDet-Plus (1-2 GFLOPs,
  weak on small objects), D-FINE / RT-DETRv2 / DEIM (transformer heads whose
  deformable attention exports as `GridSample`-like ops that tract 0.23 with
  `default-features = false` may not load: verify before spending time).
  `tract-onnx-inference.md` is the loading guide (ignore `value_info`, pin
  the input fact, outputs by outlet label, `OnceLock` plan, check against a
  Python reference on real images).
- **Parallel session.** The per-file-viewed-ranking plan
  ([plan](../_archived/20261014-per-file-viewed-ranking/plan.md)) changes `rank.rs`
  `Labeled` / `pairs` / `fit`, `Index::rank_dirs` / `rank_rows` (`viewed`),
  app `rank.rs` `refit` and `the_scores_match_rank_prior_py`, and bumps
  `SCHEMA_VERSION` to 23. At the time of writing none of its steps is on
  `origin/main`. Steps 1-3 here touch none of that (plan-folder scripts, at
  most a `crates/cli` option). **Steps 5 and 6 must start from a `main` that
  has that plan's Step 1 merged** (`Labeled` has `viewed: Vec<bool>`): check
  `git log origin/main` for it; if its PR is open, wait rather than race it.
- **Other sessions** own per-genre weighting (dlg-genre-weights),
  folder-wide training (dlg-folder-best) and the likely-pick icon
  (dlg-likely-pick-icon). The design here adds shared columns (a feature
  that reads about zero on a frame without a ball needs no per-genre gate),
  so it does not depend on them; a sports-gated interaction is reported as
  a reading aid only (below).

## Pre-registered criteria (Decision 1)

Written 2026-10-14 with this plan, before any prompt vector, detector output
or fit exists, and confirmed by the user (the bar size, the hand labels, the
choice rule and the shippability bars) before the plan was committed. The
commit that adds this file is the record; nothing in this section changes
afterwards. A deviation found while implementing goes in `learnings.md` with
its reason and does not move the bars.

### Data, model, folds

- The app dump `D:\Photos\tests\2026-10-12-rank-app\` as `gate.py` loads it:
  the same frames, picks, bursts (gap 1000 ms), scorable bursts, pair
  weights (each burst 1 in all), imputation, 32-dimension whitened PCA fit
  on the training folders, standardization, L2 pairwise logistic, `C` from
  {0.01, 0.1, 1, 10} by the inner grouped 5-fold, leave one folder out (26
  folds). The **reference** is the shared model re-run in the same process
  (expected 0.710 / 54.9%; any difference from `ranking.md` is reported).
- A **candidate model** is the shared model plus the candidate's per-frame
  columns, entered like the technical columns (standardized on the training
  folders, a pair's feature is the difference), nothing else changed.
  Candidate columns are fixed below; Candidate 1 may be run with one prompt
  revision, both reported, the first one being the one the criterion reads
  unless the revision was written before any score of the first was seen on
  the pairs (record which in `learnings.md`).
  - **Candidate 1 columns (at most 4):** the cosine of the embedding with
    the `ball visible` text vector; with the `contested carrier` vector;
    and, as contrasts, each minus the cosine with its paired negative prompt
    set ("no ball in the picture", "a single player alone"). The prompt
    ensembles are written in `ball_prompts.json` in Step 1, before any
    score.
  - **Candidate 2 columns (at most 6):** ball present (the top `sports ball`
    confidence, 0 when none); a ball indicator at the confidence the
    hand labels pick for best F1; ln(1 + persons at or above the detector's
    default confidence); contested (1 when a second person box overlaps or
    sits within one carrier-box width of the carrier, the carrier being the
    person box containing or nearest the ball center; 0 otherwise, also
    when there is no ball); the ball's size relative to the frame's long
    side; and ln(1 + persons) x ball present. Team separation by jersey
    color is **not** part of the first evaluation; it is noted as a follow-up
    if Candidate 2 ships.
  - **Combination:** the shared model plus both candidates' columns.
  - **Reading aids, not in the criterion:** each candidate column alone as
    a within-burst ranking (as `gate.py`'s baselines); each candidate's
    columns gated by the burst's sports probability from `genres.tsv`
    (the interaction the per-genre session owns); the sports-only and
    non-sports-only burst subsets; the pick vs explicit-reject bursts.
- **Hand labels** (Step 1's sample, labeled by the user before Step 2 runs):
  about 240 frames, one per burst, drawn at random from the scorable bursts
  (200 from sports bursts by `genres.tsv`'s argmax, 40 from the rest), each
  labeled `ball` (visible / not visible / unsure) and `contest` (the carrier
  and at least one opponent in frame / not / not applicable). They measure
  whether a column measures what its name says: the frame-level ROC AUC of
  the column against the label, excluding `unsure` / `not applicable`.

### Criterion

A candidate (Candidate 1, Candidate 2, or the combination) **passes** when,
held out over the same 26 folds:

1. its weighted pairwise AUC exceeds the reference's by **>= 0.010**;
2. its pooled Top-1 exceeds the reference's by **>= 2.0 points**;
3. its per-folder weighted AUC is at or above the reference's in **at least
   13 of the 20 folders with 20 or more scorable bursts** (so one large
   folder cannot carry the gain);
4. its `ball` column reads a frame-level AUC of **>= 0.75** against the
   `ball` hand label (the reason display will name the feature; a column
   that does not measure the ball must not be called that).

Candidate 2 additionally **ships only if**: the chosen detector's code and
weights are under Apache-2.0, MIT or BSD; its ONNX loads and runs in tract
0.23 with `default-features = false` and matches onnxruntime on the labeled
frames (same boxes at the chosen confidence, IoU >= 0.9); its warm cost is
**<= 132 ms per image on one thread** (SigLIP 2's) at the input size the
evaluation used; and the file is **<= 100 MB**. A detector that fails one of
these is "no-ship" whatever its AUC, and the plan records it.

**Choice:** among the passing candidates, Candidate 1 is adopted unless
Candidate 2 (shippable) beats it by >= 0.010 weighted AUC; the combination
is adopted only if it beats the better single passing candidate by >= 0.010.
**No candidate passes: Decision 1 records the numbers, Steps 4-6 are
dropped, the plan is finished.**

Also reported: Top-3, the per-folder table, the learning curve of the
adopted candidate, the weights of the fit on every folder, and the chosen
`C` per fold, as `gate.py` prints them.

## Steps

- [x] Step 1: Pre-register, prepare the evaluation harness and the hand-label sample
  - Done when:
    - This plan (with the criteria above) is committed before any score
      exists; `learnings.md` starts with the commit hash.
    - `ball_prompts.json` in this folder lists the four prompt ensembles
      (`ball visible`, `no ball`, `contested carrier`, `single player`),
      3-6 prompts each, written from the definitions, not from any frame.
    - `ballgate.py` in this folder imports the archived `gate.py` (as
      `rank_prior.py` does, `sys.path` + `sys.dont_write_bytecode`) and
      runs, for a gate directory plus one or more `extras_<name>.tsv`
      (`folder\tfile\t<column>...` per frame), the reference and each
      candidate (one extras file, or several for the combination), and
      prints: the held-out table, the per-folder paired table with the
      count of folders at or above the reference, the criterion table
      (bars 1-3 evaluated, bar 4 from a `--labels <labels.tsv>` option),
      the reading aids (each column alone, the sports-gated interaction,
      the sports / non-sports subsets, pick vs reject), and the learning
      curve for a `--curve <name>` candidate. A frame without an extras
      row gets the training mean and a missing indicator is **not** added
      (the dump covers every frame; the script fails if more than 0.1% are
      missing).
    - `ball_sample.py` draws the 240-frame sample from the app dump
      (seeded, one per scorable burst, 200 sports / 40 other) into
      `D:\Photos\tests\2026-10-14-ball-labels\frames.tsv` with a contact
      sheet or per-frame JPEGs for labeling (reuse what
      [`genre_sample.py`](../_archived/20261012-image-embeddings/genre_sample.py)
      did), and a `labels.tsv` template (`id\tball\tcontest`). The sample is
      drawn in this step and the user labels it before Step 2's fit runs.
    - A dry run of `ballgate.py` with a random-noise extras file reproduces
      the reference (0.710 / 54.9%, or the re-run's numbers recorded in
      `learnings.md`) and shows the noise candidate failing; the output
      goes in `learnings.md`, not in a results file.
  - Implementation approach:
    - Do not copy `gate.py`; extend by import so the pair construction,
      folds, `choose_c` and the metrics are the archived ones.
    - Python under `-I` from the scratchpad; the venv's interpreter
      (`%TEMP%\embedvenv\Scripts\python.exe`).
    - No crate changes in this step.

- [ ] Step 2: Candidate 1: prompt vectors, cosines, the gate run
  - Precondition: `labels.tsv` holds the user's labels for the Step 1
    sample (at least 200 frames with a `ball` label).
  - Done when:
    - `ball_text.py` (a thin variant of the archived `genre_text.py`)
      writes the four ensemble vectors to `ball_text.tsv` under the test
      directory, and `ball_scores.py` writes `extras_c1.tsv` (the four
      columns of the criterion) for every frame of the app dump from its
      `embed.tsv`.
    - `candidate1.md` in this folder records: the frame-level AUC of each
      column against the hand labels (bar 4), the `ballgate.py` tables for
      Candidate 1 against the reference, and the reading aids. If a prompt
      revision is run, both are in the file with which one the criterion
      reads and why.
    - The criterion table for Candidate 1 is copied into this plan's
      Decision 1 section as "Candidate 1: pass / fail on bars 1-4".
  - Implementation approach:
    - Vectors as `genre_vectors.py` makes them: each prompt L2-normalized,
      the mean normalized again, so a shipped version is one dot product per
      column in `rank::rows`.
    - Expect cosines in the 0.05-0.15 band; the standardization handles the
      scale, the contrast columns handle a shared offset.

- [ ] Step 3: Candidate 2: detector shortlist, previews, detections, the gate run; Decision 1
  - Done when:
    - `detectors.md` in this folder tables the shortlist (YOLOX Nano / Tiny
      / S, RTMDet tiny / s, NanoDet-Plus, and D-FINE / RT-DETRv2 / DEIM if
      their ONNX loads in tract): the license of code and weights with the
      URL checked, the pinned file (URL, size, SHA-256), the input size, the
      ops tract rejected if any, and onnxruntime's ms/image on this machine.
      Downloads go in their own directories under
      `D:\Photos\tests\2026-10-14-detectors\<model>\` and are untrusted data.
    - The previews of the 27 folders exist as JPEGs under
      `D:\Photos\tests\2026-10-14-ball-previews\` at a DCT scale near the
      detector input (640-800 px long edge). If `riffle-cli embed
      --previews` is extended to accept `--near <px>` (or a `previews`
      subcommand is added) for this, that `crates/cli` change is in this
      step's PR with its usage string and a unit test of the option parsing.
    - `detect_eval.py` runs each shortlisted model (onnxruntime) on the
      labeled frames and reports `sports ball` recall / precision at the
      default and the best-F1 confidence against the `ball` label, and the
      person count distribution; the model with the best ball F1 among the
      shippable ones (license, size, cost bars) is chosen and the choice
      written in `detectors.md`.
    - `detect_all.py` runs the chosen model on every preview and writes
      `detections.tsv` (per frame: class, confidence, box, one line per
      detection) and `detect_features.py` derives `extras_c2.tsv` (the six
      criterion columns) from it, so the feature rules can be changed
      without re-running the detector.
    - `candidate2.md` records: the frame-level AUCs against the hand labels
      (bar 4 for `ball present`, and the `contested` column against the
      `contest` label), the `ballgate.py` tables for Candidate 2 and for
      the combination (`extras_c1.tsv` + `extras_c2.tsv`), the reading aids,
      and the shippability checks: a tract load of the chosen ONNX with
      `default-features = false` (an `#[ignore]` test in `riffle-core`
      reading the path from an env var is enough; it may be removed in Step
      4 or kept as that step's seed), the box agreement with onnxruntime on
      the labeled frames, the one-thread warm ms/image in tract, and the
      file size.
    - **Decision 1** is written in this plan with the criterion tables of
      all three candidates and the choice rule's outcome: which candidate
      (or none) ships. On "none", Steps 4-6 are marked dropped, the
      Progress entry says so, and the plan is finished (the wrap-up archives
      it). On Candidate 1, Steps 4 and 5 are marked skipped and Step 6
      proceeds. On Candidate 2 or the combination, Steps 4-6 proceed.
  - Implementation approach:
    - Run the long detector passes in the background and poll (the
      image-embeddings learnings: a single blocking tool call tripped the
      stream watchdog).
    - The input for detection is the upright preview (the embedding's
      `decode_upright_near` path, orientation applied), letterboxed as the
      model's own preprocessing does; mirror that preprocessing exactly in
      Python so Step 4's Rust port has a reference.
    - Measure, do not assume, whether D-FINE / RT-DETR export plain ops;
      drop them from the shortlist at the first tract load failure and say
      which op.

- [ ] Step 4 (only if Candidate 2 or the combination ships): the detector in `riffle-core` and `riffle-cli`
  - Done when:
    - `crates/core/src/objects.rs` (name free) loads the chosen ONNX as
      `faces.rs` / `embed.rs` do (ignore `value_info`, pinned input fact,
      outputs by outlet label, one `OnceLock` plan) and exposes
      `detect(rgb, width, height) -> Vec<Detection { class, confidence, box }>`
      for the `person` and `sports ball` classes only, plus the feature
      derivation of Step 3's `detect_features.py` as a pure function over
      the detections; `OBJECTS_VERSION` names the model and its
      preprocessing.
    - Distribution decided and recorded in `learnings.md` with the numbers:
      bundled under `crates/core/models/` with its LICENSE when the file is
      small (the YuNet / face mesh precedent, about 5 MB each; measure the
      binary growth as `tract-onnx-inference.md` says), otherwise downloaded
      next to SigLIP 2 through `ai.rs` (pinned URL, size, SHA-256; `ai.rs`
      generalized from one file to a list).
    - `riffle-cli objects <dir|file>... [threads]` prints the detections and
      the derived features per file and the warm ms/image; the ignored
      cross-check against Step 3's `detections.tsv` on the labeled frames
      holds (same boxes at IoU >= 0.9 at the chosen confidence).
    - `docs/agents/tract-onnx-inference.md` gets the model's items (input
      layout, preprocessing, what loading took); `CLAUDE.md`'s layout names
      the module. `cargo test -p riffle-core -p riffle-cli` passes.
  - Implementation approach:
    - Resize through the same resampling the Python reference used (PIL's
      filter for a letterbox, or the model's own); check the agreement on
      boxes, not only that it loads.
    - No `crates/app` change in this step.

- [ ] Step 5 (only if Candidate 2 or the combination ships): detections stored by the AI pass, read by `rank_rows`
  - Precondition: the per-file-viewed-ranking plan's Step 1 is on `main`
    (`SCHEMA_VERSION` 23, `Labeled.viewed`); rebase onto it before starting.
  - Done when:
    - `index.rs`: a `SCHEMA_VERSION` bump adds `objects(path PRIMARY KEY,
      dir, size, mtime_ns, version, detections BLOB)` holding the raw
      detections (class, confidence, box) compactly, valid when `version ==
      OBJECTS_VERSION` and `size` / `mtime_ns` match the `files` row, as
      `embeddings` does; `reconcile`, `evict_folder`, `clear`,
      `rename_file`, `rename_dir` and the stray sweep carry it; tests cover
      each path and the staleness rule as the `embeddings` tests do.
    - The AI pass (`run_embed_scan`, `commands.rs`) runs the detector on
      the same preview read as the embedding, for files whose `objects`
      row is missing or stale, at the pass's priority and thread count,
      storing a failed detection as an empty row (not retried until the
      stat or the version moves, as a failed embedding is); the pass's
      events are unchanged. A file with a valid embedding but no valid
      detections row is detected without re-embedding.
    - `rank_rows` reads the detections row and `rank::Frame` carries the
      derived features (or the detections, derived in `rows`); `RankRow`
      and the rank dump test `rank_dump_of_real_folders` write the new
      values so `rank_prior.py` can read them in Step 6.
    - Measured on `D:\photos\2026\2026-09-19` (2134 ARW) with the ignored
      harness `embed_pass_on_a_real_folder`: pass wall time with and
      without the detector, peak working set, the paging probe's median /
      p95; recorded in `learnings.md`. If the probe rises above pass 3's
      current level (about 1.5x idle), lower the thread count before
      anything else.
    - `docs/agents/tauri-app.md` gets the `objects` table paragraph;
      `CLAUDE.md`'s `index.rs` sentence names it.
  - Implementation approach:
    - Follow `tauri-app.md`'s "Bumping `SCHEMA_VERSION`" entry (ranged
      guards, fixtures, the accepted-version list).
    - Storing detections rather than features means a feature-rule change
      in Step 6 or later needs no rescan, the pattern the embedding set.

- [ ] Step 6: the new ranking columns, `RANK_VERSION` 2, the regenerated prior, the reason display, docs
  - Precondition: the per-file-viewed-ranking plan's Step 1 is on `main`;
    rebase onto it before starting (it rewrites `Labeled`, `pairs`, app
    `refit` and `the_scores_match_rank_prior_py`, all touched here).
  - Done when:
    - `rank.rs`: the adopted candidate's columns join `COLUMNS` after the
      position columns (Candidate 1: cosines with text vectors stored in
      the prior JSON, computed in `rows` from `Frame::vec`; Candidate 2:
      from the stored detections of Step 5); `RANK_VERSION` 2; `REASONS`
      gains one group per cue (names decided here, e.g. `Ball in frame`,
      `Contested play`); the unit tests cover the new columns' values on
      synthetic frames and the prior's shape.
    - `rank_prior.py` is extended (a copy in this folder importing the
      archived one, or a `--extras` option added to the archived script, the
      implementer's choice recorded in `learnings.md`) to read the adopted
      extras and write the new prior `crates/core/ranking/siglip2.json`
      (`rankVersion` 2, the text vectors inside it for Candidate 1) and
      `prior_scores.tsv`; `the_scores_match_rank_prior_py` matches within
      1e-4 on the app dump (for Candidate 2 the dump is re-made by
      `rank_dump_of_real_folders` after Step 5, detections included).
    - The gate is re-run on the final column layout with `ballgate.py` and
      the numbers go in `ranking.md` of this folder next to Decision 1's,
      with the fit-on-all weights table.
    - App: `Stored::parse` already falls back to the prior at a new
      `RANK_VERSION`; `refit` and `rank_rows` compile against the new
      `Frame`; `meta.ts` `rankingRows` needs no change unless a reason name
      is formatted specially (verify in the running app that the new reason
      appears on a basketball frame).
    - Docs in the same PR: `docs/humans/usage.md` **Recommended order** (the
      new reason names and what they read, and that the ranking relearns
      from the prior after the update) with `usage.ja.md` in sync;
      `README.md` / `README.ja.md` if the ranking bullet lists the cues;
      `CLAUDE.md`'s `rank.rs` sentence; `docs/agents/tauri-app.md`'s
      ranking bullet; `todo.md` gains a manual check that the new reason
      shows on a ball-game frame and that `Refit Ranking` after the update
      reports the fit again.
    - `mise run ci`, `cargo test -p riffle-core -p riffle-app` pass.
  - Implementation approach:
    - Keep the prior's PCA basis logic untouched; the new columns are
      standardized like the technical ones and pulled toward the prior's
      weights by the same `tau`.
    - Candidate 1's text vectors live in the prior JSON (one file holds
      everything `rows` needs) rather than in `genres/siglip2.json`, which
      is the filter's; check `vite.config.ts`'s `fmt.ignorePatterns`
      already covers `crates/core/ranking/**`.

## Trade-offs and risks

- **The size of the bar (bars 1-2: +0.010 AUC, +2.0 Top-1 points).** The
  embedding's whole contribution was +0.058; the per-genre variant was held
  to the per-folder SD (0.054), which nothing short of a new signal clears.
  +0.010 is about twice the margin the ranking gate passed by and a fifth
  of the embedding's gain: large enough to be worth a model change, small
  enough that a real composition cue can show it. Bar 3 (13 of 20 folders)
  guards against one large folder carrying a pooled gain; it is a sign test
  at about p = 0.13 one-sided, a guard, not a proof.
- **Bar 4 (frame-level AUC >= 0.75 against the hand labels).** It costs the
  user about 240 labels and is the only thing that tells a feature that
  measures the ball from one that measures "a crowded basketball frame".
  Without it the reason display could name a cue the column does not read.
- **Candidate 1's small-ball risk is the point of the comparison.** If its
  `ball` column reads under 0.75 against the labels while Candidate 2's
  reads above, that is the number that justifies a detector; if both read
  high and both pass bars 1-3, the choice rule prefers the model-free
  candidate unless the detector buys another 0.010.
- **Candidate 2's cost is in the app, not the evaluation.** The evaluation
  runs in onnxruntime; only the shippability checks touch tract. If the
  chosen detector passes the AUC bars but fails a shippability bar (an op
  tract lacks, a 150 MB file, 300 ms per image), the plan records
  "no-ship" and falls back to Candidate 1 if it passed, else to none. A
  smaller detector that ships but misses small balls is the likely trade.
- **Transformer detectors may not load.** D-FINE / RT-DETR / DEIM use
  deformable attention; if their export carries `GridSample` or a custom
  op, tract 0.23 without `tract-transformers` will refuse it. Step 3 drops
  them at the first failure and says which op, rather than re-exporting.
- **Pre-registration vs prompt tuning.** Candidate 1 allows one prompt
  revision, as the genre step did; the criterion reads the first set unless
  the revision was written blind. Writing prompts after seeing which frames
  fail would make bar 4 and bars 1-3 optimistic.
- **Overlap with the per-file-viewed plan.** Steps 5 and 6 rewrite the
  same functions that plan's Step 1 rewrites. Waiting for it costs
  calendar time; racing it costs a conflict in `rank.rs`, `index.rs`
  (`SCHEMA_VERSION`) and the cross-check test. The plan waits. A
  `RANK_VERSION` bump also throws away the user's stored fit until the
  next refit; if that plan's Step 2 (startup refit) is in by then, nothing
  is visible; otherwise `Refit Ranking` brings it back, which the docs say.
- **Per-genre gating is out of scope.** A ball column reads near zero on a
  frame without a ball, so a shared weight does little harm on a landscape
  burst; the sports-gated interaction is reported for the per-genre session
  (dlg-genre-weights) to pick up, not adopted here.
- **The labeled set is one photographer's basketball.** A gain measured here
  says nothing about soccer or volleyball (different ball sizes and
  contests); the prompts and the detector classes are generic, the
  thresholds are not. `ranking.md` of this plan states that.
- **Team separation by jersey color** (part of the request's Candidate 2
  sketch) is left out of the first evaluation: it needs a color model per
  game and a definition of "opposing" the labels do not carry. If Candidate
  2 ships, it is a follow-up with its own labels.

## Decision 1 (adopt / which / drop)

(Written in Step 3 with the criterion tables of Candidate 1, Candidate 2 and
the combination, the shippability checks, and the choice rule's outcome.
Until then: pending.)

## Progress

- (2026-10-14) Plan written with the pre-registered criteria, confirmed by the user; no score computed yet.
- (2026-10-14) Step 1 complete
