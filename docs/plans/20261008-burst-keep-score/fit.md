# Step 3 fit: keep marks and failure checks from the picks, held out by folder

Measured on 2026-10-08 over the Step 1 dumps
(`D:\Photos\tests\2026-10-08-burst-keep-score\dump\`) with three scripts
that read the frames, labels and bursts through [metrics.py](metrics.py),
so every count is Step 2's ([results.md](results.md)). Newest framing
first:

- [scene.py](scene.py) (`python -I scene.py <dump-dir>`, about 3 s): the
  **burst level** of the user's second insight, strict rules scored by the
  kept share of the bursts they mark (every burst, also those without a
  pick) and by the pick share of the single frames they mark.
- [keep.py](keep.py) (`python -I keep.py <dump-dir> <frozen.json>`, about
  15 s): the **keep mark** of the reframed goal, strict rules that mark the
  frames clearly meeting the minimum conditions for a pick, scored by
  precision over the scorable bursts' frames. The keep rule of that
  round's alternative is in [frozen.json](frozen.json).
- [fit.py](fit.py) (`python -I fit.py <dump-dir> <frozen-fail-check.json>`,
  about 6 minutes): the **failure check** of the first framing, thresholds
  and logistic fits that flag technical failures while protecting the
  picks. Its chosen variant is in
  [frozen-fail-check.json](frozen-fail-check.json).

The tables under "Tables" are the scripts' output as printed (headings
demoted); the raw output, the hand-check sample and its crops are in
`D:\Photos\tests\2026-10-08-burst-keep-score\step3\` and `keepcheck\`.

## Burst level (the user's second insight)

On 2026-10-08, after approving the no-ship Decision, the user reopened it:
within a burst they also throw away good frames depending on the timing, so
which frame of a burst got picked matters little. The question becomes
whether a strict rule marks the **scenes** the user kept, and the single
frames the user picked. Measured with [scene.py](scene.py) (`python -I
scene.py <dump-dir>`, about 3 s).

### Method

- **Every burst counts** (the 27 sidecar-labeled ARW folders, gap
  1000 ms): 5026 bursts of two or more frames, including the 3214 without
  a pick that Steps 2-3 left out, and 1943 single frames.
- **Bursts.** A burst is **kept** when at least one of its frames is a
  pick (1812, a base of 36.1%), and **marked** by a rule when the rule
  (keep.py's grid of 1600 frame rules, faced frames only) marks at least
  one of its frames; `rel >= 1` in a rule therefore reads "the burst's
  sharpest frame passes". Precision = the kept share of the marked bursts;
  coverage = the share of the bursts marked; recall (the kept bursts
  marked) is for reference only.
- **Single frames.** A single frame is marked when it is faced and passes
  the rule; precision = the pick share of the marked singles (base 12.5%,
  242 of 1943).
- **Held out** as in keep.py: per folder, the rule with the most marked
  units on the other 26 folders (at least 30) among those reaching a target
  training precision (50-95%), scored on the held-out folder, pooled.
- **Burst size** (the number of frames) is not a technical feature but
  turned out to be the strongest signal, so it is reported as a baseline
  (held out, a size cut chosen by the target) and in combination with the
  grid (a size cut and a rule chosen together).
- Frames without a full face: the 1056 bursts with no faced frame (18.5%
  kept) and the 508 face-free singles (7.7% picked) get sharpness-only
  rules. The `eye_focus` caveat of the keep mark (the dumps predate #733)
  holds here too.

### Reading

- **Technical rules reach about 56% of marked bursts kept, held out.**
  With a 60% training target the chosen rules (most often absolute
  sharpness >= 200, `eye_focus` >= 0.95, eyes open >= 0.995) give **55.8%
  held out** (lift 1.55 over the 36.1% base) on 9.0% of the bursts, per
  folder 40.0-91.7% (median 55.7%). A 70% or 75% target gives 49.0% /
  48.6% on 1% of the bursts, and from 80% up no rule reaches the target on
  any training set. In-sample the best grid rule reaches 61.9% at >= 5%
  coverage and 73.1% only at 1% (52 bursts).
- **Burst size alone does better than any technical rule.** A size cut
  chosen on the training folders gives **79.3% held out at size >= 15**
  (9.6% of the bursts, 382 of the 1812 kept, per folder 73.0-89.2%) and
  **80.1% at size >= 20** (5.7%); size >= 6 gives 60.6% on a third of the
  bursts. The user shot longer bursts at the scenes they kept, and no
  technical feature carries that.
- **Technical rules add little on top of size.** Chosen together, size
  and a rule give 80.7% held out at an 80% target (size >= 15 and
  `eye_focus` >= 0.9, 9.0% of the bursts), against 79.3% for size >= 15
  alone; at 85-95% targets they give 76.2-81.7% on 1-5% of the bursts,
  no better than size alone. In-sample, size >= 15 with absolute >= 200,
  `eye_focus` >= 0.95 and eyes open >= 0.995 reaches 86.6% on 134 bursts
  (2.7%), which the held-out selection does not reproduce. No held-out
  variant reaches 85%.
- **Single frames: no rule reaches 50% even on the training folders.**
  The best grid rule in-sample is 32.0% on 25 singles (1.3%), the
  strictest `eye_focus` + eyes-open cut 22.2% (36 singles), against the
  12.5% base.
- **Without a face, sharpness does not find kept scenes.** The bursts with
  no faced frame are kept 18.5% of the time, and 24.3% when the sharpest
  frame's sharpness is at least 800 (29% of them); face-free singles stay
  at 6.6-8.1% at every floor.
- **Without sharpness cuts (the user's question: `eye_focus` already
  measures sharpness at the eyes).** The same held-out selections over the
  64 grid rules that use only `eye_focus`, eyes open and the pose, side by
  side with the full grid (held-out precision, share of bursts marked):

  | Training target | Rule alone, with sharpness | Rule alone, no sharpness | Size + rule, with sharpness | Size + rule, no sharpness |
  | --- | ---: | ---: | ---: | ---: |
  | 50% | 50.3%, 47.5% | 50.3%, 47.5% | 51.5%, 51.2% | 51.5%, 51.2% |
  | 60% | 55.8%, 9.0% | 57.5%, 3.6% | 59.9%, 33.6% | 61.0%, 33.3% |
  | 70% | 49.0%, 1.0% | 65.2%, 0.9% | 69.8%, 19.1% | 70.3%, 19.2% |
  | 75% | 48.6%, 0.7% | none | 70.2%, 10.6% | 74.8%, 10.2% |
  | 80% | none | none | 80.7%, 9.0% | 80.4%, 8.9% |
  | 85% | none | none | 76.2%, 4.6% | 82.2%, 3.8% |
  | 90% | none | none | 81.7%, 2.1% | 87.4%, 1.9% |
  | 95% | none | none | 81.5%, 1.1% | none |

  Removing sharpness costs nothing held out and helps at the strict end,
  where the smaller grid overfits less: the rule alone reaches 65.2% at a
  70% target (46 bursts, `eye_focus` >= 0.97, eyes open >= 0.98 and a
  15-10 frontal pose), and size with a rule reaches **87.4% on 1.9% of the
  bursts** (95 bursts; size >= 20, `eye_focus` >= 0.95, eyes open >= 0.995
  in 26 folds; per folder 75.7-91.7% over the 3 folders with 10 marked),
  against 80.1% on 5.7% for size >= 20 alone. Scored as fixed rules on
  every folder, dropping the sharpness cuts from the rules chosen with
  sharpness lowers precision by 0.9-11.9 points and raises their coverage
  1.1-4.5 times (for example size >= 15 with absolute >= 400, `eye_focus` >= 0.9
  and eyes open >= 0.995: 85.1% on 188 bursts; without the absolute cut
  81.7% on 383), so sharpness narrows rather than sharpens: held out its
  extra cuts do not carry over. Single frames are unchanged: no rule
  reaches 50% in training (in-sample best 25.0% on 24 singles).
- **Does eyes open help? Only together with `eye_focus`, on long bursts.**
  The user asked (2026-10-09) whether eyes open adds anything. Held out,
  over the no-sharpness grid without the pose, limited to `eye_focus`
  only, eyes open only or both (precision, share of bursts marked):

  | Target | `eye_focus` only + size | eyes open only + size | both + size | size alone |
  | --- | ---: | ---: | ---: | ---: |
  | 70% | 70.2%, 17.7% | 69.6%, 17.1% | 70.3%, 19.2% | 75.4%, 10.9% |
  | 80% | 79.7%, 8.4% | 80.1%, 7.0% | 80.4%, 8.9% | 80.1%, 5.7% |
  | 85% | 79.3%, 2.3% | 75.4%, 1.1% | 82.2%, 3.8% | none |
  | 90% | none | none | 87.4%, 1.9% | none |

  Without a size cut no eye rule reaches a 70% target (both together give
  45.9% on 0.7% at 60%). Up to 80% the three variants and size alone are
  within a point of one another; only both features together reach the
  90% target. Scored as fixed rules on every folder, the 87.4% rule (size
  >= 20, `eye_focus` >= 0.95, eyes open >= 0.995) marks 78 bursts at 92.3%
  (per folder 85.0-91.7% over 3 folders); dropping eyes open gives 84.2% on
  146, dropping `eye_focus` 82.9% on 222, and size >= 20 alone 82.3% on
  237. At size >= 15 the same pattern: 87.2% (148) with both, 81.9% (265)
  with `eye_focus` only, 79.5% (439) with eyes open only, 79.3% (482) with
  neither. So each feature alone adds 0-2 points over the burst length, and
  the two together add about ten, on 1.6-2.9% of the bursts.

## Keep mark (the reframed goal)

The user reframed the goal on 2026-10-08, before approving the first
Decision: the mark does not have to catch every pick, since the human makes
the final choice; it should narrow a burst to the frames that clearly meet
the minimum conditions for being chosen (reasonably sharp, the eyes in
focus, the eyes open, the face toward the camera), and those frames should
be almost entirely picks (precision close to 1, the marked frames a subset
of the picks). Recall is secondary.

### Method

- **Rules.** A frame is marked when every condition holds: sharpness over
  the burst maximum `rel` >= 0 / 0.5 / 0.7 / 0.9 / 1.0 (1.0 is the burst's
  sharpest frame), absolute sharpness >= 0 / 200 / 400 / 600 / 800,
  `eye_focus` >= 0 / 0.9 / 0.95 / 0.97, eyes open >= 0 / 0.98 / 0.995 /
  0.999, and `|yaw|` / `|pitch|` within none / 30-20 / 15-10 / 8-5 deg: a
  grid of 1600 rules up to cuts near the top of the picks' distributions
  (the 90th percentile of the picks is `eye_focus` 0.963 and eyes open
  1.000). The full rule applies to the **faced frames** (a cue face, an
  eyes judgment and a pose: 11,693 of the 16,522 frames of the 1802
  scorable ARW bursts, 3020 of them picks, a base rate of 25.8%). The
  other 4829 frames (1113 picks, 23.0%) get sharpness-only rules, reported
  apart.
- **Metrics.** Precision = the pick share of the marked frames; lift =
  precision over the faced frames' base rate (pooled) and over the mean
  pick share of the marked frames' own bursts (per burst); the frames
  marked (share of the faced frames), the picks marked (recall, for
  reference), the scorable bursts with at least one mark, and the
  per-folder precision (median, min-max over the folders with at least 10
  marked frames).
- **Precision is a lower bound.** A non-pick may be a technically fine
  frame the user did not choose (the Purpose's framing); a hand check of
  marked non-picks below estimates how many.
- **Held out.** A fixed grid rule needs no fitting, but choosing the best
  one is a selection. The held-out rows choose, for each folder, the grid
  rule that marks the most training frames (at least 30) among those whose
  precision on the other 25 folders reaches a target (35-90%), and score
  it on the held-out folder, pooled. The learned alternative is fit.py's
  pointwise logistic (iv) on every feature, fitted on the other folders,
  marking the frames above the score of the top 0.5-30% of the training
  faced frames, or above the cut that reaches a target training precision.
- **One mark per burst.** Among a burst's faced frames passing a rule, the
  one with the highest held-out logistic score (or the earliest), against
  marking every burst's first frame (39.5% picks, Step 2), its sharpest
  frame, its highest-scoring frame, or a random frame (29.8%, the mean pick
  share of a burst).
- **Caveat: the dumps predate the mesh `eye_focus`.** The Step 1 CLI was
  built before `20261008-mesh-eye-focus` Step 3 (#733), so `eye_focus` here
  is the eye-window cue, not the mesh cue the scan now stores.

### Reading

- **No rule comes near the target precision.** The highest held-out
  precision with any coverage is about 42%: choosing on the training
  folders by a 40% target picks `rel >= 1` (the burst's sharpest frame,
  when that frame is faced) in all 26 folds and gives **41.6% held out** (lift 1.61 pooled,
  1.37 per burst), marking 1298 frames (11.1% of the faced frames, 540
  picks) in 72.0% of the scorable bursts, per folder 24.3-57.1% (median
  40.1%). Raising the target backfires: at 45% the chosen rules give 32.7%
  held out on 1.7% of the frames, at 50% 30.0% on 0.4%, and from 60% up no
  grid rule reaches the target even on the training folders. The logistic
  does no better (38.1% at its top 2%, 35.2% at its top 10%; at a 60%
  target it marks 31 frames at 38.7%, and none from 80% up). **So
  precision of 80-90% is out of reach of these features; the ceiling is
  about 40-45%, 1.6-1.7 times the base rate.**
- **Stricter cuts stop paying off quickly.** In-sample, the best grid rule
  at a coverage floor reaches 42.9% at >= 5% of the faced frames (`rel >=
  1`, absolute >= 200, `eye_focus` >= 0.9), 45.3% at >= 2% (258 frames),
  and 56.0% only on 25 frames (0.2%), which held out does not survive.
  The rule with all four conditions at the second level (`rel` >= 0.7,
  absolute >= 400, `eye_focus` >= 0.95, eyes open >= 0.995, pose 15-10)
  marks 25 frames; at the third level nothing.
- **Sharpness carries what there is; the face conditions add little and
  the frontal pose subtracts.** Alone, the strictest `eye_focus` cut (0.97)
  gives 30.1%, the strictest eyes-open cut (0.999) 27.6%, and a frontal
  pose 22-25%, at or below the 25.8% base: a frontal face is picked less
  often than a turned one. The picks' and non-picks' distributions of
  `eye_focus` and eyes open are nearly identical at the top (90th
  percentile 0.963 against 0.961, 1.000 against 1.000), so a stricter cut
  marks fewer frames without raising the pick share. The burst's sharpest
  frame (`rel = 1`) alone gives 41.6%, and adding the face conditions moves
  it by at most 1.5 points (42.1% with absolute >= 200, `eye_focus` >= 0.9,
  eyes open >= 0.98; 39.2% when a 30-20 frontal pose is added).
- **One mark per burst equals the existing baselines.** Marking each
  burst's highest-scoring faced frame gives 41.8% (89.2% of the bursts),
  its sharpest frame 40.5%, its first frame 39.5%, a random frame 29.8%;
  adding a strict rule before the choice lowers it (37.1% at level 1).
  Within the per-folder spread (30-58% for the best per-burst variant),
  none of them differ.
- **Frames without a full face** reach 37.5% at best (the burst's
  sharpest, `rel >= 1`, 504 frames, base 23.0%); an absolute floor lowers
  it.
- **The marked non-picks are mostly fine frames.** A hand check of 12
  marked non-picks of the alternative rule (`rel >= 1`, absolute >= 200,
  `eye_focus` >= 0.9, eyes open >= 0.98; seed 0, 768 px crops at the AF
  point, the agent's own look as in Step 2) found 8 OK (sharp, eyes open,
  face visible), 4 unsure (the AF point on a jersey, a shoulder or an arm
  with the face outside the crop, or a noisy profile) and none failed. So
  the rule does pick technically OK frames (an estimated 80% or more of
  what it marks, 42% picks plus most of the rest), but the user chose
  among such frames by composition, expression and moment, which none of
  the features see. That is why precision against the picks stops at about
  40%: the marked frames cannot be a subset of the picks.

## Failure check (the first framing)

### Method

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

### Reading

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
- **Rough precision.** Over the frozen variant of [frozen-fail-check.json](frozen-fail-check.json) (`rel >= 0.0134`,
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

### Burst level

Sidecar-labeled ARW block, gap 1000 ms: 5026 bursts of two or more frames (1812 kept, 3970 with a faced frame) and 1943 single frames (242 picks, 1435 faced). Per-folder precision is over the folders with at least 10 marked.

#### Bursts of two or more frames

5026 bursts, 1812 positive (base 36.1%).

##### Bursts: fixed rules

| Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: |
| any faced frame | 40.7% | 1.13 | 3970 (79.0%) | 1617 (89.2%) | 43.2% (31.1%-63.6%), 24 |
| sharpest frame faced (`rel >= 1`) | 40.7% | 1.13 | 3209 (63.8%) | 1306 (72.1%) | 42.6% (31.0%-63.6%), 24 |
| sharpest + abs 200, ef 0.9, eo 0.98 | 47.5% | 1.32 | 1046 (20.8%) | 497 (27.4%) | 47.4% (35.7%-64.7%), 20 |
| sharpest + abs 400, ef 0.95, eo 0.995 | 55.3% | 1.53 | 152 (3.0%) | 84 (4.6%) | 50.6% (36.4%-75.0%), 6 |
| sharpest + abs 600, ef 0.97, eo 0.999 | 53.8% | 1.49 | 13 (0.3%) | 7 (0.4%) | - |
| sharpest + abs 800, ef 0.97, eo 0.999 | 50.0% | 1.39 | 6 (0.1%) | 3 (0.2%) | - |
| abs 400, ef 0.95, eo 0.995 | 61.9% | 1.72 | 299 (5.9%) | 185 (10.2%) | 62.0% (45.5%-91.7%), 14 |
| abs 800, ef 0.97, eo 0.999 | 50.0% | 1.39 | 10 (0.2%) | 5 (0.3%) | - |
| ef 0.97, eo 0.999 | 53.1% | 1.47 | 98 (1.9%) | 52 (2.9%) | 52.5% (46.7%-58.3%), 2 |
| ef 0.97, eo 0.999, pose 15-10 | 53.8% | 1.49 | 13 (0.3%) | 7 (0.4%) | - |
| level 1 everywhere | 52.8% | 1.46 | 784 (15.6%) | 414 (22.8%) | 52.7% (44.1%-75.8%), 16 |
| level 2 everywhere | 59.4% | 1.65 | 32 (0.6%) | 19 (1.0%) | - |

##### Bursts: best grid rule at a coverage floor (in-sample)

| Coverage floor | Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| >= 30% | ef >= 0.95 | 53.0% | 1.47 | 1521 (30.3%) | 806 (44.5%) | 54.2% (29.8%-66.3%), 22 |
| >= 20% | ef >= 0.95, eo >= 0.98 | 56.0% | 1.55 | 1009 (20.1%) | 565 (31.2%) | 57.5% (30.0%-70.0%), 21 |
| >= 10% | abs >= 200, ef >= 0.95, eo >= 0.995 | 60.0% | 1.66 | 552 (11.0%) | 331 (18.3%) | 58.7% (40.0%-76.2%), 20 |
| >= 5% | abs >= 400, ef >= 0.95, eo >= 0.995 | 61.9% | 1.72 | 299 (5.9%) | 185 (10.2%) | 62.0% (45.5%-91.7%), 14 |
| >= 2% | rel >= 0.7, abs >= 200, ef >= 0.97, \|yaw\| <= 30, \|pitch\| <= 20 | 66.7% | 1.85 | 108 (2.1%) | 72 (4.0%) | 70.6% (70.0%-75.0%), 3 |
| >= 1% | abs >= 400, ef >= 0.9, eo >= 0.98, \|yaw\| <= 8, \|pitch\| <= 5 | 73.1% | 2.03 | 52 (1.0%) | 38 (2.1%) | 50.0% (50.0%-50.0%), 1 |

##### Bursts: held out, the rule chosen on the other folders by a target precision

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 0 | ef >= 0.9, eo >= 0.98 (26 folds) | 50.3% | 1.40 | 2388 (47.5%) | 1202 (66.3%) | 50.2% (37.8%-70.8%), 22 |
| target 60% | 0 | abs >= 200, ef >= 0.95, eo >= 0.995 (12 folds) | 55.8% | 1.55 | 453 (9.0%) | 253 (14.0%) | 55.7% (40.0%-91.7%), 19 |
| target 70% | 0 | rel >= 0.9, ef >= 0.97, \|yaw\| <= 15, \|pitch\| <= 10 (10 folds) | 49.0% | 1.36 | 51 (1.0%) | 25 (1.4%) | 46.2% (46.2%-46.2%), 1 |
| target 75% | 18 | rel >= 0.9, abs >= 200, ef >= 0.97, \|yaw\| <= 15, \|pitch\| <= 10 (4 folds) | 48.6% | 1.35 | 37 (0.7%) | 18 (1.0%) | 50.0% (50.0%-50.0%), 1 |
| target 80% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 85% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 95% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |

##### Bursts: by size alone (no technical feature)

| Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: |
| size >= 2 | 36.1% | 1.00 | 5026 (100.0%) | 1812 (100.0%) | 40.9% (24.9%-63.6%), 25 |
| size >= 3 | 42.6% | 1.18 | 3635 (72.3%) | 1547 (85.4%) | 45.2% (32.7%-64.7%), 21 |
| size >= 4 | 52.9% | 1.47 | 2420 (48.1%) | 1280 (70.6%) | 52.3% (37.5%-70.0%), 20 |
| size >= 5 | 56.4% | 1.56 | 2030 (40.4%) | 1144 (63.1%) | 57.1% (49.2%-73.7%), 17 |
| size >= 6 | 60.7% | 1.68 | 1675 (33.3%) | 1016 (56.1%) | 60.5% (54.3%-75.7%), 16 |
| size >= 8 | 66.5% | 1.85 | 1210 (24.1%) | 805 (44.4%) | 64.7% (60.0%-85.2%), 15 |
| size >= 10 | 69.2% | 1.92 | 930 (18.5%) | 644 (35.5%) | 68.9% (61.1%-93.8%), 14 |
| size >= 15 | 79.3% | 2.20 | 482 (9.6%) | 382 (21.1%) | 79.5% (73.0%-89.2%), 11 |
| size >= 20 | 82.3% | 2.28 | 237 (4.7%) | 195 (10.8%) | 85.0% (72.2%-94.7%), 9 |

##### Bursts: size and a rule together

| Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: |
| size >= 4, no rule | 52.9% | 1.47 | 2420 (48.1%) | 1280 (70.6%) | 52.3% (37.5%-70.0%), 20 |
| size >= 4, any faced frame | 55.6% | 1.54 | 2133 (42.4%) | 1187 (65.5%) | 54.8% (33.3%-70.7%), 19 |
| size >= 4, ef 0.9, eo 0.98 | 62.3% | 1.73 | 1573 (31.3%) | 980 (54.1%) | 62.3% (35.7%-78.9%), 18 |
| size >= 4, abs 200, ef 0.95, eo 0.995 | 69.3% | 1.92 | 407 (8.1%) | 282 (15.6%) | 70.6% (59.5%-92.3%), 15 |
| size >= 4, abs 400, ef 0.95, eo 0.995 | 71.5% | 1.98 | 228 (4.5%) | 163 (9.0%) | 72.2% (52.9%-93.8%), 9 |
| size >= 8, no rule | 66.5% | 1.85 | 1210 (24.1%) | 805 (44.4%) | 64.7% (60.0%-85.2%), 15 |
| size >= 8, any faced frame | 67.3% | 1.87 | 1123 (22.3%) | 756 (41.7%) | 68.6% (60.7%-85.2%), 14 |
| size >= 8, ef 0.9, eo 0.98 | 70.8% | 1.96 | 939 (18.7%) | 665 (36.7%) | 74.1% (62.9%-82.6%), 14 |
| size >= 8, abs 200, ef 0.95, eo 0.995 | 76.2% | 2.11 | 273 (5.4%) | 208 (11.5%) | 76.4% (63.6%-100.0%), 12 |
| size >= 8, abs 400, ef 0.95, eo 0.995 | 78.0% | 2.16 | 159 (3.2%) | 124 (6.8%) | 80.0% (58.3%-100.0%), 7 |
| size >= 15, no rule | 79.3% | 2.20 | 482 (9.6%) | 382 (21.1%) | 79.5% (73.0%-89.2%), 11 |
| size >= 15, any faced frame | 79.1% | 2.19 | 473 (9.4%) | 374 (20.6%) | 78.9% (72.5%-89.2%), 11 |
| size >= 15, ef 0.9, eo 0.98 | 80.4% | 2.23 | 433 (8.6%) | 348 (19.2%) | 81.6% (72.3%-88.6%), 11 |
| size >= 15, abs 200, ef 0.95, eo 0.995 | 86.6% | 2.40 | 134 (2.7%) | 116 (6.4%) | 88.5% (61.5%-93.3%), 7 |
| size >= 15, abs 400, ef 0.95, eo 0.995 | 87.1% | 2.42 | 70 (1.4%) | 61 (3.4%) | 92.3% (92.3%-92.3%), 1 |

##### Bursts: held out, a size cut alone chosen by a target precision

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 0 | size >= 4 (27 folds) | 52.9% | 1.47 | 2420 (48.1%) | 1280 (70.6%) | 52.3% (37.5%-70.0%), 20 |
| target 60% | 0 | size >= 6 (26 folds) | 60.6% | 1.68 | 1654 (32.9%) | 1002 (55.3%) | 60.5% (54.3%-79.2%), 16 |
| target 70% | 0 | size >= 15 (26 folds) | 75.4% | 2.09 | 549 (10.9%) | 414 (22.8%) | 79.5% (63.5%-89.2%), 11 |
| target 75% | 0 | size >= 15 (27 folds) | 79.3% | 2.20 | 482 (9.6%) | 382 (21.1%) | 79.5% (73.0%-89.2%), 11 |
| target 80% | 0 | size >= 20 (26 folds) | 80.1% | 2.22 | 287 (5.7%) | 230 (12.7%) | 85.0% (72.2%-94.7%), 9 |
| target 85% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 95% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |

##### Bursts: held out, a size cut and a grid rule chosen together by a target precision

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 0 | size >= 3, ef >= 0.9 (27 folds) | 51.5% | 1.43 | 2575 (51.2%) | 1327 (73.2%) | 51.9% (37.5%-69.2%), 21 |
| target 60% | 0 | size >= 6 (21 folds) | 59.9% | 1.66 | 1691 (33.6%) | 1013 (55.9%) | 62.4% (51.5%-75.3%), 16 |
| target 70% | 0 | size >= 8, abs >= 200, ef >= 0.9 (19 folds) | 69.8% | 1.93 | 962 (19.1%) | 671 (37.0%) | 71.1% (61.8%-82.6%), 14 |
| target 75% | 0 | size >= 10, rel >= 0.5, abs >= 400, ef >= 0.9 (18 folds) | 70.2% | 1.95 | 531 (10.6%) | 373 (20.6%) | 72.4% (63.8%-88.2%), 11 |
| target 80% | 0 | size >= 15, ef >= 0.9 (24 folds) | 80.7% | 2.24 | 450 (9.0%) | 363 (20.0%) | 81.2% (73.0%-87.9%), 11 |
| target 85% | 0 | size >= 15, abs >= 400, ef >= 0.9, eo >= 0.995 (14 folds) | 76.2% | 2.11 | 231 (4.6%) | 176 (9.7%) | 80.0% (55.6%-90.9%), 9 |
| target 90% | 0 | size >= 20, rel >= 0.5, abs >= 400, ef >= 0.9, eo >= 0.995 (17 folds) | 81.7% | 2.27 | 104 (2.1%) | 85 (4.7%) | 75.7% (66.7%-85.0%), 3 |
| target 95% | 0 | size >= 20, rel >= 0.5, abs >= 600, ef >= 0.9, eo >= 0.995 (23 folds) | 81.5% | 2.26 | 54 (1.1%) | 44 (2.4%) | 74.2% (66.7%-81.8%), 2 |

#### Single frames

1943 single frames, 242 positive (base 12.5%).

##### Single frames: fixed rules

| Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: |
| any faced frame | 14.1% | 1.14 | 1435 (73.9%) | 203 (83.9%) | 14.3% (2.6%-29.2%), 19 |
| sharpest frame faced (`rel >= 1`) | 14.1% | 1.14 | 1435 (73.9%) | 203 (83.9%) | 14.3% (2.6%-29.2%), 19 |
| sharpest + abs 200, ef 0.9, eo 0.98 | 14.3% | 1.15 | 336 (17.3%) | 48 (19.8%) | 9.5% (4.2%-47.4%), 10 |
| sharpest + abs 400, ef 0.95, eo 0.995 | 19.0% | 1.53 | 63 (3.2%) | 12 (5.0%) | 9.5% (9.5%-9.5%), 1 |
| sharpest + abs 600, ef 0.97, eo 0.999 | 20.0% | 1.61 | 10 (0.5%) | 2 (0.8%) | - |
| sharpest + abs 800, ef 0.97, eo 0.999 | 16.7% | 1.34 | 6 (0.3%) | 1 (0.4%) | - |
| abs 400, ef 0.95, eo 0.995 | 19.0% | 1.53 | 63 (3.2%) | 12 (5.0%) | 9.5% (9.5%-9.5%), 1 |
| abs 800, ef 0.97, eo 0.999 | 16.7% | 1.34 | 6 (0.3%) | 1 (0.4%) | - |
| ef 0.97, eo 0.999 | 22.2% | 1.78 | 36 (1.9%) | 8 (3.3%) | 25.0% (25.0%-25.0%), 1 |
| ef 0.97, eo 0.999, pose 15-10 | 0.0% | 0.00 | 2 (0.1%) | 0 (0.0%) | - |
| level 1 everywhere | 13.5% | 1.09 | 133 (6.8%) | 18 (7.4%) | 25.0% (2.7%-27.3%), 3 |
| level 2 everywhere | 22.2% | 1.78 | 9 (0.5%) | 2 (0.8%) | - |

##### Single frames: best grid rule at a coverage floor (in-sample)

| Coverage floor | Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| >= 30% | ef >= 0.9, eo >= 0.98 | 16.3% | 1.31 | 655 (33.7%) | 107 (44.2%) | 15.9% (0.0%-37.5%), 14 |
| >= 20% | ef >= 0.9, eo >= 0.995 | 16.9% | 1.36 | 437 (22.5%) | 74 (30.6%) | 18.5% (5.9%-40.0%), 10 |
| >= 10% | ef >= 0.95, eo >= 0.995 | 18.5% | 1.48 | 238 (12.2%) | 44 (18.2%) | 20.0% (0.0%-35.0%), 7 |
| >= 5% | ef >= 0.95, eo >= 0.999 | 19.0% | 1.53 | 121 (6.2%) | 23 (9.5%) | 17.9% (3.8%-22.7%), 3 |
| >= 2% | abs >= 200, ef >= 0.97, eo >= 0.995 | 24.4% | 1.96 | 45 (2.3%) | 11 (4.5%) | 50.0% (50.0%-50.0%), 1 |
| >= 1% | abs >= 600, ef >= 0.95, eo >= 0.995 | 32.0% | 2.57 | 25 (1.3%) | 8 (3.3%) | - |

##### Single frames: held out, the rule chosen on the other folders by a target precision

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 60% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 70% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 75% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 80% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 85% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 95% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |

#### Without a faced frame (sharpness only)

1056 bursts of two or more frames have no faced frame (195 kept, base 18.5%); 508 single frames are not faced (39 picks, base 7.7%).

| Burst rule | Marked (of face-free bursts) | Precision (kept) |
| --- | ---: | ---: |
| sharpest frame abs >= 0 | 1056 (100.0%) | 18.5% |
| sharpest frame abs >= 200 | 954 (90.3%) | 18.9% |
| sharpest frame abs >= 400 | 722 (68.4%) | 19.9% |
| sharpest frame abs >= 600 | 479 (45.4%) | 23.0% |
| sharpest frame abs >= 800 | 305 (28.9%) | 24.3% |

| Single-frame rule | Marked (of face-free singles) | Precision (picked) |
| --- | ---: | ---: |
| abs >= 0 | 508 (100.0%) | 7.7% |
| abs >= 200 | 396 (78.0%) | 8.1% |
| abs >= 400 | 271 (53.3%) | 6.6% |
| abs >= 600 | 189 (37.2%) | 7.4% |
| abs >= 800 | 125 (24.6%) | 7.2% |

#### Without sharpness cuts (`eye_focus`, eyes open and pose only)

The same held-out selections with the grid limited to the rules without a relative or absolute sharpness cut (`eye_focus` already measures sharpness at the eyes): 64 of the 1600 rules.

##### Bursts, no sharpness

5026 bursts, 1812 positive (base 36.1%).

##### Bursts, no sharpness: best grid rule at a coverage floor (in-sample)

| Coverage floor | Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| >= 30% | ef >= 0.95 | 53.0% | 1.47 | 1521 (30.3%) | 806 (44.5%) | 54.2% (29.8%-66.3%), 22 |
| >= 20% | ef >= 0.95, eo >= 0.98 | 56.0% | 1.55 | 1009 (20.1%) | 565 (31.2%) | 57.5% (30.0%-70.0%), 21 |
| >= 10% | ef >= 0.95, eo >= 0.995 | 57.2% | 1.59 | 691 (13.7%) | 395 (21.8%) | 58.3% (40.0%-77.3%), 21 |
| >= 5% | ef >= 0.97, eo >= 0.98 | 59.1% | 1.64 | 279 (5.6%) | 165 (9.1%) | 57.9% (41.7%-75.0%), 13 |
| >= 2% | ef >= 0.97, eo >= 0.98, \|yaw\| <= 30, \|pitch\| <= 20 | 63.5% | 1.76 | 126 (2.5%) | 80 (4.4%) | 62.7% (50.0%-70.6%), 4 |
| >= 1% | ef >= 0.97, eo >= 0.98, \|yaw\| <= 15, \|pitch\| <= 10 | 71.2% | 1.97 | 52 (1.0%) | 37 (2.0%) | 60.0% (60.0%-60.0%), 1 |

##### Bursts, no sharpness: held out, the rule chosen on the other folders by a target precision

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 0 | ef >= 0.9, eo >= 0.98 (26 folds) | 50.3% | 1.40 | 2388 (47.5%) | 1202 (66.3%) | 50.2% (37.8%-70.8%), 22 |
| target 60% | 0 | ef >= 0.97, \|yaw\| <= 30, \|pitch\| <= 20 (24 folds) | 57.5% | 1.59 | 181 (3.6%) | 104 (5.7%) | 54.5% (45.0%-71.4%), 7 |
| target 70% | 1 | ef >= 0.97, eo >= 0.98, \|yaw\| <= 15, \|pitch\| <= 10 (24 folds) | 65.2% | 1.81 | 46 (0.9%) | 30 (1.7%) | 60.0% (60.0%-60.0%), 1 |
| target 75% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 80% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 85% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 95% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |

##### Bursts, no sharpness: held out, a size cut and a grid rule chosen together

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 0 | size >= 3, ef >= 0.9 (27 folds) | 51.5% | 1.43 | 2575 (51.2%) | 1327 (73.2%) | 51.9% (37.5%-69.2%), 21 |
| target 60% | 0 | size >= 6 (23 folds) | 61.0% | 1.69 | 1673 (33.3%) | 1020 (56.3%) | 62.8% (51.5%-75.3%), 16 |
| target 70% | 0 | size >= 8, ef >= 0.9, eo >= 0.98 (25 folds) | 70.3% | 1.95 | 966 (19.2%) | 679 (37.5%) | 74.1% (61.8%-82.6%), 14 |
| target 75% | 0 | size >= 15 (24 folds) | 74.8% | 2.07 | 511 (10.2%) | 382 (21.1%) | 77.4% (64.6%-89.2%), 11 |
| target 80% | 0 | size >= 15, ef >= 0.9 (24 folds) | 80.4% | 2.23 | 449 (8.9%) | 361 (19.9%) | 81.2% (73.0%-87.5%), 11 |
| target 85% | 0 | size >= 20, ef >= 0.9, eo >= 0.999 (25 folds) | 82.2% | 2.28 | 191 (3.8%) | 157 (8.7%) | 89.3% (60.0%-100.0%), 9 |
| target 90% | 0 | size >= 20, ef >= 0.95, eo >= 0.995 (26 folds) | 87.4% | 2.42 | 95 (1.9%) | 83 (4.6%) | 90.0% (75.7%-91.7%), 3 |
| target 95% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |

##### Single frames, no sharpness

1943 single frames, 242 positive (base 12.5%).

##### Single frames, no sharpness: best grid rule at a coverage floor (in-sample)

| Coverage floor | Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| >= 30% | ef >= 0.9, eo >= 0.98 | 16.3% | 1.31 | 655 (33.7%) | 107 (44.2%) | 15.9% (0.0%-37.5%), 14 |
| >= 20% | ef >= 0.9, eo >= 0.995 | 16.9% | 1.36 | 437 (22.5%) | 74 (30.6%) | 18.5% (5.9%-40.0%), 10 |
| >= 10% | ef >= 0.95, eo >= 0.995 | 18.5% | 1.48 | 238 (12.2%) | 44 (18.2%) | 20.0% (0.0%-35.0%), 7 |
| >= 5% | ef >= 0.95, eo >= 0.999 | 19.0% | 1.53 | 121 (6.2%) | 23 (9.5%) | 17.9% (3.8%-22.7%), 3 |
| >= 2% | ef >= 0.97, eo >= 0.995 | 23.0% | 1.84 | 74 (3.8%) | 17 (7.0%) | 34.5% (23.5%-45.5%), 2 |
| >= 1% | ef >= 0.97, eo >= 0.995, \|yaw\| <= 30, \|pitch\| <= 20 | 25.0% | 2.01 | 24 (1.2%) | 6 (2.5%) | - |

##### Single frames, no sharpness: held out, the rule chosen on the other folders by a target precision

| Training target | Folds without a rule | Most chosen rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 50% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 60% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 70% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 75% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 80% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 85% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |
| target 95% | 27 | - | - | - | 0 (0.0%) | 0 (0.0%) | - |

##### Dropping the sharpness cuts from the rules chosen with sharpness

The most chosen held-out rule per target of the with-sharpness runs, scored as a fixed rule on every folder, then the same rule with its `rel` and absolute cuts removed.

| Chosen for | Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| bursts, target 60% | abs >= 200, ef >= 0.95, eo >= 0.995 | 60.0% | 1.66 | 552 (11.0%) | 331 (18.3%) | 58.7% (40.0%-76.2%), 20 |
|  | without sharpness: ef >= 0.95, eo >= 0.995 | 57.2% | 1.59 | 691 (13.7%) | 395 (21.8%) | 58.3% (40.0%-77.3%), 21 |
| bursts, target 70% | rel >= 0.9, ef >= 0.97, \|yaw\| <= 15, \|pitch\| <= 10 | 70.4% | 1.95 | 54 (1.1%) | 38 (2.1%) | 69.2% (69.2%-69.2%), 1 |
|  | without sharpness: ef >= 0.97, \|yaw\| <= 15, \|pitch\| <= 10 | 68.4% | 1.90 | 79 (1.6%) | 54 (3.0%) | 70.7% (68.8%-72.7%), 2 |
| bursts, target 75% | rel >= 0.9, abs >= 200, ef >= 0.97, \|yaw\| <= 15, \|pitch\| <= 10 | 74.3% | 2.06 | 35 (0.7%) | 26 (1.4%) | 80.0% (80.0%-80.0%), 1 |
|  | without sharpness: ef >= 0.97, \|yaw\| <= 15, \|pitch\| <= 10 | 68.4% | 1.90 | 79 (1.6%) | 54 (3.0%) | 70.7% (68.8%-72.7%), 2 |
| size + rule, target 70% | size >= 8, abs >= 200, ef >= 0.9 | 70.3% | 1.95 | 962 (19.1%) | 676 (37.3%) | 71.1% (61.4%-84.0%), 14 |
|  | size >= 8, without sharpness: ef >= 0.9 | 69.4% | 1.93 | 1056 (21.0%) | 733 (40.5%) | 71.7% (61.2%-84.6%), 14 |
| size + rule, target 75% | size >= 10, rel >= 0.5, abs >= 400, ef >= 0.9 | 75.6% | 2.10 | 487 (9.7%) | 368 (20.3%) | 78.4% (58.6%-100.0%), 13 |
|  | size >= 10, without sharpness: ef >= 0.9 | 71.8% | 1.99 | 832 (16.6%) | 597 (32.9%) | 73.6% (61.9%-93.3%), 14 |
| size + rule, target 85% | size >= 15, abs >= 400, ef >= 0.9, eo >= 0.995 | 85.1% | 2.36 | 188 (3.7%) | 160 (8.8%) | 87.1% (61.5%-94.7%), 8 |
|  | size >= 15, without sharpness: ef >= 0.9, eo >= 0.995 | 81.7% | 2.27 | 383 (7.6%) | 313 (17.3%) | 86.0% (72.5%-90.9%), 11 |
| size + rule, target 90% | size >= 20, rel >= 0.5, abs >= 400, ef >= 0.9, eo >= 0.995 | 90.1% | 2.50 | 91 (1.8%) | 82 (4.5%) | 88.2% (80.0%-100.0%), 3 |
|  | size >= 20, without sharpness: ef >= 0.9, eo >= 0.995 | 83.7% | 2.32 | 202 (4.0%) | 169 (9.3%) | 84.8% (68.8%-100.0%), 9 |
| size + rule, target 95% | size >= 20, rel >= 0.5, abs >= 600, ef >= 0.9, eo >= 0.995 | 95.6% | 2.65 | 45 (0.9%) | 43 (2.4%) | 90.0% (90.0%-90.0%), 1 |
|  | size >= 20, without sharpness: ef >= 0.9, eo >= 0.995 | 83.7% | 2.32 | 202 (4.0%) | 169 (9.3%) | 84.8% (68.8%-100.0%), 9 |

#### Which eye feature carries the burst-level rules

Held out as above, over the no-sharpness grid without the pose, limited to rules on `eye_focus` only, eyes open only, or both; "alone" lets the selection use no size cut, "+ size" chooses a size cut and a rule together. Each cell is the held-out precision and the share of the bursts marked; "none" means no rule reaches the target on any training set.

| Target | `eye_focus` only, alone | eyes open only, alone | both, alone | size alone | `eye_focus` only + size | eyes open only + size | both + size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 60% | none | none | 45.9%, 0.7% | 60.6%, 32.9% | 61.0%, 33.3% | 60.7%, 33.3% | 61.0%, 33.3% |
| 70% | none | none | none | 75.4%, 10.9% | 70.2%, 17.7% | 69.6%, 17.1% | 70.3%, 19.2% |
| 75% | none | none | none | 79.3%, 9.6% | 79.2%, 8.9% | 79.3%, 9.6% | 74.8%, 10.2% |
| 80% | none | none | none | 80.1%, 5.7% | 79.7%, 8.4% | 80.1%, 7.0% | 80.4%, 8.9% |
| 85% | none | none | none | none | 79.3%, 2.3% | 75.4%, 1.1% | 82.2%, 3.8% |
| 90% | none | none | none | none | none | none | 87.4%, 1.9% |
| 95% | none | none | none | none | none | none | none |

Most chosen rule of the "+ size" runs:

| Target | `eye_focus` only + size | eyes open only + size | both + size |
| --- | ---: | ---: | ---: |
| 60% | size >= 6 (23 folds) | size >= 6 (26 folds) | size >= 6 (23 folds) |
| 70% | size >= 10, ef >= 0.9 (22 folds) | size >= 10, eo >= 0.98 (22 folds) | size >= 8, ef >= 0.9, eo >= 0.98 (25 folds) |
| 75% | size >= 15 (26 folds) | size >= 15 (27 folds) | size >= 15 (24 folds) |
| 80% | size >= 15, ef >= 0.9 (24 folds) | size >= 15, eo >= 0.999 (23 folds) | size >= 15, ef >= 0.9 (24 folds) |
| 85% | size >= 15, ef >= 0.97 (24 folds) | size >= 20, eo >= 0.995 (1 folds) | size >= 20, ef >= 0.9, eo >= 0.999 (25 folds) |
| 90% | - | - | size >= 20, ef >= 0.95, eo >= 0.995 (26 folds) |
| 95% | - | - | - |

##### The 87.4% rule with one eye feature dropped

Fixed rules (no fitting), so the pooled numbers over every folder are also what each folder gets when held out; the per-folder column is the spread.

| Rule | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: |
| size >= 20, `eye_focus` >= 0.95, eyes open >= 0.995 | 92.3% | 2.56 | 78 (1.6%) | 72 (4.0%) | 90.0% (85.0%-91.7%), 3 |
| eyes open dropped: size >= 20, `eye_focus` >= 0.95 | 84.2% | 2.34 | 146 (2.9%) | 123 (6.8%) | 84.6% (72.2%-91.7%), 7 |
| `eye_focus` dropped: size >= 20, eyes open >= 0.995 | 82.9% | 2.30 | 222 (4.4%) | 184 (10.2%) | 85.0% (70.6%-100.0%), 9 |
| both dropped: size >= 20 | 82.3% | 2.28 | 237 (4.7%) | 195 (10.8%) | 85.0% (72.2%-94.7%), 9 |
| size >= 15, `eye_focus` >= 0.95, eyes open >= 0.995 | 87.2% | 2.42 | 148 (2.9%) | 129 (7.1%) | 85.7% (61.5%-93.3%), 7 |
| size >= 15, `eye_focus` >= 0.95 | 81.9% | 2.27 | 265 (5.3%) | 217 (12.0%) | 81.5% (66.7%-93.8%), 9 |
| size >= 15, eyes open >= 0.995 | 79.5% | 2.21 | 439 (8.7%) | 349 (19.3%) | 81.0% (72.8%-88.5%), 11 |

### Keep mark

Sidecar-labeled ARW block, gap 1000 ms: 16522 frames in 1802 scorable bursts, 4133 picks (base 25.0%); faced frames (cue face, eyes judgment and pose) 11693, of them picks 3020 (base 25.8%). Per-folder precision is over the folders with at least 10 marked frames.

#### One condition at a time, toward the strictest cut

| Rule | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) | Bursts with a mark | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| faced, no cut | 25.8% | 1.00 | 1.03 | 11693 (100.0%) | 3020 (100.0%) | 1608 (89.2%) | 25.5% (17.1%-39.3%), 25 |
| rel >= 0.5 | 29.7% | 1.15 | 1.14 | 5853 (50.1%) | 1738 (57.5%) | 1529 (84.9%) | 29.1% (16.4%-43.0%), 25 |
| rel >= 0.7 | 31.9% | 1.24 | 1.19 | 3734 (31.9%) | 1192 (39.5%) | 1464 (81.2%) | 30.6% (16.7%-47.1%), 24 |
| rel >= 0.9 | 37.0% | 1.43 | 1.28 | 1972 (16.9%) | 729 (24.1%) | 1356 (75.2%) | 35.3% (23.5%-51.0%), 23 |
| rel >= 1 | 41.6% | 1.61 | 1.37 | 1298 (11.1%) | 540 (17.9%) | 1298 (72.0%) | 40.1% (24.3%-57.1%), 22 |
| abs >= 200 | 27.9% | 1.08 | 1.12 | 6984 (59.7%) | 1947 (64.5%) | 1353 (75.1%) | 26.7% (0.0%-44.5%), 22 |
| abs >= 400 | 30.4% | 1.18 | 1.23 | 2549 (21.8%) | 775 (25.7%) | 825 (45.8%) | 29.7% (17.3%-55.8%), 19 |
| abs >= 600 | 33.6% | 1.30 | 1.37 | 853 (7.3%) | 287 (9.5%) | 419 (23.3%) | 29.3% (9.4%-56.5%), 15 |
| abs >= 800 | 36.8% | 1.42 | 1.53 | 359 (3.1%) | 132 (4.4%) | 212 (11.8%) | 36.2% (13.6%-55.0%), 12 |
| ef >= 0.9 | 28.3% | 1.10 | 1.11 | 7910 (67.6%) | 2239 (74.1%) | 1499 (83.2%) | 27.3% (18.6%-39.9%), 24 |
| ef >= 0.95 | 28.9% | 1.12 | 1.11 | 2440 (20.9%) | 705 (23.3%) | 804 (44.6%) | 27.7% (14.3%-42.3%), 24 |
| ef >= 0.97 | 30.1% | 1.16 | 1.16 | 602 (5.1%) | 181 (6.0%) | 241 (13.4%) | 26.1% (16.7%-58.3%), 17 |
| eo >= 0.98 | 28.2% | 1.09 | 1.12 | 6760 (57.8%) | 1908 (63.2%) | 1352 (75.0%) | 28.0% (15.2%-42.9%), 24 |
| eo >= 0.995 | 28.3% | 1.09 | 1.10 | 4392 (37.6%) | 1241 (41.1%) | 1123 (62.3%) | 29.2% (14.3%-45.5%), 23 |
| eo >= 0.999 | 27.6% | 1.07 | 1.07 | 2026 (17.3%) | 560 (18.5%) | 703 (39.0%) | 28.1% (13.6%-41.2%), 20 |
| \|yaw\| <= 30, \|pitch\| <= 20 | 24.9% | 0.96 | 1.03 | 4186 (35.8%) | 1043 (34.5%) | 1052 (58.4%) | 25.9% (14.7%-42.9%), 25 |
| \|yaw\| <= 15, \|pitch\| <= 10 | 22.1% | 0.86 | 0.96 | 1347 (11.5%) | 298 (9.9%) | 540 (30.0%) | 21.1% (11.9%-61.5%), 21 |
| \|yaw\| <= 8, \|pitch\| <= 5 | 23.9% | 0.92 | 0.99 | 436 (3.7%) | 104 (3.4%) | 248 (13.8%) | 24.3% (5.3%-35.7%), 16 |

#### All four conditions at once, by strictness

Levels: rel 0 / 0.5 / 0.7 / 0.9 / 1.0 (the burst's sharpest), abs 0-800, ef 0 / 0.9 / 0.95 / 0.97, eo 0 / 0.98 / 0.995 / 0.999, pose none / 30-20 / 15-10 / 8-5 deg.

| Rule | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) | Bursts with a mark | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| level 1, rel | 31.0% | 1.20 | 1.24 | 1091 (9.3%) | 338 (11.2%) | 498 (27.6%) | 28.8% (11.9%-46.2%), 22 |
| level 1, rel + abs | 31.2% | 1.21 | 1.27 | 862 (7.4%) | 269 (8.9%) | 414 (23.0%) | 29.0% (0.0%-45.5%), 19 |
| level 1, abs | 30.1% | 1.17 | 1.21 | 1202 (10.3%) | 362 (12.0%) | 508 (28.2%) | 27.6% (0.0%-45.5%), 20 |
| level 2, rel | 29.7% | 1.15 | 1.24 | 64 (0.5%) | 19 (0.6%) | 37 (2.1%) | 15.4% (0.0%-30.8%), 2 |
| level 2, rel + abs | 44.0% | 1.70 | 1.58 | 25 (0.2%) | 11 (0.4%) | 19 (1.1%) | - |
| level 2, abs | 35.9% | 1.39 | 1.36 | 39 (0.3%) | 14 (0.5%) | 23 (1.3%) | - |
| level 3, rel | 0.0% | 0.00 | 0.00 | 1 (0.0%) | 0 (0.0%) | 1 (0.1%) | - |
| level 3, rel + abs | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| level 3, abs | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| level 3, rel = burst max + abs 800 | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| burst's sharpest + level 1 (abs 200, ef 0.9, eo 0.98, pose 30-20) | 39.2% | 1.52 | 1.39 | 194 (1.7%) | 76 (2.5%) | 194 (10.8%) | 41.2% (15.4%-60.0%), 7 |
| burst's sharpest + abs 200, ef 0.9, eo 0.98, no pose | 42.1% | 1.63 | 1.42 | 497 (4.3%) | 209 (6.9%) | 497 (27.6%) | 42.9% (26.7%-59.5%), 17 |
| burst's sharpest + abs 400, ef 0.95, eo 0.995, no pose | 35.7% | 1.38 | 1.32 | 84 (0.7%) | 30 (1.0%) | 84 (4.7%) | - |
| ef + eo strictest, no sharpness, no pose | 26.9% | 1.04 | 1.08 | 78 (0.7%) | 21 (0.7%) | 52 (2.9%) | 20.9% (18.8%-23.1%), 2 |
| ef + eo + pose strictest | 0.0% | 0.00 | 0.00 | 1 (0.0%) | 0 (0.0%) | 1 (0.1%) | - |

#### Best precision of the grid at a coverage floor (in-sample selection)

Over the 1600 grid rules on every folder; the selection makes these optimistic, the held-out table below is the honest one.

| Coverage floor | Rule | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) | Bursts with a mark | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| >= 20.0% | rel >= 0.7, ef >= 0.9 | 32.9% | 1.27 | 1.22 | 3023 (25.9%) | 995 (32.9%) | 1266 (70.3%) | 31.7% (17.0%-48.8%), 24 |
| >= 10.0% | rel >= 1 | 41.6% | 1.61 | 1.37 | 1298 (11.1%) | 540 (17.9%) | 1298 (72.0%) | 40.1% (24.3%-57.1%), 22 |
| >= 5.0% | rel >= 1, abs >= 200, ef >= 0.9 | 42.9% | 1.66 | 1.44 | 884 (7.6%) | 379 (12.5%) | 884 (49.1%) | 42.0% (32.5%-61.6%), 20 |
| >= 2.0% | rel >= 1, abs >= 400, ef >= 0.95 | 45.3% | 1.76 | 1.53 | 258 (2.2%) | 117 (3.9%) | 258 (14.3%) | 50.0% (23.1%-65.4%), 11 |
| >= 1.0% | rel >= 1, abs >= 400, ef >= 0.95 | 45.3% | 1.76 | 1.53 | 258 (2.2%) | 117 (3.9%) | 258 (14.3%) | 50.0% (23.1%-65.4%), 11 |
| >= 0.5% | rel >= 0.9, abs >= 600, ef >= 0.97 | 48.3% | 1.87 | 1.68 | 60 (0.5%) | 29 (1.0%) | 50 (2.8%) | 40.0% (40.0%-40.0%), 1 |
| >= 0.2% | rel >= 0.9, abs >= 800, ef >= 0.97 | 56.0% | 2.17 | 1.76 | 25 (0.2%) | 14 (0.5%) | 21 (1.2%) | - |

#### Held out: the rule chosen on the other folders by a target precision

Per fold, among the 1600 grid rules, the one marking the most training faced frames whose training precision reaches the target (at least 30 marked); scored on the held-out folder, pooled. "Folds without a rule" counts the folds where no rule reaches the target (their frames count as unmarked).

| Training target | Folds without a rule | Most chosen rule | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) | Bursts with a mark | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 35% | 0 | rel >= 0.9 (26 folds) | 37.0% | 1.43 | 1.28 | 1972 (16.9%) | 729 (24.1%) | 1356 (75.2%) | 35.3% (23.5%-51.0%), 23 |
| target 40% | 0 | rel >= 1 (26 folds) | 41.6% | 1.61 | 1.37 | 1298 (11.1%) | 540 (17.9%) | 1298 (72.0%) | 40.1% (24.3%-57.1%), 22 |
| target 45% | 0 | rel >= 1, abs >= 400, ef >= 0.95 (18 folds) | 32.7% | 1.26 | 1.18 | 199 (1.7%) | 65 (2.2%) | 192 (10.7%) | 33.3% (6.7%-50.0%), 8 |
| target 50% | 1 | rel >= 1, abs >= 600, ef >= 0.97 (17 folds) | 30.0% | 1.16 | 1.30 | 50 (0.4%) | 15 (0.5%) | 45 (2.5%) | 40.0% (40.0%-40.0%), 1 |
| target 60% | 26 | - | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| target 70% | 26 | - | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| target 80% | 26 | - | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 26 | - | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |

#### Held out: the pointwise logistic's top scores (learned alternative)

fit.py's (iv) on every feature, fitted on the other folders' scorable frames; the cut is set on the training faced frames.

| Cut | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) | Bursts with a mark | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| top 30.0% of training faced frames | 32.1% | 1.24 | 1.23 | 3552 (30.4%) | 1141 (37.8%) | 1317 (73.1%) | 32.9% (14.8%-50.2%), 24 |
| top 20.0% of training faced frames | 33.9% | 1.31 | 1.28 | 2382 (20.4%) | 808 (26.8%) | 1080 (59.9%) | 34.9% (15.9%-50.6%), 24 |
| top 10.0% of training faced frames | 35.2% | 1.36 | 1.29 | 1219 (10.4%) | 429 (14.2%) | 663 (36.8%) | 34.5% (15.4%-55.0%), 24 |
| top 5.0% of training faced frames | 36.8% | 1.42 | 1.37 | 644 (5.5%) | 237 (7.8%) | 391 (21.7%) | 36.9% (25.0%-54.5%), 16 |
| top 2.0% of training faced frames | 38.1% | 1.48 | 1.40 | 270 (2.3%) | 103 (3.4%) | 181 (10.0%) | 33.3% (21.7%-55.2%), 9 |
| top 1.0% of training faced frames | 34.9% | 1.35 | 1.33 | 126 (1.1%) | 44 (1.5%) | 92 (5.1%) | 60.0% (27.5%-60.0%), 3 |
| top 0.5% of training faced frames | 34.7% | 1.34 | 1.30 | 72 (0.6%) | 25 (0.8%) | 53 (2.9%) | 22.2% (22.2%-22.2%), 1 |

| Training target | Folds without a cut | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) | Bursts with a mark | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| target 35% | 0 | 33.2% | 1.28 | 1.28 | 2611 (22.3%) | 866 (28.7%) | 1132 (62.8%) | 34.7% (17.4%-52.0%), 24 |
| target 40% | 0 | 34.9% | 1.35 | 1.31 | 856 (7.3%) | 299 (9.9%) | 480 (26.6%) | 34.4% (18.2%-54.5%), 23 |
| target 45% | 0 | 38.1% | 1.47 | 1.36 | 226 (1.9%) | 86 (2.8%) | 160 (8.9%) | 32.6% (23.5%-66.7%), 8 |
| target 50% | 1 | 28.3% | 1.10 | 1.02 | 106 (0.9%) | 30 (1.0%) | 74 (4.1%) | 22.6% (16.7%-30.8%), 4 |
| target 60% | 3 | 38.7% | 1.50 | 1.35 | 31 (0.3%) | 12 (0.4%) | 26 (1.4%) | - |
| target 70% | 17 | 0.0% | 0.00 | 0.00 | 5 (0.0%) | 0 (0.0%) | 4 (0.2%) | - |
| target 80% | 26 | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |
| target 90% | 26 | - | - | - | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | - |

#### One mark per burst

Among a burst's faced frames passing the rule, the one with the highest held-out logistic score, or the earliest; bursts with no passing frame get no mark. Baselines over every scorable burst: its first frame, its highest-scoring frame, its sharpest frame, and a random frame (the mean pick share of a burst).

| Rule | Frame | Bursts marked (of scorable) | Precision | Per-folder precision median (min-max), folders |
| --- | ---: | ---: | ---: | ---: |
| faced, no cut | highest score | 1608 (89.2%) | 41.8% | 40.5% (30.0%-58.3%), 22 |
| faced, no cut | first passing | 1608 (89.2%) | 39.0% | 40.2% (26.3%-58.6%), 22 |
| level 1, rel | highest score | 498 (27.6%) | 37.1% | 36.8% (15.4%-58.8%), 18 |
| level 1, rel | first passing | 498 (27.6%) | 37.1% | 35.6% (23.1%-57.1%), 18 |
| level 2, rel | highest score | 37 (2.1%) | 35.1% | - |
| level 2, rel | first passing | 37 (2.1%) | 32.4% | - |
| level 2, rel + abs | highest score | 19 (1.1%) | 42.1% | - |
| level 2, rel + abs | first passing | 19 (1.1%) | 42.1% | - |
| level 3, rel | highest score | 1 (0.1%) | 0.0% | - |
| level 3, rel | first passing | 1 (0.1%) | 0.0% | - |
| ef + eo strictest | highest score | 52 (2.9%) | 30.8% | - |
| ef + eo strictest | first passing | 52 (2.9%) | 30.8% | - |
| every burst | first frame | 1802 (100.0%) | 39.5% | - |
| every burst | highest score | 1802 (100.0%) | 41.6% | - |
| every burst | sharpest (`rel` = 1) | 1802 (100.0%) | 40.5% | - |
| every burst | random frame (expected) | 1802 (100.0%) | 29.8% | - |

#### Frames without a full face (sharpness only)

4829 frames of the scorable bursts lack a cue face, an eyes judgment or a pose; 1113 of them are picks (base 23.0%).

| Rule | Frames marked | Picks marked (recall) | Precision |
| --- | ---: | ---: | ---: |
| rel >= 0, abs >= 0 | 4829 (100.0%) | 1113 (100.0%) | 23.0% |
| rel >= 0, abs >= 200 | 3259 (67.5%) | 818 (73.5%) | 25.1% |
| rel >= 0, abs >= 400 | 1756 (36.4%) | 436 (39.2%) | 24.8% |
| rel >= 0, abs >= 600 | 819 (17.0%) | 206 (18.5%) | 25.2% |
| rel >= 0, abs >= 800 | 398 (8.2%) | 99 (8.9%) | 24.9% |
| rel >= 0.5, abs >= 0 | 2384 (49.4%) | 660 (59.3%) | 27.7% |
| rel >= 0.5, abs >= 200 | 2158 (44.7%) | 597 (53.6%) | 27.7% |
| rel >= 0.5, abs >= 400 | 1421 (29.4%) | 374 (33.6%) | 26.3% |
| rel >= 0.5, abs >= 600 | 762 (15.8%) | 196 (17.6%) | 25.7% |
| rel >= 0.5, abs >= 800 | 385 (8.0%) | 95 (8.5%) | 24.7% |
| rel >= 0.7, abs >= 0 | 1506 (31.2%) | 451 (40.5%) | 29.9% |
| rel >= 0.7, abs >= 200 | 1411 (29.2%) | 415 (37.3%) | 29.4% |
| rel >= 0.7, abs >= 400 | 1036 (21.5%) | 296 (26.6%) | 28.6% |
| rel >= 0.7, abs >= 600 | 627 (13.0%) | 179 (16.1%) | 28.5% |
| rel >= 0.7, abs >= 800 | 346 (7.2%) | 86 (7.7%) | 24.9% |
| rel >= 0.9, abs >= 0 | 829 (17.2%) | 274 (24.6%) | 33.1% |
| rel >= 0.9, abs >= 200 | 784 (16.2%) | 257 (23.1%) | 32.8% |
| rel >= 0.9, abs >= 400 | 619 (12.8%) | 197 (17.7%) | 31.8% |
| rel >= 0.9, abs >= 600 | 426 (8.8%) | 131 (11.8%) | 30.8% |
| rel >= 0.9, abs >= 800 | 250 (5.2%) | 66 (5.9%) | 26.4% |
| rel >= 1, abs >= 0 | 504 (10.4%) | 189 (17.0%) | 37.5% |
| rel >= 1, abs >= 200 | 481 (10.0%) | 178 (16.0%) | 37.0% |
| rel >= 1, abs >= 400 | 389 (8.1%) | 138 (12.4%) | 35.5% |
| rel >= 1, abs >= 600 | 271 (5.6%) | 93 (8.4%) | 34.3% |
| rel >= 1, abs >= 800 | 171 (3.5%) | 55 (4.9%) | 32.2% |

### Failure check

Sidecar-labeled ARW block, gap 1000 ms: 26 folders with a scorable burst, 1802 scorable bursts, 4133 picks, 12389 non-picks. Per-folder spreads are over the 20 held-out folders with at least 20 picks.

#### (i) Per-feature thresholds from the picks

##### Fixed percentile

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

##### Matched to a pick false-fail

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

##### Drop one feature (the chosen set, matched)

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

##### Drop one feature (all six, matched)

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

##### Transfer to the DNG blocks

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

#### (ii) Positive-unlabeled logistic

Thresholded at the score that keeps 99 / 98 / 95% of the training picks.

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| all features, keep 99% | 1.1% (47) | 3.5% (439) | 98.2% | 2.1% (0.0%-10.2%) | 3.6% | 1.0% / 3.6% |
| all features, keep 98% | 2.2% (90) | 5.4% (671) | 97.2% | 3.6% (0.0%-13.1%) | 4.8% | 2.0% / 5.3% |
| all features, keep 95% | 5.3% (221) | 10.6% (1317) | 94.2% | 8.4% (0.0%-19.8%) | 10.0% | 5.0% / 10.7% |
| no pose, keep 99% | 1.1% (44) | 3.4% (425) | 98.3% | 1.9% (0.0%-10.2%) | 3.1% | 1.0% / 3.5% |
| no pose, keep 98% | 2.1% (86) | 5.3% (656) | 97.1% | 3.4% (0.0%-12.6%) | 4.5% | 2.0% / 5.2% |
| no pose, keep 95% | 5.4% (223) | 10.9% (1349) | 93.8% | 9.8% (0.0%-19.4%) | 9.6% | 5.0% / 10.9% |

##### Drop one group (all features, keep 98%)

| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| none dropped | 2.2% (90) | 5.4% (671) | 97.2% | 3.6% (0.0%-13.1%) | 4.8% | 2.0% / 5.3% |
| without sharp | 2.2% (91) | 3.4% (416) | 98.0% | 2.5% (0.0%-6.7%) | 5.6% | 2.0% / 3.2% |
| without ef | 2.1% (86) | 5.2% (641) | 97.1% | 3.6% (0.0%-12.1%) | 5.2% | 2.0% / 5.1% |
| without eo | 2.3% (93) | 5.2% (641) | 97.3% | 3.0% (0.0%-13.1%) | 4.9% | 2.0% / 5.1% |
| without pose | 2.1% (88) | 5.3% (654) | 97.3% | 3.3% (0.0%-13.3%) | 4.8% | 2.0% / 5.2% |
| without posediff | 2.1% (85) | 5.2% (650) | 97.2% | 3.4% (0.0%-12.8%) | 4.8% | 2.0% / 5.2% |

##### Coefficients (all folders, standardized features)

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

##### Class prior sensitivity

Elkan-Noto c = E[g | pick] = 0.266, implying 92.2% of the non-picks are technically OK under the selected-completely-at-random assumption (which the user's framing breaks: a pick is chosen by composition and moment among the OK frames, so the true share is likely higher). The 98% pick-kept threshold sits at g = 0.135; P(OK) = g / c there for an assumed OK share of the non-picks:

| Assumed OK share of non-picks | c | P(OK) at the 98% threshold |
| --- | ---: | ---: |
| 30% | 0.527 | 0.256 |
| 50% | 0.400 | 0.337 |
| 70% | 0.323 | 0.417 |
| 90% | 0.270 | 0.498 |

#### Ranking: (iii) pairwise and (iv) pointwise logistic, held out

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

#### Chosen variant

Set rel, ef, eo, matched to a 1% training pick false-fail.

##### Per held-out folder

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

##### Other gaps

| Gap ms | Set | Scorable bursts | Held-out false-fail | Held-out flagged | Mean share kept |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1000 | rel | 1802 | 1.1% | 2.8% | 98.5% |
| 1000 | rel, ef, eo | 1802 | 1.1% | 3.3% | 97.8% |
| 2000 | rel | 1786 | 1.1% | 2.8% | 98.5% |
| 2000 | rel, ef, eo | 1786 | 1.1% | 3.2% | 97.8% |
| 5000 | rel | 1539 | 1.1% | 2.8% | 98.4% |
| 5000 | rel, ef, eo | 1539 | 1.1% | 3.0% | 97.7% |
