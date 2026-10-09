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
