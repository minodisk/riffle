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

## Deferred issues (todo candidates)

- None from Step 1.
