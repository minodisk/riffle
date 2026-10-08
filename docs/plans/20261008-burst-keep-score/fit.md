# Step 3 fit: thresholds and combinations from the picks, held out by folder

Measured on 2026-10-08 with [fit.py](fit.py) over the Step 1 dumps
(`D:\Photos\tests\2026-10-08-burst-keep-score\dump\`), `python -I fit.py
<dump-dir> <frozen.json>`. It reads the frames, labels, bursts and scorable
bursts through [metrics.py](metrics.py), so every count is Step 2's
([results.md](results.md)). It runs in about 6 minutes (the logistic
fits, standard library only). The tables under "Tables" are its output as
printed (headings demoted by one level); its raw output is in
`D:\Photos\tests\2026-10-08-burst-keep-score\step3\`. The chosen variant
fitted on every ARW folder is in [frozen.json](frozen.json).

## Method

- **Held out by folder.** Every variant is fitted on the scorable bursts of
  the 27 sidecar-labeled ARW folders (26 have a scorable burst: 1802
  bursts, 4133 picks, 12,389 non-picks, gap 1000 ms) with
  leave-one-folder-out: thresholds or coefficients from the other 25
  folders, metrics on the held-out one, counts pooled. The training
  columns fit and score on every folder. The per-folder spread is over the
  20 held-out folders with at least 20 picks.
- **The DNG blocks** (68 and 32 picks in scorable bursts) are too small to
  fit on; the threshold variants fitted on every ARW folder are applied to
  them as a transfer check.
- **(i) Per-feature thresholds from the picks.** Each feature of a set at
  the p-th percentile of the training picks having it (the upper one for
  `|yaw|`, `|pitch|`, `|roll|`); a frame fails if any feature crosses, and a
  frame lacking a feature (no cue face, no judged face, no pose) is not
  failed by it, so a face-free frame is judged on sharpness alone. p = 1,
  2, 5%, and a **matched** form: one common p per set, the largest whose
  union fails at most 1 / 2 / 5% of the training picks, so every set is
  compared with sharpness alone at the same pick false-fail.
- **(ii) Positive-unlabeled logistic.** Picks 1, non-picks 0, plain
  maximum likelihood by Newton's method on the standardized features
  (`ln rel` with `rel` floored at 0.001, the `eye_focus` logit, eyes open,
  `|yaw|` / `|pitch|` / `|roll|` clipped at 90 / 60 / 60 deg, their
  differences from the burst median clipped at 60 / 45 / 45 deg, and
  indicators for no cue face, no eyes judgment and no pose, a missing value
  imputed with the training mean). Thresholded at the score that keeps 99 /
  98 / 95% of the training picks. Every fit converged in 5 iterations
  without regularization.
- **(iii) Pairwise within-burst logistic** on pick minus non-pick feature
  differences of one burst (41,944 pairs on all folders), no intercept, and
  **(iv) the pointwise logistic** of (ii) as a ranking, both with the
  position metrics of Step 2 over held-out scores.
- **Drop one feature** for the chosen set and for all six (matched), and
  drop one group for the logistic.

## Reading

- **No combination beats sharpness alone by more than the per-folder
  spread.** At a held-out pick false-fail of about 1%, sharpness over the
  burst maximum alone (`rel >= 0.027`) fails 1.1% of the picks and flags
  2.8% of the non-picks, keeping 98.5% of a burst on average. The best
  combinations at the same false-fail flag 0.5-0.8 points more:
  `rel + eye_focus + eyes open` 3.3% (gain +0.5 pt; per folder +1.2 pt
  mean, sd 2.6, positive in 12 of 20 folders), `rel + eyes open` 3.6%, the
  PU logistic at 99% 3.5%. At 2% the gains are +0.0 to +0.6 pt (sharpness
  alone 4.8%, the logistic at 98% 5.4%), and at 5% every threshold set is
  below sharpness alone (9.8%) and only the logistic is above (10.6%).
  Sharpness alone flags 0.0-9.1% per folder at 1%, and the chosen set's
  per-folder gain has an sd of 2.6 pt at 1% and 4.1 pt at 2%, five to
  twenty times its pooled gain.
- **The face checks flag few frames once the picks are protected.** The
  1st-percentile cuts are extreme (`eye_focus >= 0.14-0.18`, eyes open
  `>= 0.07-0.10` against the app's 0.772 and 0.5), so they fire on a few
  hundred frames, and a pick-matched union must lower the sharpness cut to
  pay for them (`rel >= 0.013` instead of 0.027 at 1%). Alone, at 1%,
  `eye_focus` flags 1.3% and eyes open 2.0% of the non-picks.
- **The pose adds nothing.** Adding `|yaw|` / `|pitch|` / `|roll|` to the
  thresholds lowers the flag rate at every target (all six 2.6% against
  3.3% without the pose at 1%), and dropping either pose group from the
  logistic moves its flag rate by 0.1-0.2 pt. Its logistic coefficients
  are the smallest (|w| <= 0.09 on standardized features). The pose labels
  are unreviewed, but the Decision does not rest on the pose.
- **An absolute floor does not help.** `rel + abs floor` matched is 2.9%
  at 1% (+0.1 pt) and 4.8% at 2%; absolute sharpness alone flags the same
  share as relative.
- **Ranking.** The held-out logistics rank the picks slightly above
  sharpness (pairwise AUC 0.590-0.594 against 0.577, picks in the top half
  55.6-55.8% against 54.9%) and far below the capture order (the first
  frame 0.653, 63.8%). The pose-free fits rank as well as the full ones.
- **Rough precision.** Over the frozen variant (`rel >= 0.0134`,
  `eye_focus >= 0.1406`, eyes open `>= 0.0647`, fitted on every ARW
  folder) the flagged frames are 9.0% picks (41 of 454) against 25.0% of
  all scorable frames, so a pick is 2.8 times rarer among them. Of the 30 hand-checked
  non-picks of Step 2 (drawn from rule (e)'s flags), it flags 3, all of
  them looked at as failed (01 by `rel`, 08 and 24 by `eye_focus`); the
  other 27 (16 OK, 6 failed, 5 unsure) pass. A high precision on a very
  small set: the checks that protect the picks are mostly right when they
  fire but fire on about 3% of the non-picks.
- **PU class prior.** Elkan and Noto's `c = E[g | pick]` is 0.266, which
  would make 92% of the non-picks technically OK; the selected-at-random
  assumption behind it does not hold (a pick is chosen among the OK frames
  by composition and moment), so this is a sensitivity, not an estimate.
  At the 98% threshold `g = 0.135`, `P(OK)` is 0.26-0.50 for an assumed OK
  share of 30-90%: a frame the logistic fails is still as likely as not to
  be fine.
- **DNG transfer.** The sidecar-labeled Leica block (68 picks, 130
  non-picks) fails no pick at the 1% fits and flags 2.3% (sharpness) /
  1.5% (with the face checks). The `Output/`-labeled block (32 picks) is
  noisy: sharpness alone at 1% fails 2 of its picks and flags 20.2%.
- **Gaps.** At 2000 and 5000 ms the 1% numbers are the same within 0.3 pt
  (sharpness 2.8%, the chosen set 3.0-3.3%), so the gap does not change
  the answer.
- **Caveat: the dumps predate the mesh `eye_focus`.** The Step 1 dumps
  were taken with a CLI built before `20261008-mesh-eye-focus` Step 3
  (#733) moved the cue to the mesh eye regions, so `eye_focus` here is the
  eye-window cue. Dropping `eye_focus` from the chosen set raises its flag
  rate (3.6% against 3.3%), so the feature carries no held-out worth in
  this data; a re-dump with the mesh cue could move that, and the frozen
  `eye_focus` cut would need a refit before it shipped.

## Tables

Sidecar-labeled ARW block, gap 1000 ms: 26 folders with a scorable burst, 1802 scorable bursts, 4133 picks, 12389 non-picks. Per-folder spreads are over the 20 held-out folders with at least 20 picks.

### (i) Per-feature thresholds from the picks

### Fixed percentile

Each feature of the set at the p-th percentile of the training picks; a frame fails if any feature crosses.

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged | Thresholds (all folders) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| sharpness (rel), p 1% | 1.1% (44) | 2.8% (350) | 98.5% | 1.1% (0.0%-9.1%) | 4.0% | 1.0% / 2.8% | rel >= 0.027 |
| sharpness (rel), p 2% | 2.1% (85) | 4.8% (594) | 97.3% | 2.6% (0.0%-12.4%) | 4.8% | 2.0% / 4.8% | rel >= 0.053 |
| sharpness (rel), p 5% | 5.1% (209) | 9.8% (1212) | 94.2% | 6.6% (0.0%-23.5%) | 12.4% | 5.0% / 9.6% | rel >= 0.108 |
| sharpness (abs), p 1% | 1.0% (42) | 2.8% (348) | 98.2% | 2.6% (0.0%-7.0%) | 4.8% | 1.0% / 2.8% | abs >= 15.3 |
| sharpness (abs), p 2% | 2.1% (85) | 4.6% (571) | 96.5% | 4.2% (0.0%-11.1%) | 9.5% | 2.0% / 4.6% | abs >= 27.4 |
| sharpness (abs), p 5% | 5.0% (207) | 9.1% (1122) | 92.1% | 8.6% (0.0%-18.5%) | 19.0% | 5.0% / 9.0% | abs >= 55.6 |
| rel + abs floor, p 1% | 1.3% (55) | 3.3% (408) | 97.9% | 2.9% (0.0%-9.1%) | 4.8% | 1.3% / 3.3% | rel >= 0.027, abs >= 15.3 |
| rel + abs floor, p 2% | 2.7% (111) | 5.7% (701) | 96.0% | 4.8% (0.0%-13.5%) | 9.5% | 2.6% / 5.6% | rel >= 0.053, abs >= 27.4 |
| rel + abs floor, p 5% | 7.0% (289) | 12.2% (1511) | 90.4% | 12.6% (0.0%-24.0%) | 19.0% | 6.9% / 12.0% | rel >= 0.108, abs >= 55.6 |
| `eye_focus`, p 1% | 0.9% (37) | 1.2% (147) | 99.2% | 0.6% (0.0%-3.7%) | 3.3% | 0.9% / 1.2% | ef >= 0.176 |
| `eye_focus`, p 2% | 1.8% (76) | 2.4% (294) | 98.5% | 1.4% (0.0%-5.7%) | 4.8% | 1.8% / 2.2% | ef >= 0.231 |
| `eye_focus`, p 5% | 4.6% (191) | 7.1% (875) | 95.7% | 4.9% (0.0%-13.8%) | 10.3% | 4.4% / 6.8% | ef >= 0.563 |
| eyes open, p 1% | 0.8% (32) | 1.8% (222) | 98.2% | 1.7% (0.5%-16.7%) | 5.6% | 0.7% / 1.8% | eo >= 0.095 |
| eyes open, p 2% | 1.5% (62) | 2.4% (298) | 97.4% | 2.4% (0.9%-18.5%) | 9.5% | 1.5% / 2.4% | eo >= 0.132 |
| eyes open, p 5% | 3.7% (154) | 5.6% (689) | 94.3% | 5.7% (3.0%-20.4%) | 12.3% | 3.7% / 5.4% | eo >= 0.347 |
| `eye_focus` + eyes open, p 1% | 1.7% (69) | 3.0% (369) | 97.4% | 2.9% (1.6%-16.7%) | 5.6% | 1.6% / 3.0% | ef >= 0.176, eo >= 0.095 |
| `eye_focus` + eyes open, p 2% | 3.3% (138) | 4.8% (592) | 95.8% | 4.6% (1.7%-18.5%) | 9.5% | 3.2% / 4.6% | ef >= 0.231, eo >= 0.132 |
| `eye_focus` + eyes open, p 5% | 8.1% (334) | 12.1% (1501) | 90.3% | 11.9% (5.1%-20.4%) | 13.5% | 7.8% / 11.7% | ef >= 0.563, eo >= 0.347 |
| rel + `eye_focus` + eyes open, p 1% | 2.7% (112) | 5.7% (702) | 95.9% | 5.0% (1.7%-16.7%) | 5.6% | 2.6% / 5.6% | rel >= 0.027, ef >= 0.176, eo >= 0.095 |
| rel + `eye_focus` + eyes open, p 2% | 5.3% (218) | 9.1% (1126) | 93.4% | 9.7% (1.7%-18.5%) | 9.6% | 5.1% / 9.0% | rel >= 0.053, ef >= 0.231, eo >= 0.132 |
| rel + `eye_focus` + eyes open, p 5% | 12.4% (513) | 19.3% (2396) | 85.7% | 18.0% (7.3%-28.7%) | 18.2% | 12.1% / 18.8% | rel >= 0.108, ef >= 0.563, eo >= 0.347 |
| rel + abs + `eye_focus` + eyes open, p 1% | 2.9% (120) | 6.1% (753) | 95.4% | 5.6% (1.7%-16.7%) | 7.4% | 2.8% / 6.1% | rel >= 0.027, abs >= 15.3, ef >= 0.176, eo >= 0.095 |
| rel + abs + `eye_focus` + eyes open, p 2% | 5.9% (243) | 9.8% (1219) | 92.2% | 10.0% (3.4%-20.4%) | 9.6% | 5.7% / 9.7% | rel >= 0.053, abs >= 27.4, ef >= 0.231, eo >= 0.132 |
| rel + abs + `eye_focus` + eyes open, p 5% | 14.1% (581) | 21.2% (2632) | 82.6% | 21.4% (8.3%-29.2%) | 19.0% | 13.7% / 20.7% | rel >= 0.108, abs >= 55.6, ef >= 0.563, eo >= 0.347 |
| all six (rel, ef, eo, yaw, pitch, roll), p 1% | 4.5% (186) | 7.3% (903) | 94.1% | 6.9% (3.1%-18.5%) | 6.8% | 4.2% / 7.2% | rel >= 0.027, ef >= 0.176, eo >= 0.095, \|yaw\| <= 92.9, \|pitch\| <= 53.7, \|roll\| <= 24.1 |
| all six (rel, ef, eo, yaw, pitch, roll), p 2% | 8.8% (364) | 12.6% (1563) | 89.5% | 12.5% (3.4%-25.9%) | 13.8% | 8.5% / 12.4% | rel >= 0.053, ef >= 0.231, eo >= 0.132, \|yaw\| <= 84.3, \|pitch\| <= 47.1, \|roll\| <= 20.8 |
| all six (rel, ef, eo, yaw, pitch, roll), p 5% | 21.3% (879) | 27.9% (3451) | 76.8% | 27.4% (13.5%-35.4%) | 31.0% | 20.7% / 27.2% | rel >= 0.108, ef >= 0.563, eo >= 0.347, \|yaw\| <= 72.8, \|pitch\| <= 39.8, \|roll\| <= 16.4 |

### Matched to a pick false-fail

One common p per set, the largest whose union fails at most the target share of the training picks. "Gain" is the held-out non-pick flag rate over the sharpness-alone (rel) row of the same target, pooled; the per-folder gain is the mean (sd) of the per-folder differences and the share of folders where it is positive.

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged | Gain over rel | Per-folder gain mean (sd), folders > 0 | p (all folders) | Thresholds (all folders) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| sharpness (rel), target 1% | 1.1% (44) | 2.8% (350) | 98.5% | 1.1% (0.0%-9.1%) | 4.0% | 1.0% / 2.8% | +0.0 pt | +0.0 (0.0), 0/20 | 0.0102 | rel >= 0.027 |
| sharpness (abs), target 1% | 1.0% (42) | 2.8% (348) | 98.2% | 2.6% (0.0%-7.0%) | 4.8% | 1.0% / 2.8% | -0.0 pt | +0.1 (1.1), 9/20 | 0.0102 | abs >= 15.3 |
| rel + abs floor, target 1% | 1.1% (44) | 2.9% (361) | 98.3% | 1.6% (0.0%-8.8%) | 4.8% | 1.0% / 2.9% | +0.1 pt | +0.1 (0.4), 8/20 | 0.0080 | rel >= 0.023, abs >= 12.4 |
| `eye_focus`, target 1% | 1.1% (44) | 1.3% (166) | 99.1% | 0.7% (0.0%-3.9%) | 4.0% | 1.0% / 1.3% | -1.5 pt | -1.2 (1.9), 3/20 | 0.0114 | ef >= 0.182 |
| eyes open, target 1% | 1.0% (42) | 2.0% (247) | 98.0% | 1.9% (0.7%-18.5%) | 9.5% | 1.0% / 2.0% | -0.8 pt | +0.9 (5.3), 11/20 | 0.0138 | eo >= 0.104 |
| `eye_focus` + eyes open, target 1% | 1.1% (45) | 2.2% (275) | 98.2% | 2.3% (0.0%-14.8%) | 4.8% | 1.0% / 2.2% | -0.6 pt | +0.6 (4.2), 8/20 | 0.0063 | ef >= 0.153, eo >= 0.081 |
| rel + `eye_focus` + eyes open, target 1% | 1.1% (46) | 3.3% (410) | 97.8% | 2.6% (0.0%-11.1%) | 3.1% | 1.0% / 3.3% | +0.5 pt | +1.2 (2.6), 12/20 | 0.0040 | rel >= 0.013, ef >= 0.141, eo >= 0.065 |
| rel + abs + `eye_focus` + eyes open, target 1% | 1.1% (46) | 3.3% (412) | 97.8% | 2.8% (0.0%-11.1%) | 3.1% | 1.0% / 3.3% | +0.5 pt | +1.2 (2.6), 12/20 | 0.0036 | rel >= 0.013, abs >= 5.1, ef >= 0.139, eo >= 0.065 |
| all six (rel, ef, eo, yaw, pitch, roll), target 1% | 1.2% (48) | 2.6% (322) | 98.1% | 2.2% (0.0%-7.4%) | 3.1% | 1.0% / 2.5% | -0.2 pt | +0.5 (2.4), 11/20 | 0.0024 | rel >= 0.007, ef >= 0.131, eo >= 0.055, \|yaw\| <= 113.4, \|pitch\| <= 72.0, \|roll\| <= 35.2 |
| sharpness (rel), target 2% | 2.1% (85) | 4.8% (594) | 97.3% | 2.6% (0.0%-12.4%) | 4.8% | 2.0% / 4.8% | +0.0 pt | +0.0 (0.0), 0/20 | 0.0201 | rel >= 0.053 |
| sharpness (abs), target 2% | 2.1% (85) | 4.6% (571) | 96.5% | 4.2% (0.0%-11.1%) | 9.5% | 2.0% / 4.6% | -0.2 pt | +0.5 (2.7), 8/20 | 0.0201 | abs >= 27.4 |
| rel + abs floor, target 2% | 2.0% (83) | 4.8% (600) | 96.7% | 4.2% (0.0%-10.9%) | 9.5% | 2.0% / 4.8% | +0.0 pt | +0.5 (1.9), 9/20 | 0.0165 | rel >= 0.043, abs >= 23.1 |
| `eye_focus`, target 2% | 2.2% (89) | 2.8% (345) | 98.2% | 1.6% (0.0%-6.2%) | 5.2% | 2.0% / 2.6% | -2.0 pt | -1.7 (2.9), 3/20 | 0.0226 | ef >= 0.257 |
| eyes open, target 2% | 2.0% (83) | 3.2% (399) | 96.6% | 3.3% (1.7%-20.4%) | 9.5% | 2.0% / 3.2% | -1.6 pt | +0.7 (6.7), 11/20 | 0.0273 | eo >= 0.183 |
| `eye_focus` + eyes open, target 2% | 2.0% (84) | 3.4% (417) | 97.1% | 3.5% (1.7%-18.5%) | 5.6% | 2.0% / 3.4% | -1.4 pt | +0.5 (5.6), 10/20 | 0.0125 | ef >= 0.186, eo >= 0.102 |
| rel + `eye_focus` + eyes open, target 2% | 2.2% (89) | 5.0% (619) | 96.5% | 4.4% (0.0%-16.7%) | 4.8% | 2.0% / 5.0% | +0.2 pt | +1.6 (4.1), 13/20 | 0.0079 | rel >= 0.023, ef >= 0.162, eo >= 0.085 |
| rel + abs + `eye_focus` + eyes open, target 2% | 2.2% (89) | 5.0% (623) | 96.4% | 4.7% (0.0%-16.7%) | 4.8% | 2.0% / 4.9% | +0.2 pt | +1.5 (4.1), 12/20 | 0.0072 | rel >= 0.020, abs >= 11.4, ef >= 0.158, eo >= 0.084 |
| all six (rel, ef, eo, yaw, pitch, roll), target 2% | 2.3% (93) | 4.8% (590) | 96.4% | 4.4% (0.0%-16.7%) | 4.8% | 2.0% / 4.7% | -0.0 pt | +1.3 (4.1), 12/20 | 0.0049 | rel >= 0.016, ef >= 0.147, eo >= 0.069, \|yaw\| <= 100.9, \|pitch\| <= 62.2, \|roll\| <= 29.0 |
| sharpness (rel), target 5% | 5.1% (209) | 9.8% (1212) | 94.2% | 6.6% (0.0%-23.5%) | 12.4% | 5.0% / 9.6% | +0.0 pt | +0.0 (0.0), 0/20 | 0.0501 | rel >= 0.108 |
| sharpness (abs), target 5% | 5.0% (207) | 9.1% (1122) | 92.1% | 8.6% (0.0%-18.5%) | 19.0% | 5.0% / 9.0% | -0.7 pt | +1.4 (6.3), 10/20 | 0.0501 | abs >= 55.6 |
| rel + abs floor, target 5% | 5.1% (209) | 9.2% (1139) | 92.9% | 8.6% (0.0%-18.6%) | 19.0% | 5.0% / 9.3% | -0.6 pt | +0.8 (4.4), 9/20 | 0.0365 | rel >= 0.084, abs >= 44.8 |
| `eye_focus`, target 5% | 5.3% (219) | 8.3% (1023) | 95.0% | 5.3% (0.0%-15.4%) | 10.5% | 5.0% / 7.9% | -1.5 pt | -2.5 (4.4), 5/20 | 0.0566 | ef >= 0.625 |
| eyes open, target 5% | 5.0% (206) | 7.5% (929) | 92.5% | 7.1% (4.6%-24.1%) | 15.4% | 5.0% / 7.5% | -2.3 pt | +0.8 (9.9), 11/20 | 0.0682 | eo >= 0.498 |
| `eye_focus` + eyes open, target 5% | 5.3% (218) | 7.3% (906) | 93.9% | 6.9% (3.4%-20.4%) | 9.5% | 5.0% / 7.0% | -2.5 pt | -0.7 (7.7), 9/20 | 0.0310 | ef >= 0.327, eo >= 0.206 |
| rel + `eye_focus` + eyes open, target 5% | 5.0% (208) | 9.0% (1110) | 93.5% | 9.6% (1.7%-18.5%) | 9.5% | 5.0% / 8.8% | -0.8 pt | +0.6 (5.3), 8/20 | 0.0196 | rel >= 0.051, ef >= 0.228, eo >= 0.131 |
| rel + abs + `eye_focus` + eyes open, target 5% | 5.1% (212) | 9.1% (1133) | 92.9% | 9.4% (1.7%-20.4%) | 9.5% | 5.0% / 9.1% | -0.6 pt | +0.8 (5.7), 10/20 | 0.0181 | rel >= 0.049, abs >= 24.9, ef >= 0.221, eo >= 0.121 |
| all six (rel, ef, eo, yaw, pitch, roll), target 5% | 5.1% (210) | 8.2% (1017) | 93.4% | 7.7% (3.1%-22.2%) | 10.3% | 5.0% / 8.0% | -1.6 pt | +0.3 (6.3), 8/20 | 0.0119 | rel >= 0.032, ef >= 0.183, eo >= 0.099, \|yaw\| <= 91.6, \|pitch\| <= 51.4, \|roll\| <= 23.5 |

### Drop one feature (the chosen set, matched)

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| target 1%, none dropped | 1.1% (46) | 3.3% (410) | 97.8% | 2.6% (0.0%-11.1%) | 3.1% | 1.0% / 3.3% |
| target 1%, without rel | 1.1% (45) | 2.2% (275) | 98.2% | 2.3% (0.0%-14.8%) | 4.8% | 1.0% / 2.2% |
| target 1%, without ef | 1.0% (43) | 3.6% (441) | 97.6% | 3.5% (0.0%-14.8%) | 4.8% | 1.0% / 3.4% |
| target 1%, without eo | 1.1% (47) | 2.8% (342) | 98.4% | 1.3% (0.0%-8.4%) | 3.6% | 1.0% / 2.7% |
| target 2%, none dropped | 2.2% (89) | 5.0% (619) | 96.5% | 4.4% (0.0%-16.7%) | 4.8% | 2.0% / 5.0% |
| target 2%, without rel | 2.0% (84) | 3.4% (417) | 97.1% | 3.5% (1.7%-18.5%) | 5.6% | 2.0% / 3.4% |
| target 2%, without ef | 2.0% (83) | 5.0% (621) | 96.4% | 5.0% (1.1%-18.5%) | 5.6% | 2.0% / 4.9% |
| target 2%, without eo | 2.1% (87) | 4.1% (502) | 97.6% | 1.9% (0.0%-10.9%) | 4.8% | 2.0% / 4.0% |
| target 5%, none dropped | 5.0% (208) | 9.0% (1110) | 93.5% | 9.6% (1.7%-18.5%) | 9.5% | 5.0% / 8.8% |
| target 5%, without rel | 5.3% (218) | 7.3% (906) | 93.9% | 6.9% (3.4%-20.4%) | 9.5% | 5.0% / 7.0% |
| target 5%, without ef | 5.1% (211) | 9.7% (1200) | 92.8% | 9.5% (1.7%-22.2%) | 9.5% | 5.0% / 9.6% |
| target 5%, without eo | 5.3% (217) | 8.7% (1075) | 94.7% | 6.3% (0.0%-17.0%) | 11.2% | 5.0% / 8.5% |

### Drop one feature (all six, matched)

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| target 1%, none dropped | 1.2% (48) | 2.6% (322) | 98.1% | 2.2% (0.0%-7.4%) | 3.1% | 1.0% / 2.5% |
| target 1%, without rel | 1.1% (47) | 2.2% (271) | 98.1% | 2.2% (0.0%-9.3%) | 4.6% | 0.9% / 2.0% |
| target 1%, without ef | 1.1% (46) | 2.8% (344) | 97.9% | 2.6% (0.0%-9.3%) | 4.6% | 1.0% / 2.6% |
| target 1%, without eo | 1.2% (48) | 2.1% (265) | 98.6% | 1.8% (0.0%-5.6%) | 2.8% | 1.0% / 2.0% |
| target 1%, without yaw | 1.2% (49) | 2.8% (350) | 98.0% | 2.5% (0.0%-9.3%) | 3.1% | 1.0% / 2.7% |
| target 1%, without pitch | 1.2% (51) | 2.8% (347) | 98.0% | 2.4% (0.0%-7.4%) | 4.6% | 1.0% / 2.7% |
| target 1%, without roll | 1.2% (48) | 2.7% (339) | 98.0% | 2.3% (0.0%-9.3%) | 4.6% | 1.0% / 2.6% |
| target 2%, none dropped | 2.3% (93) | 4.8% (590) | 96.4% | 4.4% (0.0%-16.7%) | 4.8% | 2.0% / 4.7% |
| target 2%, without rel | 2.2% (91) | 3.4% (418) | 96.9% | 3.1% (1.7%-18.5%) | 4.8% | 2.0% / 3.3% |
| target 2%, without ef | 2.2% (92) | 4.7% (586) | 96.3% | 4.5% (1.7%-16.7%) | 4.8% | 2.0% / 4.6% |
| target 2%, without eo | 2.2% (92) | 4.0% (491) | 97.2% | 3.2% (0.0%-9.8%) | 4.8% | 2.0% / 3.8% |
| target 2%, without yaw | 2.2% (89) | 4.8% (595) | 96.5% | 4.5% (0.0%-16.7%) | 4.8% | 2.0% / 4.7% |
| target 2%, without pitch | 2.2% (91) | 4.8% (592) | 96.5% | 4.0% (0.0%-14.8%) | 4.8% | 2.0% / 4.7% |
| target 2%, without roll | 2.3% (93) | 4.9% (612) | 96.4% | 4.5% (0.0%-16.7%) | 4.8% | 2.0% / 4.8% |
| target 5%, none dropped | 5.1% (210) | 8.2% (1017) | 93.4% | 7.7% (3.1%-22.2%) | 10.3% | 5.0% / 8.0% |
| target 5%, without rel | 5.1% (212) | 6.5% (801) | 93.8% | 6.1% (3.4%-24.1%) | 10.3% | 5.0% / 6.3% |
| target 5%, without ef | 5.2% (216) | 8.2% (1016) | 93.1% | 8.1% (3.4%-24.1%) | 10.3% | 5.0% / 8.0% |
| target 5%, without eo | 5.3% (218) | 7.4% (921) | 94.4% | 5.6% (0.0%-13.3%) | 10.3% | 5.0% / 7.2% |
| target 5%, without yaw | 5.3% (219) | 8.5% (1057) | 93.3% | 8.2% (3.4%-22.2%) | 10.3% | 5.0% / 8.4% |
| target 5%, without pitch | 5.1% (211) | 8.1% (1003) | 93.5% | 7.4% (3.4%-20.4%) | 10.3% | 5.0% / 8.1% |
| target 5%, without roll | 5.2% (216) | 8.6% (1070) | 93.2% | 8.3% (1.7%-22.2%) | 10.0% | 5.0% / 8.4% |

### Transfer to the DNG blocks

Thresholds fitted on every ARW folder (matched), applied to the DNG blocks (no AF point, so no `eye_focus`).

| Block | Fit | Set | Pick false-fail | Non-pick flagged | Mean share kept |
| --- | ---: | ---: | ---: | ---: | ---: |
| dng-sidecar | target 1% | rel | 0.0% (0/68) | 2.3% (3/130) | 98.5% |
| dng-sidecar | target 1% | rel, ef, eo | 0.0% (0/68) | 1.5% (2/130) | 99.1% |
| dng-sidecar | target 1% | rel, ef, eo, yaw, pitch, roll | 1.5% (1/68) | 1.5% (2/130) | 98.8% |
| dng-sidecar | target 2% | rel | 0.0% (0/68) | 3.8% (5/130) | 97.5% |
| dng-sidecar | target 2% | rel, ef, eo | 0.0% (0/68) | 3.8% (5/130) | 97.2% |
| dng-sidecar | target 2% | rel, ef, eo, yaw, pitch, roll | 1.5% (1/68) | 3.1% (4/130) | 97.7% |
| dng-sidecar | target 5% | rel | 1.5% (1/68) | 7.7% (10/130) | 94.6% |
| dng-sidecar | target 5% | rel, ef, eo | 2.9% (2/68) | 6.2% (8/130) | 94.8% |
| dng-sidecar | target 5% | rel, ef, eo, yaw, pitch, roll | 2.9% (2/68) | 6.2% (8/130) | 95.0% |
| dng-output | target 1% | rel | 6.2% (2/32) | 20.2% (22/109) | 91.1% |
| dng-output | target 1% | rel, ef, eo | 0.0% (0/32) | 16.5% (18/109) | 93.4% |
| dng-output | target 1% | rel, ef, eo, yaw, pitch, roll | 0.0% (0/32) | 1.8% (2/109) | 98.3% |
| dng-output | target 2% | rel | 15.6% (5/32) | 33.0% (36/109) | 86.8% |
| dng-output | target 2% | rel, ef, eo | 6.2% (2/32) | 19.3% (21/109) | 90.3% |
| dng-output | target 2% | rel, ef, eo, yaw, pitch, roll | 3.1% (1/32) | 18.3% (20/109) | 91.2% |
| dng-output | target 5% | rel | 28.1% (9/32) | 43.1% (47/109) | 78.6% |
| dng-output | target 5% | rel, ef, eo | 15.6% (5/32) | 35.8% (39/109) | 82.8% |
| dng-output | target 5% | rel, ef, eo, yaw, pitch, roll | 9.4% (3/32) | 25.7% (28/109) | 83.4% |

### (ii) Positive-unlabeled logistic

Thresholded at the score that keeps 99 / 98 / 95% of the training picks.

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| all features, keep 99% | 1.1% (47) | 3.5% (439) | 98.2% | 2.1% (0.0%-10.2%) | 3.6% | 1.0% / 3.6% |
| all features, keep 98% | 2.2% (90) | 5.4% (671) | 97.2% | 3.6% (0.0%-13.1%) | 4.8% | 2.0% / 5.3% |
| all features, keep 95% | 5.3% (221) | 10.6% (1317) | 94.2% | 8.4% (0.0%-19.8%) | 10.0% | 5.0% / 10.7% |
| no pose, keep 99% | 1.1% (44) | 3.4% (425) | 98.3% | 1.9% (0.0%-10.2%) | 3.1% | 1.0% / 3.5% |
| no pose, keep 98% | 2.1% (86) | 5.3% (656) | 97.1% | 3.4% (0.0%-12.6%) | 4.5% | 2.0% / 5.2% |
| no pose, keep 95% | 5.4% (223) | 10.9% (1349) | 93.8% | 9.8% (0.0%-19.4%) | 9.6% | 5.0% / 10.9% |

### Drop one group (all features, keep 98%)

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| none dropped | 2.2% (90) | 5.4% (671) | 97.2% | 3.6% (0.0%-13.1%) | 4.8% | 2.0% / 5.3% |
| without sharp | 2.2% (91) | 3.4% (416) | 98.0% | 2.5% (0.0%-6.7%) | 5.6% | 2.0% / 3.2% |
| without ef | 2.1% (86) | 5.2% (641) | 97.1% | 3.6% (0.0%-12.1%) | 5.2% | 2.0% / 5.1% |
| without eo | 2.3% (93) | 5.2% (641) | 97.3% | 3.0% (0.0%-13.1%) | 4.9% | 2.0% / 5.1% |
| without pose | 2.1% (88) | 5.3% (654) | 97.3% | 3.3% (0.0%-13.3%) | 4.8% | 2.0% / 5.2% |
| without posediff | 2.1% (85) | 5.2% (650) | 97.2% | 3.4% (0.0%-12.8%) | 4.8% | 2.0% / 5.2% |

### Coefficients (all folders, standardized features)

Newton iterations 5, converged True, ridge 0.0.

| Term | Coefficient |
| --- | ---: |
| intercept | -1.1057 |
| ln rel | +0.2852 |
| `eye_focus` logit | +0.0645 |
| eyes open | +0.1657 |
| \|yaw\| (<= 90) | -0.0174 |
| \|pitch\| (<= 60) | +0.0447 |
| \|roll\| (<= 60) | +0.0217 |
| \|yaw - median\| (<= 60) | -0.0159 |
| \|pitch - median\| (<= 45) | -0.0877 |
| \|roll - median\| (<= 45) | +0.0144 |
| no cue face | +0.3130 |
| no eyes judgment | -0.2474 |
| no pose | +0.0383 |

### Class prior sensitivity

Elkan-Noto c = E[g | pick] = 0.266, implying 92.2% of the non-picks are technically OK under the selected-completely-at-random assumption (which the user's framing breaks: a pick is chosen by composition and moment among the OK frames, so the true share is likely higher). The 98% pick-kept threshold sits at g = 0.135; P(OK) = g / c there for an assumed OK share of the non-picks:

| Assumed OK share of non-picks | c | P(OK) at the 98% threshold |
| --- | ---: | ---: |
| 30% | 0.527 | 0.256 |
| 50% | 0.400 | 0.337 |
| 70% | 0.323 | 0.417 |
| 90% | 0.270 | 0.498 |

### Ranking: (iii) pairwise and (iv) pointwise logistic, held out

Position metrics as in results.md over the held-out scores; baselines need no fit.

| Score | Picks in top half | Top third | Worst pick median | Worst pick p90 | Bursts with worst pick >= 0.9 | Pairwise AUC | Absolute AUC | Top-1 hit | Mean pick q |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| (iii) pairwise, all features | 55.6% | 42.9% | 0.60 | 1.00 | 26.8% | 0.592 | 0.584 | 41.3% | 0.429 |
| (iii) pairwise, no pose | 55.8% | 42.6% | 0.60 | 1.00 | 27.3% | 0.590 | 0.585 | 41.0% | 0.432 |
| (iv) pointwise, all features | 55.8% | 42.6% | 0.60 | 1.00 | 26.5% | 0.594 | 0.580 | 41.6% | 0.429 |
| (iv) pointwise, no pose | 55.8% | 42.7% | 0.60 | 1.00 | 27.2% | 0.593 | 0.584 | 41.2% | 0.431 |
| Sharpness / burst max | 54.9% | 41.7% | 0.60 | 1.00 | 28.0% | 0.577 | 0.582 | 40.5% | 0.438 |
| First frame | 63.8% | 46.7% | 0.50 | 1.00 | 16.3% | 0.653 | 0.621 | 39.5% | 0.378 |
| Random (seed 0) | 46.7% | 33.6% | 0.75 | 1.00 | 37.1% | 0.495 | 0.496 | 29.9% | 0.503 |

Pairwise fit on all folders: 41944 pairs, Newton iterations 5, converged True, ridge 0.0.

| Term | Coefficient |
| --- | ---: |
| ln rel | +0.3115 |
| `eye_focus` logit | +0.0659 |
| eyes open | +0.2581 |
| \|yaw\| (<= 90) | -0.1070 |
| \|pitch\| (<= 60) | +0.0588 |
| \|roll\| (<= 60) | +0.0396 |
| \|yaw - median\| (<= 60) | -0.0341 |
| \|pitch - median\| (<= 45) | -0.1060 |
| \|roll - median\| (<= 45) | -0.0001 |
| no cue face | +0.2740 |
| no eyes judgment | -0.3349 |
| no pose | +0.0140 |

### Chosen variant

Set rel, ef, eo, matched to a 1% training pick false-fail.

### Per held-out folder

| Folder | Picks | Non-picks | Chosen: false-fail / flagged | rel alone: false-fail / flagged |
| --- | ---: | ---: | ---: | ---: |
| `2026-06-05` | 199 | 662 | 1.5% / 2.4% | 4.0% / 3.6% |
| `2026-06-13` | 117 | 423 | 0.9% / 5.4% | 0.0% / 3.8% |
| `2026-06-14` | 143 | 626 | 1.4% / 5.3% | 0.7% / 4.3% |
| `2026-06-20` | 127 | 398 | 0.0% / 1.5% | 0.0% / 0.8% |
| `2026-07-02` | 13 | 59 | 0.0% / 8.5% | 0.0% / 0.0% |
| `2026-07-05` | 251 | 721 | 0.8% / 5.8% | 1.2% / 4.6% |
| `2026-07-11` | 29 | 55 | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-18` | 271 | 500 | 0.0% / 3.8% | 0.4% / 4.6% |
| `2026-07-21` | 3 | 4 | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-22` | 19 | 43 | 5.3% / 7.0% | 0.0% / 0.0% |
| `2026-07-24` | 15 | 26 | 0.0% / 3.8% | 0.0% / 0.0% |
| `2026-07-26` | 7 | 32 | 0.0% / 6.2% | 0.0% / 3.1% |
| `2026-07-30` | 65 | 163 | 3.1% / 1.8% | 0.0% / 0.0% |
| `2026-07-31` | 34 | 96 | 0.0% / 2.1% | 0.0% / 0.0% |
| `2026-08-01` | 433 | 1187 | 1.2% / 1.4% | 0.9% / 1.5% |
| `2026-08-02` | 21 | 54 | 0.0% / 11.1% | 0.0% / 0.0% |
| `2026-08-08` | 54 | 131 | 1.9% / 3.8% | 0.0% / 0.0% |
| `2026-08-15` | 21 | 59 | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-08-16` | 6 | 19 | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-08-22` | 266 | 1005 | 1.1% / 2.3% | 0.0% / 0.4% |
| `2026-08-29` | 468 | 1212 | 0.6% / 3.6% | 0.9% / 4.0% |
| `2026-09-13-b` | 223 | 723 | 2.2% / 5.1% | 3.1% / 4.7% |
| `2026-09-19` | 274 | 898 | 1.8% / 1.0% | 0.7% / 0.2% |
| `2026-09-27-a` | 718 | 1920 | 1.3% / 2.7% | 1.5% / 3.1% |
| `2026-09-27-b` | 170 | 571 | 1.8% / 7.4% | 1.8% / 9.1% |
| `2026-10-03` | 186 | 802 | 0.5% / 2.5% | 0.0% / 0.7% |

### Other gaps

| Gap ms | Set | Scorable bursts | Held-out false-fail | Held-out flagged | Mean share kept |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1000 | rel | 1802 | 1.1% | 2.8% | 98.5% |
| 1000 | rel, ef, eo | 1802 | 1.1% | 3.3% | 97.8% |
| 2000 | rel | 1786 | 1.1% | 2.8% | 98.5% |
| 2000 | rel, ef, eo | 1786 | 1.1% | 3.2% | 97.8% |
| 5000 | rel | 1539 | 1.1% | 2.8% | 98.4% |
| 5000 | rel, ef, eo | 1539 | 1.1% | 3.0% | 97.7% |

