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

# Discard model: improve on `eye_focus` with the 1,200 blind labels, confirmed once on folders not yet sampled

## Purpose

Riffle's AI goal is to **suggest frames to discard** (sorted last, not hidden)
so the human picks from what remains; the label is **"OK to discard?"**. The
bar to beat is the plain `eye_focus` cue (the in-focus probability of the
eyes of the face nearest the AF point): weighted discard AUC **0.785**
[0.723, 0.844] on the model-search confirmation (round 2), above every
learned model so far (`G0 ⊕ ef` 0.684, `G0` 0.630;
[model-search Decision](../_archived/20261020-model-search/plan.md)), and
0.777 on round 1. The face-choice evaluation found no better face rule
(`af-whole` the only near-tie, +0.014 [−0.018, +0.052], filling more frames:
4.5% missing against 12.4%;
[facechoice.md, Decision](../_archived/20261023-face-choice/facechoice.md)).

This plan does the two remaining TODOs of the todo item "Core: improve on
`eye_focus` for the discard question using the 1,200 blind labels":

1. **train** a small, pre-registered set of discard scores on the 1,200 blind
   labels (both rounds), leave-one-folder-out over the 27 dump folders, and
   pick one winner by a mechanical rule; the question it tests is whether
   `eye_focus` plus a few technical columns, a fill for the frames where
   `eye_focus` is missing (12.4% of the dump, sorted last today, which is
   where `eye_focus`'s `PL` comes from: 4.44% at q5, 9.14% at q10), and a
   monotone or simple form can order the discard labels better than
   `eye_focus` alone;
2. **confirm once** on a fresh blind round of about 600 frames drawn from
   folders **not in the dump**, with the bars pre-registered before any label
   is read, and end with a Decision naming the follow-up.

**Offline evaluation only; no app behavior changes.** Selection happens on
reused labels (round 2 confirmed the model search; round 1 has been read by
four plans), so only the fresh round decides.

## Decisions (by the user, at planning time)

- **Fresh population: six folders, 14,525 frames.** The four ARW folders
  shot after the dump (`2026-10-11`, `2026-10-12a`, `2026-10-12b`,
  `2026-10-17`) **plus** `2026-09-13-a` and `2026-09-27-c`, whose culling the
  user confirmed is now finished (they were left out of the dump as
  unfinished on 2026-10-08). (Rejected: the four post-dump folders only,
  9,665 frames.)
- The user accepted the plan's proposals for the other choices in
  Trade-offs: pooled training weights (2a), round 2 primary with round 1 as
  replication (3a), the five bars without the extra cut-level `RR` condition
  and with `PL` read on the fresh folders' picks (4a), and no `G0` reading on
  the fresh folders (5a).

### What is known (investigated 2026-10-23)

- **Labels.** Round 1: `D:\Photos\tests\2026-10-18-discard-labels\`
  (`labels.tsv` 600 rows, 211 keep / 389 discard; `manifest.tsv` with `band`,
  `inclusion_prob`; `strata.tsv`; population = dump minus the 1,200 strict
  frames = 31,259). Round 2: `D:\Photos\tests\2026-10-20-model-search\`
  (600 rows, 248 keep / 352 discard; `manifest.tsv`, `strata.tsv` with 31
  cells; population = dump minus the 1,800 earlier-labeled frames = 30,659).
  Both are rows of the 27-folder dump `D:\Photos\tests\2026-10-12-rank-app\`
  (`features\<folder>.tsv`, `<folder>.output.txt`, `embed.tsv`,
  `genre_text.tsv`; 32,459 frames, 4,419 picks; loaded by the archived
  `gate.load` / `gate.build` / `folder_gate.extend`). Per-face dump:
  `D:\Photos\tests\2026-10-23-face-choice\faces\<folder>.tsv` (crop and
  whole sets per frame; per face `eye_focus`, `state`, `scored`, `ear`,
  `yaw` / `pitch` / `roll`, `score`, box; loaded by
  `facechoiceread.load_faces`). Every dump frame has a trusted AF point.
- **Known numbers to reproduce**: round 2's `eye_focus` (missing last) row:
  AUC 0.785 [0.723, 0.844] (cluster [0.693, 0.881]), `RR` 5.5 / 12.9 / 12.8%,
  `PL` 4.44 / 9.14 / 15.59%, removed 4.80 / 9.70 / 19.64% at q5 / q10 / q20,
  base rate 42.31% [35.85, 48.91]; round 1's: AUC 0.777 [0.716, 0.841], `RR`
  16.8 / 14.3 / 28.3%, base rate 48.45% [42.03, 55.11]; `af-whole` fills
  4.5% missing against `af`'s 12.4% on the dump (facechoice.md, "Results").
- **Harness, imported by path, none modified**: `badfilter` (`read_tsv`,
  `strat_counts`, `cluster_counts`, `graded_cut`, `finite_by_folder`,
  `aucs`, `auc_pair`, `quant`, `iv`, `civ`, `Tee`; it brings `rejectfilter`
  = `rf` (`HGB`, `load_ear`, `bottom`), `gate` (`load`, `build`, `TECH`,
  `CS`, `logit`, `table`), `folder_gate` (`extend`, `design`, `choose_c`),
  `folderwide` and the archived bad-photo `sample` = `bs` (`allocate`,
  `under`, `listing`, `sha256`, `cell`)); then `discardfilter` (`metrics`,
  `unweighted`, `counts`, `QS`, `PL_BAR`, `REMOVED_BAR`); `retargetfit.kish`;
  `searchfit`'s weight clip (5 × the median sampled `1/π`, then rescaled to
  mean 1) and inner-`GroupKFold` `C` choice with the `C = 1.0` fallback;
  `facechoiceread.load_faces` and its 0.8 floor; `confirmsample.py` (strata,
  allocation, the copy with SHA-256 checks, the audit hook on the earlier
  rounds' folders) and `confirmlabels.py` (rating 1 = keep, unmarked =
  discard, unexpected marks listed). **Do not call `badfilter.load`** (it
  opens the strict labels); load as `facechoiceread.py` does. **Name no
  script `sample.py`.**
- **Folders not in the dump** (label-free inventory of `D:\Photos\2026`,
  read-only):
  - **Four ARW folders shot after the dump and culled** (`.dop` picks and
    `Output\` exports present): `2026-10-11` (3,190 frames, 134 picks, 23
    rejects), `2026-10-12a` (3,185, 304, 20), `2026-10-12b` (1,615, 285, 22),
    `2026-10-17` (1,675, 205, 0): 9,665 frames, 928 picks.
  - `2026-09-13-a` (3,045 ARW, 620 picks, 1,424 unflagged `.dop`) and
    `2026-09-27-c` (1,815 ARW, 247 picks, 3 rejects, 1,565 unflagged), left
    out of the dump as "unfinished (culling not done)" on 2026-10-08; both
    now have `Output\` and `Output-sequenced\`, and the user confirmed their
    culling is finished. With the four above they make the **fresh
    population: six folders, 14,525 frames**.
  - Excluded: `2026-08-29-l` (211) and `2026-09-05` (411), Leica DNG without
    an AF point (the no-AF cue path, never exercised in the dump);
    `2026-06-02` / `06-06` / `06-09` / `06-21` / `06-27`, every frame a
    pick (no negatives, `PL` undefined); the January–May folders and
    `100LEICA`, no sidecars at all (no picks).
- **Reproducing the dump for new folders** (today's CLI): `riffle-cli
  features <dir> [threads]` (its header is a superset of the dump's;
  `gate.load` reads columns by name), the `Output\` listing as
  `features\<folder>.output.txt` (exports count as picks in `gate.load`;
  `_archived/20261008-burst-keep-score/dump.sh` has the one-liner),
  `riffle-cli embed <vision_model.onnx> siglip2 <dir> [threads] --near 404
  --out <embed.tsv>` (`gate.load` drops frames without a vector, so it is
  required even though no candidate uses the embedding; the model is
  `D:\Photos\tests\2026-10-12-embed-models\siglip2-b32-256\vision_model.onnx`),
  `riffle-cli facechoice <dir> <threads> --check --out <faces\folder.tsv>`
  (as `D:\Photos\tests\2026-10-23-face-choice\run.sh`), and a copy of the
  dump's `genre_text.tsv`. `compare.md` in the rank-app dump showed the CLI
  features equal the app's stored values, and facechoice `--check` reproduced
  the stored `eye_focus` on all 32,459 frames on 2026-10-23. Cost references:
  features about 11 min for the 27 folders on 24 threads, embed 18.8 ms per
  image, facechoice 1,571 s for 32,459 files on 12 threads, so the six fresh
  folders take about 25–30 min in all.
- **`.dop` flag**: `ShouldProcess` 0 = pick, 1 = reject, 2 = unflagged
  (`crates/core/src/dop.rs`); `gate.load`'s pick is `dop == Pick`, or
  exported in `Output\`, or (no `.dop` and `xmp == Pick`).
- **Python**: `D:\Photos\tests\.venv-eval\Scripts\python.exe -I -B` on
  script files only, never `python -` heredocs (the Git Bash stub hangs;
  kill a hung process by its PID, never by image name); run from the
  scratchpad; each tool call under 10 minutes, long runs in the background;
  check `git status` for stray `__pycache__/`. `cargo` through `mise exec --
  cargo ...` in Git Bash.
- **Where the data goes**: `D:\Photos\tests\2026-10-23-discard-model\`
  (`dump\` for the fresh folders, `scores_model.tsv`, `scores_fresh.tsv`,
  `manifest.tsv`, `strata.tsv`, `sample\`, `labels.tsv`, `*.out.md`). The
  library `D:\Photos\2026` is **read-only**.
- **Out of scope (do not touch)**: `rank.rs`'s `pairs`, `Labeled`, `fit`;
  the `views` table and its data; the dlg-genre-weights, dlg-folder-best and
  dlg-likely-pick-icon sessions; no app code changes. GitHub CI's Ubuntu job
  has been slow (apt about 26 min); doc-only PRs may wait.

### The candidates (pre-registered in Step 1, fit in Step 2, read in Step 3)

Every score is "higher keeps"; a missing score sorts last (`-inf` in the cut,
the folder's lowest finite value minus 1 in the AUC), as every earlier read.

- **References, not candidates**: `ef` (today's `eye_focus`, missing last),
  random (seed 0).
- **Columns.** `S` (3): logit `eye_focus` **filled** by the `af-whole` face's
  value where today's is missing (`facechoiceread`'s rule, the whole set's
  `nearest_face`), the indicator "no `eye_focus` today", ln relative
  sharpness (`gate.TECH`'s). `T` (10): `S` plus the indicator "no `eye_focus`
  after the fill", EAR of the judged face (`rf.load_ear`), |yaw|, |pitch|,
  |roll| (clipped as `gate.tech_row`), the indicator "no pose", and the
  number of whole-set faces at or above 0.8 capped at 3. NaN is mean-filled
  on the training folders for the logistic forms and kept for HGB.
- **Candidates (5)**: `ef-fill` (label-free rule: today's `eye_focus`, the
  missing filled by `af-whole`'s, the rest last); `LR/S` and `LR/T`
  (standardized L2 logistic, `C` by inner `GroupKFold` over `gate.CS` on the
  weighted discard AUC, `C = 1.0` when no `C` wins more than half the folds);
  `HGB-mono/T` (`rf.HGB` with `monotonic_cst`: keep rises with the
  `eye_focus` logit, sharpness and EAR, falls with |yaw|, |pitch|, |roll|,
  0 on the indicators and the face count); `HGB/T` (`rf.HGB` as is, the
  reading of what the constraints cost). Nothing else is searched: no
  embedding columns, no burst context, no hyperparameter grid.
- **Training**: the labeled frames of the 26 other folders, target
  `discard`, weights `1/π` of each frame's own round, **clipped at 5 × that
  round's median sampled `1/π` and rescaled to mean 1 within the round** (so
  the two rounds carry equal total weight; Kish ESS printed per fold with
  and without the clip). One leave-one-folder-out fit per fold; every dump
  frame scored by a model that did not see its folder.
- **Read (Step 3)**, per round with its own population, `π` and strata
  (round 2 primary, round 1 replication, as face-choice): weighted discard
  AUC with the stratified (1,000 draws, seed 0) and folder-cluster
  bootstraps; `DP` / `RR` / `PL` / removed at q5 / q10 / q20 with the
  per-folder bottom-q cut on the full dump; paired differences against `ef`
  (one count matrix per round); the **missing-face subset** (frames without
  today's `eye_focus`) and its complement read beside, so the text can say
  whether the gain sits in the frames `eye_focus` leaves missing.
- **Selection rule, mechanical**: among the candidates whose q5 or q10 cut
  passes the label-free conditions on the full dump (`PL` <= 5%, removed >=
  4.5%), the one with the highest weighted AUC on round 2; ties by round 1's AUC
  (amended 2026-10-24 by the user from q10 only, on Step 2's label-free
  readings, where no candidate's q10 `PL` is <= 5%; see discardmodel.md,
  "Amendment (2026-10-24)").
  **Stop rule**: the winner goes to the fresh round only if its AUC − `ef`
  point is above zero on **both** rounds; otherwise Steps 4–6 are skipped
  and the Decision (Step 7) is "nothing, keep `eye_focus`" (the model-search
  Decision's own condition for that branch).
- **Multiplicity**: 5 candidates × 2 rounds × 3 cuts, read without
  correction; hence the mechanical rule and the single fresh read. One
  photographer, mostly basketball.

### The confirmation (pre-registered in Step 5, before any fresh label)

- **Population**: every frame of the six fresh folders (no earlier sample to
  exclude). **Strata**: the winner's per-folder bottom bands `b05` / `b10` /
  `b20` on the fresh folders, an `efonly` band (`ef`'s bottom 20% outside
  the winner's bottom 20%, so "beats `ef`" is read on a sample holding both
  cuts), and per-folder `control` with a floor of 3; **600 frames, 120 per
  stratum**; `π` = drawn / cell population; one `default_rng(20261023)`
  generator in the order `b05`, `b10`, `b20`, `efonly`, the `control` cells
  by folder name, then the shuffle. Copies named `<nnnn>_<folder>_<file>`
  in `sample\`, no sidecar, SHA-256 checked, the sampler refusing to write
  under `D:\Photos\2026`.
- **Scores read once on the fresh labels**: the winner (refit once on all 27
  dump folders with the same recipe, scored on every fresh frame), `ef`,
  random; metrics as discard.md's with the fresh `1/π`; the stratified and
  cluster bootstraps; the base rate.
- **Bars**: a cut of the winner at **q5 or q10** is usable when all hold:
  1. weighted `RR` <= **25%**, stratified 97.5% quantile <= **35%**;
  2. `PL` <= **5%** (the fresh folders' picks, label-free);
  3. removes >= **4.5%** of the fresh population (label-free);
  4. `DP`'s stratified 2.5% quantile above the fresh base rate (point);
  5. **beats `ef`**: weighted AUC − `ef`'s, paired, stratified **2.5%
     quantile above zero** (the key bar; one value for both cuts).
  q20 is read and reported, not eligible. Best cut: the deepest passing
  one. A pass whose cluster interval crosses a bar is reported as fragile.
  There is no "beats `G0`" condition: `G0` is not the reference any more
  and is not computed on the fresh folders.
- **Follow-up rule**: (1) pass → an **in-app suggestion plan** (discard
  candidates sorted last at the best cut's depth, with a rescue loop); (2)
  close (at q5 or q10 the `RR` point <= 25% with conditions 2–3 holding and
  only quantile or "beats" conditions failing; or the AUC − `ef` point above
  zero with the interval spanning zero) → **more labels** (another fresh
  round, about 1.5 s per frame); (3) otherwise → **nothing, `eye_focus`
  stays** as the cue.

## Steps

- [x] Step 1: Pre-register the exploration in `discardmodel.md`, before any fit exists
  - Done when:
    - `discardmodel.md` in this plan folder has a "Pre-registration of the
      exploration" section merged **before `discardfit.py` exists and before
      any candidate has been fit or read**; the merge commit is the record.
      It states what is known (the numbers above with their sources), that
      the design was chosen after reading `eye_focus`'s and `af-whole`'s
      numbers on both rounds (selection bias the Decision weighs), the
      candidates, columns, forms, weights, the read, the selection rule, the
      stop rule and the multiplicity statement exactly as above, plus the
      user's decisions above with the rejected options.
    - It lists the fresh-folder inventory above (label-free) and names the
      fresh population as the user chose it (six folders, 14,525 frames).
    - `mise run lint` passes.
  - Implementation approach:
    - Doc-only PR; model the text on modelsearch.md's "Pre-registration of
      the exploration" and facechoice.md's "Pre-registration"; discard.md's
      vocabulary. Confirm `monotonic_cst` with NaN on sklearn 1.9.1 from the
      documentation only (modelsearch.md has a section on it); if
      unsupported, say `HGB-mono/T` uses mean-filled columns.

- [x] Step 2: `discardfit.py`, every candidate's held-out score on every dump frame, label-free readings only
  - Done when:
    - `discardfit.py` in this plan folder (`python -I -B discardfit.py
      <app-dump-dir> <face-choice-dir> <reject-filter-eval-dir>
      <strict-labels-dir> <discard-labels-dir> <model-search-dir> <out-dir>
      [jobs]`) builds the `S` / `T` columns
      from the dump and the per-face dump, fits the four learned candidates
      leave-one-folder-out on the 27 folders with the pooled, per-round
      clipped weights, builds `ef-fill`, and writes `scores_model.tsv`
      (`folder`, `file`, one column per candidate, full `repr` precision,
      every frame scored out of fold) into `<out-dir>`, stdout saved as
      `discardfit.out.md`.
    - It prints **label-free readings only**: per candidate and q the
      removed share and `PL` on the full dump with the two label-free
      verdicts; per fold the training counts, the clip cap per round and
      the Kish ESS with and without the clip; the chosen `C` per fold; the
      missing shares (today's `eye_focus` 12.4%, after the fill printed beside
      `af-whole`'s 4.5%, expected at or below it); wall time. The labels enter training only; **no score is
      read against `labels.tsv`**; an audit hook records the files opened
      under the two rounds' folders (`labels.tsv`, `manifest.tsv` and
      `strata.tsv` only, for the training weights), under the strict-labels
      folder (`manifest.tsv` only; its `labels.tsv` is never opened) and
      under the reject-filter-eval folder (`scores_reject.tsv` only).
    - `learnings.md` records the wall times; `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 1 merged. Import `badfilter` first (for `rf`, `gate`,
      `folder_gate`), then `discardfilter`, `retargetfit` (`kish`),
      `facechoiceread` by path (`load_faces`, `choose`). Reuse `searchfit`'s
      clip and inner-`C` recipe; joblib over folds; seconds to a few
      minutes (the training sets are at most 1,200 rows).

- [ ] Step 3: `discardread.py`, the exploration read of both rounds and the winner by the pre-registered rule
  - Done when:
    - `discardread.py` in this plan folder (same arguments, without `[jobs]`, and
      the same audit allow-list as Step 2, plus `scores_discard.tsv` under
      the model-search folder and `scores_model.tsv` in `<out-dir>`, as
      `facechoiceread.py`'s) loads the dump,
      both rounds' `labels.tsv` / `manifest.tsv` / `strata.tsv`, round 2's
      `scores_discard.tsv` and `scores_model.tsv`, prints Markdown tables saved as
      `discardread.out.md` in `<out-dir>` and **refuses to run when that
      file exists**: the checks (both rounds' `ef` rows and base rates
      reproduce; the strata recompute); per round the AUC table (every
      candidate and reference with stratified, cluster and unweighted
      values and the paired AUC − `ef`), the cut tables at q5 / q10 / q20
      (`DP`, `RR`, `PL`, removed, `RR` − `ef`), the missing-face subset and
      its complement; the **selection rule applied** with the winner named
      and the **stop rule's verdict** (fresh round or not).
    - `discardmodel.md` gains a "Results of the exploration" section with
      the tables and a "Reading" paragraph stating the facts (including
      where the gain sits) without deciding anything.
    - `learnings.md` records deviations; `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 2 merged. Reuse `facechoiceread.py`'s per-round cell
      construction, `auc_draws`, `cut_metrics` and `diff` (import by path or
      repeat the few lines, as that script did with `confirmread`). Metric
      only, about two minutes. If the stop rule says "no fresh round", mark
      Steps 4–6 as skipped in this plan and go to Step 7.

- [ ] Step 4: `freshdump.sh`, the dump of the fresh folders, label-free
  - Done when:
    - `freshdump.sh` in this plan folder runs, per fresh folder and
      resumable (a folder whose outputs exist is skipped), a copied release
      `riffle-cli.exe`: `features` into `dump\features\<folder>.tsv`, the
      `Output\` listing into `dump\features\<folder>.output.txt`, `embed
      ... siglip2 ... --near 404` appended into `dump\embed.tsv`, and
      `facechoice --check` into `dump\faces\<folder>.tsv`, with stderr in
      `dump\logs\`; `dump\genre_text.tsv` is a copy of the rank-app dump's.
    - The dump exists under `D:\Photos\tests\2026-10-23-discard-model\dump\`
      for every fresh folder, and `learnings.md` records: the file count per
      folder equals the folder's RAW count, 0 errors, 0 facechoice `--check`
      mismatches, the share of frames with a trusted AF point, the pick and
      reject counts `gate.load` derives (and how many come from `Output\`),
      the missing-`eye_focus` share before and after the `af-whole` fill,
      and the wall times. The library is only read (size and mtime listing
      before and after, as `bs.listing`).
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 3 merged with a "fresh round" verdict. Follow
      `_archived/20261008-burst-keep-score/dump.sh` and the face-choice
      `run.sh`; run in the background, one folder at a time, and wait with
      a bounded `tail -f | grep -m1`, never a sleep. A small label-free
      Python check (`gate.load` on the new dump) may print the counts.

- [ ] Step 5: Pre-register the confirmation and draw the fresh blind sample
  - Done when:
    - `discardmodel.md` has a "Pre-registration of the confirmation" section
      merged **before any fresh label exists**: the frozen winner (as Step 3
      named it) and its refit recipe on all 27 folders, the fresh population
      and strata, the allocation and seed, the bars, the follow-up rule and
      the multiplicity statement, exactly as "The confirmation" above with
      the user's decisions recorded; an "As drawn" paragraph (label-free) is
      added after the draw.
    - `freshsample.py` in this plan folder (`python -I -B freshsample.py
      <app-dump-dir> <face-choice-dir> <discard-labels-dir>
      <model-search-dir> <fresh-dump-dir> <out-dir> [n] [--no-copy]`)
      refits the winner once on all 27 dump folders, scores every fresh
      frame (written as `scores_fresh.tsv` with `ef` and `ef-fill` beside),
      cuts the strata, draws, writes `manifest.tsv` and `strata.tsv`, prints
      the tables and checks (the sum of `1/π` equals the population; the
      winner's and `ef`'s `PL` and removed shares on the fresh folders;
      the overlap table for `efonly`), copies the sample with the SHA-256
      checks and refuses to write under `D:\Photos\2026`.
    - `labeling.md` in this plan folder: the model-search guide with the new
      path and folder count, the same "OK to discard?" definition, rating 1
      = keep, `aiEnabled` off, never Pick / Reject, the stopping rule.
    - `mise run lint` passes. **The plan pauses here for the user's
      labeling** (about 1.5 s per frame, 600 frames about 15 minutes).
      Nothing reads the sample's sidecars before the user says "done".
  - Implementation approach:
    - Assumes Step 4 merged. Reuse `confirmsample.py` by path for its
      helpers and copy loop pattern (`--no-copy` first, then the copy in the
      background, about 27 GB and 15 minutes). The fresh dump loads through
      `gate.load` on `<fresh-dump-dir>` with the copied `genre_text.tsv`;
      the winner's `T` columns on the fresh frames use the fresh per-face
      dump; mean fills and standardization come from the 27-folder fit.

- [ ] Step 6: The confirmatory read, once
  - Done when:
    - After the user says "done", `freshlabels.py` (the `confirmlabels.py`
      recipe with the five strata `b05`, `b10`, `b20`, `efonly`, `control`)
      writes `labels.tsv` into the data folder; `learnings.md` records the
      session length and anything odd the user reported.
    - `freshread.py` in this plan folder reads **once** (it refuses to run
      when `freshread.out.md` exists): the winner, `ef`, `ef-fill` and
      random at q5 / q10 / q20 on the fresh labels with the fresh `π`, the
      stratified (five strata) and cluster bootstraps, the base rate, the
      bars applied with a verdict per cut and the follow-up branch; stdout
      saved as `freshread.out.md`. Every computation runs before anything
      is printed, the label-free checks first.
    - `discardmodel.md` gains a "Results of the confirmation" section with
      the tables, the verdict and a "Reading" paragraph; the exploration
      numbers of the winner (round 2 and round 1) beside the confirmation
      numbers so the shrinkage is visible.
    - `learnings.md` records deviations; `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 5 merged and the labeling done. Reuse Step 3's functions
      and `confirmread.py`'s cell construction from `strata.tsv`. Dry-run
      the script once on synthetic labels in the scratchpad (as face-choice
      did) so the real read cannot crash after opening the labels.

- [ ] Step 7: The Decision
  - Done when:
    - A "Decision" section in this plan, written against the Step 5 bars
      (or, if the stop rule ended the plan at Step 3, against the stop rule):
      whether the winner passed, what it would suggest (depth, `RR`, `PL`),
      how far the exploration numbers shrank on the fresh folders, whether
      the gain sat in the missing-face frames, and the follow-up by the
      pre-registered rule: an in-app suggestion plan (with the surface it
      needs: discard candidates sorted last at the best cut's depth, a
      rescue loop, and what the scan must store beyond today's columns),
      more labels, or nothing / keep `eye_focus`; the selection-bias and
      one-photographer caveats weighed; the todo item's follow-up named.
    - `learnings.md` records the compute and labeling time spent;
      `mise run lint` passes.
  - Implementation approach:
    - Doc-only PR; do not move a bar.

## Trade-offs and risks

Choices made at planning time (see "Decisions" above; Step 1 and Step 5
record them):

1. **The fresh population.** (a) The four post-dump ARW folders only (9,665
   frames, 928 picks). (b) Add `2026-09-13-a` and `2026-09-27-c` (14,525
   frames): more frames and two more folder-cluster units. **Chosen: (b)**,
   the user confirmed their culling is finished. Not proposed: the Leica DNG
   folders (no AF point: the no-AF path, which no label has ever covered; a
   separate question).
2. **Training weights across the two rounds.** (a) Pool the 1,200 labels,
   each frame weighted by its own round's `1/π`, clipped and rescaled to
   mean 1 within the round. (b) Train on round 2 only. **Chosen: (a).**
3. **The selection statistic.** (a) Round 2 primary with round 1 as
   replication, the stop rule needing a positive AUC − `ef` point on both.
   (b) One pooled weighted AUC over both rounds. **Chosen: (a).**
4. **The bars.** (a) The five conditions above. (b) Add a cut-level
   condition, "`RR` at the cut not above `ef`'s at q5 by more than +5 points
   (97.5% quantile)". **Chosen: (a)**, with `PL` read on the fresh folders'
   picks rather than the dump's.
5. **A `G0` reading on the fresh folders.** (a) None. (b) Add it as a
   reading. **Chosen: (a).**

Risks:

- **The gain may be nothing but the fill.** If `ef-fill` (label-free) wins,
  the learned forms added nothing; that is a valid outcome and the cheapest
  to ship. If no candidate beats `ef` on both rounds, the stop rule ends the
  plan without a labeling round, which is what the model-search Decision
  asked for.
- **Shrinkage.** The model-search winner lost 0.079 AUC from exploration to
  confirmation, and the fresh folders are a new population (different
  dates, maybe different genres), so the confirmation is harder than round
  2 was; the Decision reports the shrinkage.
- **`PL` on the fresh folders rests on their picks** (928 on the four
  post-dump folders plus 867 on `2026-09-13-a` / `2026-09-27-c`); a folder
  culled lightly inflates `PL`. Step 4 prints the pick shares per folder
  before any label.
- **The missing-face frames** carry today's `PL` (sorted last, they hold
  picks); a candidate that fills them may lower `PL` and raise `RR` at q5
  at once, which the missing-face subset read makes visible.
- **The winner's refit on all 27 folders** differs from the LOFO fits that
  were read; the refit is the pre-registered scoring model and its
  reproduction (a spot fold of the LOFO recipe) is checked in Step 5.
- **Selection on reused labels** is the design's bias; the mechanical rule,
  the stop rule and the one fresh read are the protections.

## Progress

- (2026-10-23) Plan written
- (2026-10-24) Step 1 complete
- (2026-10-24) Step 2 complete
- (2026-10-24) Selection rule amended by the user: q5 or q10 eligible (was q10 only), on label-free readings only
