# Provisional cuts of the good-photo mark (Step 4)

The first cuts of `goodPhoto` (`crates/app/ui/src/focus.ts`), chosen by hand
on 2026-10-09 from the re-dump's distributions, to be tuned in Step 5 from
what the user sees in the app. Not from labels.

## The re-dump

`riffle-cli features` built from `main` at 69d49ccd (after #741, so the
`eye_focus` is the mesh-based cue of #733 and the EAR and pose are the AF
face's), 24 threads, over six sidecar-labeled ARW folders:
`2026-09-19`, `2026-09-27-a`, `2026-10-03`, `2026-07-11`, `2026-08-08`,
`2026-07-24`. The dumps (`<folder>.tsv`, `<folder>.output.txt`, `<folder>.log`)
and the binary are under `D:\Photos\tests\2026-10-09-good-mark\dump\`; no
file failed. [provisional.py](provisional.py) reads them:

```sh
python provisional.py /d/Photos/tests/2026-10-09-good-mark/dump 0.998 0.3 60 45
```

**The dump's EAR and pose are the index's.** The app's index held stored
eyes for one folder only, `2026-09-27-c` (unfinished, so not in the data
set); its dump (`D:\Photos\tests\2026-10-09-good-mark\check\`) matches the
index on all 1815 files: `eye_focus`, `eyes_ear` and yaw / pitch / roll
alike, `None` where the index has `NULL`. The dump's eyes path picks the
same face as the cue's mesh, so `features` needed no new column.

## Distributions over the faced AF frames

8915 faced AF frames (`af` with a cue face); 1207 of them picks (13.5%). The
EAR and the pose exist where the mesh ran (faces of 60 px and more): 7261
frames with an EAR, 7213 with a pose. Pick / non-pick for information only.

| Feature | Set | n | p5 | p10 | p25 | p50 | p75 | p90 | p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| eye_focus | all | 8915 | 0.384 | 0.701 | 0.878 | 0.940 | 0.997 | 0.999 | 1.000 |
| eye_focus | picks | 1207 | 0.539 | 0.817 | 0.908 | 0.963 | 0.998 | 1.000 | 1.000 |
| eye_focus | non-picks | 7708 | 0.373 | 0.678 | 0.873 | 0.937 | 0.996 | 0.999 | 1.000 |
| EAR | all | 7261 | 0.093 | 0.137 | 0.214 | 0.290 | 0.350 | 0.408 | 0.459 |
| EAR | picks | 1046 | 0.115 | 0.160 | 0.235 | 0.305 | 0.358 | 0.409 | 0.456 |
| EAR | non-picks | 6215 | 0.091 | 0.134 | 0.211 | 0.288 | 0.348 | 0.407 | 0.459 |
| abs(yaw) | all | 7213 | 2.4 | 4.7 | 11.5 | 25.9 | 45.8 | 65.4 | 74.6 |
| abs(yaw) | picks | 1037 | 2.2 | 4.3 | 11.8 | 25.1 | 43.9 | 65.0 | 74.3 |
| abs(yaw) | non-picks | 6176 | 2.4 | 4.7 | 11.4 | 26.1 | 46.3 | 65.5 | 74.6 |
| abs(pitch) | all | 7213 | 1.4 | 2.6 | 6.5 | 14.3 | 26.3 | 37.5 | 43.9 |
| abs(pitch) | picks | 1037 | 1.5 | 2.9 | 7.1 | 14.2 | 25.7 | 35.9 | 40.8 |
| abs(pitch) | non-picks | 6176 | 1.4 | 2.6 | 6.4 | 14.4 | 26.5 | 37.7 | 44.3 |

Two things stand out. The mesh `eye_focus` is saturated: half the faced AF
frames are at 0.94 or more and a quarter at 0.997 or more, so today's
`candidate` (0.772) is 87.4% of them. And picks and non-picks have nearly the
same distributions on every feature, as Steps 2-3 found: the features say
"technically fine" for most non-picks too.

## The starting point the plan named does not narrow

At `eye_focus` >= 0.9, EAR >= 0.2 (open probability 0.861), |yaw| <= 60,
|pitch| <= 45 the AND marks 41.8% of the faced AF frames, against today's
87.4%: half the icons gone, still far from "a few percent". Each cut tried,
the total over the six folders (AND = the share of faced AF frames marked):

| `eye_focus` | EAR | \|yaw\| | \|pitch\| | AND | Picks among AND |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0.9 | 0.2 | 60 | 45 | 41.8% (3727) | 16.6% |
| 0.95 | 0.2 | 60 | 45 | 30.5% (2722) | 18.2% |
| 0.99 | 0.2 | 60 | 45 | 24.7% (2201) | 18.9% |
| 0.99 | 0.25 | 60 | 45 | 20.3% (1807) | 20.1% |
| 0.99 | 0.25 | 45 | 45 | 18.7% (1668) | 19.8% |
| 0.99 | 0.25 | 30 | 30 | 12.6% (1127) | 19.6% |
| 0.99 | 0.3 | 60 | 45 | 13.7% (1217) | 21.2% |
| 0.995 | 0.3 | 60 | 45 | 12.2% (1088) | 21.3% |
| **0.998** | **0.3** | **60** | **45** | **9.0% (801)** | **21.0%** |
| 0.999 | 0.3 | 60 | 45 | 6.2% (552) | 21.4% |

## The provisional cuts

- **`GOOD_EYE_FOCUS` = 0.998** (the 79th percentile of `eye_focus`): the cue
  is saturated, so "high" has to sit in its top fifth to select at all.
- **`GOOD_EYE_EAR` = 0.30** (the 54th percentile of the EAR; open probability
  0.991, against 0.964 at 0.25 and 0.861 at 0.20): "wide open", about the
  pick median (0.305).
- **`GOOD_MAX_YAW` = 60, `GOOD_MAX_PITCH` = 45** (86% / 96% of the poses
  within): the loose form, only extreme turns excluded, roll unused.

With the loose pose cut the AND marks 9.0% of the faced AF frames, under the
~10% the plan named, so the loose form stays for the user's look. The
user's preference as recorded in the plan: the loose pose cut stays if the
AND is selective enough and the user does not see turned faces marked;
otherwise the tight "both eyes visible" cut, about |yaw| <= 45 (measured in
Step 5 if needed), is the next thing to try. At these cuts it would mark
8.3% (the pose cut is the weakest of the three once the eye cuts are this
high).

## Per folder at the provisional cuts

Shares of the faced AF frames. Before: today's green icon (`candidate`).
After: the AND of the three cuts (`goodPhoto`). The single-cut columns count
the frames that pass that cut alone.

| Folder | Faced AF | Before (`candidate`) | `eye_focus` | EAR | Pose | After (AND) | Picks among AND | Picks among `candidate` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-07-11` | 441 | 98.4% | 36.5% | 32.9% | 81.0% | 7.9% (35) | 25.7% | 12.4% |
| `2026-07-24` | 173 | 98.3% | 34.1% | 33.5% | 65.3% | 16.2% (28) | 28.6% | 17.1% |
| `2026-08-08` | 367 | 95.9% | 35.1% | 24.5% | 92.1% | 7.9% (29) | 31.0% | 17.3% |
| `2026-09-19` | 1952 | 86.9% | 25.7% | 45.5% | 72.1% | 13.4% (261) | 17.6% | 14.2% |
| `2026-09-27-a` | 4367 | 84.3% | 13.2% | 38.1% | 60.4% | 6.6% (288) | 26.0% | 15.4% |
| `2026-10-03` | 1615 | 90.0% | 27.4% | 30.3% | 72.3% | 9.9% (160) | 13.1% | 10.4% |
| **Total** | 8915 | 87.4% | 21.0% | 37.4% | 67.5% | 9.0% (801) | 21.0% | 14.2% |

The mark is selective now (7-16% per folder) but its picks share rises only
from 14% to 21%: as the plan's Purpose says, a non-pick is unlabeled, so
this column is no measure of "not a miss"; the user's look is.

## The openness anchor (Step 4b)

The meta pane's eyes row shows an openness, 0 at `EYES_CLOSED_EAR` (0.137)
and below to 100 at `EYES_WIDE_OPEN_EAR` and above, linear between
(`eyesOpenness` in `crates/app/ui/src/focus.ts`). The 100 anchor is the
**90th percentile of the EAR over the faced AF frames with an EAR**: 0.4076
over 7261 frames of the re-dump above (the `EAR | all` row's p90), rounded
to **0.41**. It is a display scale, independent of `GOOD_EYE_EAR` (0.30,
openness 60).

## The fair tier (Step 4c)

A second tier below good, `photoTier`'s `"fair"` in
`crates/app/ui/src/focus.ts`: a focus candidate that is not good but clears
looser cuts, chosen so that good + fair mark about 20% of the faced AF frames
pooled. [provisional.py](provisional.py) takes the fair cuts as four more
arguments:

```sh
python provisional.py /d/Photos/tests/2026-10-09-good-mark/dump 0.998 0.3 60 45 0.99 0.25 60 45
```

| Fair `eye_focus` | Fair EAR | Fair \|yaw\| | Fair \|pitch\| | Fair | Good + fair |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0.995 | 0.25 | 60 | 45 | 9.2% (817) | 18.1% |
| **0.99** | **0.25** | **60** | **45** | **11.3% (1006)** | **20.3%** |
| 0.99 | 0.25 | 45 | 45 | 10.4% (927) | 19.4% |
| 0.99 | 0.24 | 60 | 45 | 12.3% (1097) | 21.3% |
| 0.98 | 0.25 | 60 | 45 | 12.8% (1139) | 21.8% |
| 0.99 | 0.22 | 60 | 45 | 14.3% (1275) | 23.3% |

- **`FAIR_EYE_FOCUS` = 0.99** (the 67th percentile of `eye_focus`).
- **`FAIR_EYE_EAR` = 0.25** (the 35th percentile of the EAR; open probability
  0.964, openness 41).
- **`FAIR_MAX_YAW` = 60, `FAIR_MAX_PITCH` = 45**, the good tier's loose pose
  cut: tightening yaw to 45 moves the total by under one point.

Per folder at these cuts (shares of the faced AF frames):

| Folder | Faced AF | Good | Fair | Good + fair | Picks among good | Picks among fair |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-07-11` | 441 | 7.9% (35) | 19.0% (84) | 27.0% | 25.7% | 16.7% |
| `2026-07-24` | 173 | 16.2% (28) | 10.4% (18) | 26.6% | 28.6% | 27.8% |
| `2026-08-08` | 367 | 7.9% (29) | 23.4% (86) | 31.3% | 31.0% | 18.6% |
| `2026-09-19` | 1952 | 13.4% (261) | 8.3% (162) | 21.7% | 17.6% | 19.8% |
| `2026-09-27-a` | 4367 | 6.6% (288) | 9.7% (424) | 16.3% | 26.0% | 25.2% |
| `2026-10-03` | 1615 | 9.9% (160) | 14.4% (232) | 24.3% | 13.1% | 9.1% |
| **Total** | 8915 | 9.0% (801) | 11.3% (1006) | 20.3% | 21.0% | 19.4% |

As for good, the picks columns are for information only.

## The Step 5 cuts (from the user's stars)

The user scored a 60-frame sample (20 good, 20 fair, 20 in neither tier at
the Step 4c cuts, `D:\photos\samples\ARW\good-mark-2026-10-09\`, 1-5 stars in
its `.xmp` sidecars) and named three 1-star good frames, each a different
failure: `2026-07-11__DSC2638` (a baby lying down, the face rotated far
in-plane, the mesh fitted upright off the face), `2026-07-11__DSC2827`
(turned, yaw -49, the foreshortened eye inflating the EAR) and
`2026-09-19__DSC3345` (the AF face cut by the image's left edge and out of
focus). Step 5 adds two exclusions and tightens the pose cut for both tiers;
the cue, the mesh, `EYES_CLOSED_EAR` and `CANDIDATE_LOGIT` are unchanged.

**Two new measures**, computed in pass 2 from the mesh the cue already runs
(`crates/core/src/candidate.rs`) and stored in `files.eye_offset` /
`files.edge_gap` (`SCHEMA_VERSION` 19, `FACES_VERSION` 8), so the thresholds
stay in `crates/app/ui/src/focus.ts`:

- `eye_offset` (`mesh_eye_offset`): the larger distance between a mesh eyelid
  contour's center and YuNet's eye landmark of the same face, in the pairing
  of the eyes that keeps it smaller, over the face box's longer side.
- `edge_gap` (`edge_gap`): the smallest distance from the face box, or from
  either mesh eye region grown as the cue grows it, to an image edge, over the
  box's longer side; negative outside. "Extends outside the image" alone (a
  gap under 0) does not catch `DSC3345`: YuNet's box of a face the frame cuts
  covers only what is visible, and ends 1 px inside the edge (gap 0.008).

The re-dump: the same six folders with a `riffle-cli features` that prints
the two columns, under `D:\Photos\tests\2026-10-09-good-mark\step5\dump\`
(the sample's dump is `step5\samples.tsv`, the output of the script below
`step5\tune.txt`). Every other column matches the Step 4 re-dump (the Step 4c
rule gives the same 20.3% and the same sample tiers). [tune.py](tune.py)
reads them:

```sh
python tune.py /d/Photos/tests/2026-10-09-good-mark/step5/dump \
  /d/Photos/tests/2026-10-09-good-mark/step5/samples.tsv \
  /d/photos/samples/ARW/good-mark-2026-10-09 30 0.1 0.02
```

| Measure | n | p1 | p2 | p5 | p10 | p25 | p50 | p75 | p90 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| eye_offset | 7261 | 0.013 | 0.016 | 0.021 | 0.027 | 0.040 | 0.062 | 0.111 | 0.189 | 0.246 | 0.359 |
| edge_gap | 7261 | 0.222 | 0.370 | 0.721 | 1.313 | 3.038 | 4.537 | 5.669 | 6.556 | 7.051 | 7.818 |

### The cuts chosen

- **`MAX_EYE_OFFSET` = 0.10.** On the sample every tier frame whose mesh sits
  on the face is under 0.09 (the highest, `DSC3254` at 0.086 and `DSC2827` at
  0.087); `DSC2638` is at 0.125, and the other tier frames above 0.1 scored
  1-3 stars (`DSC6920` 0.177 and `DSC5035` 0.220 at 1-2, `DSC4673` 0.329 at 3,
  `DSC3345` 0.352 at 1). Over the re-dump it excludes 28.6% of the frames with
  a mesh, mostly turned or soft ones the cuts drop anyway, but only 100 of the
  1321 frames (7.6%) the cuts alone would put in a tier. 0.12 gives the same
  sample tiers; 0.08 also drops a 3-star fair frame (`DSC3254`, 0.086) and
  1.0 point of the re-dump (12.7%, against 14.0% at 0.12); 0.10 is the round
  value between.
- **`MIN_EDGE_GAP` = 0.02.** `DSC3345` is at 0.008. Of the faced AF frames
  with a mesh, 23 (0.3%) are under 0.02 and 3 of them would be tiered; a look
  at the frames between 0 and 0.06 found faces touching the edge at 0.001-0.013
  (`2026-07-11\_DSC3003`, forehead cut by the top edge; `2026-07-24\_DSC4992`)
  and whole faces near it from about 0.03 (`2026-09-19\_DSC3043`,
  `2026-09-27-a\_DSC6862`). The sample's two large faces near the top edge,
  `DSC4884` (0.093) and `DSC0148` (0.065), are whole and stay above it (a
  first try that tested the 1.25x face crop the mesh is fitted on flagged
  both, so it was dropped).
- **`GOOD_MAX_YAW` = `FAIR_MAX_YAW` = 30**, the "both eyes visible" form the
  plan kept as the fallback (56% of the 7213 poses are within). On the sample
  the tiers keep their mean stars from 30 down to 20 and lose them from 35 up
  (35 lets in `DSC2373`, 2 stars, at yaw -31; 40 also `DSC2050`, 1 star, at
  -37), and over the re-dump 25 instead of 30 marks 1.7 points fewer frames:

| \|yaw\| cut | Good n | Good mean | Fair n | Fair mean | Tiered 1-2 star | Re-dump good + fair |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 9 | 4.00 | 11 | 3.73 | 1 | 10.3% |
| 25 | 10 | 3.90 | 13 | 3.69 | 1 | 12.0% |
| **30** | **10** | **3.90** | **13** | **3.69** | **1** | **13.7%** |
| 35 | 11 | 3.91 | 16 | 3.56 | 2 | 15.0% |
| 40 | 13 | 3.85 | 18 | 3.39 | 3 | 16.3% |
| 45 | 14 | 3.79 | 18 | 3.39 | 3 | 17.0% |
| 60 | 17 | 3.53 | 18 | 3.39 | 4 | 18.1% |

The eye cuts (`GOOD_EYE_FOCUS` 0.998, `GOOD_EYE_EAR` 0.30, `FAIR_EYE_FOCUS`
0.99, `FAIR_EYE_EAR` 0.25) and the pitch cut (45) stay.

### The sample, re-scored

| Rule | Tier | n | Mean stars | 1 | 2 | 3 | 4 | 5 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Step 4c | good | 20 | 3.20 | 3 | 1 | 8 | 5 | 3 |
| Step 4c | fair | 20 | 3.25 | 2 | 2 | 9 | 3 | 4 |
| Step 4c | none | 20 | 2.40 | 4 | 7 | 6 | 3 | 0 |
| Step 5 | good | 10 | 3.90 | 0 | 0 | 4 | 3 | 3 |
| Step 5 | fair | 13 | 3.69 | 0 | 1 | 6 | 2 | 4 |
| Step 5 | none | 37 | 2.43 | 9 | 9 | 13 | 6 | 0 |

Spearman of the tier against the stars: 0.29 before, 0.54 now. Of the eight
1-2 star frames the Step 4c rule tiered, one stays: `2026-09-19__DSC1805`
(fair, 2 stars; yaw 18.5, pitch -21.8, eye offset 0.037, nothing the
exclusions see). Seventeen frames left a tier: the three named ones, the
other four of 1-2 stars (`DSC5035` and `DSC6920` by the eye offset,
`DSC2373` and `DSC2050` by the yaw cut), and ten of 3-4 stars, nine of them
by the yaw cut (the cost: `DSC0173`, `DSC4884` and `DSC0148` at 4 stars, yaw
39, -31 and -35) and one by the eye offset (`DSC4673`, 3 stars).

The three named frames, now in neither tier:

| Frame | eye_focus | EAR | yaw | pitch | eye_offset | edge_gap | Excluded by |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `2026-07-11__DSC2638` | 0.999 | 0.360 | 3.0 | -11.8 | **0.125** | 0.224 | the eye offset |
| `2026-07-11__DSC2827` | 0.998 | 0.303 | **-48.6** | 5.2 | 0.087 | 1.692 | the yaw cut |
| `2026-09-19__DSC3345` | 0.998 | 0.314 | -15.3 | 2.7 | **0.352** | **0.008** | the edge gap and the eye offset |

### Per folder at the Step 5 cuts

Shares of the faced AF frames (the picks columns for information only, as
before):

| Folder | Faced AF | Good | Fair | Good + fair | Step 4c good + fair | Picks among good | Picks among fair |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-07-11` | 441 | 3.4% (15) | 10.4% (46) | 13.8% | 27.0% | 33.3% | 17.4% |
| `2026-07-24` | 173 | 11.0% (19) | 6.4% (11) | 17.3% | 26.6% | 31.6% | 36.4% |
| `2026-08-08` | 367 | 4.6% (17) | 17.7% (65) | 22.3% | 31.3% | 41.2% | 15.4% |
| `2026-09-19` | 1952 | 8.9% (173) | 5.3% (103) | 14.1% | 21.7% | 15.0% | 16.5% |
| `2026-09-27-a` | 4367 | 3.9% (169) | 6.1% (268) | 10.0% | 16.3% | 26.6% | 22.0% |
| `2026-10-03` | 1615 | 8.7% (140) | 12.0% (194) | 20.7% | 24.3% | 12.9% | 8.2% |
| **Total** | 8915 | 6.0% (533) | 7.7% (687) | 13.7% | 20.3% | 20.1% | 16.6% |

The 60 frames are a small sample scored by one person and drawn from the
Step 4c tiers, so the means are a direction, not a precision figure; no
labeled `miss-truth.md` set was made.

## One tier (Step 5b)

The user rated a second batch of 120 frames drawn without regard to the
tiers (seed 20261010, twelve folders; `samples-manifest-batch2.tsv` and
`samples-scored-batch2.tsv` under `D:\Photos\tests\2026-10-09-good-mark\`).
On it the good / fair split does not separate 4-5 stars (good 22% at 4+,
fair 55%, none 22%) but passing (3 stars or more) is separated: good 9 / 9,
fair 9 / 11, none 64 / 100; on the first batch at the Step 5 cuts good
10 / 10, fair 12 / 13. Good and fair together: 40 / 43 pass, no 1-star
frame. So the two tiers fold into one, `good`, meaning "not a miss".

The folded cuts in `crates/app/ui/src/focus.ts` are the Step 5 fair cuts.
The Step 5 good cuts were stricter on the eyes only, with the same pose cut
and exclusions, so the folded tier marks exactly what good or fair marked:

- **`GOOD_EYE_FOCUS` = 0.99** (the re-dump's 67th percentile).
- **`GOOD_EYE_EAR` = 0.25** (the 35th percentile; open probability 0.964,
  openness 41).
- **`GOOD_MAX_YAW` = 30, `GOOD_MAX_PITCH` = 45.**
- **`MAX_EYE_OFFSET` = 0.10, `MIN_EDGE_GAP` = 0.02**, unchanged.

Per folder, the share of the faced AF frames the folded tier marks (the Step
5 good + fair column above):

| Folder | Faced AF | Good |
| --- | ---: | ---: |
| `2026-07-11` | 441 | 13.8% (61) |
| `2026-07-24` | 173 | 17.3% (30) |
| `2026-08-08` | 367 | 22.3% (82) |
| `2026-09-19` | 1952 | 14.1% (276) |
| `2026-09-27-a` | 4367 | 10.0% (437) |
| `2026-10-03` | 1615 | 20.7% (334) |
| **Total** | 8915 | 13.7% (1220) |

The cuts are re-tuned on all 180 rated frames after the face mesh's roll
correction lands.

## The Step 7 cuts (re-tuned on 180 rated frames)

The face-mesh roll correction was not adopted
(`docs/plans/20261010-mesh-roll/`), so the values the rule reads are those of
Step 5. The 180 rated frames were re-dumped with `riffle-cli features` and
`riffle-cli meshfit` (for YuNet's eye distance over the box side) built from
`main` at c1843389, under `D:\Photos\tests\2026-10-09-good-mark\step7\`
(`samples.tsv`, `meshfit.tsv`, the script's output `retune.txt`); every
`features` column the rule reads matches the Step 5 sample dump and the batch
2 manifest. The folder shares use the Step 5 dumps of six folders
(`step5\dump\`) and the batch 2 dumps of six more (`batch2\dump\`), 19422
faced AF frames. [retune.py](retune.py) reads them:

```sh
T=/d/Photos/tests/2026-10-09-good-mark
python retune.py $T/step7/samples.tsv $T/step7/meshfit.tsv \
  /d/photos/samples/ARW/good-mark-2026-10-09 $T/samples-manifest.tsv \
  $T/step5/dump $T/batch2/dump
```

The stars: batch 1 (60, drawn from the Step 4c tiers, used for Step 5) 9 /
10 / 22 / 12 / 7 frames at 1-5 stars, batch 2 (120, drawn without regard to
the tiers) 11 / 28 / 51 / 16 / 14. Per variant (one change from the Step 5b
cuts unless several are named): the frames marked, the share of 3+ stars
among them, the 1- and 2-star frames among them, the share of the batch
marked, the share of its 3+ star frames marked.

| Variant | B1 n | B1 3+ | B1 1 / 2 | B2 n | B2 3+ | B2 1 / 2 | Pooled n | Pooled 3+ | Pooled 1 / 2 | Marked | 3+ captured |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Step 5b cuts | 23 | 95.7% | 0 / 1 | 20 | 90.0% | 0 / 2 | 43 | 93.0% | 0 / 3 | 23.9% | 32.8% |
| \|yaw\| <= 35 | 27 | 92.6% | 0 / 2 | 24 | 91.7% | 0 / 2 | 51 | 92.2% | 0 / 4 | 28.3% | 38.5% |
| \|yaw\| <= 40 | 31 | 90.3% | 1 / 2 | 26 | 92.3% | 0 / 2 | 57 | 91.2% | 1 / 4 | 31.7% | 42.6% |
| \|yaw\| <= 45 | 32 | 90.6% | 1 / 2 | 27 | 88.9% | 0 / 3 | 59 | 89.8% | 1 / 5 | 32.8% | 43.4% |
| `eye_focus` >= 0.98 / 0.97 | 23 | 95.7% | 0 / 1 | 20 | 90.0% | 0 / 2 | 43 | 93.0% | 0 / 3 | 23.9% | 32.8% |
| `eye_focus` >= 0.95 | 23 | 95.7% | 0 / 1 | 22 | 90.9% | 0 / 2 | 45 | 93.3% | 0 / 3 | 25.0% | 34.4% |
| `eye_focus` >= 0.90 | 23 | 95.7% | 0 / 1 | 26 | 92.3% | 0 / 2 | 49 | 93.9% | 0 / 3 | 27.2% | 37.7% |
| `eye_offset` <= 0.15 | 24 | 91.7% | 1 / 1 | 23 | 82.6% | 0 / 4 | 47 | 87.2% | 1 / 5 | 26.1% | 33.6% |
| `eye_offset` <= 0.20 | 25 | 88.0% | 2 / 1 | 23 | 82.6% | 0 / 4 | 48 | 85.4% | 2 / 5 | 26.7% | 33.6% |
| no `eye_offset` cut | 27 | 85.2% | 2 / 2 | 23 | 82.6% | 0 / 4 | 50 | 84.0% | 2 / 6 | 27.8% | 34.4% |
| `eye_offset` only at YuNet eye distance >= 0.15 or 0.20 | 25 | 88.0% | 1 / 2 | 20 | 90.0% | 0 / 2 | 45 | 88.9% | 1 / 4 | 25.0% | 32.8% |
| no `edge_gap` cut | 23 | 95.7% | 0 / 1 | 20 | 90.0% | 0 / 2 | 43 | 93.0% | 0 / 3 | 23.9% | 32.8% |
| \|yaw\| <= 35, `eye_focus` >= 0.95 | 27 | 92.6% | 0 / 2 | 26 | 92.3% | 0 / 2 | 53 | 92.5% | 0 / 4 | 29.4% | 40.2% |
| **\|yaw\| <= 35, `eye_focus` >= 0.90** | **27** | **92.6%** | **0 / 2** | **31** | **93.5%** | **0 / 2** | **58** | **93.1%** | **0 / 4** | **32.2%** | **44.3%** |
| \|yaw\| <= 35, `eye_focus` >= 0.772 (the candidate cut) | 27 | 92.6% | 0 / 2 | 31 | 93.5% | 0 / 2 | 58 | 93.1% | 0 / 4 | 32.2% | 44.3% |
| \|yaw\| <= 40, `eye_focus` >= 0.90 | 31 | 90.3% | 1 / 2 | 34 | 94.1% | 0 / 2 | 65 | 92.3% | 1 / 4 | 36.1% | 49.2% |
| \|yaw\| <= 35, `eye_focus` >= 0.90, EAR >= 0.22 | 28 | 92.9% | 0 / 2 | 39 | 89.7% | 0 / 4 | 67 | 91.0% | 0 / 6 | 37.2% | 50.0% |
| \|yaw\| <= 35, `eye_focus` >= 0.90, `eye_offset` <= 0.15 | 29 | 86.2% | 1 / 3 | 35 | 88.6% | 0 / 4 | 64 | 87.5% | 1 / 7 | 35.6% | 45.9% |
| \|yaw\| <= 35, `eye_focus` >= 0.90, offset gated at 0.20 | 29 | 86.2% | 1 / 3 | 33 | 90.9% | 1 / 2 | 62 | 88.7% | 2 / 5 | 34.4% | 45.1% |

- **The yaw cut goes to 35.** It adds seven 3-5 star frames turned 31-35 deg
  (among them `DSC4884` and `DSC0148`, the 4-star frames Step 5 named as
  the cost of 30) and one 2-star frame (`DSC2373`, yaw -31.2). 40 lets in
  the 1-star `2026-10-03__DSC2050` (yaw -36.6).
- **The `eye_focus` cut goes to 0.90.** From 0.99 down to 0.95 it adds two
  3-star frames, to 0.90 four more, all 5 stars (at |yaw| <= 35 five, the
  fifth 3 stars); all seven rated frames from 0.90 to 0.99 that clear the
  other cuts at |yaw| <= 35 pass. Below 0.90 no
  rated frame clears the other cuts (the 2-star frames there, such as
  `DSC5396` at 0.899 and `DSC3385` at 0.879, miss only the eye offset), so
  the sample cannot tell 0.90 from the candidate cut (0.772); 0.90 is kept,
  marking 1.4 points fewer of the folders' faced AF frames. The gain below
  0.99 rests on batch 2 alone: batch 1 was drawn from the Step 4c tiers and
  has no frame between 0.90 and 0.99 that clears the other cuts.
- **`MAX_EYE_OFFSET` stays 0.10.** Every way of loosening it lets a 1-star
  frame in: 0.15 / 0.20 / none the misfit `2026-07-11__DSC2638` (0.125,
  YuNet eye distance 0.247, so gating does not save it either at 0.20) and
  at 0.20 also `2026-09-27-a__DSC6920` (0.177); applying it only where
  YuNet's eye distance is at least 0.15 or 0.20 lets `DSC6920` in (eye
  distance 0.057: YuNet sees a profile, the mesh a face turned 27 deg), and
  at the chosen cuts also `2026-09-27-b__DSC8931` (1 star, 0.196, eye
  distance 0.027). At the chosen cuts the offset alone keeps 12 frames out:
  three 1-star (`DSC2638`, `DSC6920`, `DSC8931`), five 2-star and four of
  3-4 stars. So on these 180 frames the frames the offset drops are mostly
  not fine ones; the
  mesh-roll finding (most frames over 0.10 have a fitted mesh) is about
  frames the other cuts drop anyway. A mesh-fit check that does not read
  YuNet's eye points needs a new pass-2 value and was not tried.
- **The edge exclusion stays.** Removing it changes nothing on the 180
  (`DSC3345` is also over the eye offset) and marks 0.3 points more of
  `2026-07-11`, where it drops the faces the frame cuts.
- **The EAR cut stays 0.25.** 0.22 adds nine frames, two of them 2 stars.

The 1-2 star frames the chosen cuts mark, all 2 stars:
`2026-07-05__DSC1180` (`eye_focus` 0.990, yaw -1.4, EAR 0.281),
`2026-07-05__DSC2385` (0.998, -21.7, 0.285), `2026-09-19__DSC1805` (0.999,
18.5, 0.256) and `2026-09-19__DSC2373` (0.996, -31.2, 0.323). The three
frames the user named in Step 5 stay out (`DSC2638` and `DSC3345` by the eye
offset, `DSC2827` by the yaw at -48.6).

Per folder, the share of the faced AF frames marked (the first six folders
are the Step 5 re-dump, the other six the batch 2 dumps):

| Folder | Faced AF | Step 5b cuts | Step 7 cuts |
| --- | ---: | ---: | ---: |
| `2026-07-11` | 441 | 13.8% | 18.6% |
| `2026-07-24` | 173 | 17.3% | 22.0% |
| `2026-08-08` | 367 | 22.3% | 31.1% |
| `2026-09-19` | 1952 | 14.1% | 25.1% |
| `2026-09-27-a` | 4367 | 10.0% | 18.0% |
| `2026-10-03` | 1615 | 20.7% | 30.0% |
| `2026-06-14` | 1360 | 5.1% | 11.7% |
| `2026-07-05` | 1231 | 6.8% | 13.7% |
| `2026-07-18` | 1196 | 24.3% | 30.2% |
| `2026-08-22` | 2481 | 12.8% | 20.2% |
| `2026-08-29` | 3157 | 7.2% | 14.0% |
| `2026-09-27-b` | 1082 | 11.7% | 18.1% |
| The six Step 5 folders | 8915 | 13.7% | 22.4% |
| The six batch 2 folders | 10507 | 10.6% | 17.4% |
| **Total** | 19422 | 12.0% | 19.7% |
