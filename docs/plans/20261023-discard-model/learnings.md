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

## Step 4 (`freshdump.sh`, the fresh dump)

- Run once in the background (2026-10-24 23:35 to about 23:58) with a
  frozen copy of the release `riffle-cli.exe` (built from `5b93ca61`, SHA-256
  `841560e4...`, copied to `D:\Photos\tests\2026-10-23-discard-model\`) on 12
  threads into `D:\Photos\tests\2026-10-23-discard-model\dump\`: every stage
  exited 0, nothing was resumed (no stage skipped). The waits were a bounded
  `timeout 580 tail -F wall.log | grep -m1` on the lines the script prints
  (`tail -f` on a log that did not exist yet exits at once, hence `-F`).
- `embed --out` overwrites, so each folder's vectors go to
  `dump\embed\<folder>.tsv` (resumable per folder) and `dump\embed.tsv` is
  rebuilt as their concatenation at the end of every run (14,525 lines);
  `dump\genre_text.tsv` is byte-identical to the rank-app dump's.
- **Library read-only**: the size / mtime listing of the six folders
  (`find -printf '%p\t%s\t%T@'`, recursive as `bs.listing`; 26,410 files) was
  taken before the run (`logs\listing.before.tsv`) and after it
  (`logs\listing.after.tsv`); identical (`cmp`), "library unchanged".
- `freshcheck.py` (this plan folder, label-free) printed the table below,
  saved as `dump\freshcheck.out.md`. File counts: the RAW count is the `.ARW`
  count of the before-listing; features rows, embed rows, face-dump files and
  `gate.load` frames all equal it. Errors from the stage logs; the `--check`
  mismatches from facechoice's own `check:` line.

| Folder | RAW | errors (features / embed / facechoice) | `--check` mismatches | trusted AF | picks (from `Output\` only) | pick % | rejects | missing `eye_focus` | after the `af-whole` fill | wall (features + embed + faces) |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|
| 2026-10-11 | 3,190 | 0 / 0 / 0 | 0 | 100% | 134 (0) | 4.20% | 23 | 10.31% | 1.76% | 61 + 75 + 154 s |
| 2026-10-12a | 3,185 | 0 / 0 / 0 | 0 | 100% | 304 (0) | 9.54% | 20 | 5.84% | 0.75% | 108 + 70 + 131 s |
| 2026-10-12b | 1,615 | 0 / 0 / 0 | 0 | 100% | 285 (0) | 17.65% | 22 | 6.44% | 0.68% | 34 + 36 + 93 s |
| 2026-10-17 | 1,675 | 0 / 0 / 0 | 0 | 100% | 205 (0) | 12.24% | 0 | 6.21% | 0.72% | 34 + 39 + 69 s |
| 2026-09-13-a | 3,045 | 0 / 0 / 0 | 0 | 100% | 620 (0) | 20.36% | 0 | 12.28% | 1.48% | 62 + 70 + 132 s |
| 2026-09-27-c | 1,815 | 0 / 0 / 0 | 0 | 100% | 247 (0) | 13.61% | 3 | 11.57% | 1.32% | 37 + 42 + 88 s |
| all | 14,525 | 0 / 0 / 0 | 0 | 100% | 1,795 (0) | 12.36% | 68 | 9.00% | 1.18% | 336 + 332 + 667 s = 1,335 s (22 min) |

- The picks and rejects match the plan's inventory (134 / 304 / 285 / 205 /
  620 / 247; rejects 23 / 20 / 22 / 0 / 3, and `2026-09-13-a`'s, "not
  counted" there, is 0). None comes from `Output\` only: every `Output\`
  name equals a dump file's stem, and `gate.load` counts `Output\` only for
  a `_<n>` virtual-copy name whose stem is not itself a file. So two
  exported frames of `2026-09-13-a` with an unflagged `.dop`
  (`_DSC9344`, `_DSC9346`) are not picks under `gate.load` (as in the 27
  dump folders, same rule); the plan's "exported in `Output\`" counts as pick
  only in that copy sense.
- The missing-`eye_focus` share is 9.00% on the fresh folders (12.4% on the
  dump) and 1.18% after the fill (4.5% for `af-whole` on the dump).
- **For Step 5**:
  - Today's `riffle-cli features` prints `eye_focus` (and `sharpness`, EAR)
    to 4 decimals, while the rank-app dump's `features\*.tsv` carry full
    precision; `|features eye_focus − af|` is at most 5.0e-05 on the 13,218
    frames with both. The per-face dump's `eye_focus` is full precision
    (`--check` compares it to `scan::extract_analysis` exactly, 0
    mismatches). Step 5 can take the fresh `ef` from either; the choice
    changes logits by at most about 5e-05 / (p(1 − p)) and may break ties.
  - `folder_gate.extend` opens `prior_scores.tsv` (the app's ranking prior),
    which the fresh dump does not have; `freshcheck.py` stops at
    `gate.build`. The `T` columns need only `gate.build`'s `tech`,
    `rf.load_ear` and the per-face dump, so Step 5 can skip `extend` (or
    write the frames' `ln sharpness` itself).
- `freshdump.sh` gets a shellcheck SC2012 info on the `ls Output` one-liner,
  kept as `dump.sh` has it (plan folders are not shellchecked by `mise run
  ci`). `git status` showed no `__pycache__/`.

## Step 5 (pre-registration of the confirmation, `freshsample.py`, the draw)

- The pre-registration was written into `discardmodel.md` before
  `freshsample.py` existed; the "As drawn" paragraph after the run.
- **The two open points of Step 4, resolved in the pre-registration**:
  - The fresh `ef` (the reference, the fill and `T`'s first column) is the
    per-face dump's full-precision `af` value, not the features TSV's
    4-decimal `eye_focus`. Checked: missing on the same frames, largest gap
    5.00e-05. Sharpness and EAR stay the features TSV's 4-decimal values.
  - `T` is built from `gate.build`'s `tech`, `rf.load_ear` and the per-face
    dump without `folder_gate.extend` (which needs `prior_scores.tsv` and
    only appends columns). Checked on the 27-folder dump: equal to Step 2's
    path (`extend`, stored `eye_focus`) on all 32,459 frames, and the spot
    fold `2026-09-27-a` (`C` 0.1) reproduces `scores_model.tsv`'s `LR/T`
    exactly (difference 0).
- **Refit choice the plan left open**: the `C = 1.0` fallback is a rule over
  the 27 LOFO fits, so for the single refit the inner `GroupKFold` choice (5
  folds over the 27 folders) is used as it comes; it chose 0.1, the LOFO
  majority. Stated in the pre-registration before the run.
- `freshsample.py` imports `discardfit.py` by path (`fold`, `choose_c`,
  `per_round_mean1`, `T_NAMES`) instead of repeating it, and reuses
  `confirmsample.py`'s draw and copy pattern. Its audit hook allows only
  `labels.tsv` / `manifest.tsv` under the two rounds' folders and records
  any sidecar opened under `sample\` (none).
- Runs from the scratchpad: `--no-copy` 9 s, then the copy in the background
  349 s (copy 230 s, SHA-256 of sources and copies 109 s; 26.1 GB, not the
  27 GB guessed). The copy run's `manifest.tsv`, `strata.tsv` and
  `scores_fresh.tsv` were byte-identical (md5) to the `--no-copy` run's, and
  its tables identical. The wait was a bounded `timeout 570 tail -F | grep
  -m1 "freshsample: "` on the stderr log, the line the script prints last.
- **Label-free outcome worth noting for Step 6/7**: on the fresh folders the
  winner's q10 `PL` is 5.91% (> 5%), so bar 2 already fails at q10; only q5
  (`PL` 2.56%, removed 4.98%) can pass. `ef`'s fresh `PL` is 4.07 / 7.30 /
  14.99%, `ef-fill`'s 2.84 / 5.96 / 13.09%.
- A first attempt to append the section with a Bash heredoc failed on the
  shell's quote parsing (nothing written); the Edit tool was used instead.
- No `__pycache__/` in `git status` (`-B` and `sys.dont_write_bytecode`).
- **Deviation: the labeling question was reworded (Round 1 review, item 1).**
  The earlier guides (`_archived/20261018-discard-labels/labeling.md`,
  `_archived/20261020-model-search/labeling.md`) asked whether it is fine if
  Riffle **hid** the frame from what you pick from, and whether you would
  mind if Riffle had **hidden** it. This round's `labeling.md` asks whether it
  is fine if Riffle suggested the frame for discard and **sorted it last**,
  and whether you would mind if Riffle had **sorted it last**, while still
  saying the question is "the same as in the earlier rounds". The new wording
  follows the plan's "sorted last, not hidden" Purpose, but sorted last is the
  milder outcome, so the fresh labels may call more frames OK to discard than
  round 1 and round 2 did. The fresh labels are taken under the "sorted last"
  wording, and the guide is not edited because the user may already be
  labeling. Step 6's Reading and the Step 7 Decision must weigh this when they
  compare the fresh `RR`, the base rate (bar 4) and the shrinkage with the
  two exploration rounds: a lower fresh `RR` or base rate may be partly the
  wording, not only the folders.
- **Deviation: the pre-registration is not merged before labeling may have
  started (Round 1 review, item 2).** At the time of the review the branch
  `discard-model-step-5` had no upstream and no PR. The pre-label record is
  the local commit `5e0e1209` (2026-10-25T00:48:36+09:00); the copies in
  `sample\` were checked as sidecar-free at draw time. If labeling began
  before the merge, the merge came after labeling started, and the section's
  "the merge commit is the record" is then read as this commit and its
  timestamp. The draw itself (strata, `π`, seed order, the refit recipe and
  its checks) is consistent with the pre-registration and is not redone.

## Step 6 (`freshlabels.py`, `freshread.py`, the confirmation read)

- **Labeling session**: the user labeled the 600 frames in **under one
  hour** (the guide guessed 15–30 minutes, the plan about 1.5 s per frame,
  15 minutes) and confirmed that **`aiEnabled` stayed off for the whole
  labeling**. Nothing else odd was reported.
- `freshlabels.py` is `confirmlabels.py` with the five strata (`b05`, `b10`,
  `b20`, `efonly`, `control`); run once from the scratchpad, its stdout saved
  as `freshlabels.out.md` in the data folder. 127 keep / 473 discard. The
  sidecar format was "both": 128 copies with a `.dop` (`<name>.dop`) and an
  `.xmp` (`<stem>.xmp`), 0 disagreements on rating 1, 472 with no sidecar.
  **One anomaly** (counted, not dropped): `0250_2026-10-11__DSC5031.ARW` has
  rating 0 in both sidecars, a mark taken back with `0` (allowed by the
  guide), so it is `discard` under the recipe (the label follows the rating-1
  mark alone). No Pick, Reject or color label anywhere. The sample copies
  and their sidecars were only read.
- `freshread.py` takes `<fresh-dump-dir> <out-dir>` only: the fresh `ef` and
  `ef-fill` come from Step 5's `scores_fresh.tsv` (checked against the
  features TSV's `eye_focus`, as Step 5 did), so the per-face dump is not
  reloaded. Its audit hook allows `scores_fresh.tsv`, `manifest.tsv`,
  `strata.tsv` and `labels.tsv` in the data folder (the `dump\` subtree
  aside) and nothing under `sample\`.
- **Dry run first**: the script ran unchanged with `<out-dir>` a scratchpad
  folder holding copies of `scores_fresh.tsv`, `manifest.tsv` and
  `strata.tsv` and a synthetic `labels.tsv` (seed 7, discard with a
  per-stratum probability); all checks passed and the tables were sane, and
  a second run there refused on the existing `freshread.out.md`. Then the
  real read ran **once**: 25 s, 19 label-free and 6 label checks passed.
- **Choices the text left open**:
  - The "close" clause "only quantile or 'beats' conditions failing" is
    implemented as "`RR` point <= 25% with conditions 2 and 3 holding": once
    those hold, every remaining condition (the `RR` 97.5% quantile, bar 4,
    bar 5) is a quantile or a "beats" condition, so the clause adds nothing
    further.
  - Fragility: a usable cut is fragile when the cluster version of bar 1's
    quantile, bar 4 or bar 5 fails (no cut was usable, so it did not arise).
  - The missing-face subset and its complement are read beside (AUC only),
    as Step 3 did, because Step 7 must say whether the gain sat in the
    missing-face frames; this reading decides nothing and was not in the
    confirmation's pre-registration.
- **Result**: `LR/T` does not pass (bar 5 fails at q5 and q10, AUC − `ef`
  −0.013 [−0.048, +0.023]; bar 2 fails at q10). The follow-up rule gives
  **(2) more labels**, through the first "close" clause at q5 (`RR` 8.3%,
  `PL` 2.56%, removed 4.98%). `ef`'s own q5 cut would meet the same three
  conditions, which the Decision should weigh.
- No `__pycache__/` (`-B` and `sys.dont_write_bytecode`).
