# Learnings: model-search

## Step 1: the pre-registration of the exploration

- **Lockbox draw**: `lockbox.py` (named so it cannot shadow the archived
  `sample` modules), run as `python -I -B lockbox.py
  D:\Photos\tests\2026-10-12-rank-app D:\Photos\tests\2026-10-20-model-search`
  from the scratchpad, 6 s. It imports only the archived `gate` and reads
  `gate.load`'s frames for each folder's frame and pick counts: no label and
  no model score. The counts match `folderwide.out.md` (27 / 25 qualifying,
  32,459 frames, 4,419 picks). The rule fixed the lockbox size at 7 and
  repeated `rng.choice(27, 7, replace=False)` on one `default_rng(0)` until
  the constraints held; draw 2 was accepted: 7 folders, all qualifying,
  9,591 frames (29.5%), 1,301 picks (29.4%), near the top of the 20–30%
  range because two large folders (`2026-08-01`, `2026-08-29`) are in it.
- **Choices the plan left open, fixed in modelsearch.md**: the two fit sets
  (the pick set on the 20 exploration folders only, its combinations built
  from its own `All@32 / HGB / pick` rather than the stored `G0`, which was
  trained with the lockbox; the discard set on all 27); the Step 5 pick
  read as one fit on all 20 exploration folders; the global clip cap from
  the 600 `inclusion_prob` values; the inner-`C` criterion for the discard
  target (weighted clipped discard AUC per inner fold); the `C = 1.0`
  fallback trigger (no `C` chosen by more than half the folds); a missing
  `eye_focus` stays in the upper part of `G0 | ef-rule`; `stack` uses
  `ln sharpness` with `C = 1.0`; the empty-set branch of the discard rule
  (q5, then no winner); "16 of 25 scaled" as 5 of 7 lockbox folders.
- **Local review led to two more pre-registered decisions** (modelsearch.md):
  a `stack` winner is trained for the Step 5 pick confirmation on the pick fit
  set's held-out `G0` scores of the 20 exploration folders (as in the
  exploration), with its `eye_focus` logit / `ln sharpness` fill and
  standardization fit on those folders; and `stack` is the one candidate whose
  held-out claim is not exact: its training rows' `G0` scores came from models
  that saw the held-out folder (its picks), accepted and not nested. Steps 3
  and 5 should read `stack`'s out-of-folder numbers with that caveat.
- **References are trained with more folders than the pick candidates**:
  the stored `W0` / `F0+ctx` / `G0` are leave-one-folder-out on all 27, so
  on the exploration and lockbox folders they saw the other lockbox folders.
  Recorded as favoring the references.
- **sklearn 1.9.1 docs** (no fit): no documented restriction on
  `monotonic_cst` with NaN; constraints are binary-only and hold over the
  positive class's probability, so the discard target negates every sign.
  `HGB-mono` keeps the NaN; the mean-filled fallback applies only if Step 2
  hits an error or a spot-check violation. Also: `LogisticRegression`'s
  `penalty` is deprecated since 1.8 (removal in 1.10), so `LR-L1` is
  `l1_ratio=1.0, solver="liblinear"`; Step 2 should not pass `penalty`.
  `liblinear` penalizes the intercept, unlike `lbfgs`; noted, not changed.
- While writing the known numbers, `folderwide.out.md`'s per-folder `W0` /
  `F0+ctx` table (which includes the lockbox folders) was read; it predates
  this plan and holds references only. The draw does not use it.

## Step 2: `searchfit.py`, the held-out scores

- **No candidate score was read against `labels.tsv` or against the picks.**
  The discard labels and the picks enter training only; `searchfit.py`
  computes no `DP`, `RR`, `P@q` or AUC. The one reading that touches the
  picks is the pre-registered label-free `PL` of the discard task's cuts on
  the discard fit set. The strict labels were not opened: an audit hook
  records every file opened under the strict-labels directory, and the run
  opened `manifest.tsv` only (for the population).
- **Run**: `python -I -B searchfit.py` with the seven directories and 8 jobs,
  from the scratchpad, in the background. **1,220 s in total** (load 4 s,
  the `HGB-mono` spot checks 7 s, the fits 1,201 s summed over 66 candidates
  × 2 fit sets plus `stack`, the spot-fold checks 4 s), well under the
  hour the plan allowed. Per run: an `HGB` pick-target candidate 6–27 s
  (`All@32` the slowest), a discard-target one 1–15 s, `LR-L2` 1–8 s, `LR-L1`
  on the pick target the slowest logistic, 7–46 s (`liblinear` on ~30k rows;
  `All@16` 46 s on the discard fit set); no form was pathologically slow.
  The per-candidate times are in `searchfit.out.md`'s "Wall time".
- **Resumable**: every fitted run's held-out scores, chosen `C` per fold,
  lockbox-row count and time are cached in
  `D:\Photos\tests\2026-10-20-model-search\searchfit-cache\` (137 `.npz`,
  35 MB); a rerun fits only what is missing. Delete the folder to refit.
- **Checks, all pass**: the local `design` equals `folder_gate.design`
  exactly at `All@32` (`"full"`) and `T+C` (`"noemb"`) on the spot fold
  `2026-09-27-a`; the discard fit set's `All@32 / HGB / pick` reproduces the
  stored `G0` to 5.0e-06 on the spot fold (and on every frame, so the
  discard set's combinations built from the stored `G0` match its own
  column); `All@32 / HGB / discard` with the clip off reproduces the stored
  `D0` exactly on the spot fold; every score finite (the pick set's on the
  22,868 exploration frames, `nan` on the 9,591 lockbox frames); 0 lockbox
  rows inside any pick-set fold's training folders; 0 strict-sample frames
  among the labeled training frames.
- **`HGB-mono` keeps the NaN**: the fit with `monotonic_cst` and NaN raised
  nothing on sklearn 1.9.1 and, on the spot fold for both targets (the pick target's check trained on the exploration folders only, so no lockbox frame or pick enters it), no step
  against a constraint over a 9-point grid of each constrained column's
  non-missing values (200 held-out rows, other columns as they are). The
  mean-filled fallback was not triggered.
- **Weights**: the median sampled `1/π` is 10.320, so the cap is 51.600;
  129 of the 600 labeled frames are clipped. Kish ESS per fold, `1/π` →
  clipped: pick fit set (332–425 labeled training frames) 109.1–138.0 →
  212.2–273.1; discard fit set (504–597) 166.1–195.0 → 326.8–387.7 (the
  unclipped range equals the archived `D0`'s).
- **`C` fallback triggered three times** (no `C` chosen by more than half
  the folds, refit with `C` = 1.0): pick fit set `T+C / LR-L1 / discard`
  (0.1 in 9 of 20 folds) and `Emb@8 / LR-L2 / discard` (0.1 in 8 of 20);
  discard fit set `Emb@32 / LR-L1 / discard` (0.1 in 12 of 27). Elsewhere
  the choice was stable: `LR-L2 / discard` mostly 0.01 (all folds at
  `All@16` / `All@32`), `LR-L1 / discard` mostly 0.1, the pick target mostly
  0.01 on the discard fit set and 0.01–0.1 on the pick fit set.
- **Label-free verdicts (discard fit set)**: every candidate removes >= 4.5%
  of the population at every cut; `PL` <= 5% holds for all 69 at q5, 43 at
  q10 and none at q20. The 26 q10 failures: every `T+C` discard-target
  candidate, the `All@4` / `All@8` discard targets under `HGB`, `HGB-reg`,
  `HGB-mono` and `LR-L1`, the `All@16` discard targets under the three HGB
  forms, `All@32 / HGB / discard`, all 8 `Emb@8` candidates and
  `Emb@32 / HGB-reg / discard`.
- **Choices the text left open**:
  - The pick target's logistic weights are rescaled to mean 1 (as the
    pre-registration says for every form), unlike `folder_gate.fit`, which
    passes the raw per-folder class weights; so the `C` grid regularizes
    differently from `F0`'s, and `searchfit.choose_c` reimplements
    `folder_gate.choose_c`'s criterion with the candidate's weights. Inner
    fits rescale their own training weights to mean 1; a discard inner fold
    with one class is skipped.
  - `G0 ⊕ ef`: the normalized rank is `(average rank − 1) / (m − 1)` over the
    folder's frames with a score (0.5 for a folder of one); `eye_focus` is
    ranked among the frames that have it. `G0 | ef-rule` is `G0`'s
    normalized rank plus 2 in the upper part (beyond the rank span, so the parts never tie), so the per-folder cut orders
    the lower part first, each part by `G0`.
  - `stack`'s mean fill and standardization are fit on every frame of the
    training folders (as `design` does), its fit on their labeled frames.
- **Tripped**: a stray `python3 -` (meant as a no-op) hung on the Git Bash
  stub as the plan warns; it was killed by PID. Use script files only.

## Step 3: `searchread.py`, the exploration read

- **Run**: `python -I -B searchread.py` with the seven directories and 8
  jobs, from the scratchpad: 33 s (load 5 s, cuts and metrics 1 s, the AUC
  bootstraps of 75 scores 26 s with `retargetread.auc_of` per draw in a
  joblib loop over the scores); `--check-all` alone 4 s. Every reference
  check passes digit for digit (discard.md's `G0` / `eye_focus` / random,
  retarget.md's `D0` AUC, folderwide.out.md's `W0` / `F0+ctx`, the
  rejectfilter `G0` frame AUC 0.670).
- **Guard kept**: the pick task read the 18 qualifying exploration folders
  of `scores_pick.tsv` only (a check row counts 0 lockbox folders among
  them); `--check-all` loads no candidate file. Step 5 is the first read of
  the lockbox folders' picks against a candidate.
- **Winners by the rule**: discard `G0 ⊕ ef` (runner-up `All@32 / LR-L2 /
  discard`); pick `All@32 / HGB-mono / pick` (runner-up `G0 ⊕ ef`). No
  candidate's mean `P@10` reaches `W0`'s on the exploration folders.
- **Choices the text left open**:
  - `--check-all` is a separate invocation that runs only the pick task's
    code path on the 25 qualifying folders of all 27 for the stored `W0`,
    `F0+ctx` and `G0` and writes `searchread.check-all.out.md`; the main run
    runs the same function and prints its rows among the reference checks.
  - The top-`q` order is `np.lexsort((-tiebreak, -score))`, the reverse of
    `rf.bottom` and the order `folderwide.r_precision` uses; `k = floor(q *
    n / 100)` (>= 2 on a qualifying folder). The lift check uses
    `folder_gate`'s `ceil(n / 10)`.
  - "Pooled" is `folderwide.pooled`, the frame-weighted mean of the
    per-folder values (as folderwide.out.md's "Pooled"), not a ratio of
    sums.
  - `eye_focus` in the pick task is missing last as `-inf` (as
    `folderwide`'s `EYE`); in the AUC as `bf.finite_by_folder`'s folder
    minimum minus 1.
  - `W0` has no score in `2026-07-23` (4 sampled frames): its cuts leave the
    folder out and its AUC is read on the other 596 labels; its paired AUC
    differences are not printed. Every candidate covers every frame.
  - The reading aids' "against `G0`" is the stored `G0` on both sides; on
    the pick side it was trained with the lockbox folders, the pick-set
    candidates without (the pick set's own `All@32 / HGB / pick` reads −1.6
    points of `P@10` against it). The target comparison is printed as the
    paired discard-target minus pick-target difference per design (each
    target's difference against `G0` is in the dimension-curve tables).
  - The two label-free combinations on the pick winner are read on both
    tasks (pick fit set for the pick side, discard fit set for the discard
    side), point values only.
  - The full tables (every score and cut with cluster intervals, every
    candidate's pick row and paired differences, every reading aid with its
    intervals) are in `searchread.out.md` only; modelsearch.md carries the
    selection tables, the winners beside the references and the reading
    aids as point matrices plus the `All@32` form comparison and the
    combinations with intervals.
- **Facts for Step 4 to weigh** (no bar applied here): the 600 labels were
  drawn in `G0`'s bands, and the discard winner mixes the ranks of `G0` and
  `eye_focus`, the two scores the design was chosen after; three `LR-L1`
  discard-target models read AUC − `eye_focus` with the stratified 2.5%
  quantile above zero, two of them failing the q10 `PL` condition, so the
  rule did not select them.

## Step 4: the pre-registration of the confirmation and the fresh sample

- **Order kept**: the "Pre-registration of the confirmation" (winners, the
  strata, the allocation, the draw seed, the bars, the follow-up rule) was
  written into modelsearch.md before `confirmsample.py` first ran; the
  "As drawn" paragraph was added after the draw and holds label-free numbers
  only. No lockbox pick and no fresh label has been read; the new sample
  has no sidecar, and none of its files other than the copies' bytes (for
  the SHA-256 check) was read.
- **Facts written beside the bars, not moving them**: `G0 ⊕ ef` did not beat
  `eye_focus` on AUC in the exploration (−0.015 [-0.062, +0.030]), so
  condition 6 is the likely failure; the pick winner's `P@10` was below
  `W0`'s on the exploration folders (−0.9 points); the winner's q20 already
  fails `PL` (5.73%), so a pass needs q5 or q10. New and label-free: on the
  fresh population the winner's q5 removes **4.52%**, 0.02 points above the
  4.5% bar (the earlier samples took more of the winner's bottom 5% than of
  the rest, 223 of 1,609 frames).
- **Importing the discard-labels `sample.py` by path**: `badfilter` already
  imports the bad-photo `sample` under the module name `sample`, so a plain
  `import sample` after it returns the cached bad-photo module.
  `confirmsample.py` loads the discard-labels file with
  `importlib.util.spec_from_file_location("discard_sample", ...)`; that file
  only re-imports the cached `badfilter` and defines constants (`CUTS`,
  `CONTROL_FLOOR`, `LIBRARY`, `SIDECARS`) and `bs`, the bad-photo `sample`,
  whose `allocate`, `under`, `listing`, `sha256` and `cell` are the helpers
  reused. Its copy loop is inline in its `main`, so it is repeated in
  `confirmsample.py`, not imported.
- **Choices the text left open**:
  - The draw uses its own generator, `default_rng(20261021)`, rather than
    the archived sampler's continuation of the seed-0 tie-break generator,
    so the new draw does not share a random stream with the earlier one; the
    tie-break draw stays `default_rng(0).random(n)` as every read.
  - A `control` folder cell with no population frame would be skipped (none
    is empty: the smallest, `2026-07-23`, has 4).
  - The manifest adds the winner's score, its per-folder rank share, `G0`'s
    rank share and `G0`'s band (`b05` / `b10` / `b20` / `above`), and drops
    the discard-labels manifest's `strict_label` column; Step 5's
    `confirmlabels.py` must drop it from the archived `labels.py`'s `KEPT`
    and use the five strata.
  - `--no-copy` writes the manifest and prints the tables and the draw
    checks without copying, so the draw was checked (5 s) before the copy.
  - An audit hook records every file opened under the two earlier rounds'
    folders: `manifest.tsv` only in each (no `labels.tsv`).
- **Copy**: 27.4 GB, run in the background with its stdout in
  `confirmsample.out.md` and its progress in `confirmsample.err` (data
  folder), waited on with a bounded `tail -f | grep -m1` rather than a
  sleep: copy 504 s, SHA-256 of sources and copies 316 s, 832 s in total,
  so one run is longer than a 10-minute tool call; a rerun skips the copies
  whose size and mtime match, so it is resumable. The background run drew
  the same sample as the `--no-copy` run (its tables are identical). Every
  copy check passes.

## Step 5: the confirmatory read

- **Labeling session**: the user labeled `sample\` and said "done"; about
  **15 minutes** for 600 frames, about **1.5 s per frame**, half the
  pre-registered estimate (about 3 s per frame, about 30 minutes, from the
  discard-labels round). No odd frame was reported. The user confirmed that
  `aiEnabled` stayed off throughout labeling. A later "more labels" round
  sized at 3 s per frame is therefore likely to take about half its estimate.
- **`confirmlabels.py`**: loads the archived discard-labels `labels.py` by
  path (`importlib`, module name `discard_labels`) and reuses its `xmp_of`,
  `dop_of`, `read` and `table`; its own `main` is that file's with `KEPT`
  minus `strict_label` and the five strata. Runs in about a second; 600
  labeled (248 keep / 352 discard), 248 `.dop` + `.xmp` pairs, 0
  disagreements, **0 unexpected marks**, 0 skipped; stdout saved as
  `confirmlabels.out.md` in the data folder.
- **`confirmread.py`**: imports `badfilter` first, then this plan's
  `searchread` (which brings `discardfilter`, `retargetread` and
  `searchfit`) and reuses `pick_task`, `pdiff`, `summ`, `auc_draws`; the
  count matrix is `discardfilter.counts` with the cell list `b05`, `b10`,
  `b20`, `g0only`, `rest` (the `control` folder cells). It refuses to run
  when `confirmread.out.md` exists, and every computation (cuts, AUC
  bootstraps, the pick fits) runs before anything is printed, with the checks
  printed before the results; it ran **once**, every check passing.
- **Choices the text left open**:
  - The pick winner's single fit is a local `pick_fit` replicating
    `searchfit.fold` for `All@32 / HGB-mono / pick` but scoring every lockbox
    frame from one model; checked by refitting it on 19 exploration folders,
    which reproduces `scores_pick.tsv` on the spot fold exactly.
  - "Fragile" is computed per cut as any of conditions 1, 4, 5, 6 failing on
    the cluster interval; no cut passed, so it never applied.
  - The follow-up's first "close" branch is read as: at q5 or q10 the `RR`
    point <= 25% with conditions 2 and 3 holding (every other condition is
    then a quantile or "beats" one). The pick "close" branch is read
    literally (gain positive and interval spanning zero).
  - Pick bar 1 uses the difference of the means, equal to the mean paired
    difference on the same 7 folders.
- **Wall time far above the reference**: 2,759 s in total (load 264 s
  against 5 s in Step 3; the AUC bootstraps of 4 scores 204 s against 26 s
  for 75; the two `HGB-mono` fits 2,286 s against about 25 s each in Step 2).
  The machine was under heavy load from other processes during the run
  (about a dozen other `python.exe` in the task list); the results do not
  depend on it. The run went past a 10-minute tool call, so it continued in
  the background and was waited on.
- **Results**: discard winner does not pass (condition 6 fails at every cut;
  condition 5 also at q5), follow-up (2); pick winner does not pass (all
  three bars), follow-up (3). Numbers in modelsearch.md, "Results of the
  confirmation".

## Deferred issues (todo candidates)

- None from Step 1.
- None from Step 2.
- None from Step 3.
- **Pending manual check (Step 4): the fresh sample opens unjudged in
  Riffle.** On Windows, before labeling: turn `aiEnabled` off, open
  `D:\Photos\tests\2026-10-20-model-search\sample`, sort by File name, let
  the scan finish. Expected: 600 files `0001_…` to `0600_…`, every one
  without a rating, Pick / Reject flag or color label, no `.xmp` / `.dop`
  in the folder before the first key press, and no recommended icon or
  ranking rows. `confirmsample.py` checked 0 sidecars in `sample\` and the
  copies' bytes; the in-app view is not checked. Step 4's checkbox was
  ticked on the automated criteria. Basis: Step 4 of
  `docs/plans/20261020-model-search/plan.md`, `labeling.md`.
- None from Step 5. Bearing on the Step 4 pending check: the labeled
  sidecars hold no mark other than rating 1 and the user reported
  `aiEnabled` off throughout; whether the sample first opened unjudged in the
  app was not reported separately.
