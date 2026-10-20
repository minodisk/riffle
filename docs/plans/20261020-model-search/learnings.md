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
  nothing on sklearn 1.9.1 and, on the spot fold for both targets, no step
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
    normalized rank plus 1 in the upper part, so the per-folder cut orders
    the lower part first, each part by `G0`.
  - `stack`'s mean fill and standardization are fit on every frame of the
    training folders (as `design` does), its fit on their labeled frames.
- **Tripped**: a stray `python3 -` (meant as a no-op) hung on the Git Bash
  stub as the plan warns; it was killed by PID. Use script files only.

## Deferred issues (todo candidates)

- None from Step 1.
- None from Step 2.
