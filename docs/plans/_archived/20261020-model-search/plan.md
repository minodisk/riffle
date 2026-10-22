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

# Model search: an offline exploration of pick-suggestion and discard-suggestion models with a guarded confirmation

## Purpose

Riffle should **suggest** two things in a folder: frames to pick (the top)
and frames to reject, hidden or sorted last (the bottom). The user asked for
trial-and-error to find better-performing models for both. What exists:

- **`G0`** (`HistGradientBoostingClassifier` on 46 columns, trained on the
  dump's picks, held out by folder: 11 technical columns, 3 burst-context
  columns, 32 whitened PCA dimensions of the SigLIP embedding). On the 600
  blind "OK to discard?" labels its discard AUC is 0.710; at q5 `RR` 17.3%
  [11.3, 24.0] and `PL` 0.68%; under the user's looser hide / sort-last bar
  it meets every condition except "beats `G0`", which it cannot meet
  ([discard-retarget Decision](../_archived/20261019-discard-retarget/plan.md)).
  Folder-wide, frame AUC 0.670 (mean over the 25 qualifying folders).
- **`eye_focus` alone** has the best discard AUC, 0.777, but a high `PL`
  (4.44% at q5, 9.14% at q10).
- **`D0` / `D0+B`** (the discard target on the same features) read a higher
  AUC (0.761) but cut worse at the bottom (`RR` 32.7–42.7%, `PL` 2.3%
  at q5); Kish ESS of their `1/π` training weights only 166–195 on 504–597
  frames. The burst-relative `B` columns moved nothing.
- **`W0`** (the shipped in-burst model) folder-wide: frame AUC 0.666,
  R-precision 28.6% (chance 13.8%), lift at the top tenth 2.32
  ([folder-wide-ranking-eval](../_archived/20261015-folder-wide-ranking-eval/plan.md)).

Every round so far changed one thing at a time against a bar. This plan is
the **exploratory** round the user asked for: a bounded search over feature
sets (embedding only, technical + context only, all; embedding PCA dims 0 /
4 / 8 / 16 / 32), model forms (HGB as is, a regularized HGB, HGB with
monotone constraints on the cues, L2 and L1 logistic), targets (pick,
discard with clipped `1/π` weights) and score combinations (`G0` with
`eye_focus`), read on **two tasks**, each with its own metrics: **pick
suggestion** (the folder's top q% against the dump's picks) and **discard
suggestion** (the folder's bottom q% against the blind discard labels). It
ends with **one confirmatory read** of one winner per task, behind an
explicit guard against overfitting the reused labels, and a Decision naming
the follow-up: an in-app suggestion plan, more labels, or nothing. **No app
code changes in this plan.**

The user's stated goal (auto memory, `goal-absolute-photo-quality`) was
mainly to cull the bad; on 2026-10-20 the user explicitly asked for pick
suggestions too, so both tasks are in scope. Whenever a reading optimizes
the top rather than the bottom, the text says so.

### Choices (decided by the user on 2026-10-20, before Step 1)

1. **Overfitting guard: the hybrid form of (b).** The **discard** task
   explores freely on all 27 folders and 600 labels and is confirmed by a
   **fresh blind round** of 600 frames (about 30 minutes) drawn from the dump
   minus the 1,800 labeled frames. The **pick** task is confirmed on a
   **lockbox** of about 7 folders drawn in Step 1 (20–30% of the frames, >= 5
   qualifying pick folders); the pick exploration uses the other folders
   only, and the lockbox folders' picks are not read against a pick
   candidate until Step 5. (Rejected: (a) a lockbox for both tasks; (b) a
   fresh round with no pick guard.)
2. **Pick bar: relative.** Mean `P@10` over the qualifying folders read >=
   +2.0 points above `W0`'s, with the folder-bootstrap 2.5% quantile above
   zero and at least as many folders at or above `W0` as 16 of 25 (scaled
   to the lockbox's qualifying folders). (Rejected: absolute `P@10` >= 40%.)
3. **Discard bar: the looser one** (`RR` <= 25%, 97.5% quantile <= 35%),
   plus "beats `G0`" and "beats `eye_focus`-alone"; the latter **in AUC with
   the paired 2.5% quantile above zero**. (Rejected: the strict 15% / 25%;
   the point-above form.)
4. **Search space: full, 69 candidates.** (Rejected: the reduced 25.)
5. Smaller choices taken as proposed: the discard selection depth is q10;
   the weight cap is 5 × the median sampled `1/π` (about 50).

### What is known (investigated 2026-10-20)

- **Data.** The dump of 27 folders and 32,459 frames
  (`D:\Photos\tests\2026-10-12-rank-app\`), read by the archived
  `gate.load` / `gate.build` / `folder_gate.extend` into `d` (`emb` 768-d
  unit vectors, `bursts`, `tech` 11 columns, `context` 3, `pick`, `reject`,
  `size`, `single`, `weight` = per-folder class weights); the raw EAR via
  `rf.load_ear`. Stored held-out scores: `W0`, `F0+ctx` in
  `D:\Photos\tests\2026-10-15-folder-wide-eval\scores.tsv`; `R0` / `G0` in
  `D:\Photos\tests\2026-10-16-reject-filter-eval\scores_reject.tsv`;
  `G0+B` / `D0` / `D0+B` in
  `D:\Photos\tests\2026-10-19-discard-retarget\scores_retarget.tsv`;
  `burst_features.tsv` there too (not used here).
- **Labels.** 600 blind discard labels
  `D:\Photos\tests\2026-10-18-discard-labels\labels.tsv` (211 keep / 389
  discard; `band`, `inclusion_prob`; `strata.tsv`, `manifest.tsv` beside
  it; population = dump minus the 1,200 strict-sample frames = 31,259
  frames; `1/π` from 2.3 to 238). 1,200 strict bad labels
  `D:\Photos\tests\2026-10-17-bad-labels\labels.tsv` (80 bad; its own
  `stratum` / `π`), the secondary coherence check only. The dump's picks
  (4,419 on the full dump) and explicit rejects.
- **Known numbers to reproduce** (reference checks): discard.md's `G0` /
  `eye_focus` / random rows and AUCs (above); the discardable base rate
  48.45% [42.03, 55.11]; folderwide.out.md's `W0` frame AUC 0.666 and
  R-precision 28.6 over the 25 qualifying folders (>= 50 frames, >= 5
  picks); rejectfilter.out.md's `G0` frame AUC 0.670.
- **Design functions.** `folder_gate.design(d, train_mask, variant)` with
  `"full"` (tech + context + 32 PCA), `"noemb"` (tech + context), `"emb"`
  (32 PCA only), `"context"`; `PCA_DIMS` is the module constant
  `gate.PCA_DIMS` = 32, so other dimensionalities need a local design
  function (same recipe: mean-fill the tech NaN on the training folders,
  `PCA(n, whiten=True, random_state=0)` fit on the training folders,
  standardize on them) rather than an edit of the archive.
  `rejectfilter.g0_fold` restores the tech NaN for HGB; `badmodel.fold`
  has the `L0` logistic recipe (standardized, class-balanced weights,
  `C=1.0`); `folder_gate.choose_c` the inner `GroupKFold` choice of `C`
  over `gate.CS` = (0.01, 0.1, 1, 10); `retargetfit.d0_fold` the
  discard-target recipe (labeled frames of the other folders, `1/π`
  rescaled to mean 1, `-decision_function` so higher keeps).
  `rf.HGB` = 200 iterations, learning rate 0.05, 15 leaves, l2 1.0, seed 0.
- **Metric harnesses, imported by path, none modified**: `badfilter` (bring
  it first; it imports `rejectfilter`, `gate`, `folder_gate`, `folderwide`
  and the archived bad-photo `sample`): `graded_cut`, `strat_counts`,
  `cluster_counts`, `ratio`, `quant`, `aucs`, `finite_by_folder`, `Tee`,
  `read_tsv`; then put `_archived/20261018-discard-labels` on `sys.path`
  and import `discardfilter` (`metrics`, `unweighted`, `counts`, `QS`,
  `PL_BAR`, `REMOVED_BAR`); `retargetread.py` has the per-draw AUC loop
  and the paired-effect pattern; `folderwide` has `r_precision`,
  `boot_folders`, `pooled`, `MIN_FRAMES` / `MIN_PICKS`; `folder_gate` has
  `paired`, `signed_rank`, `mean_sd`. That folder's own `sample.py` would
  shadow the bad-photo `sample` if imported first. **Name no script of
  this plan `sample.py`.**
- **Python**: `D:\Photos\tests\.venv-eval\Scripts\python.exe -I -B`
  (sklearn 1.9.1, numpy 2.5.4; keep them pinned), on script files only,
  run from the scratchpad; never `python -` heredocs (the Git Bash stub
  hangs; kill a hung process by PID). Check `git status` for stray
  `__pycache__/`.
- **Where the data goes**: `D:\Photos\tests\2026-10-20-model-search\`.
  The library `D:\Photos\2026` is read-only; the fresh blind sample is
  copied under the data folder as discard-labels did.
- **Compute reference**: 27 HGB folds on about 31k rows took 17–32 s on 8
  jobs; the discard target 15 s; metric reads 12–16 s. The full search (66
  fits) should stay under about an hour.
- **Out of scope (do not touch)**: `rank.rs`'s `pairs`, `Labeled`, `fit`;
  the `views` table and its data; the dlg-genre-weights, dlg-folder-best and
  dlg-likely-pick-icon sessions. No app code changes.

### The search space (69 candidates)

- **Feature sets (7)**: `T+C` (11 technical + 3 context, no embedding);
  `All@4`, `All@8`, `All@16`, `All@32` (technical + context + k whitened
  PCA dims); `Emb@8`, `Emb@32` (PCA dims only). `All@32` is `G0`'s design.
- **Model forms (5)**: `HGB` (`rf.HGB` as is); `HGB-reg` (a fixed
  regularized setting, e.g. 100 iterations, 7 leaves, l2 10,
  `min_samples_leaf` 50, decided in Step 1, not tuned); `HGB-mono` (`rf.HGB`
  plus `monotonic_cst`: keep rises with logit `eye_focus`, `eyes_open`,
  both sharpness columns and `tier`, falls with |yaw|, |pitch|, |roll|;
  0 on the missing indicators, the context and the PCA columns; not
  defined on `Emb@*`); `LR-L2` (standardized, `C` by inner `GroupKFold`
  over `gate.CS`); `LR-L1` (`penalty="l1"`, `solver="liblinear"` or
  `"saga"`, same `C` grid). 7 × 5 − 2 = **33 designs**.
- **Targets (2)**: `pick` (every frame of the training folders, the dump's
  per-folder class weights rescaled to mean 1, as `g0_fold`); `discard`
  (the labeled frames of the training folders, **clipped `1/π`**: the
  weight capped at 5 × the median sampled `1/π`, about 50, then rescaled to
  mean 1; Kish ESS reported per fold). 33 × 2 = **66 fitted candidates**,
  each one leave-one-folder-out fit per fold, every frame scored by a
  model that did not see its folder.
- **Score combinations (3)**, label-free to build, on `G0`: `G0 ⊕ ef`
  (the mean of the per-folder ranks of `G0` and `eye_focus`, a missing
  `eye_focus` taking `G0`'s rank alone); `G0 | ef-rule` (frames with
  `eye_focus` < `gate.GOOD_EYE_FOCUS` sorted below the rest, each part by
  `G0`); `stack` (a 3-column logistic on the held-out `G0` score, logit
  `eye_focus` and log sharpness, trained on the discard labels with clipped
  weights, held out by folder). The same two combinations applied to the
  pick-side winner are printed as a reading only, not as candidates.
- **References, not candidates**: `G0`, `W0`, `F0+ctx`, `eye_focus`
  (missing last), random (seed 0), and `D0` (unclipped weights, stored) as
  the clipped-weight sensitivity reading.
- Nothing else is searched: no hyperparameter grid beyond the two fixed
  HGB settings and the logistic `C`, no feature engineering beyond PCA
  dimensionality.

### The two tasks and their metrics

- **Pick suggestion (top)**, on the qualifying folders, label-free except
  for the dump's picks: per folder, the top `floor(q · n / 100)` frames by
  score (ties by the seed-0 draw), q = 5 / 10 / 20; **`P@q`** the pick share
  among them, **`R@q`** the share of the folder's picks among them,
  R-precision as before, and the frame AUC; the mean over the qualifying
  folders with the per-folder SD, the pooled value, and paired per-folder
  differences against the references with `fw.boot_folders` and the
  Wilcoxon signed-rank p. This task optimizes the top; it is reported as
  such.
- **Discard suggestion (bottom)**, exactly discard.md's: per folder on the
  full dump, the bottom `floor(q · n / 100)` by score (`bf.graded_cut`),
  q = 5 / 10 / 20; `DP`, `RR`, `PL` (full dump, label-free), `DR`, `MR`,
  `DO`, removed share (population); the weighted discard AUC; HT weights
  `1/π`; the 1,000-draw stratified bootstrap (one count matrix for every
  candidate, so differences are paired) and the folder-cluster bootstrap
  beside it; the strict-bad coherence check on the winner only.

## Steps

- [x] Step 1: Pre-register the exploration in `modelsearch.md`, before any new score exists
  - Done when:
    - `modelsearch.md` in this plan folder has a "Pre-registration of the
      exploration" section merged **before `searchfit.py` exists and
      before any score of a candidate has been computed or read**; the
      merge commit is the record. It states what is known (the numbers
      above, quoted with their sources), that this plan's design was
      chosen after reading `G0`'s, `eye_focus`'s and `D0`'s numbers on the
      same 600 labels (a selection bias the Decision weighs), and the
      user's choices with the rejected options.
    - It fixes: the search space exactly as above with the candidate count
      (69) and the two HGB settings and the weight cap written out; the two
      tasks' metrics; the **selection rule per task**, applied mechanically
      in Step 3:
      - discard: among the candidates whose q10 cut passes the label-free
        conditions (`PL` <= 5% on the full dump, removed >= 4.5% of the
        population), the one with the highest weighted `DP@10`; ties by
        the weighted discard AUC; the q5 ranking printed beside it as a
        reading; a candidate's rank at q5 does not change the winner;
      - pick: among all candidates, the one with the highest mean `P@10`
        over the qualifying exploration folders; ties by mean frame AUC;
      - one winner per task; a winner may be the same candidate for both
        tasks; the references are not eligible.
    - It fixes the **guard** (choice 1, hybrid): the **pick lockbox**
      folders drawn here by `numpy.random.default_rng(0)` from the 27 under
      the constraints that they hold 20–30% of the frames and at least 5
      qualifying pick folders, listed by name with their frame and pick
      counts. The pick-task reads of Step 3 use only the other folders; the
      candidates are fit leave-one-folder-out among the exploration folders
      for the pick task, so no lockbox folder's frame or pick enters a pick
      fit or a pick read before Step 5. The discard task explores on all 27
      folders and 600 labels (the lockbox folders' frames and labels
      included), and its confirmation is the fresh blind round of Step 4 (600
      frames, about 3 s per frame, about 30 minutes; the population the dump
      minus the 1,800 already-labeled frames; the strata fixed in Step 4).
      Where the pick and discard reads need different fits (the pick
      winner refit without the lockbox, the discard winner fit on all 27
      folders), the pre-registration says which fit each read uses.
    - It fixes the multiplicity statement (69 candidates × 2 tasks × 3
      cuts, read without correction, which is why the selection rule is
      mechanical and the confirmation is a single read) and the population
      caveat (one photographer, mostly basketball).
    - `mise run lint` passes.
  - Implementation approach:
    - Doc-only PR. Model the text on retarget.md's Pre-registration and
      keep discard.md's vocabulary. Confirm the `monotonic_cst` + NaN
      combination on sklearn 1.9.1 by reading the sklearn documentation
      only (no fit); if unsupported, the pre-registration says `HGB-mono`
      uses the mean-filled technical columns. The lockbox draw may run a
      small label-free script (folder sizes and pick counts only).

- [x] Step 2: `searchfit.py`, every candidate's held-out score on every dump frame, label-free readings only
  - Done when:
    - `searchfit.py` in this plan folder (`python -I -B searchfit.py
      <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir>
      <discard-labels-dir> <strict-labels-dir> <retarget-dir> <out-dir>
      [jobs]`) fits the 66 candidates as pre-registered and builds the 3
      combinations, and writes the scores (`folder`, `file`, one column per
      candidate, higher keeps, full `repr` precision; every frame scored out
      of fold) into `<out-dir>`, the pick-task fits (exploration folders
      only, lockbox rows `nan`) and the discard-task fits (all 27 folders)
      in separate files as the pre-registration fixes, stdout saved as
      `searchfit.out.md`.
    - It prints **label-free readings only**: per candidate and q the
      removed share and `PL` (full dump) with the two label-free verdicts;
      per discard-target candidate the per-fold training counts and Kish
      ESS with and without the clip; the chosen `C` per fold for the
      logistic forms; wall time per candidate and in total; the checks:
      `All@32 / HGB / pick` on all 27 folders reproduces the stored `G0` to
      about 1e-5 on a spot fold; `All@32 / HGB / discard` with the clip
      disabled reproduces the stored `D0` on a spot fold; every written
      score finite; no lockbox folder's frame enters any pick-task training
      set; the strict labels are not opened.
    - **No candidate score is read against `labels.tsv` or against the
      picks** (the labels and picks enter training only); `learnings.md`
      says so and records the wall times.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 1 merged. Import `badfilter` first, then
      `discardfilter`, then `retargetfit` (for `base_design`, `d0_fold`,
      `kish`) from the archive. A local `design(d, train_mask, tech, dims)`
      replicates `folder_gate.design`'s recipe for the other PCA sizes;
      check it equals `folder_gate.design(d, m, "full")` at 32 and
      `"noemb"` at 0 on one fold.
    - joblib over folds as the archived scripts (8 jobs); run in the
      background with the output file, and measure whether the total stays
      under about an hour; if a form is much slower (L1 with `saga`), note
      it and keep going.

- [x] Step 3: `searchread.py`, the exploration read of both tasks and the winners by the pre-registered rule
  - Done when:
    - `searchread.py` in this plan folder (same arguments as Step 2)
      loads the dump, the stored scores, the candidate scores, the discard
      labels and `strata.tsv`, and prints Markdown tables saved as
      `searchread.out.md` in `<out-dir>`: the reference checks (`G0`,
      `eye_focus`, random reproduce discard.md on all 27 folders; `W0` /
      `F0+ctx` reproduce folderwide.out.md when the code path is run on all
      27 folders with a `--check-all` flag that reads no candidate against
      the picks); the **pick table** on the exploration folders (every
      candidate and reference: mean `P@5` / `P@10` / `P@20`, `R@10`,
      R-precision, frame AUC, pooled values); the **discard table** (every
      candidate and reference at q5 / q10 / q20: `DP`, `RR`, `PL`, `DR`,
      `DO`, removed, intervals; the AUC table); the **reading aids**: the
      PCA-dimension curve per form and target, the form comparison at
      `All@32`, the target comparison per design, each as paired
      differences against `G0` with stratified and cluster intervals; the
      **selection rule applied** per task with the winner named and its
      runner-up; the strict-bad coherence check on the discard winner's
      bands.
    - `modelsearch.md` gains a "Results of the exploration" section with
      these tables and a "Reading" paragraph stating the facts without
      deciding anything about the bars.
    - `learnings.md` records deviations and the choices the text left open.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 2 merged. Reuse `discardfilter.metrics` / `counts`,
      `retargetread`'s per-draw AUC loop, `folderwide.r_precision` /
      `boot_folders` / `pooled`; add the top-q precision and recall next
      to `r_precision`. Metric-only, seconds to a minute to run.

- [x] Step 4: Pre-register the confirmation (the frozen winners and the bars) and draw the fresh blind sample
  - Done when:
    - `modelsearch.md` has a "Pre-registration of the confirmation" section
      merged **before any lockbox pick or fresh label is read**. It names
      the two winners (one per task) exactly as Step 3's rule chose them,
      and fixes the **bars**:
      - **discard**: a usable discard cut of the discard winner at q5 or
        q10 (q20 read too) is one with weighted `RR` <= 25% and its
        stratified 97.5% quantile <= 35%, `PL` <= 5% (full dump), removed
        >= 4.5%, `DP`'s 2.5% quantile above the discardable base rate
        measured on the confirmation labels, **beats `G0`** (weighted `DP`
        minus `G0`'s at the same q with the paired 2.5% quantile above
        zero) and **beats `eye_focus`-alone** (weighted discard AUC minus
        `eye_focus`'s with the paired 2.5% quantile above zero); best cut
        the deepest passing one;
      - **pick**: on the lockbox's qualifying folders, mean `P@10` >= +2.0
        points above `W0`'s with the folder-bootstrap 2.5% quantile above
        zero and at least 64% of the lockbox's qualifying folders (16 of 25
        scaled) at or above `W0`; frame AUC and `R@10` reported beside;
      - the **follow-up rule**: (1) a winner passes its bars → an in-app
        suggestion plan for that task (discard: sorted-last candidates with
        a rescue loop at the best cut's depth; pick: a strip mark or sort
        for the folder's top 10%), (2) no pass but the discard winner's
        `RR` point meets the bar with only a quantile or the "beats"
        condition failing, or its `DP` − `G0` point is positive with the
        interval spanning zero (for pick: the `P@10` gain point is positive
        with the interval spanning zero) → **more labels or more folders**
        (sized at about 3 s per frame), (3) otherwise → **nothing**;
        applied per task.
    - `confirmsample.py` in this plan folder (`python -I -B
      confirmsample.py <app-dump-dir> ... <out-dir> [n]`) draws the fresh
      sample as the archived discard-labels `sample.py` did: population the
      dump minus the 1,800 frames of the two earlier samples; strata the
      discard winner's per-folder bottom bands `b05` / `b10` / `b20` on the
      full dump, a `g0only` band (frames in `G0`'s bottom 20% but not the
      winner's, so "beats `G0`" is read on a tight paired sample) and
      per-folder `control` with a floor of 3; 600 frames split evenly over
      the five strata (allocation fixed here); `manifest.tsv` and
      `strata.tsv` with `π`; shuffled copies named `<nnnn>_<folder>_<file>`
      in `<out-dir>\sample\` (no sidecar; refuses to write under
      `D:\Photos\2026`; copies checked by size and SHA-256); and
      `labeling.md` in this plan folder, the discard-labels guide with the
      new path and the same "OK to discard?" definition, rating-1 key,
      `aiEnabled` off, stopping rule.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 3 merged. Doc PR plus the sampler; import the archived
      discard-labels `sample` by path after `badfilter` and reuse its
      allocation and copy helpers; print tables and checks before copying;
      copy in the foreground or in resumable chunks.

- [x] Step 5: The confirmatory read, once
  - Done when:
    - The user labels the sample folder (manual; the plan pauses here),
      then `confirmlabels.py` (the archived discard-labels `labels.py`
      recipe: rating 1 = keep, unmarked = discard, unexpected marks listed)
      writes `labels.tsv` into the data folder; `learnings.md` records the
      session length and anything odd the user reported.
    - `confirmread.py` in this plan folder reads **once**: the discard
      winner, `G0`, `eye_focus` and random at q5 / q10 / q20 on the fresh
      labels with the new `π`, the stratified (five strata) and cluster
      bootstraps, the base rate, the bars applied with a verdict per cut;
      and the pick winner (refit on the exploration folders as fixed) on
      the lockbox folders' picks against `W0`, the bars applied. Its stdout
      is saved as `confirmread.out.md`.
    - `modelsearch.md` gains a "Results of the confirmation" section with
      the tables, the verdict per winner and a "Reading" paragraph; the
      exploration numbers of the same winners are shown beside the
      confirmation numbers so the shrinkage is visible.
    - `learnings.md` records deviations; `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 4 merged. Reuse Step 3's functions; the fresh sample's
      strata differ from discard-labels', so the count matrix is built
      from the new `strata.tsv` as `discardfilter.counts` does with its
      own cell list. Do not re-read the exploration labels against a new
      score here.

- [x] Step 6: The Decision
  - Done when:
    - A "Decision" section in this plan, written against the Step 4 bars
      with the confirmation numbers: per task, whether the winner passed,
      what it would suggest (depth, `RR` / `PL` or `P@10` / `R@10`), how
      far the exploration numbers shrank on the confirmation (the measure
      of how much the search overfit), what the search taught about the
      space (the dimension curve, forms, targets, the combinations, in
      one paragraph each, as readings), and the follow-up by the
      pre-registered rule: an in-app suggestion plan (with the surface it
      needs), more labels, or nothing; stated as a recommendation with the
      conditions under which it would change and the selection-bias caveat
      weighed.
    - `learnings.md` records the compute and labeling time spent;
      `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 5 merged. Doc-only PR; do not move a bar.

## Trade-offs and risks

- **The choices** are recorded under Purpose, "Choices", with the rejected
  options.
- **The pick lockbox is small.** About 7 folders (5 or more qualifying)
  make the pick confirmation's folder bootstrap wide; a real +2-point gain
  may fail the interval condition. A fail is then weakly informative, and
  the Decision says so.
- **Under the looser discard bar `G0` q5 / q10 already meet conditions
  1–4**, so a pass rests on the "beats" conditions; "beats `eye_focus`-alone"
  in AUC with the paired interval is demanding (the best discard AUC seen is
  0.761 against 0.777, so the search must find about +0.05 over the best
  known model).
- **Selection bias is the point of the design.** 69 candidates × 2 tasks ×
  3 cuts are read on the same labels the design was chosen after; the
  mechanical selection rule (fixed before any score) and the one
  confirmatory read are the only protections. The Decision reports the
  exploration-to-confirmation shrinkage of the winners as the measure.
- **The discard target may again lose at the bottom.** `D0`'s pattern (AUC
  up, cuts down) may hold for every discard-target candidate; the clip and
  the smaller feature sets are the hypotheses this round tests. If so, the
  discard winner will be a pick-target candidate or a combination, which
  is a valid outcome.
- **Logistic `C` by inner CV on the discard target** has few labeled frames
  per inner fold (about 400); if the choice is unstable across folds, the
  pre-registration's fallback (`C = 1.0` as `L0`) applies and is recorded.
- **`HGB-mono` with NaN** may be unsupported on sklearn 1.9.1 for
  `monotonic_cst`; the fallback (mean-filled technical columns for that
  form) is pre-registered in Step 1 if the documentation says so.
- **Fail path is a valid outcome.** The artifacts are the held-out scores
  of every candidate on every frame, the two tasks' tables, the dimension /
  form / target readings and a confirmation that says how much the search
  overfit; the Decision says whether the features, the forms or the labels
  were the limit.

## Decision (neither winner passes; discard: more labels, after improving on `eye_focus`; pick: nothing, `W0` stays)

Written 2026-10-23 at Step 6, against the bars and the follow-up rule
pre-registered in Step 4 ([modelsearch.md](modelsearch.md), "Pre-registration
of the confirmation"). The confirmation numbers come from modelsearch.md
"Results of the confirmation", the exploration numbers from "Results of the
exploration"; discard values are weighted by the fresh `1/π` with the
stratified bootstrap 95% interval, pick values are means over the 7 lockbox
folders with the folder-bootstrap 95% interval. No bar or rule was changed
after the fresh labels or the lockbox picks were read.

**1. Discard: `G0 ⊕ ef` does not pass.** Conditions 1–4 hold at q5 and q10,
condition 6 (beats `eye_focus`-alone) fails at every cut, and condition 5
(beats `G0`) fails at q5 as well:

| Cut | Hides (fresh population) | `DP` % | `RR` % | `PL` % | `DP` − `G0`, points | AUC − `eye_focus` | Usable |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| q5 | 4.52% | 76.7 [69.2, 84.2] | 23.3 [15.8, 30.8] | 0.77 | +0.1 [-7.0, +6.9] | -0.101 [-0.159, -0.044] | no (5, 6) |
| q10 | 9.34% | 76.2 [70.8, 81.7] | 23.8 [18.3, 29.2] | 2.10 | +6.3 [+0.9, +11.4] | (same) | no (6) |

What it would have suggested: at q10, sorted last, 9.34% of a folder's
frames, about one in four of them (`RR` 23.8%) a frame the user would rescue,
with 2.10% of the picks among them. The condition 6 failure is not marginal:
the whole paired interval is below zero (cluster [-0.201, -0.016]). The
winner's AUC, 0.684 [0.610, 0.756], sits between `G0`'s 0.630 and
`eye_focus`'s 0.785 [0.723, 0.844]; averaging `G0`'s rank into `eye_focus`'s
lowered the threshold-free order below `eye_focus`'s while keeping `PL` low
(`eye_focus` alone: `PL` 4.44% at q5, 9.14% at q10, so q10 fails condition 2
for it).

**2. Pick: `All@32 / HGB-mono / pick` does not pass.** Every bar fails: mean
`P@10` 28.0% against `W0`'s 33.2%, −5.2 [-10.8, -0.7] points, 3 of 7 folders
at or above `W0`. What it would have suggested: a folder's top 10% with 28.0%
of them picks (`W0` 33.2%) and 20.2% of the picks found (`R@10`; `W0` 24.0%);
frame AUC −0.001 [-0.039, +0.036]. The 7-folder lockbox makes a fail weakly
informative in general (Trade-offs), but here the interval lies below zero,
so the read is not merely underpowered; `W0` was trained with the other
lockbox folders and the winner without them, which favors `W0` and is part
of the gap. The winner's `P@5` (+5.5 [-3.7, +16.4]) and its lead on the three
largest folders are single readings without a bar.

**3. Shrinkage from the exploration to the confirmation** (modelsearch.md,
"Exploration beside confirmation"):

- Discard winner: `DP@5` 88.4 → 76.7 (−11.7 points), `DP@10` 90.0 → 76.2
  (−13.8), `RR@5` / `RR@10` 11.6 → 23.3 / 10.0 → 23.8 (about doubled), AUC
  0.763 → 0.684 (−0.079); its lead over `G0` at q10 +13.7 → +6.3 and at q5
  +5.7 → +0.1; its AUC lead over `G0` held (+0.053 → +0.055); its AUC gap to
  `eye_focus` widened from −0.015 to −0.101.
- Part of that fall is the population, not the selection: the discardable
  base rate fell from 48.45% to 42.31%, and the unselected references fell
  too (`G0` `DP@10` 76.3 → 69.9, AUC 0.710 → 0.630; random `DP@10` 61.5 →
  46.1), while `eye_focus` held (AUC 0.777 → 0.785). The part attributable
  to the search is the lost lead over `G0` (halved at q10, gone at q5) and
  over `eye_focus`, the comparisons the selection had favored.
- Pick winner: `P@10` − `W0` −0.9 [-2.7, +0.5] → −5.2 [-10.8, -0.7]; its
  `P@10` 30.4 → 28.0 while `W0`'s rose 31.2 → 33.2 on the lockbox folders.
- Reading: the leads over `G0` at the cuts shrank by half (q10, +13.7 → +6.3
  [+0.9, +11.4], still above zero) or vanished (q5); the near-tie with
  `eye_focus` became a clear deficit; the pick gap to `W0` widened. What held
  was the leads over random (`DP@10` +28.5 → +30.2, q5 +36.4 → +32.5) and the
  AUC lead over `G0` (+0.053 → +0.055), whose interval now spans zero.

**4. What the search taught about the space**, as readings of the
exploration (one selection over 69 candidates on reused labels, a few of
several hundred intervals read without correction; none of these was
confirmed):

- **The dimension curve.** On the pick side the HGB pick-target forms do best
  at `All@32` (`P@10` − `G0` −1.6 / −1.9 / −0.2) and every form loses at
  `T+C` (−5.8 to −7.2), so the embedding dims carry pick information that the
  technical and context columns do not. On the discard side the logistic
  discard-target AUCs are flat from `All@4` to `All@16` (0.796–0.809; the
  `All@32 / LR-L1` form drops to 0.787), so the discard order needs few
  embedding dims; `Emb@8` is the lowest or near it in every row, while
  `Emb@32` beats `T+C` on the pick side (`P@10` − `G0` −2.9 / −6.0 / −2.9 /
  −3.2 against −5.8 / −6.2 / −6.0 / −6.3) and `Emb@32 / LR-L2 / discard`
  reads AUC − `G0` +0.046.
- **The forms.** No form beat `W0` on the pick task. At `All@32`, `HGB-mono`
  was the only pick-target form whose `DP@10` − `G0` interval was above zero
  (+5.3 [+1.3, +9.1]) and the pick winner, and it lost on the lockbox. On the
  discard target at `All@32` the logistic forms raised the AUC most
  (+0.097 for `LR-L2`, +0.077 for `LR-L1` over `G0`) and were the only
  discard-target forms with a positive `DP@5` − `G0`, while the HGB forms on
  that target lowered `DP@5` (−12.8 to −19.0); across the full `DP@10` grid
  some HGB discard-target cells are positive (`T+C / HGB / discard` +14.8).
  On about 600 weighted labels the simpler form may generalize better; this
  rests on the `All@32` column alone.
- **The targets.** The discard target raised the AUC over the pick target in
  31 of 33 designs and lowered `P@10` in all 33 (−1.8 to −8.5 points), and
  its `DP@10` effect ranged from −29.7 to +26.0: it reorders the whole folder
  better but its bottom cut is unstable, `D0`'s pattern again. The weight
  clip, one of the two hypotheses for that pattern, moved nothing (`All@32 /
  HGB / discard` against `D0`: `DP@10` +3.3 [-14.1, +20.8], AUC −0.005).
- **The combinations.** `G0 ⊕ ef` moved `DP@10` most (+13.7) and won the
  discard task; `stack` (+6.9 [-4.2, +16.4]) and `G0 | ef-rule` (+3.4 [-3.6,
  +10.7]) less; none moved `P@10`. Confirmed: mixing `G0` into `eye_focus`
  costs order (AUC −0.101 against `eye_focus` alone) to buy a low `PL`, and
  on fresh labels the trade no longer beats `G0` at q5.
- **Where the limit is** (judgment). Not the forms: five forms over the same
  columns span a narrow pick range below `W0`, and on the discard task a
  single cue, `eye_focus`, ordered the fresh labels better than any
  combination containing it. The features are the limit on both tasks: the
  discard signal lives in `eye_focus`, which the models see as one column
  and do not exceed, and the pick signal of these columns tops out near
  `W0`. The labels limit what can be trained on the discard target (about
  600 weighted frames, Kish ESS a third of that unclipped) and are the input
  the follow-up adds.

**5. Recommendation by the pre-registered follow-up rule.**

- **Discard: branch (2), more labels or more folders.** The `RR` point meets
  the 25% bar at q5 and q10 with `PL` and the removed share holding; the
  only failing conditions are "beats `G0`" (q5) and "beats `eye_focus`"
  (both cuts). Facts the recommendation rests on: on the fresh labels the
  plain `eye_focus` cue was the strongest discard signal (AUC 0.785 against
  the winner's 0.684, paired −0.101 [−0.159, −0.044]); in the exploration
  three `LR-L1` discard-target models read AUC − `eye_focus` +0.031 with
  stratified 2.5% quantiles of +0.004 to +0.006, two of them failing the q10
  `PL` condition and none read on the fresh labels. Judgment: since the
  winner's interval against `eye_focus` lies wholly below zero, more labels
  alone are unlikely to flip the "beats `eye_focus`" bar for this winner. The
  useful follow-up is to **improve on `eye_focus` itself and use the labels
  for training** (the two blind rounds, 1,200 frames), then **confirm once
  on a fresh blind round drawn from folders not yet sampled** (both rounds
  so far drew from the same 27 folders), sized at about 3 s per frame as the
  rule says (this round's labeling took about 1.5 s per frame, so 600 frames
  about 15 minutes).
  - **Next step**: the user asked whether `eye_focus`'s face choice (the face
    nearest the AF point) differs from the face a human looks at. A
    separate plan, `docs/plans/20261021-face-choice` (not yet written to the
    repository), will evaluate face-choice rules (the largest face, the
    sharpest, the highest detector score, the mean over faces) on the two
    blind rounds. Reading the rules on both rounds makes them exploration
    labels, so a winning rule still needs the fresh-folder confirmation
    above before any app work.
  - The surface an in-app plan would need if a later round passes:
    discard candidates **sorted last** (not hidden) at the best cut's depth
    with a rescue loop, as the follow-up rule's branch (1) names. Nothing is
    built now.
- **Pick: branch (3), nothing.** The `P@10` gain point is negative (−5.2),
  so the "close" branch does not apply. **`W0` stays** as the shipped
  in-burst ranking; no strip mark or sort for a folder's top 10% follows.

The recommendation would change:

- **discard → an in-app suggestion plan** only if a fresh blind round, from
  folders not yet sampled, puts a cut of a new score over all six
  conditions, "beats `eye_focus`-alone" included;
- **discard → nothing** if the face-choice plan and a model trained on the
  1,200 labels both fail to put an AUC − `eye_focus` point above zero on the
  labels they explore on; a fresh round is then not worth its labeling time;
- **pick → a new round** only with information this search did not have
  (new features, or more folders of picks so that a lockbox is not 7
  folders); this round gives no evidence that another form or feature set
  over the same columns would beat `W0`.

**Selection bias weighed.** The discard winner is a rank mix of `G0` and
`eye_focus`, the two scores this plan's design was chosen after, it was
chosen as the best of 69 on reused labels, and the fresh strata were cut by
the winner and `G0` themselves (the `1/π` weights keep the estimates
unbiased for the population; they do not make it a new population). Every
one of these biases ran in the winner's favor, and the winner still failed,
so the negative verdict is robust to them; what they do weaken is the
exploration's positive readings (item 4) and the three `LR-L1` AUCs above
`eye_focus`, which are leads to test, not findings. `eye_focus` was a
reference, not selected, and its AUC held across both rounds (0.777,
0.785). Both rounds come from one photographer's 27 folders, mostly
basketball.

**Facts the user may weigh, not a recommendation.** The open todo item
"Core: decide whether `G0` as-is at per-folder q5-q10, sorted last, is worth
shipping" stays open for the user. On the fresh labels `G0` at q5 again meets
conditions 1–4: `RR` 23.5 [15.7, 30.9] (cluster 97.5% quantile 34.9, just
under 35), `PL` 0.68%, removed 4.53%, `DP` 2.5% quantile 69.1 above the
42.31% base rate; at q10 its `RR` is 30.1 [23.7, 35.8] and fails. This round's
strata were cut for the winner, not drawn to test `G0`, so it is a reading,
not the pre-registered round that item asks for.

## Progress

- (2026-10-20) Plan written; the user's choices decided (see Purpose,
  "Choices")
- (2026-10-20) Step 1 complete
- (2026-10-21) Step 2 complete
- (2026-10-21) Step 3 complete
- (2026-10-21) Step 4 complete
- (2026-10-21) Step 5 complete
- (2026-10-23) Step 6 complete
