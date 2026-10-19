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

# Discard retarget: a 2x2 offline evaluation of the discard target and burst-relative features against the blind discard labels

## Purpose

The archived [discard-labels](../_archived/20261018-discard-labels/plan.md)
plan measured `G0` (gradient boosting on the `F0+ctx` columns, trained on
**pick vs non-pick**, held out by folder) against 600 blind "OK to discard?"
labels and found no usable "hide / sort last" cut: q5 rescue rate `RR`
17.3% [11.3, 24.0] against the 15% bar, q10 23.7%, q20 27.9%. Its Decision
also found that `G0`'s target is the wrong one: 48.45% of the population's
frames are OK to discard but only 51.7% of the non-picks are, and 24.4% of
the sampled picks are too; threshold-free, `eye_focus` alone (AUC 0.777)
ordered the labels better than `G0` (0.710). By the pre-registered follow-up
rule the Decision recommended a later plan that **trains on the rescue labels
with the controls as negatives**.

This plan is that evaluation, as one integrated **2x2** that separates two
effects on the same labeled frames with paired intervals:

- **(A) retarget**: train on the 600 blind discard labels (211 keep / 389
  discard, `1/π` weights) instead of on the dump's picks, with `G0`'s
  features;
- **(B) burst-relative features**: a small fixed set of columns that compare
  a frame with the other frames of its burst (embedding similarity to its
  nearest burst mate, its sharpness / eye-focus / eye-openness relative to
  the burst), added to `G0`'s features.

Cells: `G0` (pick target, base features; the known reference), `G0+B` (pick
target, base + B), `D0` (discard target, base features; effect A) and `D0+B`
(discard target, base + B; A+B), read beside `eye_focus` alone and random.
Everything is read on the same 600 labeled frames with the same
Horvitz–Thompson weighting, stratified bootstrap and per-folder cut grid as
the archived `discardfilter.py`, against bars pre-registered **before any
score of a new model is read**. It ends at a Decision naming the follow-up:
an in-app "discard candidates sorted last" view with a rescue loop, more
labels, or nothing. **No app code changes in this plan.**

### Choices (decided by the user on 2026-10-19, before any new score exists)

1. **The bar: looser**, `RR` <= **25%** with its stratified 97.5% quantile
   <= **35%** (rejected: keep `RR` <= 15% with the quantile <= 25%). Both
   options keep `PL` <= 5% (full dump), removed share >= 4.5%, beat random
   and beat `G0` paired.
2. **A fresh blind evaluation sample: none now.** Leave-one-folder-out on the
   600 labels decides; a pass triggers a confirmatory blind round as the first
   step of the in-app plan (rejected: draw one in this plan).
3. **The 1,200 strict labels in training: out.** They stay the secondary
   coherence check (rejected: in, as positives only).
4. **The "beat random" form: the discardable base rate.** A cut beats random
   when its `DP`'s stratified 2.5% quantile is above the base rate's point
   estimate (48.45%) (rejected: the paired difference against the seed-0
   random cut at the same `q`, which removes only 27 sampled frames at q5).
   The seed-0 random rows are printed either way.

### What is known (investigated 2026-10-19)

- **Data.** The dump of 27 folders and 32,459 frames
  (`D:\Photos\tests\2026-10-12-rank-app\`: `features/<folder>.tsv`,
  `embed.tsv`, `genre_text.tsv`, `prior_scores.tsv`), read by the archived
  `gate.load` / `gate.build` / `folder_gate.extend` into `d` with `emb`
  (L2-normalized 768-d SigLIP vectors), `bursts` (`(folder, start, end)`
  slices, gap 1,000 ms), `tech` (11 columns), `context` (3), `pick`,
  `reject`, `size`, `single`; the raw EAR via `rf.load_ear`. Stored
  held-out scores: `W0` etc. in
  `D:\Photos\tests\2026-10-15-folder-wide-eval\scores.tsv`, `R0` / `G0` in
  `D:\Photos\tests\2026-10-16-reject-filter-eval\scores_reject.tsv`.
- **The labels.** `D:\Photos\tests\2026-10-18-discard-labels\labels.tsv`
  (600 rows: `name`, `label` = `keep` / `discard`, `folder`, `file`, `band`,
  `inclusion_prob`, `pick`, `reject`, `single`, `burst_size`, ...),
  `manifest.tsv` (plus the cue values and stored scores), `strata.tsv`
  (bands `b05` / `b10` / `b20` as `G0`'s per-folder bottom 5 / 10 / 20% on
  the full dump; `control` per folder, `π` 0.0042–0.43, 150 frames for
  25,071). Population: the dump minus the 1,200 strict-sample frames,
  31,259 frames. The strict labels:
  `D:\Photos\tests\2026-10-17-bad-labels\labels.tsv` (`label` = `bad` /
  `not_bad`, 80 bad, its own `stratum` / `π`).
- **`G0`'s features and fit** (`rejectfilter.g0_fold`): the 46 columns of
  `folder_gate.design(d, train_mask, "full")`: `tech` (log of the frame's
  sharpness over the **burst's top sharpness**, logit `eye_focus`,
  `eyes_open`, |yaw|, |pitch|, |roll|, `tier`, three missing indicators,
  and absolute log sharpness), `context` (log **burst size**, **position in
  the burst**, first-of-burst), and the 32 whitened PCA dimensions of the
  embedding fit on the training folders; the NaN of the technical columns
  restored after the mean fill; `HistGradientBoostingClassifier(**rf.HGB)`
  (200 iterations, learning rate 0.05, 15 leaves, l2 1.0, seed 0) on
  `pick`, per-folder class weights rescaled to mean 1 per fold; one fit per
  held-out folder. **So three burst-relative columns are already in the
  base features**; effect B is the increment over them.
- **`badmodel.py`'s label-trained scheme** (`fold`): the same design, the
  model fit on the other folders' **sampled** frames only with `1/π`
  weights rescaled to mean 1, scored on the held-out folder. It read its
  models on a coarse within-sample cut because it scored only the sampled
  frames; a held-out model can score **every dump frame of its held-out
  folder**, so this plan uses the full-dump per-folder cuts of
  `discardfilter.py` (`bf.graded_cut`, `floor(q * n / 100)`, ties by the
  seed-0 draw) for every cell, and `PL` / removed share stay exact and
  label-free.
- **`G0`'s numbers at these cuts are known** (discard.md "Results"): q5 `DP`
  82.7 / `RR` 17.3 [11.3, 24.0] / `PL` 0.68 / removed 4.92; q10 76.3 /
  23.7 [18.7, 28.7] / 2.10 / 9.87; q20 72.1 / 27.9 [23.3, 32.5] / 5.18 /
  19.80; AUC 0.710; `DP` minus random q5 +30.7 [-3.8, +63.9] (random q5
  removes only 27 sampled frames); `eye_focus` AUC 0.777, q10 `RR` 14.3%
  at `PL` 9.14%; random AUC 0.528; base rate 48.45% [42.03, 55.11]. `G0` is
  a **known reference, not a blind test**; the design of this plan (the
  retarget, the cue-like B features) was chosen after reading those
  numbers, which the pre-registration states as a selection bias on the
  same 600 frames.
- **Harnesses, imported by path, none modified**: `badfilter` (brings
  `rejectfilter`, `gate`, `folder_gate`, `folderwide` and the archived
  bad-photo `sample`): `graded_cut`, `g0_q95_cut`, `strat_counts`,
  `cluster_counts`, `ratio`, `quant`, `aucs`, `finite_by_folder`, `Tee`,
  `read_tsv`; `discardfilter`: `metrics`, `unweighted`, `counts`, `share`.
  **Import `badfilter` first**, then put `_archived/20261018-discard-labels`
  on `sys.path` and import `discardfilter`: that folder's own `sample.py`
  would otherwise shadow the archived bad-photo `sample` that `badfilter`
  imports (the discard-labels learnings, Step 1). This plan names none of
  its scripts `sample.py`.
- **Python**: the venv `%TEMP%\embedvenv\Scripts\python.exe -I -B` on script
  files only, run from the scratchpad; never `python -` heredocs (the Git
  Bash stub hangs). Check `git status` for stray `__pycache__/`.
- **Where the data goes**: `D:\Photos\tests\2026-10-19-discard-retarget\`
  (features, scores, outputs). The library `D:\Photos\2026` is read only;
  nothing is copied.
- **Out of scope, owned elsewhere (do not touch)**: `rank.rs`'s `pairs`,
  `Labeled`, `fit`; the `views` table and its data; the dlg-genre-weights,
  dlg-folder-best and dlg-likely-pick-icon sessions. No app code changes.

## Steps

- [ ] Step 1: `burstfeat.py`, the burst-relative feature set `B` on every dump frame, label-free
  - Done when:
    - `burstfeat.py` in this plan folder (`python -I -B burstfeat.py
      <app-dump-dir> <out-dir>`) loads the dump through the archived
      harnesses and writes `burst_features.tsv` (`folder`, `file`, one
      column per feature) into `<out-dir>`, with exactly these **7
      columns**, each NaN where undefined (a single, or a cue missing on
      the frame or on every other frame of the burst):
      1. `nn_cos`: the highest cosine between the frame's embedding and any
         other frame of its burst (duplicates);
      2. `sharp_z`: the z-score of the frame's log sharpness among the
         burst's frames with a sharpness (0 when their SD is 0);
      3. `sharp_rank`: the share of the burst's other frames with a sharpness
         that are sharper (0 = the sharpest; "the burst holds a sharper
         frame" is `sharp_rank` > 0);
      4. `ef_gap`: the frame's logit `eye_focus` minus the highest logit
         `eye_focus` among the burst's other frames (`gate.logit`, the
         same clip as `tech_row`);
      5. `ef_rank`: the share of the burst's other frames with an
         `eye_focus` whose `eye_focus` is higher;
      6. `ear_gap`: the frame's raw EAR minus the highest EAR among the
         burst's other frames ("the burst holds a frame with the eyes more
         open");
      7. `ear_rank`: the share of the burst's other frames with an EAR whose
         EAR is higher.
    - It prints (Markdown) and `learnings.md` records, all label-free: the
      count of frames with a value per column (singles are 1,943; `ef` and
      EAR are missing on many frames), the per-column quantiles over the
      dump, the Spearman correlation of each B column with the three
      burst-relative base columns already in `G0` (`tech[:, 0]`,
      `context`), and the checks: every (`folder`, `file`) of the dump has
      a row; `sharp_rank` is 0 exactly where `tech[:, 0]` is 0 among frames
      with a sharpness; `nn_cos` is NaN exactly on the singles.
    - **No label and no model score is read in this step** (`labels.tsv`
      is not opened); the script reads only the dump.
    - `mise run lint` passes.
  - Implementation approach:
    - Import `badfilter as bf` by path as the archived `discardfilter.py`
      does; take `gate`, `folder_gate`, `rf` from it; EAR via
      `rf.load_ear(dump, d)`; `eye_focus` from `rows[i]["ef"]`; bursts from
      `d["bursts"]`; cosines from `d["emb"]` (already unit norm).
    - Ranks and gaps are over the **other** frames of the burst so a frame
      never compares with itself; ties count as not higher.
    - Script and docs only; no Rust, no app change.

- [ ] Step 2: Pre-register the cells, training, evaluation, effects and bars in `retarget.md`, before any new score exists
  - Done when:
    - `retarget.md` in this plan folder has a "Pre-registration" section
      merged **before `retargetfit.py` exists and before any score of
      `G0+B`, `D0` or `D0+B` has been computed or read**; the merge commit
      is the record. It states what is known (the `G0`, `eye_focus` and
      random numbers above, quoted from discard.md; the Step 1 label-free
      feature facts), that no new model score existed when it was written,
      that the plan's design was chosen after reading `G0`'s numbers on the
      same 600 labels (a selection bias the Decision must weigh), and the
      four user choices with the rejected options.
    - It fixes, in this order:
      - **Labels, population and weights**: the 600 `labels.tsv` rows
        (`discard` = positive), the population of 31,259 frames, `1/π` from
        `inclusion_prob`; every estimate Horvitz–Thompson weighted with the
        unweighted value beside it (as discard.md). The 1,200 strict labels
        do not enter training (choice 3); they stay the secondary coherence
        check.
      - **Cells and their training**:
        - `G0`: the stored `scores_reject.tsv` column, unchanged.
        - `G0+B`: `rejectfilter.g0_fold` with the 7 B columns appended
          raw (NaN kept) to the 46-column design; pick target, the dump's
          per-folder weights rescaled to mean 1, `rf.HGB`, one fit per
          held-out folder on **every dump frame** of the other 26 folders.
        - `D0`: `rf.HGB` on the 46 columns (NaN of the technical columns
          restored as `g0_fold` does), target `discard`, fit on the **other
          folders' labeled frames** with `1/π` weights rescaled to mean 1 per
          fold (`badmodel.fold`'s `B0` recipe), scored on **every dump
          frame** of the held-out folder; 27 folds; `-decision_function` so
          higher keeps, like every score here.
        - `D0+B`: `D0` with the 7 B columns appended.
        - **No hyperparameter or feature selection**: `rf.HGB` as is for
          all four cells (so effect A is the target alone and effect B the
          columns alone), the B set exactly Step 1's; hence no nested CV is
          needed and the leave-one-folder-out read is the estimate. The
          Kish effective sample size of each fold's training weights is
          reported (label-free; the `control` weights reach 238).
        - References: `eye_focus` (missing last) and random (seed-0 `tb`),
          as discard.md.
      - **Cuts**: per folder on the full dump, `floor(q * n / 100)` by the
        score, ties by the seed-0 draw, `q` = 5 / 10 / 20, for every cell
        and reference (`bf.graded_cut`); no `quantile@95` for the new cells
        (it is a pick-recall device). The label-free `PL` (full dump) and
        removed share (population) of each cell's cut are reported first
        and decide those conditions before any label is read.
      - **Primary metrics** per cell and cut, exactly discard.md's: `DP`,
        `RR`, `PL`, `DR`, `MR`, `DO`, removed share; the weighted AUC of
        `discard` vs `keep` per cell.
      - **Effects, paired on the same draws**, in points of `DP` and `DR`
        at each `q` and in AUC: A = `D0` − `G0` and `D0+B` − `G0+B`;
        B = `G0+B` − `G0` and `D0+B` − `D0`; the interaction
        (`D0+B` − `D0`) − (`G0+B` − `G0`); each new cell against `eye_focus`
        and random. The stratified bootstrap resamples the labeled frames
        with the models held fixed (training variance is not captured; the
        folder-cluster bootstrap is the sensitivity reading).
      - **Intervals**: the 1,000-draw stratified bootstrap within `b05`,
        `b10`, `b20` and each (`control`, folder) cell, seed 0, one count
        matrix reused across every quantity and cell; the folder-cluster
        bootstrap beside it.
      - **Bars** (the chosen looser option written out with the rejected
        one): a cut of a **new cell** is a **usable discard cut** when its
        weighted `RR` <= 25% and its stratified 97.5% quantile <= 35%,
        `PL` <= 5% (full dump), it removes >= 4.5% of the population's
        frames, its `DP`'s stratified 2.5% quantile is above the
        discardable base rate's point estimate (48.45%, the "beat random"
        form of choice 4), **and** its `DP` beats `G0` at the same `q` with
        the paired 2.5% quantile above zero. Best cut: the deepest passing
        one of the cell with the largest `DP` gain over `G0` there. The
        pre-registration states which of these conditions `G0` itself
        already meets under the chosen bar (under the looser bar, `G0` q5
        meets the `RR` conditions), so the reader knows what a pass of a
        new cell adds.
      - **Follow-up rule**: (1) a usable cut exists → the Decision
        recommends a later plan for an in-app **discard candidates sorted
        last** view with a rescue loop at that depth, whose first step is a
        **fresh blind confirmatory round** (candidates and controls mixed
        and unmarked, as discard-labels Step 1), because the pass was read
        on the labels the model was selected and trained on and because of
        the automation-bias finding; (2) no usable cut but effect A or A+B
        positive at q5 (the paired `DP` 2.5% quantile above zero against
        `G0`) → **more labels**: a second blind round sized at about 3 s
        per frame (600 frames about 30 minutes, 1,000 about 50 minutes) to
        retrain and re-read, in a later plan; (3) otherwise → **nothing**.
      - **Multiplicity** (3 new cells, 3 cuts, 2 references, about 10
        paired effects) and the population caveat (one photographer, 27
        folders, mostly basketball; 600 labels).
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 1 merged (its label-free facts quoted). Model the text
      on discard.md's Pre-registration and keep its vocabulary. Doc-only
      PR; the four choices are recorded under "Choices" here and in
      `retarget.md`.

- [ ] Step 3: `retargetfit.py`, the held-out scores of `G0+B`, `D0` and `D0+B` on every dump frame, with the label-free readings only
  - Done when:
    - `retargetfit.py` in this plan folder (`python -I -B retargetfit.py
      <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir>
      <discard-labels-dir> <strict-labels-dir> <out-dir> [jobs]`) fits the
      three new cells exactly as pre-registered (27 folds each, joblib as
      `rejectfilter` / `badmodel`) and writes `scores_retarget.tsv`
      (`folder`, `file`, `G0+B`, `D0`, `D0+B`; higher keeps; a value on
      every dump frame, each from the model that did not see its folder)
      into `<out-dir>`, stdout saved as `retargetfit.out.md`.
    - It prints **label-free readings only**: per cell and `q`, the removed
      share (population and full dump) and `PL` (full dump and population),
      with the verdict of those two bars; the per-fold training counts
      (frames, positives) and Kish effective sample size; the wall time per
      cell; and the checks: `G0+B` with the B columns zeroed out
      reproduces the stored `G0` within float noise on a spot fold (the
      refit of `g0_fold` must match `scores_reject.tsv`'s `G0` to about
      1e-5 on that fold's frames, as the archived check did for `R0`);
      every score finite; the strict labels do not enter training.
    - **No new score is read against `labels.tsv`** in this step (the
      labels are read for training only; no `DP` / `RR` / AUC is computed);
      `learnings.md` says so and records the wall times.
    - `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 2 merged. Build the design per fold with
      `folder_gate.design(d, train_mask, "full")` plus the B columns joined
      from `burst_features.tsv` (recompute them with Step 1's function
      instead if that is simpler; either way the check that the file and
      the recomputation agree is printed). For `D0` / `D0+B` the training
      rows are the labeled frames of the other folders (`labels.tsv` joined
      by (`folder`, `file`)), the scored rows every frame of the held-out
      folder.
    - Measure the wall time; `G0+B` fits 27 HGB models on about 31k rows
      each, which `rejectfilter` ran with 8 jobs.

- [ ] Step 4: `retargetread.py`, every pre-registered metric, effect and bar against the labels, and the Results in `retarget.md`
  - Done when:
    - `retargetread.py` in this plan folder (`python -I -B retargetread.py
      <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir>
      <discard-labels-dir> <strict-labels-dir> <out-dir>`) loads the dump,
      the stored scores, `scores_retarget.tsv`, `labels.tsv`, `strata.tsv`
      and the strict labels, computes everything in the Pre-registration,
      prints Markdown tables and saves `retargetread.out.md` in
      `<out-dir>`.
    - **Reference checks** recorded in `learnings.md`: the bands and `π`
      recomputed from the dump match `strata.tsv`; the sum of `1/π` is
      31,259; `G0`'s rows reproduce discard.md's main table (q5 `RR` 17.3
      [11.3, 24.0], AUC 0.710); `eye_focus` and random reproduce theirs;
      the base rate reads 48.45%.
    - `retarget.md` gains a "Results" section: the main table (the four
      cells, `eye_focus`, random at q5 / q10 / q20 with `DP`, `RR`, `PL`,
      `DR`, `MR`, `DO`, removed share, intervals); the AUC table; the
      **effects table** (A, B, interaction, each new cell vs `eye_focus`
      and random, in `DP` / `DR` points per `q` and in AUC, stratified and
      folder-cluster intervals); the per-band `discard` rate of each new
      cell's own bottom bands beside the strict-bad rate from the strict
      sample (the coherence check); the bars applied per new cell and cut
      under the chosen bar, and, in a second table, under the rejected bar
      option, marked as a reading only; the follow-up rule's inputs; wall
      time; a "Reading" paragraph stating the facts without deciding.
    - `learnings.md` records deviations and the choices the text left open.
  - Implementation approach:
    - Assumes Step 3 merged. Import `badfilter` first, then
      `discardfilter` (see "What is known"); reuse `discardfilter.metrics`,
      `counts`, `unweighted`, `bf.graded_cut`, `bf.aucs`, the count
      matrices. Metric-only work, seconds to run; no refit.

- [ ] Step 5: The Decision
  - Done when:
    - A "Decision" section in this plan, written against the Step 2 bars
      with the numbers that decided it: (1) effect A (does the discard
      target beat the pick target on the same features, at each cut and
      threshold-free); (2) effect B and the interaction; (3) whether a
      usable discard cut exists under the chosen bar, the best cut, what it
      rescues, loses and misses, and how it reads under the other bar
      option; (4) each new cell against `eye_focus` alone and random; (5)
      the follow-up by the pre-registered rule: a later plan for an in-app
      discard candidates sorted-last view with a rescue loop (with its
      depth, the confirmatory blind round it starts with and the app
      surface it would need), a later plan for more labels (with the size
      at about 3 s per frame), or nothing; stated as a recommendation, with
      the conditions under which it would change and the selection-bias
      caveat weighed.
    - `learnings.md` records the compute spent and what the next round
      would need; `mise run lint` passes.
  - Implementation approach:
    - Assumes Step 4 merged. Doc-only PR; write it only after the Results
      exist; do not move a bar.

## Trade-offs and risks

- **The four choices** are recorded under Purpose, "Choices", with the
  rejected options. Under the chosen looser bar `G0` q5 (17.3 [11.3, 24.0])
  and `G0` q10 (23.7 [18.7, 28.7]) already meet the `RR` conditions, so a
  pass of a new cell rests on the "beat random" (base rate) and "beat `G0`"
  conditions; the Decision must say that `G0` itself would have met the
  rescue conditions under this bar.
- **Model selection on the same 600 labels can overfit**: the plan's design
  was chosen after reading `G0`'s numbers on them, and `D0` / `D0+B` are
  trained on them. Handling: no hyperparameter or feature selection at all
  (fixed `rf.HGB`, a fixed 7-column B set), so leave-one-folder-out is an
  honest estimate of the fixed recipe and nested CV is unnecessary; what
  remains is the design bias, stated. A pass triggers a confirmatory blind
  round before any app work.
- **The base-rate form of "beat random"** reads random's expected `DP` as
  the discardable base rate; it is the same quantity in expectation as the
  seed-0 random cut, far better powered, but a change from the archived
  bar's form.
- **Model form.** HGB only, so that A and B are pure effects of the target
  and the columns; the bad-photo round found a logistic model (`L0`) with
  the highest AUC, so a linear `D0` might read better, but adding it doubles
  the cells and the looks. Left out; the Decision may name it for a later
  round.
- **Heavy weights.** `1/π` of the controls ranges 2.3 to 238 per frame, so a
  few control frames of large folders carry much of the `discard` target's
  negatives; the Kish effective sample size is printed per fold so the
  Decision can say how many frames the fit effectively saw. A clipped-weight
  variant is not pre-registered (one more look); if the ESS is very low it is
  named as the next round's question.
- **Bootstrap scope.** The intervals resample the labeled frames with the
  models fixed, as before; the training variance of `D0` / `D0+B` (575–597
  training frames per fold) is not in them. The folder-cluster bootstrap
  partially covers it and is reported beside.
- **B already partly in the base features.** `log(sharp / burst top)`, log
  burst size and the position are `G0` columns, so a small B effect may mean
  "already captured", not "burst context is useless"; the Step 1 correlation
  table is there to read that apart.
- **Fail path is a valid outcome.** If no cell passes, the artifacts are the
  feature file, the held-out scores of three new cells on every frame, the
  paired A / B effects with intervals, and a Decision that says whether the
  target or the features were the problem.

## Progress

- (2026-10-19) Plan written; the four choices decided by the user (see
  Purpose, "Choices")
