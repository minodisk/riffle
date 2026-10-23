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
    per-round rescale to mean 1 are this plan's addition (stated so in the
    pre-registration).
- The face-choice `af-whole` read is +0.014 on round 2 but −0.029
  [−0.071, +0.014] on round 1; the plan quoted round 2 only. Recorded in the
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
