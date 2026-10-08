# Step 2 results: bursts, pick-protecting metrics and baselines

Measured on 2026-10-08 with [metrics.py](metrics.py) over the Step 1 dumps
(`D:\Photos\tests\2026-10-08-burst-keep-score\dump\`, see [data.md](data.md)),
`python -I metrics.py <dump-dir> <sample.tsv>`. The tables under "Tables"
are the script's output as printed (headings demoted by one level); its
working copy and raw output are in
`D:\Photos\tests\2026-10-08-burst-keep-score\scripts\` and `step2\`.

## Definitions

- **Burst**: the rule of `crates/app/ui/src/burst.ts` `groupBursts`
  (capture time, then subsec right-padded, then file name; a new burst
  where the gap to the previous frame is over the gap, so a gap equal to it
  stays in; a missing subsec is 0 ms). Measured at 1000 ms (the app's
  `BURST_GAP_MS`), 2000 ms and 5000 ms.
- **Labels**: the rules of [data.md](data.md) as
  [inventory.py](inventory.py) applies them (frames by base stem, the
  `.dop` flag, a picked virtual copy in `Output/`, XMP where no `.dop`,
  `Output/` for the sidecar-less DNG folders). A non-pick is unlabeled.
- **Scorable burst**: two or more frames, at least one pick and at least
  one non-pick. Every metric is over the scorable bursts; picks outside
  them (single frames and all-picked bursts) are counted in each block's
  Counts table.
- **Scores** (higher is better): sharpness over the burst maximum (`rel`)
  and as is; `eye_focus`; the eyes-open probability (`1 -` the closed
  probability, only for a judged face of at least 60 px); `-|yaw|`,
  `-|pitch|`, `-|roll|`, and `-|angle - burst median|` (median over the
  burst's frames with a pose). Baselines: random (seed 0), the first frame
  of the burst (`-position`), and the app's sharpness over the burst
  maximum (the first row of every table).
- **Missing features**: "missing passes" (the plain row) never flags a
  frame lacking the feature and leaves it out of that feature's ranking;
  "missing last" ranks it below every frame with the feature and always
  flags it. A pick-keeping threshold is taken over the picks that have the
  feature in both rows.
- **Pick-keeping threshold**: the 1st (5th) percentile of the block's
  pooled picks, so 99% (95%) of the picks with the feature pass; it is
  applied to every folder of the block.
- **Hand rules**: (a) `rel` below 0.7 / 0.8; (b) the cue state
  `NotCandidate` (`eye_focus` below `candidate_probability()` = 0.772, the
  app's `Not a candidate`; the dump's state and `eye_focus` agree on every
  row); (c) eyes open below 0.5; (d) `|yaw|` over 45 or `|pitch|` over 30
  deg, and 60 / 45 as the second cut (each axis alone also shown); (e) any
  of a-d; (f) a-c only. A frame without a face is judged by (a) alone.
- **Position**: q = (rank - 1) / (size - 1) over the frames ranked, 0
  best, ties at their average rank.

## Reading

All numbers at 1000 ms on the sidecar-labeled ARW block (1802 scorable
bursts, 4133 picks, 12,389 non-picks) unless said otherwise. The two DNG
blocks have 54 and 16 scorable bursts (68 and 32 picks); they agree in
direction but are too small to set a threshold on (at 68 picks the 1st
percentile is the minimum).

- **The hand sharpness cut fails most picks.** `rel < 0.7` fails 60.2% of
  the picks and `rel < 0.8` 67.7% (non-picks flagged 71.0% / 78.9%); the
  median pick sits at 0.585 of its burst's maximum and the 25th percentile
  at 0.32. The sharpness score is taken on the AF region, whose content
  (a face, a jersey's lettering, a ball) changes from frame to frame within
  a burst, so the burst maximum is often a different texture, not a
  sharper frame (hand check 02, 06, 11, 27 below). Only the 99% point
  (`rel >= 0.027`) protects the picks: it fails 1.0% and flags 2.8% of
  the non-picks (per folder: pick false-fail 0-3.5%, non-pick flag
  0-9.1%). The absolute form at its 99% point (15.3) flags the same 2.8%.
- **The AF eye cue and the eyes-open probability are the only checks that
  flag non-picks more than picks at a usable false-fail.** (b) fails 7.5%
  of picks and flags 12.2% of non-picks; (c) fails 5.1% / flags 7.6%. At
  their 99% points (`eye_focus >= 0.176`, eyes open `>= 0.095`) they fail
  0.9% / 0.7% and flag 1.2% / 1.8%. Per folder (b) fails up to 11.6% of
  the picks (`2026-07-05`) and (c) up to 28.6% (`2026-07-26`, 7 picks).
  What they add over sharpness: (b) flags 235 non-picks (1.9% of all
  non-picks) that `rel < 0.7` does not, against 91 picks (2.2%) likewise;
  (c) adds 267 non-picks against 96 picks. The union of the three 99%
  points (sharpness, `eye_focus`, eyes open) fails 2.6% of the picks (106)
  and flags 5.6% of the non-picks (698), keeping 96.0% of a burst on
  average (computed with the script's functions; not a row of the tables).
- **The pose does not separate picks from non-picks.** (d) 45 / 30 fails
  26.5% of the picks and flags 26.5% of the non-picks; 60 / 45 fails 10.1%
  and flags 10.7%. The pose's pairwise AUCs are 0.48-0.53 (random 0.495),
  the burst-median differences 0.49-0.53. Its 99% points (`|yaw| <= 92.9`,
  `|pitch| <= 53.7`) flag under 1% of the non-picks. Profiles and faces
  looking up are picked as often as frontal ones (hand check 03, 05, 13,
  14, 28: flagged by the pose only, all technically fine). With (d), (e)
  fails 73.3% of the picks; without it, (f) 64.5%, still dominated by (a).
- **Ranking: every feature is weak, and the first frame beats them all.**
  Pairwise AUC within a burst: sharpness 0.577, `eye_focus` 0.564, eyes
  open 0.542, random 0.495, first frame **0.653**. 39.5% of the first
  frames of the scorable bursts are picks against 19.2% from the sixth
  frame on. Picks in the top half of their burst: sharpness 54.9%, first
  frame 63.8%, random 46.7%. Every score leaves the worst pick at the
  bottom (q >= 0.9) in 28-38% of the bursts it ranks (the "missing passes"
  rows); the first frame in 16.3%. No
  single feature is a ranking; the choice among frames is, as the framing
  says, the human's, and the capture order carries more of it than any
  technical feature measured here.
- **Missing features.** In the ARW block 1812 scorable frames have no cue
  face, 4767 no eyes judgment and 4829 no pose. Treating them as failing
  ("missing last") fails 12-31% of the picks for any face feature, so a
  frame without a face must pass the face checks, as the requirement says.
  Scorable bursts face-free / mixed / all-faced: 35 / 621 / 1146 (ARW),
  14 / 5 / 35 (sidecar DNG), 1 / 3 / 12 (`Output/` DNG). The DNG blocks
  have no `eye_focus` at all (no AF point), so (b) never fires there.
- **Gaps.** At 2000 and 5000 ms the ARW block has 1786 and 1539 scorable
  bursts (the bursts merge; the median size of a 2+ burst goes 3, 4, 6);
  the face checks' rates barely move ((b) false-fail 7.5 / 7.3 / 7.2%), the
  sharpness `rel` cuts get worse as the burst maximum rises with the burst
  size, and the sharpness pairwise AUC stays at 0.57-0.58. Nothing argues
  for moving `BURST_GAP_MS`.
- **For Step 3.** Sharpness only works at its 99% point, where it flags
  2.8% of the non-picks; the AF eye cue and eyes open add about as much
  again at about the same pick cost, and the pose adds nothing. The best
  hand rule that keeps the picks is therefore the union of the 99% points
  of sharpness, `eye_focus` and eyes open (2.6% false-fail / 5.6% flagged),
  against sharpness alone at 1.0% / 2.8%. Whether a fitted combination
  beats that by more than the per-folder spread is Step 3's question; the
  hand check below says the flags of the hand rules (e) are mostly fine
  frames.

## Hand check of flagged non-picks

30 non-picks drawn at random (seed 0) from those flagged by rule (e) (a
0.7, b, c, d 45 / 30) at 1000 ms over all three blocks. For each, a
768 px crop at full size around the AF point (`riffle-cli crop`; the
centre for the DNG), the same crop of its burst's sharpest frame, and the
face crops of `riffle-cli eyecrops`, all in
`D:\Photos\tests\2026-10-08-burst-keep-score\handcheck\` (`NN-<stem>-af.png`,
`NN-best-<stem>-af.png`, `NN-faces\`). **The tally is the agent's own look
at those crops, not the user's**; the user can re-check it from that
folder. Failed: soft, missed focus or motion blur on the subject, eyes
shut mid-blink, or the face turned away from the camera (the back of the
head). OK: the subject sharp, eyes open, face visible (profiles and faces
looking up or down count as OK).

| # | Folder | File | Fired | Look | Note |
| ---: | --- | --- | --- | --- | --- |
| 01 | `2026-09-13-b` | `_DSC1266.ARW` | a, b, d | failed | Out of focus |
| 02 | `2026-09-19` | `_DSC3040.ARW` | a | OK | Face sharp; the burst max is a shirt print |
| 03 | `2026-06-13` | `_DSC3904.ARW` | d | OK | Sharp three-quarter view |
| 04 | `2026-08-22` | `_DSC1559.ARW` | a | failed | Blurred |
| 05 | `2026-09-27-a` | `_DSC7466.ARW` | d | OK | Sharp, looking up |
| 06 | `2026-09-27-a` | `_DSC6204.ARW` | a | OK | Sharp |
| 07 | `2026-09-19` | `_DSC2362.ARW` | a, d | failed | Turned away (back of the head) |
| 08 | `2026-08-29` | `_DSC4133.ARW` | a, b | failed | Missed focus |
| 09 | `2026-09-27-a` | `_DSC5733.ARW` | a, d | unsure | Small face looking up; AF region slightly soft |
| 10 | `2026-09-13-b` | `_DSC0137.ARW` | a, d | OK | Sharp profile |
| 11 | `2026-10-03` | `_DSC2528.ARW` | a | OK | Sharp |
| 12 | `2026-08-01` | `_DSC8671.ARW` | a, d | unsure | Profile looking down |
| 13 | `2026-09-27-a` | `_DSC7258.ARW` | d | OK | Sharp, looking up |
| 14 | `2026-07-05` | `_DSC2435.ARW` | d | OK | Sharp, eyes open |
| 15 | `2026-08-22` | `_DSC2761.ARW` | a, d | failed | Turned away (back of the head) |
| 16 | `2026-07-05` | `_DSC2445.ARW` | a, d | OK | Sharp ball and profile; pitch -68 is a misread |
| 17 | `2026-06-20` | `_DSC6541.ARW` | a, b | failed | Soft, the subject's back |
| 18 | `2026-05-26` | `BF_00862-DxO_DeepPRIME 3.dng` | a | OK | Sharp (no face) |
| 19 | `2026-08-22` | `_DSC1159.ARW` | a, c | unsure | Laughing, eyes narrowed; slightly soft |
| 20 | `2026-09-27-a` | `_DSC8502.ARW` | a | OK | Sharp |
| 21 | `2026-10-03` | `_DSC3170.ARW` | a, d | OK | Sharp, looking up |
| 22 | `2026-07-18` | `_DSC3370.ARW` | a, d | OK | Sharp, looking up |
| 23 | `2026-08-29` | `_DSC4480.ARW` | a | unsure | AF region is background and an arm |
| 24 | `2026-06-20` | `_DSC6772.ARW` | b | failed | The subject's back to the camera |
| 25 | `2026-06-14` | `_DSC5576.ARW` | a, d | failed | Motion blur |
| 26 | `2026-08-29` | `_DSC5749.ARW` | a | unsure | Two players overlapping; the subject is unclear |
| 27 | `2026-09-27-a` | `_DSC5465.ARW` | a | OK | Sharp (rel 0.113) |
| 28 | `2026-09-27-b` | `_DSC9615.ARW` | d | OK | Sharp profile |
| 29 | `2026-06-20` | `_DSC6871.ARW` | c | OK | Looking down, eyes not shut |
| 30 | `2026-08-29` | `_DSC6846.ARW` | a, c | failed | Mid-blink |

Tally: **failed 9, OK 16, unsure 5** (a rough precision of 9 / 25 = 36%
over the sure ones). By what fired: (a) alone, 9 frames: 1 failed, 6 OK,
2 unsure; (d) alone, 5: all OK; (b) or (c) among what fired, 7: 5 failed
(01, 08, 17, 24, 30), 1 OK (29, a child looking down, the known
closed-eyes misread) and 1 unsure. So most of the hand rule's flags come
from the sharpness cut and the pose, and both flag fine frames; the AF eye
and eyes-open checks flagged real failures in most of the few cases drawn.
Twenty-nine of the 30 are ARW, as the pool is.

## Tables

### Burst statistics

Bursts per gap; "No pick" to "All picked" count the bursts of two or more frames.

#### Gap 1000 ms

| Folder | Bursts | Single frames | Bursts of 2+ | Frames in 2+ | Median size (2+) | Max size | Size 2 | 3-5 | 6-10 | 11+ | No pick | One pick | Several picks | All picked | Scorable |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 241 | 4 | 237 | 1333 | 4.0 | 26 | 32 | 131 | 43 | 31 | 116 | 74 | 47 | 0 | 121 |
| `2026-06-13` | 210 | 6 | 204 | 927 | 3.0 | 15 | 54 | 88 | 49 | 13 | 112 | 78 | 14 | 0 | 92 |
| `2026-06-14` | 285 | 69 | 216 | 1451 | 3.0 | 41 | 64 | 70 | 37 | 45 | 150 | 36 | 30 | 1 | 65 |
| `2026-06-20` | 261 | 28 | 233 | 1022 | 3.0 | 21 | 62 | 119 | 38 | 14 | 135 | 77 | 21 | 0 | 98 |
| `2026-07-02` | 32 | 21 | 11 | 85 | 5.0 | 18 | 3 | 3 | 1 | 4 | 4 | 3 | 4 | 0 | 7 |
| `2026-07-05` | 182 | 5 | 177 | 1410 | 4.0 | 32 | 49 | 50 | 24 | 54 | 101 | 29 | 47 | 1 | 75 |
| `2026-07-11` | 359 | 291 | 68 | 177 | 2.0 | 13 | 50 | 16 | 1 | 1 | 39 | 29 | 0 | 0 | 29 |
| `2026-07-18` | 377 | 96 | 281 | 1449 | 4.0 | 26 | 52 | 129 | 80 | 20 | 171 | 44 | 66 | 1 | 109 |
| `2026-07-21` | 27 | 23 | 4 | 9 | 2.0 | 3 | 3 | 1 | 0 | 0 | 1 | 3 | 0 | 0 | 3 |
| `2026-07-22` | 101 | 64 | 37 | 125 | 3.0 | 8 | 18 | 13 | 6 | 0 | 20 | 16 | 1 | 0 | 17 |
| `2026-07-23` | 17 | 16 | 1 | 2 | 2.0 | 2 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| `2026-07-24` | 163 | 122 | 41 | 96 | 2.0 | 5 | 32 | 9 | 0 | 0 | 26 | 15 | 0 | 0 | 15 |
| `2026-07-26` | 16 | 6 | 10 | 58 | 5.0 | 10 | 2 | 4 | 4 | 0 | 4 | 5 | 1 | 0 | 6 |
| `2026-07-30` | 563 | 397 | 166 | 539 | 2.0 | 13 | 87 | 59 | 17 | 3 | 109 | 51 | 6 | 0 | 57 |
| `2026-07-31` | 159 | 88 | 71 | 261 | 3.0 | 11 | 27 | 32 | 10 | 2 | 37 | 30 | 4 | 2 | 32 |
| `2026-08-01` | 779 | 215 | 564 | 3186 | 3.0 | 45 | 191 | 209 | 73 | 91 | 388 | 103 | 73 | 0 | 176 |
| `2026-08-02` | 69 | 32 | 37 | 118 | 2.0 | 10 | 20 | 11 | 6 | 0 | 17 | 19 | 1 | 0 | 20 |
| `2026-08-08` | 191 | 95 | 96 | 320 | 3.0 | 14 | 45 | 43 | 7 | 1 | 44 | 50 | 2 | 0 | 52 |
| `2026-08-15` | 182 | 120 | 62 | 204 | 3.0 | 16 | 29 | 27 | 5 | 1 | 41 | 21 | 0 | 0 | 21 |
| `2026-08-16` | 19 | 9 | 10 | 50 | 4.0 | 12 | 3 | 4 | 2 | 1 | 5 | 4 | 1 | 0 | 5 |
| `2026-08-22` | 462 | 53 | 409 | 2624 | 3.0 | 56 | 76 | 185 | 73 | 75 | 293 | 64 | 52 | 0 | 116 |
| `2026-08-29` | 601 | 66 | 535 | 3385 | 3.0 | 40 | 181 | 189 | 66 | 99 | 402 | 53 | 80 | 0 | 133 |
| `2026-09-13-b` | 318 | 66 | 252 | 1729 | 4.0 | 34 | 56 | 99 | 53 | 44 | 172 | 37 | 43 | 1 | 79 |
| `2026-09-19` | 316 | 23 | 293 | 2111 | 5.0 | 42 | 48 | 122 | 70 | 53 | 185 | 48 | 60 | 1 | 107 |
| `2026-09-27-a` | 613 | 22 | 591 | 4808 | 4.0 | 42 | 127 | 214 | 91 | 159 | 389 | 80 | 122 | 3 | 199 |
| `2026-09-27-b` | 152 | 6 | 146 | 1234 | 5.0 | 35 | 31 | 47 | 23 | 45 | 90 | 19 | 37 | 0 | 56 |
| `2026-10-03` | 274 | 0 | 274 | 1803 | 6.0 | 33 | 48 | 86 | 103 | 37 | 162 | 64 | 48 | 0 | 112 |
| **arw** | 6969 | 1943 | 5026 | 30516 | 3.0 | 56 | 1391 | 1960 | 882 | 793 | 3214 | 1052 | 760 | 10 | 1802 |
| `2026-08-29-l` | 122 | 78 | 44 | 133 | 3.0 | 8 | 20 | 21 | 3 | 0 | 29 | 14 | 1 | 0 | 15 |
| `2026-09-05` | 226 | 132 | 94 | 279 | 2.5 | 11 | 47 | 40 | 6 | 1 | 54 | 34 | 6 | 1 | 39 |
| **dng-sidecar** | 348 | 210 | 138 | 412 | 3.0 | 11 | 67 | 61 | 9 | 1 | 83 | 48 | 7 | 1 | 54 |
| `2026-01-08` | 3 | 3 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-02-14` | 12 | 12 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-03-29` | 104 | 76 | 28 | 79 | 2.0 | 18 | 21 | 6 | 0 | 1 | 18 | 5 | 5 | 2 | 8 |
| `2026-04-12` | 9 | 8 | 1 | 2 | 2.0 | 2 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| `2026-04-18` | 3 | 3 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-05-22` | 35 | 33 | 2 | 4 | 2.0 | 2 | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 |
| `2026-05-26` | 22 | 1 | 21 | 162 | 5.0 | 30 | 6 | 5 | 5 | 5 | 13 | 4 | 4 | 0 | 8 |
| **dng-output** | 188 | 136 | 52 | 247 | 2.0 | 30 | 30 | 11 | 5 | 6 | 33 | 9 | 10 | 3 | 16 |

#### Gap 2000 ms

| Folder | Bursts | Single frames | Bursts of 2+ | Frames in 2+ | Median size (2+) | Max size | Size 2 | 3-5 | 6-10 | 11+ | No pick | One pick | Several picks | All picked | Scorable |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 222 | 3 | 219 | 1334 | 4.0 | 33 | 27 | 108 | 51 | 33 | 99 | 72 | 48 | 0 | 120 |
| `2026-06-13` | 188 | 4 | 184 | 929 | 4.0 | 17 | 46 | 75 | 46 | 17 | 97 | 69 | 18 | 0 | 87 |
| `2026-06-14` | 208 | 36 | 172 | 1484 | 5.0 | 44 | 45 | 48 | 27 | 52 | 109 | 32 | 31 | 1 | 62 |
| `2026-06-20` | 220 | 22 | 198 | 1028 | 3.0 | 24 | 47 | 87 | 44 | 20 | 104 | 73 | 21 | 0 | 94 |
| `2026-07-02` | 18 | 4 | 14 | 102 | 2.0 | 25 | 9 | 1 | 0 | 4 | 9 | 1 | 4 | 0 | 5 |
| `2026-07-05` | 153 | 4 | 149 | 1411 | 5.0 | 36 | 35 | 40 | 17 | 57 | 78 | 24 | 47 | 0 | 71 |
| `2026-07-11` | 252 | 154 | 98 | 314 | 2.0 | 22 | 56 | 35 | 4 | 3 | 59 | 35 | 4 | 0 | 39 |
| `2026-07-18` | 323 | 57 | 266 | 1488 | 4.5 | 29 | 47 | 112 | 81 | 26 | 156 | 43 | 67 | 1 | 109 |
| `2026-07-21` | 19 | 12 | 7 | 20 | 2.0 | 7 | 5 | 1 | 1 | 0 | 2 | 5 | 0 | 0 | 5 |
| `2026-07-22` | 77 | 35 | 42 | 154 | 3.0 | 14 | 19 | 17 | 5 | 1 | 20 | 20 | 2 | 0 | 22 |
| `2026-07-23` | 12 | 8 | 4 | 10 | 2.5 | 3 | 2 | 2 | 0 | 0 | 2 | 2 | 0 | 0 | 2 |
| `2026-07-24` | 120 | 63 | 57 | 155 | 2.0 | 6 | 33 | 22 | 2 | 0 | 34 | 22 | 1 | 0 | 23 |
| `2026-07-26` | 10 | 2 | 8 | 62 | 4.5 | 19 | 1 | 3 | 1 | 3 | 4 | 2 | 2 | 0 | 4 |
| `2026-07-30` | 349 | 153 | 196 | 783 | 3.0 | 18 | 79 | 81 | 25 | 11 | 123 | 66 | 7 | 0 | 73 |
| `2026-07-31` | 106 | 38 | 68 | 311 | 3.0 | 17 | 25 | 26 | 11 | 6 | 34 | 26 | 8 | 1 | 33 |
| `2026-08-01` | 592 | 109 | 483 | 3292 | 3.0 | 46 | 134 | 181 | 65 | 103 | 310 | 96 | 77 | 1 | 172 |
| `2026-08-02` | 44 | 13 | 31 | 137 | 3.0 | 10 | 11 | 9 | 11 | 0 | 9 | 18 | 4 | 0 | 22 |
| `2026-08-08` | 144 | 57 | 87 | 358 | 3.0 | 22 | 34 | 34 | 16 | 3 | 34 | 47 | 6 | 0 | 53 |
| `2026-08-15` | 116 | 57 | 59 | 267 | 3.0 | 24 | 23 | 20 | 13 | 3 | 30 | 27 | 2 | 0 | 29 |
| `2026-08-16` | 11 | 4 | 7 | 55 | 6.0 | 17 | 1 | 2 | 2 | 2 | 2 | 4 | 1 | 0 | 5 |
| `2026-08-22` | 335 | 24 | 311 | 2653 | 6.0 | 56 | 55 | 99 | 61 | 96 | 205 | 53 | 53 | 0 | 106 |
| `2026-08-29` | 473 | 38 | 435 | 3413 | 4.0 | 50 | 119 | 145 | 58 | 113 | 308 | 44 | 83 | 0 | 127 |
| `2026-09-13-b` | 247 | 42 | 205 | 1753 | 6.0 | 37 | 29 | 68 | 58 | 50 | 125 | 36 | 44 | 0 | 80 |
| `2026-09-19` | 241 | 13 | 228 | 2121 | 6.0 | 47 | 27 | 76 | 62 | 63 | 124 | 44 | 60 | 1 | 103 |
| `2026-09-27-a` | 475 | 14 | 461 | 4816 | 6.0 | 47 | 79 | 147 | 71 | 164 | 277 | 63 | 121 | 2 | 182 |
| `2026-09-27-b` | 123 | 4 | 119 | 1236 | 6.0 | 47 | 19 | 38 | 14 | 48 | 68 | 15 | 36 | 0 | 51 |
| `2026-10-03` | 243 | 0 | 243 | 1803 | 6.0 | 39 | 28 | 75 | 97 | 43 | 136 | 56 | 51 | 0 | 107 |
| **arw** | 5321 | 970 | 4351 | 31489 | 4.0 | 56 | 1035 | 1552 | 843 | 921 | 2558 | 995 | 798 | 7 | 1786 |
| `2026-08-29-l` | 86 | 44 | 42 | 167 | 3.0 | 16 | 15 | 20 | 4 | 3 | 27 | 12 | 3 | 0 | 15 |
| `2026-09-05` | 170 | 71 | 99 | 340 | 3.0 | 11 | 44 | 44 | 9 | 2 | 54 | 38 | 7 | 1 | 44 |
| **dng-sidecar** | 256 | 115 | 141 | 507 | 3.0 | 16 | 59 | 64 | 13 | 5 | 81 | 50 | 10 | 1 | 59 |
| `2026-01-08` | 3 | 3 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-02-14` | 12 | 12 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-03-29` | 84 | 46 | 38 | 109 | 2.0 | 18 | 27 | 9 | 1 | 1 | 26 | 6 | 6 | 2 | 10 |
| `2026-04-12` | 8 | 6 | 2 | 4 | 2.0 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 2 | 0 |
| `2026-04-18` | 3 | 3 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-05-22` | 30 | 23 | 7 | 14 | 2.0 | 2 | 7 | 0 | 0 | 0 | 2 | 5 | 0 | 0 | 5 |
| `2026-05-26` | 19 | 1 | 18 | 162 | 7.0 | 30 | 3 | 5 | 5 | 5 | 10 | 4 | 4 | 0 | 8 |
| **dng-output** | 159 | 94 | 65 | 289 | 2.0 | 30 | 39 | 14 | 6 | 6 | 38 | 15 | 12 | 4 | 23 |

#### Gap 5000 ms

| Folder | Bursts | Single frames | Bursts of 2+ | Frames in 2+ | Median size (2+) | Max size | Size 2 | 3-5 | 6-10 | 11+ | No pick | One pick | Several picks | All picked | Scorable |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 157 | 1 | 156 | 1336 | 5.0 | 179 | 15 | 68 | 42 | 31 | 59 | 53 | 44 | 0 | 97 |
| `2026-06-13` | 146 | 3 | 143 | 930 | 5.0 | 34 | 27 | 55 | 34 | 27 | 66 | 55 | 22 | 0 | 77 |
| `2026-06-14` | 146 | 14 | 132 | 1506 | 7.0 | 61 | 19 | 34 | 26 | 53 | 73 | 25 | 34 | 1 | 58 |
| `2026-06-20` | 165 | 17 | 148 | 1033 | 5.5 | 36 | 26 | 48 | 42 | 32 | 69 | 50 | 29 | 0 | 79 |
| `2026-07-02` | 10 | 0 | 10 | 106 | 5.5 | 27 | 3 | 2 | 1 | 4 | 5 | 1 | 4 | 0 | 5 |
| `2026-07-05` | 116 | 1 | 115 | 1414 | 8.0 | 64 | 19 | 26 | 15 | 55 | 50 | 23 | 42 | 0 | 65 |
| `2026-07-11` | 155 | 59 | 96 | 409 | 3.0 | 33 | 34 | 45 | 14 | 3 | 54 | 35 | 7 | 0 | 42 |
| `2026-07-18` | 205 | 22 | 183 | 1523 | 6.0 | 65 | 26 | 58 | 57 | 42 | 94 | 31 | 58 | 1 | 88 |
| `2026-07-21` | 12 | 4 | 8 | 28 | 2.5 | 10 | 4 | 3 | 1 | 0 | 3 | 3 | 2 | 0 | 5 |
| `2026-07-22` | 44 | 9 | 35 | 180 | 4.0 | 18 | 8 | 16 | 8 | 3 | 11 | 20 | 4 | 0 | 24 |
| `2026-07-23` | 9 | 6 | 3 | 12 | 3.0 | 7 | 1 | 1 | 1 | 0 | 1 | 2 | 0 | 0 | 2 |
| `2026-07-24` | 66 | 16 | 50 | 202 | 3.0 | 13 | 18 | 21 | 9 | 2 | 28 | 18 | 4 | 0 | 22 |
| `2026-07-26` | 7 | 1 | 6 | 63 | 8.0 | 22 | 0 | 3 | 0 | 3 | 2 | 2 | 2 | 0 | 4 |
| `2026-07-30` | 192 | 45 | 147 | 891 | 4.0 | 40 | 31 | 69 | 28 | 19 | 72 | 61 | 14 | 0 | 75 |
| `2026-07-31` | 75 | 16 | 59 | 333 | 3.0 | 38 | 17 | 23 | 11 | 8 | 24 | 28 | 7 | 1 | 34 |
| `2026-08-01` | 394 | 49 | 345 | 3352 | 5.0 | 73 | 60 | 116 | 52 | 117 | 192 | 70 | 83 | 0 | 153 |
| `2026-08-02` | 26 | 5 | 21 | 145 | 6.0 | 33 | 7 | 3 | 8 | 3 | 4 | 12 | 5 | 0 | 17 |
| `2026-08-08` | 87 | 17 | 70 | 398 | 4.0 | 26 | 17 | 29 | 14 | 10 | 21 | 35 | 14 | 0 | 49 |
| `2026-08-15` | 63 | 18 | 45 | 306 | 4.0 | 50 | 7 | 20 | 13 | 5 | 16 | 24 | 5 | 0 | 29 |
| `2026-08-16` | 7 | 2 | 5 | 57 | 10.0 | 20 | 0 | 1 | 2 | 2 | 1 | 2 | 2 | 0 | 4 |
| `2026-08-22` | 210 | 8 | 202 | 2669 | 10.0 | 77 | 16 | 42 | 45 | 99 | 110 | 42 | 50 | 0 | 92 |
| `2026-08-29` | 320 | 14 | 306 | 3437 | 8.0 | 59 | 48 | 86 | 49 | 123 | 197 | 26 | 83 | 0 | 109 |
| `2026-09-13-b` | 178 | 27 | 151 | 1768 | 8.0 | 43 | 15 | 40 | 35 | 61 | 78 | 29 | 44 | 0 | 73 |
| `2026-09-19` | 142 | 4 | 138 | 2130 | 9.0 | 127 | 15 | 31 | 34 | 58 | 63 | 28 | 47 | 0 | 75 |
| `2026-09-27-a` | 332 | 5 | 327 | 4825 | 10.0 | 83 | 34 | 85 | 50 | 158 | 163 | 48 | 116 | 1 | 163 |
| `2026-09-27-b` | 83 | 3 | 80 | 1237 | 11.0 | 53 | 10 | 19 | 8 | 43 | 39 | 12 | 29 | 0 | 41 |
| `2026-10-03` | 113 | 0 | 113 | 1803 | 8.0 | 217 | 10 | 34 | 23 | 46 | 56 | 29 | 28 | 0 | 57 |
| **arw** | 3460 | 366 | 3094 | 32093 | 6.0 | 217 | 487 | 978 | 622 | 1007 | 1551 | 764 | 779 | 4 | 1539 |
| `2026-08-29-l` | 52 | 14 | 38 | 197 | 3.0 | 34 | 13 | 14 | 8 | 3 | 17 | 16 | 5 | 1 | 20 |
| `2026-09-05` | 119 | 40 | 79 | 371 | 4.0 | 16 | 23 | 34 | 18 | 4 | 35 | 33 | 11 | 1 | 43 |
| **dng-sidecar** | 171 | 54 | 117 | 568 | 3.0 | 34 | 36 | 48 | 26 | 7 | 52 | 49 | 16 | 2 | 63 |
| `2026-01-08` | 3 | 3 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-02-14` | 12 | 12 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-03-29` | 56 | 24 | 32 | 131 | 3.0 | 24 | 12 | 16 | 3 | 1 | 21 | 6 | 5 | 1 | 10 |
| `2026-04-12` | 8 | 6 | 2 | 4 | 2.0 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 2 | 0 |
| `2026-04-18` | 3 | 3 | 0 | 0 | - | - | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `2026-05-22` | 24 | 14 | 10 | 23 | 2.0 | 3 | 7 | 3 | 0 | 0 | 2 | 7 | 1 | 1 | 7 |
| `2026-05-26` | 13 | 0 | 13 | 163 | 10.0 | 30 | 0 | 2 | 5 | 6 | 6 | 2 | 5 | 0 | 7 |
| **dng-output** | 119 | 62 | 57 | 321 | 3.0 | 30 | 21 | 21 | 8 | 7 | 29 | 15 | 13 | 4 | 24 |

### Sidecar-labeled ARW folders, gap 1000 ms

#### Counts

| Count | Value |
| --- | ---: |
| Folders | 27 |
| Frames / picks | 32459 / 4420 |
| Bursts (all) / single-frame | 6969 / 1943 |
| Bursts of 2+ dropped: no pick | 3214 |
| Bursts of 2+ dropped: every frame picked | 10 |
| Scorable bursts | 1802 |
| Frames / picks / non-picks in scorable bursts | 16522 / 4133 / 12389 |
| Picks outside scorable bursts (single frames, all-picked bursts) | 287 |
| Scorable frames without a cue face (`eye_focus`) | 1812 |
| Scorable frames without an eyes judgment (< 60 px or none) | 4767 |
| Scorable frames without a pose | 4829 |
| Scorable bursts face-free / mixed / all-faced | 35 / 621 / 1146 |

#### Hand threshold rules

A frame lacking the feature a rule reads is not flagged by it.

| Rule | Pick false-fail | Non-pick flagged | Mean share of a burst kept |
| --- | ---: | ---: | ---: |
| (a) rel sharpness < 0.7 | 60.2% (2490) | 71.0% (8792) | 47.6% |
| (a) rel sharpness < 0.8 | 67.7% (2798) | 78.9% (9771) | 39.2% |
| (b) `eye_focus` < candidate | 7.5% (309) | 12.2% (1514) | 92.2% |
| (c) eyes open < 0.5 | 5.1% (209) | 7.6% (937) | 92.5% |
| (d) \|yaw\| > 45 | 17.7% (731) | 18.2% (2255) | 82.3% |
| (d) \|pitch\| > 30 | 11.9% (491) | 11.8% (1456) | 88.3% |
| (d) \|yaw\| > 45 or \|pitch\| > 30 | 26.5% (1094) | 26.5% (3285) | 74.0% |
| (d) \|yaw\| > 60 or \|pitch\| > 45 | 10.1% (418) | 10.7% (1325) | 89.7% |
| (e) all checks (a 0.7, d 45/30) | 73.3% (3029) | 80.9% (10028) | 31.7% |
| (e) all checks (a 0.8, d 45/30) | 78.5% (3243) | 86.0% (10655) | 26.2% |
| (e) all checks (a 0.7, d 60/45) | 67.4% (2784) | 77.1% (9554) | 38.0% |
| (f) a-c (a 0.7) | 64.5% (2667) | 74.9% (9275) | 42.0% |
| (f) a-c (a 0.8) | 71.2% (2944) | 81.6% (10111) | 34.7% |

Per folder (pick false-fail / non-pick flagged):

| Folder | (a) rel sharpness < 0.8 | (b) `eye_focus` < candidate | (c) eyes open < 0.5 | (d) \|yaw\| > 45 or \|pitch\| > 30 | (e) all checks (a 0.7, d 45/30) | (f) a-c (a 0.7) | (f) a-c (a 0.8) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 70.4% / 74.6% | 2.5% / 8.8% | 3.0% / 4.7% | 30.7% / 28.1% | 72.9% / 79.2% | 63.3% / 70.2% | 70.9% / 76.4% |
| `2026-06-13` | 50.4% / 73.0% | 5.1% / 9.7% | 4.3% / 6.9% | 29.9% / 17.7% | 64.1% / 73.0% | 48.7% / 67.1% | 54.7% / 75.9% |
| `2026-06-14` | 75.5% / 85.1% | 9.1% / 13.3% | 4.2% / 6.2% | 30.1% / 29.6% | 78.3% / 86.1% | 72.7% / 82.3% | 79.0% / 87.1% |
| `2026-06-20` | 48.8% / 65.3% | 3.1% / 8.8% | 4.7% / 10.1% | 37.0% / 30.7% | 67.7% / 74.4% | 48.8% / 64.3% | 53.5% / 70.1% |
| `2026-07-02` | 46.2% / 55.9% | 0.0% / 0.0% | 7.7% / 16.9% | 0.0% / 0.0% | 38.5% / 37.3% | 38.5% / 37.3% | 46.2% / 59.3% |
| `2026-07-05` | 80.5% / 86.4% | 11.6% / 14.7% | 4.8% / 5.5% | 32.7% / 28.2% | 86.9% / 88.9% | 79.3% / 85.4% | 83.7% / 88.6% |
| `2026-07-11` | 20.7% / 38.2% | 0.0% / 1.8% | 10.3% / 9.1% | 31.0% / 41.8% | 41.4% / 63.6% | 20.7% / 34.5% | 27.6% / 45.5% |
| `2026-07-18` | 62.4% / 78.2% | 1.8% / 4.2% | 3.0% / 9.2% | 39.5% / 27.8% | 76.0% / 80.4% | 57.2% / 73.4% | 63.8% / 81.2% |
| `2026-07-21` | 0.0% / 75.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 75.0% | 0.0% / 75.0% | 0.0% / 75.0% |
| `2026-07-22` | 26.3% / 39.5% | 0.0% / 0.0% | 15.8% / 20.9% | 21.1% / 23.3% | 42.1% / 39.5% | 31.6% / 32.6% | 36.8% / 46.5% |
| `2026-07-23` | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-07-24` | 13.3% / 30.8% | 0.0% / 0.0% | 13.3% / 11.5% | 20.0% / 11.5% | 40.0% / 42.3% | 26.7% / 34.6% | 26.7% / 38.5% |
| `2026-07-26` | 57.1% / 71.9% | 0.0% / 9.4% | 28.6% / 21.9% | 28.6% / 15.6% | 57.1% / 78.1% | 57.1% / 75.0% | 57.1% / 78.1% |
| `2026-07-30` | 33.8% / 50.3% | 0.0% / 1.8% | 13.8% / 15.3% | 15.4% / 18.4% | 46.2% / 52.1% | 38.5% / 42.9% | 43.1% / 56.4% |
| `2026-07-31` | 41.2% / 45.8% | 0.0% / 0.0% | 2.9% / 8.3% | 20.6% / 18.8% | 38.2% / 43.8% | 23.5% / 30.2% | 41.2% / 50.0% |
| `2026-08-01` | 63.7% / 79.4% | 6.5% / 13.6% | 4.8% / 7.2% | 24.5% / 21.2% | 70.4% / 80.4% | 62.1% / 75.3% | 67.7% / 82.0% |
| `2026-08-02` | 47.6% / 63.0% | 0.0% / 0.0% | 14.3% / 24.1% | 23.8% / 38.9% | 52.4% / 70.4% | 42.9% / 63.0% | 47.6% / 66.7% |
| `2026-08-08` | 29.6% / 46.6% | 0.0% / 0.8% | 9.3% / 16.0% | 9.3% / 7.6% | 31.5% / 44.3% | 24.1% / 41.2% | 37.0% / 57.3% |
| `2026-08-15` | 38.1% / 49.2% | 0.0% / 0.0% | 4.8% / 5.1% | 23.8% / 22.0% | 57.1% / 64.4% | 42.9% / 47.5% | 42.9% / 52.5% |
| `2026-08-16` | 16.7% / 26.3% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 10.5% | 0.0% / 10.5% | 16.7% / 26.3% |
| `2026-08-22` | 64.7% / 78.3% | 5.6% / 9.2% | 7.9% / 11.5% | 15.4% / 23.9% | 67.3% / 77.0% | 60.5% / 72.6% | 69.2% / 81.4% |
| `2026-08-29` | 76.7% / 83.3% | 13.0% / 20.2% | 5.3% / 7.0% | 22.2% / 24.8% | 79.7% / 83.6% | 74.6% / 79.9% | 80.6% / 85.9% |
| `2026-09-13-b` | 76.2% / 84.2% | 10.3% / 16.9% | 3.1% / 7.1% | 17.5% / 17.7% | 78.9% / 84.6% | 72.2% / 80.9% | 77.6% / 86.2% |
| `2026-09-19` | 67.9% / 75.6% | 6.9% / 9.5% | 2.6% / 6.3% | 31.8% / 33.4% | 71.5% / 80.1% | 59.1% / 69.0% | 70.8% / 78.3% |
| `2026-09-27-a` | 79.0% / 84.9% | 11.0% / 15.5% | 5.3% / 6.0% | 25.9% / 27.9% | 81.5% / 87.0% | 76.3% / 81.9% | 82.7% / 86.9% |
| `2026-09-27-b` | 75.9% / 88.3% | 7.6% / 16.3% | 4.1% / 4.6% | 22.4% / 24.3% | 71.8% / 86.3% | 71.2% / 83.7% | 78.2% / 89.5% |
| `2026-10-03` | 56.5% / 79.9% | 4.8% / 8.2% | 5.4% / 9.0% | 36.6% / 43.3% | 71.5% / 87.9% | 55.9% / 76.8% | 62.4% / 83.8% |

#### Thresholds at a pick-keeping point

The threshold keeps 99% (95%) of the block's picks that have the feature; the false-fail column counts every pick, so under "missing last" it includes the picks lacking the feature.

| Score | Picks kept | Threshold | Pick false-fail | Non-pick flagged | Mean share of a burst kept |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sharpness / burst max | 99% | rel >= 0.027 | 1.0% (41) | 2.8% (346) | 98.5% |
| Sharpness / burst max | 95% | rel >= 0.108 | 5.0% (206) | 9.6% (1189) | 94.3% |
| Sharpness (absolute) | 99% | abs >= 15.3 | 1.0% (41) | 2.8% (348) | 98.2% |
| Sharpness (absolute) | 95% | abs >= 55.6 | 5.0% (206) | 9.0% (1121) | 92.1% |
| `eye_focus` | 99% | >= 0.176 | 0.9% (36) | 1.2% (149) | 99.2% |
| `eye_focus` | 95% | >= 0.563 | 4.4% (183) | 6.8% (846) | 95.8% |
| `eye_focus` (missing last) | 99% | >= 0.176 | 12.0% (496) | 12.1% (1501) | 88.3% |
| `eye_focus` (missing last) | 95% | >= 0.563 | 15.6% (643) | 17.7% (2198) | 85.0% |
| Eyes open | 99% | >= 0.095 | 0.7% (30) | 1.8% (220) | 98.3% |
| Eyes open | 95% | >= 0.347 | 3.7% (151) | 5.4% (664) | 94.5% |
| Eyes open (missing last) | 99% | >= 0.095 | 27.3% (1127) | 31.4% (3890) | 70.3% |
| Eyes open (missing last) | 95% | >= 0.347 | 30.2% (1248) | 35.0% (4334) | 66.5% |
| \|yaw\| | 99% | <= 92.9 deg | 0.7% (29) | 0.6% (77) | 99.3% |
| \|yaw\| | 95% | <= 72.8 deg | 3.6% (150) | 4.0% (500) | 96.2% |
| \|yaw\| (missing last) | 99% | <= 92.9 deg | 27.6% (1142) | 30.6% (3793) | 71.0% |
| \|yaw\| (missing last) | 95% | <= 72.8 deg | 30.6% (1263) | 34.0% (4216) | 67.9% |
| \|pitch\| | 99% | <= 53.7 deg | 0.7% (30) | 0.9% (107) | 99.1% |
| \|pitch\| | 95% | <= 39.8 deg | 3.7% (151) | 4.4% (545) | 95.7% |
| \|pitch\| (missing last) | 99% | <= 53.7 deg | 27.7% (1143) | 30.9% (3823) | 70.9% |
| \|pitch\| (missing last) | 95% | <= 39.8 deg | 30.6% (1264) | 34.4% (4261) | 67.5% |
| \|roll\| | 99% | <= 24.1 deg | 0.7% (28) | 0.7% (86) | 99.3% |
| \|roll\| | 95% | <= 16.4 deg | 3.6% (150) | 3.2% (395) | 96.3% |
| \|roll\| (missing last) | 99% | <= 24.1 deg | 27.6% (1141) | 30.7% (3802) | 71.0% |
| \|roll\| (missing last) | 95% | <= 16.4 deg | 30.6% (1263) | 33.2% (4111) | 68.0% |
| \|yaw - burst median\| | 99% | <= 90.5 deg | 0.7% (30) | 0.9% (106) | 99.5% |
| \|yaw - burst median\| | 95% | <= 54.9 deg | 3.7% (151) | 4.3% (538) | 97.4% |
| \|yaw - burst median\| (missing last) | 99% | <= 90.5 deg | 27.7% (1143) | 30.8% (3822) | 71.2% |
| \|yaw - burst median\| (missing last) | 95% | <= 54.9 deg | 30.6% (1264) | 34.3% (4254) | 69.1% |
| \|pitch - burst median\| | 99% | <= 51.1 deg | 0.7% (30) | 0.9% (116) | 99.4% |
| \|pitch - burst median\| | 95% | <= 30.1 deg | 3.6% (149) | 4.5% (560) | 97.2% |
| \|pitch - burst median\| (missing last) | 99% | <= 51.1 deg | 27.7% (1143) | 30.9% (3832) | 71.1% |
| \|pitch - burst median\| (missing last) | 95% | <= 30.1 deg | 30.5% (1262) | 34.5% (4276) | 68.9% |
| \|roll - burst median\| | 99% | <= 19.8 deg | 0.7% (30) | 1.0% (125) | 99.3% |
| \|roll - burst median\| | 95% | <= 12.0 deg | 3.5% (146) | 4.1% (503) | 97.2% |
| \|roll - burst median\| (missing last) | 99% | <= 19.8 deg | 27.7% (1143) | 31.0% (3841) | 71.1% |
| \|roll - burst median\| (missing last) | 95% | <= 12.0 deg | 30.5% (1259) | 34.1% (4219) | 68.9% |

Per folder at the 99% point, missing passes (pick false-fail / non-pick flagged):

| Folder | Sharpness / burst max | Sharpness (absolute) | `eye_focus` | Eyes open | \|yaw\| | \|pitch\| | \|roll\| | \|yaw - burst median\| | \|pitch - burst median\| | \|roll - burst median\| |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-06-05` | 3.5% / 3.3% | 2.0% / 3.9% | 0.5% / 0.6% | 0.5% / 1.1% | 0.0% / 0.6% | 0.0% / 0.8% | 0.5% / 0.3% | 0.5% / 0.3% | 0.5% / 1.2% | 1.0% / 0.6% |
| `2026-06-13` | 0.0% / 3.8% | 0.0% / 4.5% | 2.6% / 2.4% | 0.0% / 1.2% | 0.0% / 0.0% | 0.0% / 0.2% | 0.0% / 0.0% | 0.9% / 0.5% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-06-14` | 0.7% / 4.3% | 0.0% / 3.2% | 2.1% / 2.9% | 0.7% / 1.9% | 0.7% / 1.1% | 0.7% / 1.3% | 0.7% / 1.0% | 2.1% / 1.1% | 0.0% / 0.6% | 1.4% / 2.1% |
| `2026-06-20` | 0.0% / 0.8% | 0.0% / 1.5% | 0.8% / 1.3% | 0.0% / 1.5% | 0.0% / 0.5% | 0.8% / 0.8% | 0.0% / 1.3% | 0.0% / 0.5% | 0.0% / 0.5% | 0.0% / 0.8% |
| `2026-07-02` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 7.7% / 8.5% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-05` | 1.2% / 4.6% | 0.8% / 3.7% | 2.4% / 3.3% | 0.8% / 1.0% | 0.4% / 0.8% | 1.6% / 1.1% | 0.8% / 0.8% | 0.8% / 1.1% | 3.2% / 1.2% | 0.8% / 1.4% |
| `2026-07-11` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 1.8% | 0.0% / 0.0% | 0.0% / 0.0% | 3.4% / 1.8% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-18` | 0.4% / 4.6% | 0.7% / 5.0% | 0.0% / 0.2% | 0.0% / 3.2% | 1.1% / 1.8% | 0.4% / 0.6% | 1.5% / 1.2% | 0.0% / 0.0% | 0.0% / 0.6% | 1.1% / 1.4% |
| `2026-07-21` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-22` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 5.3% / 7.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-23` | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-07-24` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 13.3% / 3.8% | 0.0% / 0.0% | 0.0% / 3.8% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-26` | 0.0% / 3.1% | 0.0% / 28.1% | 0.0% / 0.0% | 0.0% / 6.2% | 0.0% / 3.1% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-30` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 3.1% / 2.5% | 1.5% / 0.0% | 1.5% / 0.6% | 1.5% / 0.6% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-07-31` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 4.2% | 0.0% / 0.0% | 2.9% / 2.1% | 0.0% / 2.1% | 0.0% / 1.0% | 0.0% / 2.1% | 0.0% / 2.1% |
| `2026-08-01` | 0.9% / 1.5% | 1.4% / 2.1% | 0.0% / 0.3% | 0.7% / 1.6% | 2.8% / 1.3% | 1.4% / 1.3% | 1.4% / 0.9% | 2.1% / 1.2% | 0.9% / 1.2% | 1.6% / 1.4% |
| `2026-08-02` | 0.0% / 0.0% | 0.0% / 3.7% | 0.0% / 0.0% | 4.8% / 16.7% | 0.0% / 1.9% | 0.0% / 1.9% | 0.0% / 1.9% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-08-08` | 0.0% / 0.0% | 1.9% / 0.8% | 0.0% / 0.0% | 3.7% / 8.4% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-08-15` | 0.0% / 0.0% | 4.8% / 0.0% | 0.0% / 0.0% | 0.0% / 1.7% | 0.0% / 0.0% | 0.0% / 0.0% | 4.8% / 1.7% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 1.7% |
| `2026-08-16` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |
| `2026-08-22` | 0.4% / 0.4% | 0.0% / 0.2% | 0.4% / 0.3% | 1.5% / 3.0% | 0.4% / 0.2% | 0.8% / 0.3% | 0.4% / 0.7% | 0.4% / 0.3% | 0.4% / 0.5% | 0.4% / 1.1% |
| `2026-08-29` | 0.9% / 4.0% | 0.6% / 3.7% | 0.2% / 2.0% | 0.9% / 1.7% | 0.2% / 0.7% | 0.9% / 1.4% | 0.4% / 0.7% | 0.6% / 1.1% | 1.3% / 1.7% | 0.9% / 1.2% |
| `2026-09-13-b` | 3.1% / 4.6% | 3.1% / 3.9% | 0.9% / 1.9% | 0.4% / 1.7% | 0.0% / 0.3% | 0.0% / 0.4% | 0.4% / 0.3% | 0.0% / 1.1% | 0.0% / 0.3% | 0.4% / 0.4% |
| `2026-09-19` | 0.7% / 0.2% | 0.7% / 0.4% | 3.3% / 1.6% | 0.0% / 0.9% | 0.0% / 0.8% | 0.0% / 0.9% | 0.4% / 1.0% | 0.7% / 1.3% | 0.4% / 1.2% | 0.0% / 0.9% |
| `2026-09-27-a` | 1.1% / 3.0% | 1.4% / 3.3% | 1.0% / 0.5% | 0.4% / 1.0% | 1.0% / 0.4% | 0.8% / 1.1% | 0.4% / 0.6% | 0.8% / 1.4% | 0.8% / 1.4% | 1.0% / 1.2% |
| `2026-09-27-b` | 1.8% / 9.1% | 1.8% / 7.0% | 0.6% / 2.1% | 0.6% / 0.5% | 1.2% / 0.9% | 0.6% / 0.7% | 0.0% / 0.0% | 1.2% / 0.9% | 1.2% / 0.7% | 0.0% / 0.7% |
| `2026-10-03` | 0.0% / 0.7% | 0.0% / 0.7% | 0.5% / 0.9% | 0.5% / 1.7% | 0.0% / 0.0% | 1.1% / 0.2% | 1.6% / 0.6% | 0.0% / 0.2% | 0.5% / 0.7% | 0.5% / 0.4% |

#### Pick position and AUC

Position q = (rank - 1) / (size - 1) over the frames ranked (0 best). Top half: q < 1/2; top third: q < 1/3. Worst pick: the largest q of a burst's picks. Pairwise AUC: pick / non-pick pairs of one burst. Absolute AUC: every frame of the scorable bursts. Top-1: the burst's best frame is a pick (ties split).

| Score | Bursts | Picks | Picks in top half | Top third | Worst pick median | Worst pick p90 | Bursts with worst pick >= 0.9 | Pairwise AUC | Absolute AUC | Top-1 hit | Mean pick q |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Sharpness / burst max | 1802 | 4133 | 54.9% | 41.7% | 0.60 | 1.00 | 28.0% | 0.577 | 0.582 | 40.5% | 0.438 |
| Sharpness (absolute) | 1802 | 4133 | 54.9% | 41.7% | 0.60 | 1.00 | 28.0% | 0.577 | 0.547 | 40.5% | 0.438 |
| `eye_focus` | 1692 | 3655 | 52.9% | 39.8% | 0.61 | 1.00 | 29.9% | 0.564 | 0.556 | 36.1% | 0.453 |
| `eye_focus` (missing last) | 1802 | 4133 | 52.1% | 39.2% | 0.62 | 1.00 | 29.4% | 0.552 | 0.543 | 35.6% | 0.460 |
| Eyes open | 1439 | 3001 | 50.0% | 37.5% | 0.67 | 1.00 | 30.2% | 0.542 | 0.542 | 33.4% | 0.470 |
| Eyes open (missing last) | 1802 | 4133 | 46.6% | 35.9% | 0.60 | 1.00 | 22.5% | 0.541 | 0.537 | 32.2% | 0.471 |
| \|yaw\| | 1435 | 2982 | 49.9% | 36.5% | 0.67 | 1.00 | 33.3% | 0.528 | 0.510 | 33.8% | 0.479 |
| \|yaw\| (missing last) | 1802 | 4133 | 46.8% | 34.6% | 0.60 | 1.00 | 23.5% | 0.533 | 0.520 | 32.5% | 0.474 |
| \|pitch\| | 1435 | 2982 | 46.3% | 33.3% | 0.75 | 1.00 | 37.1% | 0.487 | 0.488 | 31.3% | 0.505 |
| \|pitch\| (missing last) | 1802 | 4133 | 44.2% | 32.2% | 0.65 | 1.00 | 24.7% | 0.510 | 0.509 | 30.5% | 0.489 |
| \|roll\| | 1435 | 2982 | 45.3% | 33.7% | 0.75 | 1.00 | 37.8% | 0.479 | 0.489 | 32.2% | 0.510 |
| \|roll\| (missing last) | 1802 | 4133 | 43.2% | 32.1% | 0.67 | 1.00 | 24.8% | 0.506 | 0.510 | 31.3% | 0.494 |
| \|yaw - burst median\| | 1435 | 2982 | 46.3% | 34.4% | 0.67 | 1.00 | 30.0% | 0.523 | 0.527 | 31.7% | 0.487 |
| \|yaw - burst median\| (missing last) | 1802 | 4133 | 44.5% | 33.6% | 0.57 | 1.00 | 21.8% | 0.530 | 0.529 | 30.9% | 0.481 |
| \|pitch - burst median\| | 1435 | 2982 | 45.9% | 33.8% | 0.62 | 1.00 | 30.2% | 0.519 | 0.532 | 32.5% | 0.486 |
| \|pitch - burst median\| (missing last) | 1802 | 4133 | 44.2% | 33.2% | 0.58 | 1.00 | 20.5% | 0.528 | 0.532 | 31.6% | 0.479 |
| \|roll - burst median\| | 1435 | 2982 | 44.4% | 32.7% | 0.67 | 1.00 | 32.8% | 0.493 | 0.503 | 32.3% | 0.503 |
| \|roll - burst median\| (missing last) | 1802 | 4133 | 43.2% | 32.3% | 0.60 | 1.00 | 22.6% | 0.514 | 0.517 | 31.4% | 0.490 |
| Random (seed 0) | 1802 | 4133 | 46.7% | 33.6% | 0.75 | 1.00 | 37.1% | 0.495 | 0.496 | 29.9% | 0.503 |
| First frame | 1802 | 4133 | 63.8% | 46.7% | 0.50 | 1.00 | 16.3% | 0.653 | 0.621 | 39.5% | 0.378 |

### Sidecar-labeled Leica DNG folders, gap 1000 ms

#### Counts

| Count | Value |
| --- | ---: |
| Folders | 2 |
| Frames / picks | 622 / 100 |
| Bursts (all) / single-frame | 348 / 210 |
| Bursts of 2+ dropped: no pick | 83 |
| Bursts of 2+ dropped: every frame picked | 1 |
| Scorable bursts | 54 |
| Frames / picks / non-picks in scorable bursts | 198 / 68 / 130 |
| Picks outside scorable bursts (single frames, all-picked bursts) | 32 |
| Scorable frames without a cue face (`eye_focus`) | 198 |
| Scorable frames without an eyes judgment (< 60 px or none) | 59 |
| Scorable frames without a pose | 60 |
| Scorable bursts face-free / mixed / all-faced | 14 / 5 / 35 |

#### Hand threshold rules

A frame lacking the feature a rule reads is not flagged by it.

| Rule | Pick false-fail | Non-pick flagged | Mean share of a burst kept |
| --- | ---: | ---: | ---: |
| (a) rel sharpness < 0.7 | 27.9% (19) | 40.0% (52) | 69.8% |
| (a) rel sharpness < 0.8 | 36.8% (25) | 51.5% (67) | 57.5% |
| (b) `eye_focus` < candidate | 0.0% (0) | 0.0% (0) | 100.0% |
| (c) eyes open < 0.5 | 5.9% (4) | 10.0% (13) | 91.1% |
| (d) \|yaw\| > 45 | 11.8% (8) | 10.0% (13) | 89.1% |
| (d) \|pitch\| > 30 | 2.9% (2) | 3.1% (4) | 97.2% |
| (d) \|yaw\| > 45 or \|pitch\| > 30 | 13.2% (9) | 13.1% (17) | 86.7% |
| (d) \|yaw\| > 60 or \|pitch\| > 45 | 2.9% (2) | 5.4% (7) | 96.5% |
| (e) all checks (a 0.7, d 45/30) | 36.8% (25) | 46.9% (61) | 60.4% |
| (e) all checks (a 0.8, d 45/30) | 44.1% (30) | 55.4% (72) | 51.2% |
| (e) all checks (a 0.7, d 60/45) | 32.4% (22) | 45.4% (59) | 64.4% |
| (f) a-c (a 0.7) | 30.9% (21) | 42.3% (55) | 66.8% |
| (f) a-c (a 0.8) | 38.2% (26) | 52.3% (68) | 56.3% |

Per folder (pick false-fail / non-pick flagged):

| Folder | (a) rel sharpness < 0.8 | (b) `eye_focus` < candidate | (c) eyes open < 0.5 | (d) \|yaw\| > 45 or \|pitch\| > 30 | (e) all checks (a 0.7, d 45/30) | (f) a-c (a 0.7) | (f) a-c (a 0.8) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-08-29-l` | 35.3% / 60.0% | 0.0% / 0.0% | 0.0% / 5.7% | 29.4% / 25.7% | 41.2% / 57.1% | 17.6% / 48.6% | 35.3% / 60.0% |
| `2026-09-05` | 37.3% / 48.4% | 0.0% / 0.0% | 7.8% / 11.6% | 7.8% / 8.4% | 35.3% / 43.2% | 35.3% / 40.0% | 39.2% / 49.5% |

#### Thresholds at a pick-keeping point

The threshold keeps 99% (95%) of the block's picks that have the feature; the false-fail column counts every pick, so under "missing last" it includes the picks lacking the feature.

| Score | Picks kept | Threshold | Pick false-fail | Non-pick flagged | Mean share of a burst kept |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sharpness / burst max | 99% | rel >= 0.104 | 0.0% (0) | 7.7% (10) | 95.0% |
| Sharpness / burst max | 95% | rel >= 0.362 | 4.4% (3) | 16.2% (21) | 88.7% |
| Sharpness (absolute) | 99% | abs >= 4.4 | 0.0% (0) | 0.8% (1) | 99.4% |
| Sharpness (absolute) | 95% | abs >= 21.6 | 4.4% (3) | 10.0% (13) | 91.2% |
| `eye_focus` | 99% | - | - | - | - |
| `eye_focus` | 95% | - | - | - | - |
| `eye_focus` (missing last) | 99% | - | - | - | - |
| `eye_focus` (missing last) | 95% | - | - | - | - |
| Eyes open | 99% | >= 0.090 | 0.0% (0) | 2.3% (3) | 98.3% |
| Eyes open | 95% | >= 0.246 | 2.9% (2) | 8.5% (11) | 93.1% |
| Eyes open (missing last) | 99% | >= 0.090 | 39.7% (27) | 26.9% (35) | 67.2% |
| Eyes open (missing last) | 95% | >= 0.246 | 42.6% (29) | 33.1% (43) | 61.9% |
| \|yaw\| | 99% | <= 115.3 deg | 0.0% (0) | 0.0% (0) | 100.0% |
| \|yaw\| | 95% | <= 59.3 deg | 2.9% (2) | 5.4% (7) | 96.5% |
| \|yaw\| (missing last) | 99% | <= 115.3 deg | 39.7% (27) | 25.4% (33) | 68.6% |
| \|yaw\| (missing last) | 95% | <= 59.3 deg | 42.6% (29) | 30.8% (40) | 65.2% |
| \|pitch\| | 99% | <= 56.3 deg | 0.0% (0) | 0.0% (0) | 100.0% |
| \|pitch\| | 95% | <= 27.8 deg | 2.9% (2) | 4.6% (6) | 95.8% |
| \|pitch\| (missing last) | 99% | <= 56.3 deg | 39.7% (27) | 25.4% (33) | 68.6% |
| \|pitch\| (missing last) | 95% | <= 27.8 deg | 42.6% (29) | 30.0% (39) | 64.5% |
| \|roll\| | 99% | <= 21.4 deg | 0.0% (0) | 2.3% (3) | 98.8% |
| \|roll\| | 95% | <= 17.2 deg | 2.9% (2) | 6.9% (9) | 95.1% |
| \|roll\| (missing last) | 99% | <= 21.4 deg | 39.7% (27) | 27.7% (36) | 67.4% |
| \|roll\| (missing last) | 95% | <= 17.2 deg | 42.6% (29) | 32.3% (42) | 63.7% |
| \|yaw - burst median\| | 99% | <= 139.8 deg | 0.0% (0) | 0.0% (0) | 100.0% |
| \|yaw - burst median\| | 95% | <= 23.4 deg | 2.9% (2) | 6.9% (9) | 95.0% |
| \|yaw - burst median\| (missing last) | 99% | <= 139.8 deg | 39.7% (27) | 25.4% (33) | 68.6% |
| \|yaw - burst median\| (missing last) | 95% | <= 23.4 deg | 42.6% (29) | 32.3% (42) | 63.6% |
| \|pitch - burst median\| | 99% | <= 38.0 deg | 0.0% (0) | 0.0% (0) | 100.0% |
| \|pitch - burst median\| | 95% | <= 19.4 deg | 2.9% (2) | 5.4% (7) | 96.4% |
| \|pitch - burst median\| (missing last) | 99% | <= 38.0 deg | 39.7% (27) | 25.4% (33) | 68.6% |
| \|pitch - burst median\| (missing last) | 95% | <= 19.4 deg | 42.6% (29) | 30.8% (40) | 65.0% |
| \|roll - burst median\| | 99% | <= 21.6 deg | 0.0% (0) | 0.8% (1) | 99.4% |
| \|roll - burst median\| | 95% | <= 7.1 deg | 2.9% (2) | 8.5% (11) | 92.7% |
| \|roll - burst median\| (missing last) | 99% | <= 21.6 deg | 39.7% (27) | 26.2% (34) | 68.0% |
| \|roll - burst median\| (missing last) | 95% | <= 7.1 deg | 42.6% (29) | 33.8% (44) | 61.3% |

Per folder at the 99% point, missing passes (pick false-fail / non-pick flagged):

| Folder | Sharpness / burst max | Sharpness (absolute) | Eyes open | \|yaw\| | \|pitch\| | \|roll\| | \|yaw - burst median\| | \|pitch - burst median\| | \|roll - burst median\| |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-08-29-l` | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 2.9% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 2.9% |
| `2026-09-05` | 0.0% / 10.5% | 0.0% / 1.1% | 0.0% / 3.2% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 2.1% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% |

#### Pick position and AUC

Position q = (rank - 1) / (size - 1) over the frames ranked (0 best). Top half: q < 1/2; top third: q < 1/3. Worst pick: the largest q of a burst's picks. Pairwise AUC: pick / non-pick pairs of one burst. Absolute AUC: every frame of the scorable bursts. Top-1: the burst's best frame is a pick (ties split).

| Score | Bursts | Picks | Picks in top half | Top third | Worst pick median | Worst pick p90 | Bursts with worst pick >= 0.9 | Pairwise AUC | Absolute AUC | Top-1 hit | Mean pick q |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Sharpness / burst max | 54 | 68 | 55.9% | 45.6% | 0.33 | 1.00 | 27.8% | 0.556 | 0.621 | 50.0% | 0.412 |
| Sharpness (absolute) | 54 | 68 | 55.9% | 45.6% | 0.33 | 1.00 | 27.8% | 0.556 | 0.611 | 50.0% | 0.412 |
| `eye_focus` | 0 | 0 | - | - | - | - | - | - | - | - | - |
| `eye_focus` (missing last) | 54 | 68 | 0.0% | 0.0% | 0.50 | 0.50 | 0.0% | 0.500 | 0.500 | 36.3% | 0.500 |
| Eyes open | 37 | 41 | 43.9% | 39.0% | 0.50 | 1.00 | 18.9% | 0.537 | 0.553 | 27.0% | 0.465 |
| Eyes open (missing last) | 54 | 68 | 27.9% | 25.0% | 0.50 | 1.00 | 13.0% | 0.537 | 0.448 | 29.6% | 0.489 |
| \|yaw\| | 37 | 41 | 39.0% | 34.1% | 0.50 | 1.00 | 24.3% | 0.486 | 0.473 | 33.8% | 0.483 |
| \|yaw\| (missing last) | 54 | 68 | 25.0% | 23.5% | 0.50 | 1.00 | 16.7% | 0.508 | 0.416 | 34.2% | 0.497 |
| \|pitch\| | 37 | 41 | 46.3% | 43.9% | 0.50 | 1.00 | 29.7% | 0.551 | 0.569 | 45.9% | 0.441 |
| \|pitch\| (missing last) | 54 | 68 | 30.9% | 27.9% | 0.50 | 1.00 | 20.4% | 0.548 | 0.459 | 42.5% | 0.474 |
| \|roll\| | 37 | 41 | 39.0% | 31.7% | 0.50 | 1.00 | 27.0% | 0.463 | 0.510 | 28.4% | 0.502 |
| \|roll\| (missing last) | 54 | 68 | 25.0% | 20.6% | 0.50 | 1.00 | 16.7% | 0.494 | 0.433 | 30.5% | 0.507 |
| \|yaw - burst median\| | 37 | 41 | 43.9% | 43.9% | 0.50 | 1.00 | 24.3% | 0.612 | 0.574 | 41.9% | 0.438 |
| \|yaw - burst median\| (missing last) | 54 | 68 | 29.4% | 29.4% | 0.50 | 1.00 | 18.5% | 0.584 | 0.462 | 39.8% | 0.472 |
| \|pitch - burst median\| | 37 | 41 | 31.7% | 31.7% | 0.50 | 1.00 | 37.8% | 0.453 | 0.487 | 33.8% | 0.535 |
| \|pitch - burst median\| (missing last) | 54 | 68 | 22.1% | 20.6% | 0.50 | 1.00 | 25.9% | 0.489 | 0.423 | 34.2% | 0.530 |
| \|roll - burst median\| | 37 | 41 | 34.1% | 31.7% | 0.50 | 1.00 | 24.3% | 0.463 | 0.536 | 33.8% | 0.492 |
| \|roll - burst median\| (missing last) | 54 | 68 | 23.5% | 22.1% | 0.50 | 1.00 | 16.7% | 0.494 | 0.445 | 34.2% | 0.501 |
| Random (seed 0) | 54 | 68 | 44.1% | 35.3% | 0.50 | 1.00 | 35.2% | 0.511 | 0.498 | 37.0% | 0.484 |
| First frame | 54 | 68 | 41.2% | 36.8% | 0.50 | 1.00 | 29.6% | 0.567 | 0.544 | 35.2% | 0.480 |

### `Output/`-labeled DNG folders, gap 1000 ms

#### Counts

| Count | Value |
| --- | ---: |
| Folders | 7 |
| Frames / picks | 383 / 70 |
| Bursts (all) / single-frame | 188 / 136 |
| Bursts of 2+ dropped: no pick | 33 |
| Bursts of 2+ dropped: every frame picked | 3 |
| Scorable bursts | 16 |
| Frames / picks / non-picks in scorable bursts | 141 / 32 / 109 |
| Picks outside scorable bursts (single frames, all-picked bursts) | 38 |
| Scorable frames without a cue face (`eye_focus`) | 141 |
| Scorable frames without an eyes judgment (< 60 px or none) | 27 |
| Scorable frames without a pose | 27 |
| Scorable bursts face-free / mixed / all-faced | 1 / 3 / 12 |

#### Hand threshold rules

A frame lacking the feature a rule reads is not flagged by it.

| Rule | Pick false-fail | Non-pick flagged | Mean share of a burst kept |
| --- | ---: | ---: | ---: |
| (a) rel sharpness < 0.7 | 65.6% (21) | 78.0% (85) | 46.1% |
| (a) rel sharpness < 0.8 | 68.8% (22) | 80.7% (88) | 38.0% |
| (b) `eye_focus` < candidate | 0.0% (0) | 0.0% (0) | 100.0% |
| (c) eyes open < 0.5 | 6.2% (2) | 6.4% (7) | 90.9% |
| (d) \|yaw\| > 45 | 9.4% (3) | 20.2% (22) | 81.1% |
| (d) \|pitch\| > 30 | 0.0% (0) | 3.7% (4) | 98.5% |
| (d) \|yaw\| > 45 or \|pitch\| > 30 | 9.4% (3) | 22.9% (25) | 80.0% |
| (d) \|yaw\| > 60 or \|pitch\| > 45 | 3.1% (1) | 5.5% (6) | 93.0% |
| (e) all checks (a 0.7, d 45/30) | 71.9% (23) | 84.4% (92) | 32.0% |
| (e) all checks (a 0.8, d 45/30) | 75.0% (24) | 87.2% (95) | 23.8% |
| (e) all checks (a 0.7, d 60/45) | 71.9% (23) | 81.7% (89) | 36.9% |
| (f) a-c (a 0.7) | 68.8% (22) | 78.9% (86) | 42.8% |
| (f) a-c (a 0.8) | 71.9% (23) | 81.7% (89) | 34.6% |

Per folder (pick false-fail / non-pick flagged):

| Folder | (a) rel sharpness < 0.8 | (b) `eye_focus` < candidate | (c) eyes open < 0.5 | (d) \|yaw\| > 45 or \|pitch\| > 30 | (e) all checks (a 0.7, d 45/30) | (f) a-c (a 0.7) | (f) a-c (a 0.8) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-01-08` | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-02-14` | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-03-29` | 47.1% / 63.2% | 0.0% / 0.0% | 5.9% / 10.5% | 5.9% / 5.3% | 47.1% / 52.6% | 47.1% / 47.4% | 52.9% / 63.2% |
| `2026-04-12` | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-04-18` | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-05-22` | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-05-26` | 93.3% / 84.4% | 0.0% / 0.0% | 6.7% / 5.6% | 13.3% / 26.7% | 100.0% / 91.1% | 93.3% / 85.6% | 93.3% / 85.6% |

#### Thresholds at a pick-keeping point

The threshold keeps 99% (95%) of the block's picks that have the feature; the false-fail column counts every pick, so under "missing last" it includes the picks lacking the feature.

| Score | Picks kept | Threshold | Pick false-fail | Non-pick flagged | Mean share of a burst kept |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sharpness / burst max | 99% | rel >= 0.014 | 0.0% (0) | 15.6% (17) | 94.7% |
| Sharpness / burst max | 95% | rel >= 0.017 | 3.1% (1) | 17.4% (19) | 92.4% |
| Sharpness (absolute) | 99% | abs >= 9.3 | 0.0% (0) | 3.7% (4) | 97.3% |
| Sharpness (absolute) | 95% | abs >= 10.4 | 3.1% (1) | 9.2% (10) | 95.0% |
| `eye_focus` | 99% | - | - | - | - |
| `eye_focus` | 95% | - | - | - | - |
| `eye_focus` (missing last) | 99% | - | - | - | - |
| `eye_focus` (missing last) | 95% | - | - | - | - |
| Eyes open | 99% | >= 0.356 | 0.0% (0) | 4.6% (5) | 94.1% |
| Eyes open | 95% | >= 0.438 | 3.1% (1) | 5.5% (6) | 91.6% |
| Eyes open (missing last) | 99% | >= 0.356 | 25.0% (8) | 22.0% (24) | 85.0% |
| Eyes open (missing last) | 95% | >= 0.438 | 28.1% (9) | 22.9% (25) | 82.4% |
| \|yaw\| | 99% | <= 76.0 deg | 0.0% (0) | 2.8% (3) | 95.6% |
| \|yaw\| | 95% | <= 47.4 deg | 3.1% (1) | 16.5% (18) | 88.7% |
| \|yaw\| (missing last) | 99% | <= 76.0 deg | 25.0% (8) | 20.2% (22) | 86.5% |
| \|yaw\| (missing last) | 95% | <= 47.4 deg | 28.1% (9) | 33.9% (37) | 79.5% |
| \|pitch\| | 99% | <= 22.5 deg | 0.0% (0) | 9.2% (10) | 96.2% |
| \|pitch\| | 95% | <= 19.9 deg | 3.1% (1) | 14.7% (16) | 94.2% |
| \|pitch\| (missing last) | 99% | <= 22.5 deg | 25.0% (8) | 26.6% (29) | 87.0% |
| \|pitch\| (missing last) | 95% | <= 19.9 deg | 28.1% (9) | 32.1% (35) | 85.0% |
| \|roll\| | 99% | <= 14.0 deg | 0.0% (0) | 4.6% (5) | 98.3% |
| \|roll\| | 95% | <= 12.8 deg | 3.1% (1) | 4.6% (5) | 96.2% |
| \|roll\| (missing last) | 99% | <= 14.0 deg | 25.0% (8) | 22.0% (24) | 89.1% |
| \|roll\| (missing last) | 95% | <= 12.8 deg | 28.1% (9) | 22.0% (24) | 87.0% |
| \|yaw - burst median\| | 99% | <= 49.9 deg | 0.0% (0) | 11.0% (12) | 94.3% |
| \|yaw - burst median\| | 95% | <= 46.0 deg | 3.1% (1) | 11.9% (13) | 93.6% |
| \|yaw - burst median\| (missing last) | 99% | <= 49.9 deg | 25.0% (8) | 28.4% (31) | 85.1% |
| \|yaw - burst median\| (missing last) | 95% | <= 46.0 deg | 28.1% (9) | 29.4% (32) | 84.5% |
| \|pitch - burst median\| | 99% | <= 20.3 deg | 0.0% (0) | 6.4% (7) | 97.6% |
| \|pitch - burst median\| | 95% | <= 17.1 deg | 3.1% (1) | 12.8% (14) | 94.9% |
| \|pitch - burst median\| (missing last) | 99% | <= 20.3 deg | 25.0% (8) | 23.9% (26) | 88.4% |
| \|pitch - burst median\| (missing last) | 95% | <= 17.1 deg | 28.1% (9) | 30.3% (33) | 85.7% |
| \|roll - burst median\| | 99% | <= 13.7 deg | 0.0% (0) | 4.6% (5) | 96.8% |
| \|roll - burst median\| | 95% | <= 6.8 deg | 3.1% (1) | 14.7% (16) | 90.8% |
| \|roll - burst median\| (missing last) | 99% | <= 13.7 deg | 25.0% (8) | 22.0% (24) | 87.7% |
| \|roll - burst median\| (missing last) | 95% | <= 6.8 deg | 28.1% (9) | 32.1% (35) | 81.7% |

Per folder at the 99% point, missing passes (pick false-fail / non-pick flagged):

| Folder | Sharpness / burst max | Sharpness (absolute) | Eyes open | \|yaw\| | \|pitch\| | \|roll\| | \|yaw - burst median\| | \|pitch - burst median\| | \|roll - burst median\| |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-01-08` | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-02-14` | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-03-29` | 0.0% / 0.0% | 0.0% / 5.3% | 0.0% / 10.5% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 0.0% | 0.0% / 5.3% |
| `2026-04-12` | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-04-18` | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-05-22` | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - | - / - |
| `2026-05-26` | 0.0% / 18.9% | 0.0% / 3.3% | 0.0% / 3.3% | 0.0% / 3.3% | 0.0% / 11.1% | 0.0% / 5.6% | 0.0% / 13.3% | 0.0% / 7.8% | 0.0% / 4.4% |

#### Pick position and AUC

Position q = (rank - 1) / (size - 1) over the frames ranked (0 best). Top half: q < 1/2; top third: q < 1/3. Worst pick: the largest q of a burst's picks. Pairwise AUC: pick / non-pick pairs of one burst. Absolute AUC: every frame of the scorable bursts. Top-1: the burst's best frame is a pick (ties split).

| Score | Bursts | Picks | Picks in top half | Top third | Worst pick median | Worst pick p90 | Bursts with worst pick >= 0.9 | Pairwise AUC | Absolute AUC | Top-1 hit | Mean pick q |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Sharpness / burst max | 16 | 32 | 46.9% | 34.4% | 0.85 | 1.00 | 37.5% | 0.477 | 0.630 | 43.8% | 0.494 |
| Sharpness (absolute) | 16 | 32 | 46.9% | 34.4% | 0.85 | 1.00 | 37.5% | 0.477 | 0.605 | 43.8% | 0.494 |
| `eye_focus` | 0 | 0 | - | - | - | - | - | - | - | - | - |
| `eye_focus` (missing last) | 16 | 32 | 0.0% | 0.0% | 0.50 | 0.50 | 0.0% | 0.500 | 0.500 | 35.9% | 0.500 |
| Eyes open | 15 | 24 | 70.8% | 54.2% | 0.25 | 1.00 | 20.0% | 0.646 | 0.551 | 53.3% | 0.339 |
| Eyes open (missing last) | 16 | 32 | 56.2% | 43.8% | 0.33 | 1.00 | 18.8% | 0.626 | 0.494 | 52.8% | 0.365 |
| \|yaw\| | 15 | 24 | 41.7% | 33.3% | 0.89 | 1.00 | 46.7% | 0.548 | 0.610 | 33.3% | 0.543 |
| \|yaw\| (missing last) | 16 | 32 | 31.2% | 28.1% | 0.89 | 1.00 | 43.8% | 0.561 | 0.530 | 34.0% | 0.525 |
| \|pitch\| | 15 | 24 | 45.8% | 37.5% | 0.67 | 1.00 | 20.0% | 0.500 | 0.500 | 46.7% | 0.437 |
| \|pitch\| (missing last) | 16 | 32 | 34.4% | 31.2% | 0.62 | 1.00 | 18.8% | 0.530 | 0.462 | 46.5% | 0.436 |
| \|roll\| | 15 | 24 | 54.2% | 33.3% | 1.00 | 1.00 | 53.3% | 0.516 | 0.497 | 46.7% | 0.509 |
| \|roll\| (missing last) | 16 | 32 | 40.6% | 28.1% | 0.89 | 1.00 | 43.8% | 0.540 | 0.460 | 46.5% | 0.497 |
| \|yaw - burst median\| | 15 | 24 | 45.8% | 33.3% | 0.50 | 1.00 | 26.7% | 0.561 | 0.703 | 33.3% | 0.475 |
| \|yaw - burst median\| (missing last) | 16 | 32 | 37.5% | 28.1% | 0.50 | 1.00 | 25.0% | 0.570 | 0.588 | 34.0% | 0.472 |
| \|pitch - burst median\| | 15 | 24 | 54.2% | 41.7% | 0.50 | 1.00 | 20.0% | 0.535 | 0.617 | 43.3% | 0.429 |
| \|pitch - burst median\| (missing last) | 16 | 32 | 40.6% | 31.2% | 0.50 | 1.00 | 18.8% | 0.553 | 0.535 | 43.4% | 0.432 |
| \|roll - burst median\| | 15 | 24 | 50.0% | 45.8% | 0.67 | 1.00 | 33.3% | 0.497 | 0.628 | 53.3% | 0.445 |
| \|roll - burst median\| (missing last) | 16 | 32 | 37.5% | 37.5% | 0.67 | 1.00 | 31.2% | 0.528 | 0.542 | 52.8% | 0.449 |
| Random (seed 0) | 16 | 32 | 40.6% | 25.0% | 0.94 | 1.00 | 56.2% | 0.509 | 0.472 | 12.5% | 0.570 |
| First frame | 16 | 32 | 56.2% | 37.5% | 0.50 | 1.00 | 18.8% | 0.586 | 0.679 | 50.0% | 0.397 |

### Other gaps

Hand rules at 2000 and 5000 ms (pick false-fail / non-pick flagged / mean share kept), and the sharpness-over-max ranking.

| Gap ms | Block | Scorable bursts | (a) rel sharpness < 0.8 | (b) `eye_focus` < candidate | (c) eyes open < 0.5 | (d) \|yaw\| > 45 or \|pitch\| > 30 | (e) all checks (a 0.7, d 45/30) | (f) a-c (a 0.7) | Sharpness / max pairwise AUC | Picks in top half |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1000 | arw | 1802 | 67.7% / 78.9% / 39.2% | 7.5% / 12.2% / 92.2% | 5.1% / 7.6% / 92.5% | 26.5% / 26.5% / 74.0% | 73.3% / 80.9% / 31.7% | 64.5% / 74.9% / 42.0% | 0.577 | 54.9% |
| 1000 | dng-sidecar | 54 | 36.8% / 51.5% / 57.5% | 0.0% / 0.0% / 100.0% | 5.9% / 10.0% / 91.1% | 13.2% / 13.1% / 86.7% | 36.8% / 46.9% / 60.4% | 30.9% / 42.3% / 66.8% | 0.556 | 55.9% |
| 1000 | dng-output | 16 | 68.8% / 80.7% / 38.0% | 0.0% / 0.0% / 100.0% | 6.2% / 6.4% / 90.9% | 9.4% / 22.9% / 80.0% | 71.9% / 84.4% / 32.0% | 68.8% / 78.9% / 42.8% | 0.477 | 46.9% |
| 2000 | arw | 1786 | 70.5% / 80.9% / 35.6% | 7.3% / 12.1% / 92.0% | 5.3% / 7.9% / 92.0% | 26.6% / 25.9% / 74.1% | 75.4% / 82.5% / 29.2% | 67.4% / 77.0% / 38.8% | 0.570 | 54.4% |
| 2000 | dng-sidecar | 59 | 41.6% / 57.4% / 53.4% | 0.0% / 0.0% / 100.0% | 6.5% / 8.2% / 91.2% | 14.3% / 14.4% / 83.8% | 39.0% / 53.8% / 55.9% | 32.5% / 50.3% / 62.1% | 0.556 | 51.9% |
| 2000 | dng-output | 23 | 67.5% / 76.0% / 46.0% | 0.0% / 0.0% / 100.0% | 5.0% / 5.8% / 94.8% | 15.0% / 21.5% / 77.4% | 65.0% / 80.2% / 43.6% | 60.0% / 74.4% / 55.5% | 0.464 | 42.5% |
| 5000 | arw | 1539 | 77.5% / 84.9% / 29.9% | 7.2% / 11.9% / 91.5% | 5.3% / 7.8% / 92.0% | 26.8% / 25.7% / 74.8% | 80.1% / 85.3% / 26.3% | 74.2% / 81.2% / 33.7% | 0.578 | 54.8% |
| 5000 | dng-sidecar | 63 | 53.4% / 67.9% / 44.8% | 0.0% / 0.0% / 100.0% | 5.7% / 8.7% / 92.1% | 17.0% / 17.3% / 81.6% | 50.0% / 63.8% / 46.8% | 43.2% / 60.9% / 52.3% | 0.529 | 54.5% |
| 5000 | dng-output | 24 | 75.0% / 80.0% / 38.6% | 0.0% / 0.0% / 100.0% | 6.8% / 5.3% / 95.8% | 13.6% / 20.7% / 82.7% | 70.5% / 78.7% / 45.7% | 70.5% / 76.7% / 47.9% | 0.484 | 43.2% |

