# Learnings: face-choice

## Step 1: pre-registration

- The scan's cue lives in `crates/core/src/scan.rs` (`riffle_core::scan`),
  not in `crates/app`; the plan's `scan::extract_analysis` reference is to
  that module.
- "Missing last" in the archived reads means the lowest score: `-inf` in
  the per-folder cut (so a missing frame is removed first, which is why the
  stored `eye_focus`'s `PL` at q5, 4.44%, is close to random's) and the
  folder's lowest finite score minus 1 in the AUC. The pre-registration
  states it that way rather than as "sorted after every value".
- Choices the plan text left open, fixed in `facechoice.md`: the 0.8 floor
  applies to `largest`, `sharpest`, `score` and `mean` and to the no-AF
  fallback, while `af-whole` takes `nearest_face` over the whole set at the
  detector's 0.6 (the crop set's floor, so only the detection differs from
  `af`); `score` needs its top face at or above 0.8 (the face `chosen_face`
  would take); ties follow `max_by` (later face) and `min_by` (earlier
  face); the disagreement subset needs a face from both rules, frames with
  exactly one counted and left out; the multi-face subset is 2+ whole faces
  at or above 0.8; the share each rule leaves missing is printed with the
  face counts (a reading).
- The manifests print `eye_focus` to 6 significant digits, so the `af`
  reproduction against them is pre-registered at |difference| <= 1e-6 (the
  1e-9 tolerance is Step 2's check against `scan::extract_analysis`).
- Risk for Steps 2–3: the dump's stored `eye_focus` dates from 2026-10-12;
  `git log` shows no change to `candidate.rs`, `faces.rs`, `eyes.rs` or the
  cue path of `scan.rs` since then that should alter the value (#867 only
  carries every face's values to the UI), but if `af` does not reproduce,
  the read stops before any label is opened.
