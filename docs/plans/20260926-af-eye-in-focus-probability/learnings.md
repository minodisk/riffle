# Learnings

## Step 1: combined score in riffle-core and `riffle-cli candidates`

- Validation runs with the release CLI (`riffle-cli candidates <dir>...`, 24
  threads, pooled over the given folders):
  - Training (`2026-06-05`, `2026-07-31`, `2026-09-13-a`, `2026-09-19`,
    `2026-09-19-focus-sample-2`): 500 files, 406 with a face and a label (339
    in focus, 67 off). AUC lap 0.816, combined 0.852. Candidates 332, in focus
    309 (93.1%); coverage 309/339 = **91.2%** (Purpose: 91.4%, i.e. 310).
  - Held-out (`2026-06-14`, `2026-07-18`, `2026-08-01`, `2026-08-22`): 400
    files, 400 faced and labeled (342 / 58). AUC lap 0.635, combined 0.754.
    Candidates 366, in focus 326 (89.1%); coverage 326/342 = 95.3%. Exact.
  - Pass-2 time (`extract_faces` only, cold cache): 11.4 s for the 500
    training files, 8.2 s for the 400 held-out files (a warm re-run of the
    training set: 2.7 s). The edge width is computed on the already-decoded
    luma, so it adds no decode.
- The one-frame coverage gap on the training set is not a port difference.
  The frozen `threshold` is exactly the logit of `_DSC1686.ARW`
  (`2026-06-05`..`2026-09-19` training set, a Pick) computed from the
  reference CSV's rounded columns (`lap` printed `{:.2}` = 128.08,
  `edge_w_rel` `{:.5}` = 0.11097): that gives 1.2194865955352432, bit for bit
  the threshold. From the full-precision values (lap 128.0778..., edge_w_rel
  0.110972850...) the same frame's logit is 1.21944427..., 4.2e-5 below the
  threshold, so it is `NotCandidate` (p 0.772). Every other number, including
  both AUCs and the held-out set, matches exactly. The coefficients were left
  as frozen. If 91.4% must hold exactly, the choice is the user's: lower
  `CANDIDATE_LOGIT` by ~5e-5 (a threshold change, not a refit) or accept
  91.2%.
- Decision (user, after the first Step 1 commit): lower `CANDIDATE_LOGIT`
  from the frozen 1.2194865955352432 to 1.2194, just below `_DSC1686.ARW`'s
  full-precision logit, so the training coverage matches the original
  definition (the `lap >= 80` coverage, 91.4%). Coefficients unchanged.
  Before the change, the logits of every training, held-out and reserved
  frame were printed at full precision: only `_DSC1686.ARW` lies in
  [1.2, 1.24), so nothing else crosses. Re-run with 1.2194: training
  candidates 333, in focus 310 (93.1%), coverage 310/339 = 91.4%, AUC 0.816
  / 0.852; held-out unchanged (366 / 326, 89.1% / 95.3%, AUC 0.635 / 0.754).
  The plan's Frozen model line records the change; the ulp round-trip clamp
  and its test are written against `CANDIDATE_LOGIT`, so they follow the new
  value unchanged.
- No-edge fallback (`edge_width` is `None` -> `NotCandidate`, probability 0):
  triggered on 0 of 406 training faces, 0 of 400 held-out faces and 0 of 200
  faces in the two reserved folders.
- Reserved folders `2026-08-29-focus-sample` and `2026-09-13-b-focus-sample`:
  100 ARW each, no XMP sidecars yet (no labels). Follow-up: run
  `riffle-cli candidates D:\Photos\tests\2026-08-29-focus-sample D:\Photos\tests\2026-09-13-b-focus-sample`
  once labeled and record AUC / precision / coverage.
- Round-tripping the state through a stored probability: several logits a
  few ulps below `CANDIDATE_LOGIT` have a sigmoid equal to
  `sigmoid(CANDIDATE_LOGIT)`, so a probability compared against
  `candidate_probability()` could read back `Candidate` for a frame the logit
  called `NotCandidate`. `scored` lowers such a probability to
  `candidate_probability().next_down()`; a test sweeps 64 ulps either side of
  the threshold.
- `riffle-app` had to change in this step too, because `Cue::eye_sharpness`
  is gone: `run_faces_scan` now writes `cue.eye_focus` into the old
  `eye_sharpness` column, and `candidate()` reads it as a probability. Until
  Step 2 bumps `FACES_VERSION` and swaps the column, rows filled by the old
  pass hold Laplacian values (almost all >= 0.772) and read back as
  `Candidate`. Step 2 must land before a release.
- The CLI's AUC skips frames with no edge width (as the reference skipped
  non-finite values); precision / coverage count every labeled faced frame
  whose preview decoded. Unfaced labeled frames are no longer counted in the
  coverage (the old CLI counted them), matching the reference pooling.
- In this environment the Bash tool's heredoc tripped on a long inline Python
  script (`unexpected EOF`); the Edit tool was simpler for multi-line Rust
  replacements.

## Deferred issues (todo candidates)

- Run `riffle-cli candidates` on `D:\Photos\tests\2026-08-29-focus-sample` and `D:\Photos\tests\2026-09-13-b-focus-sample` once they carry XMP pick / reject labels, and record AUC / precision / coverage (basis: Step 1 found no XMP sidecars in either; files: `crates/cli/src/main.rs`, `crates/core/src/candidate.rs`).
