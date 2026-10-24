# Learnings: discard model

## Step 1 (pre-registration of the exploration)

- Harness names checked against the archived scripts before quoting them.
  Everything the plan names exists, with these differences in how it is
  reached, which `discardmodel.md` states the accurate way:
  - `badfilter` imports the bad-photo `sample` as `sample` (so
    `bf.sample`), not as `bs`; `bs` is the alias the later samplers use
    (`rf, bs = bf.rf, bf.sample` in discard-labels `sample.py`,
    `ds.bs` in `confirmsample.py`). `folderwide` is `bf.fw`.
  - `nearest_face` is not a Python name: it is the Rust
    `candidate::nearest_face`, whose choice the per-face dump records in
    its `nearest` column, which `facechoiceread.choose` reads for `af` /
    `af-whole`.
  - `facechoiceread.load_faces` keeps per face only the box, area, score,
    the 0.8-floor flag, `nearest` and `eye_focus`; the dump's `ear` /
    pose / `state` columns are not loaded by it. The `T` columns' EAR and
    pose come from the frame-level dump (`rf.load_ear`, `gate.tech_row`),
    so this does not matter for Step 2, but a per-face pose would need its
    own read.
  - `searchfit`'s clip is `np.minimum(w, CAP_FACTOR * median(w))` over one
    round, then `mean1` on each fit's training rows. The per-round cap and
    per-round rescale to mean 1 (over each fold's training rows of that round, no further rescale) are this plan's addition (stated so in the
    pre-registration).
- The face-choice `af-whole` read is +0.014 on round 2 but −0.028
  [−0.065, +0.009] on round 1; the plan quoted round 2 only. Recorded in the
  "What was known" table since it bears on the stop rule (positive on both
  rounds). `ef-fill` is not the `af-whole` rule: it only fills the frames
  today leaves missing.
- `monotonic_cst` with NaN: taken from modelsearch.md's documentation
  section (no new fetch, no fit), plus its learnings' record that the fits
  raised nothing and passed the spot check.
- The ARW counts of the six fresh folders were relisted (file names only,
  read-only): 3,190 / 3,185 / 1,615 / 1,675 / 3,045 / 1,815 = 14,525, as
  the plan says. `2026-09-13-a`'s reject count was not in the plan's
  inventory and is marked "not counted".

## Step 2 (`discardfit.py`)

- Run once into `D:\Photos\tests\2026-10-23-discard-model\` on 12 jobs:
  `scores_model.tsv` (32,459 rows, `ef-fill` written `nan` where it stays
  missing, the learned columns finite everywhere) and `discardfit.out.md`.
  Wall time 13 s in all: load 5 s, the `HGB-mono/T` spot check 1 s, `LR/S`
  3 s (both runs, `C` fallback), `LR/T` under 1 s, `HGB-mono/T` 2 s,
  `HGB/T` 2 s, the cuts under 1 s. Every check passed, including the
  reproduction of `ef`'s `PL` (4.44 / 9.14 / 15.59%) and removed shares
  (round 2 4.80 / 9.70 / 19.64%, round 1 4.83 / 9.78 / 19.87%).
- Implementation choices within the pre-registration:
  - The per-round rescale to mean 1 is applied at every fit, including the
    inner `GroupKFold` fits of the `C` choice (over the inner training rows
    of each round); `searchfit.choose_c` itself rescales the pooled inner
    rows, so it is repeated here with that one change rather than called.
    The inner validation AUC uses the fold's weights (per-round mean 1).
  - The Kish ESS "without the clip" is the same recipe with the raw `1/π`
    (rescaled to mean 1 within each round, pooled).
  - The rounds' `1/π` come from `manifest.tsv`; `labels.tsv` rows are
    checked against it (0 rows differ). `strata.tsv` is on the allow-list
    but not opened (not needed for the weights); nothing is opened under
    the reject-filter-eval folder either (no `G0` reading in this step).
  - The fill touched 3,020 frames, none of which had a frame-level EAR or
    pose, so the `T` EAR / pose columns are exactly the dump's.
  - `HGB-mono/T` with NaN kept: the spot fold (`2026-09-27-a`) fit and broke
    no constraint over the non-missing grid (largest step 0), so no
    mean-filled fallback.
  - `LR/S` took the `C = 1.0` fallback (11 / 8 / 8 folds at 0.1 / 1 / 10);
    `LR/T` chose 0.1 in 15 of 27 folds, no fallback.
- Label-free outcome worth noting for Step 3: **no candidate passes the
  q10 label-free conditions** (`PL` at q10 6.00–6.99% for all five, above
  the 5% bar; removed 9.96% passes). Under the pre-registered selection
  rule that means no winner and the stop rule ends the plan at Step 3
  (Steps 4–6 skipped), which Step 3 has to state from its own read.

## Amendment of the selection rule (2026-10-24, after Step 2)

- The user amended the selection rule after seeing only Step 2's label-free
  readings (no candidate score read against `labels.tsv`): a candidate is
  eligible when its q5 **or** q10 cut passes `PL` <= 5% and removed >=
  4.5% on the full dump (was q10 only), matching the confirmation bars'
  "q5 or q10". Prompted by every candidate's q10 `PL` being 6.00–6.99%
  (`ef`'s own 9.14%) while all pass at q5 (`PL` 2.47–3.91%). Rejected:
  keeping q10 only, which would have ended the plan with no winner. At q5
  both references pass too (`ef` `PL` 4.44%, random 4.71%). Only the
  removed-share condition is fixed at q5 (4.96% for every score,
  `floor(q * n / 100)` fixes it); `PL` <= 5% can still fail at q5 for a
  score that cuts picks faster than random. On Step 2's readings every
  candidate passes it, so in this run the filter excludes nobody and the
  selection reduces to the highest round-2 AUC. (Corrected in Step 3, after
  Step 2's review: this note and the amendment first said the q5 condition
  "cannot fail".)
- Recorded as a dated "Amendment (2026-10-24)" subsection under the
  pre-registration's "Selection rule and stop rule", leaving the merged
  text as it was; it runs against the pre-registration's own "nothing
  changes after a candidate's score exists", which the Decision should
  weigh. `plan.md`'s Selection rule bullet now says q5 or q10. `discardfit.py`
  was re-run (label-free only) so its prose and eligibility table state the
  amended rule (q5 passes, q10 passes, eligible); `scores_model.tsv` came out
  byte-identical (md5 494ca4b9...), only `discardfit.out.md` changed. Step 3's `discardread.py` must apply the amended
  rule.

## Step 3 (`discardread.py`)

- Run once into `D:\Photos\tests\2026-10-23-discard-model\`
  (`discardread.out.md`), 91 s in all: load 6 s, cuts, checks, metrics and
  bootstraps 86 s (seven scores × three frame sets × two rounds of
  stratified and cluster AUC draws). All 29 label-free checks passed before
  any `labels.tsv` was opened and all 17 label checks after.
- **Result**: every candidate eligible (all pass at q5, none at q10);
  winner `LR/T` (round 2 AUC 0.812, AUC − `ef` +0.027 [−0.020, +0.078];
  round 1 0.795, +0.018 [−0.018, +0.053]); the stop rule sends it to the
  fresh round, so Steps 4–6 go ahead (not skipped).
- **Dry run first**: a copy of the script in the scratchpad (its `HERE`
  pointed at this plan folder, the label-check gate off) ran on synthetic
  random labels (copies of both rounds' `manifest.tsv` / `strata.tsv`,
  round 2's `scores_discard.tsv` and `scores_model.tsv`, `labels.tsv`
  written from the manifests with seed-7 labels). After it, two
  print-only fixes (the subset note printed under the complement too, and
  the base-rate table's subset names), and the dry run was repeated on the
  fixed copy; the refusal on an existing `discardread.out.md` was tested
  there too. Then the real read ran once.
- **Choices the text left open**:
  - The face-choice folder argument is used for two label-free checks:
    `ef-fill` in `scores_model.tsv` equals the fill recomputed from the
    per-face dump on every frame, and `af` reproduces the stored
    `eye_focus`. The audit hook gets a fifth root, `<out-dir>`, allowing
    `scores_model.tsv` only.
  - Step 2's `PL` and removed shares (q5 / q10 / q20, all seven scores) are
    checked as label-free reproduction rows; the eligibility uses the
    removed share of the full dump, as Step 2's verdicts did.
  - The missing-face subset is "no `eye_focus` today" (`ef` NaN, 4,022
    dump frames); its complement the rest. The subset tables use the
    full-dump cut, restricting only the labeled frames read (as
    face-choice's multi-face read). On the subset `ef` is constant within
    a folder (lowest finite value minus 1), so its subset AUC orders
    folders only; on the complement `ef-fill` equals `ef`, so its
    difference is 0 by construction. Where `ef`'s cut holds no sampled
    complement frame (round 1, q5) the `RR` cells print `nan`.
  - The random reference is the seed-0 draw `tb` itself (the cut's tie
    breaker), its AUC on the same draw.
  - "Ties by round 1's AUC" is an exact tie of the round-2 point; there was
    none.
- **Fixed from Step 2's review**: the amendment in `discardmodel.md` and its
  note above said the q5 label-free condition "cannot fail"; only the
  removed-share condition is fixed at q5, and `PL` <= 5% can still fail.
  Both sentences corrected, marked as corrected in Step 3.
- **Slip**: one `python -` heredoc was run by mistake (meant as a no-op);
  it hung on the Git Bash stub as the plan warns and was killed by its PID
  (and its parent shell's), not by image name. Another hung `python3 -`
  process (PID 71899, started 08:44, not from this step) was left alone.
