# Step 3: the sharpness score on face-free frames

## The pre-registered criterion (written before the numbers)

The sharpness score "separates well" on face-free frames, and Step 4 ships a
Good / Bad from it, only if both hold for the form (absolute or within-burst
relative):

1. **Held-out AUC >= 0.70, pick vs non-pick.** The score has no fitted
   parameter and its direction (sharper is better) is fixed in advance, so
   the held-out AUC is the folder-stratified one: pick / non-pick pairs
   within a folder only (within a burst for the relative form).
2. **A cut whose precision is at least twice the base while it marks at
   least 20% of the face-free frames,** with the cut held out by folder:
   for each folder, the cut is chosen on the other folders (the highest
   precision among the cuts that mark at least 20% of their face-free
   frames) and applied to it; the pooled held-out precision is compared
   with the pooled base rate.
3. **The same on pick vs reject** wherever a folder has 30 or more
   face-free rejects.

Non-picks are unlabeled (failed, or fine and not chosen), so a pass on (1)
and (2) with too few rejects for (3) still counts as a pass; a failure on
(1) or (2) is a no-ship whatever (3) says.

## Data and how to reproduce

The 36 folders of the `20261008-burst-keep-score` data set (27 ARW, 2
sidecar-labeled and 7 `Output/`-labeled Leica DNG folders) plus
`D:\Photos\samples\ARW\good-mark-2026-10-09`, re-dumped on 2026-10-10 with a
release `riffle-cli features` built from `main` at `3df2fcb6` (Step 2 merged,
`FACES_VERSION` 9, 24 threads, no errors). The dumps, `dump.sh`, the CLI and
the hand-check crops are in `D:\Photos\tests\2026-10-10-facefree\`:

```sh
T=/d/Photos/tests/2026-10-10-facefree
sh $T/dump.sh $T/riffle-cli.exe 24 <folder>...
python -I facefree.py $T/dump                               # every table below
python -I facefree.py $T/dump $T/handcheck $T/riffle-cli.exe 200   # + the hand check
```

Against the burst-best-mark dumps of the same 36 folders (`22a309f9`, before
Step 2), the re-dump changes the cue `state` of 708 frames, all without an AF
point: 549 `Unknown` -> `Candidate` and 159 `Unknown` -> `NotCandidate` (Step
2's no-AF face). No ARW row changes state and no row changes `sharpness`. The
face-free grouping by `state == Unknown` matches the `af` / `cue_side` and
`noaf` / `judged_side` grouping on every frame (0 mismatches below).

Labels: the user named `good-mark-2026-10-09` as the folder with real reject
flags beyond `2026-09-19`. Its 180 frames now carry 20 rejects (`.dop`
`ShouldProcess = 1`, XMP `xmpDM:good="False"`, all 1-star) and 160 unflagged
2- to 5-star frames, but every one of them has a face at the AF point, so it
adds no face-free frame. `2026-09-13-a` and `2026-09-27-c` hold no reject
(`.dop` or XMP). On the face-free frames the data holds **9 rejects** (7 in
`2026-09-19`, 2 in `2026-09-13-b`), 549 picks and 3803 unlabeled non-picks.

## Reading

- **No form passes.** The folder-stratified AUC of pick vs non-pick is 0.579
  for the absolute score, 0.536 within face-free bursts and 0.598 within
  every burst (face-free frames against their whole burst), all under 0.70.
  No cut reaches twice the base: the absolute cut held out by folder is
  `abs >= 200` in all 34 folders, marking 51.4% at 14.7% precision against
  a 12.6% base (lift 1.17); the best in-sample absolute cut is `abs >= 1000`
  at lift 1.22 and 6.7% coverage. The relative cuts held out reach lift 1.25
  (face-free bursts, `rel >= 0.9`) and 0.95 (every burst).
- **Singles are worse than random.** On the 391 face-free singles the
  stratified AUC is 0.489 and every cut from 200 up has a lower pick share
  than the frames below it.
- **Pick vs reject cannot be tested to the criterion.** No folder has 30
  face-free rejects (most: 7). On the 9 there are, the pooled AUC is 0.663
  and the within-folder one 0.622 (209 pairs); `abs >= 200` marks 3 of the 9.
- **Why.** On a face-free frame with an AF point the score is the texture
  under the point, not whether the subject is in focus: the hand check's
  `failed-27` (`2026-08-29/_DSC6421.ARW`, a pick, score 3.1) is a flat
  background under the AF point, and `marked-11`
  (`2026-09-27-a/_DSC7109.ARW`, not picked, 1316) a sharp jersey. This is
  the same finding as the burst-best-mark measurement (the burst's sharpest
  frame is good in 47.6% of bursts, random 45.8%).
- The earlier plan's face-free lifts (1.0 - 1.3 against non-picks) hold on
  the post-Step-2 set.

## Hand check

20 face-free frames at `abs >= 200` (the held-out cut) and 20 below it, seed
0, `riffle-cli crop` cuts of 512 px of the full-size embedded JPEG around
the AF point (the centre for a DNG without one) in
`D:\Photos\tests\2026-10-10-facefree\handcheck\` (the list with each frame's
score and labels is `list.tsv` there). 2 of the 20 marked frames are picks, 3
of the 20 below the cut are.

| Crop | Frame | Score | Pick |
| --- | --- | ---: | --- |
| `marked-00` | `2026-09-05/L1005149.DNG` | 481.4 | |
| `marked-01` | `2026-09-13-b/_DSC0006.ARW` | 270.4 | |
| `marked-02` | `2026-06-05/_DSC1231.ARW` | 262.1 | |
| `marked-03` | `2026-08-01/_DSC7654.ARW` | 245.0 | |
| `marked-04` | `2026-09-27-a/_DSC7602.ARW` | 478.3 | |
| `marked-05` | `2026-09-27-a/_DSC5445.ARW` | 333.6 | |
| `marked-06` | `2026-09-05/L1005290.DNG` | 2091.7 | |
| `marked-07` | `2026-08-08/_DSC0102.ARW` | 708.2 | |
| `marked-08` | `2026-09-27-a/_DSC4311.ARW` | 300.6 | |
| `marked-09` | `2026-08-29/_DSC5152.ARW` | 585.5 | |
| `marked-10` | `2026-07-31/_DSC6325.ARW` | 433.4 | |
| `marked-11` | `2026-09-27-a/_DSC7109.ARW` | 1316.4 | |
| `marked-12` | `2026-07-05/_DSC2305.ARW` | 287.9 | |
| `marked-13` | `2026-08-01/_DSC8647.ARW` | 453.5 | |
| `marked-14` | `2026-07-05/_DSC2307.ARW` | 362.4 | |
| `marked-15` | `2026-06-14/_DSC5316.ARW` | 205.1 | |
| `marked-16` | `2026-08-01/_DSC7231.ARW` | 389.7 | |
| `marked-17` | `2026-09-27-b/_DSC9444.ARW` | 641.2 | yes |
| `marked-18` | `2026-07-11/_DSC2632.ARW` | 215.3 | |
| `marked-19` | `2026-08-15/_DSC0593.ARW` | 221.8 | yes |
| `failed-20` | `2026-06-14/_DSC5036.ARW` | 7.2 | |
| `failed-21` | `2026-06-13/_DSC3631.ARW` | 8.1 | |
| `failed-22` | `2026-09-05/L1005192.DNG` | 142.4 | |
| `failed-23` | `2026-09-27-b/_DSC9505.ARW` | 159.9 | |
| `failed-24` | `2026-06-14/_DSC5509.ARW` | 86.2 | |
| `failed-25` | `2026-09-13-b/_DSC0602.ARW` | 197.5 | |
| `failed-26` | `2026-09-27-a/_DSC7333.ARW` | 182.9 | yes |
| `failed-27` | `2026-08-29/_DSC6421.ARW` | 3.1 | yes |
| `failed-28` | `2026-07-31/_DSC6104.ARW` | 173.9 | |
| `failed-29` | `2026-09-27-b/_DSC9722.ARW` | 122.7 | |
| `failed-30` | `2026-09-27-a/_DSC7713.ARW` | 193.6 | yes |
| `failed-31` | `2026-08-02/_DSC9940.ARW` | 23.3 | |
| `failed-32` | `2026-06-05/_DSC1644.ARW` | 159.8 | |
| `failed-33` | `2026-06-05/_DSC0474.ARW` | 35.9 | |
| `failed-34` | `2026-06-13/_DSC4447.ARW` | 27.9 | |
| `failed-35` | `2026-09-27-a/_DSC4082.ARW` | 3.1 | |
| `failed-36` | `2026-03-29/L1001411.DNG` | 14.0 | |
| `failed-37` | `2026-10-03/_DSC2719.ARW` | 122.1 | |
| `failed-38` | `2026-09-05/L1005415.DNG` | 70.2 | |
| `failed-39` | `2026-08-01/_DSC8607.ARW` | 33.5 | |

## Numbers (`facefree.py` output)

### The face-free set

| Folder | Kind | Frames | Face-free | AF, no face near it | No AF, no face | Grouping mismatches | Picks | Rejects | Non-picks (unlabeled) | Pick base |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-01-08` | dng-output | 3 | 3 | 0 | 3 | 0 | 2 | 0 | 1 | 66.7% |
| `2026-02-14` | dng-output | 12 | 1 | 0 | 1 | 0 | 0 | 0 | 1 | 0.0% |
| `2026-03-29` | dng-output | 155 | 38 | 0 | 38 | 0 | 6 | 0 | 32 | 15.8% |
| `2026-04-12` | dng-output | 10 | 6 | 0 | 6 | 0 | 3 | 0 | 3 | 50.0% |
| `2026-04-18` | dng-output | 3 | 3 | 0 | 3 | 0 | 1 | 0 | 2 | 33.3% |
| `2026-05-22` | dng-output | 37 | 37 | 37 | 0 | 0 | 19 | 0 | 18 | 51.4% |
| `2026-05-26` | dng-output | 163 | 33 | 0 | 33 | 0 | 0 | 0 | 33 | 0.0% |
| `2026-06-05` | arw | 1337 | 429 | 429 | 0 | 0 | 44 | 0 | 385 | 10.3% |
| `2026-06-13` | arw | 933 | 161 | 161 | 0 | 0 | 7 | 0 | 154 | 4.3% |
| `2026-06-14` | arw | 1520 | 160 | 160 | 0 | 0 | 10 | 0 | 150 | 6.2% |
| `2026-06-20` | arw | 1050 | 102 | 102 | 0 | 0 | 15 | 0 | 87 | 14.7% |
| `2026-07-02` | arw | 106 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | - |
| `2026-07-05` | arw | 1415 | 184 | 184 | 0 | 0 | 37 | 0 | 147 | 20.1% |
| `2026-07-11` | arw | 468 | 27 | 27 | 0 | 0 | 2 | 0 | 25 | 7.4% |
| `2026-07-18` | arw | 1545 | 349 | 349 | 0 | 0 | 41 | 0 | 308 | 11.7% |
| `2026-07-21` | arw | 32 | 2 | 2 | 0 | 0 | 0 | 0 | 2 | 0.0% |
| `2026-07-22` | arw | 189 | 13 | 13 | 0 | 0 | 2 | 0 | 11 | 15.4% |
| `2026-07-23` | arw | 18 | 1 | 1 | 0 | 0 | 0 | 0 | 1 | 0.0% |
| `2026-07-24` | arw | 218 | 45 | 45 | 0 | 0 | 5 | 0 | 40 | 11.1% |
| `2026-07-26` | arw | 64 | 12 | 12 | 0 | 0 | 1 | 0 | 11 | 8.3% |
| `2026-07-30` | arw | 936 | 76 | 76 | 0 | 0 | 3 | 0 | 73 | 3.9% |
| `2026-07-31` | arw | 349 | 60 | 60 | 0 | 0 | 9 | 0 | 51 | 15.0% |
| `2026-08-01` | arw | 3401 | 533 | 533 | 0 | 0 | 47 | 0 | 486 | 8.8% |
| `2026-08-02` | arw | 150 | 21 | 21 | 0 | 0 | 3 | 0 | 18 | 14.3% |
| `2026-08-08` | arw | 415 | 48 | 48 | 0 | 0 | 12 | 0 | 36 | 25.0% |
| `2026-08-15` | arw | 324 | 36 | 36 | 0 | 0 | 4 | 0 | 32 | 11.1% |
| `2026-08-16` | arw | 59 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | - |
| `2026-08-22` | arw | 2677 | 196 | 196 | 0 | 0 | 22 | 0 | 174 | 11.2% |
| `2026-08-29-l` | dng-sidecar | 211 | 11 | 0 | 11 | 0 | 0 | 0 | 11 | 0.0% |
| `2026-08-29` | arw | 3451 | 294 | 294 | 0 | 0 | 36 | 0 | 258 | 12.2% |
| `2026-09-05` | dng-sidecar | 411 | 207 | 0 | 207 | 0 | 35 | 0 | 172 | 16.9% |
| `2026-09-13-b` | arw | 1795 | 282 | 282 | 0 | 0 | 31 | 2 | 249 | 11.0% |
| `2026-09-19` | arw | 2134 | 182 | 182 | 0 | 0 | 21 | 7 | 154 | 11.5% |
| `2026-09-27-a` | arw | 4830 | 463 | 463 | 0 | 0 | 88 | 0 | 375 | 19.0% |
| `2026-09-27-b` | arw | 1240 | 158 | 158 | 0 | 0 | 17 | 0 | 141 | 10.8% |
| `2026-10-03` | arw | 1803 | 188 | 188 | 0 | 0 | 26 | 0 | 162 | 13.8% |
| `good-mark-2026-10-09` | arw | 180 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | - |
| **Total** | | 33644 | 4361 | 4059 | 302 | 0 | 549 | 9 | 3803 | 12.6% |

### Absolute threshold

#### All face-free frames: pooled

4361 face-free frames, 549 picks (base 12.6%), 9 rejects.

| Cut | Marked (coverage) | Picks marked (recall) | Precision | Lift | Rejects marked | Below the cut: pick share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| abs >= 100 | 3082 (70.7%) | 444 (80.9%) | 14.4% | 1.14 | 5 of 9 | 8.2% |
| abs >= 200 | 2240 (51.4%) | 329 (59.9%) | 14.7% | 1.17 | 3 of 9 | 10.4% |
| abs >= 300 | 1629 (37.4%) | 234 (42.6%) | 14.4% | 1.14 | 3 of 9 | 11.5% |
| abs >= 400 | 1156 (26.5%) | 160 (29.1%) | 13.8% | 1.10 | 2 of 9 | 12.1% |
| abs >= 600 | 654 (15.0%) | 95 (17.3%) | 14.5% | 1.15 | 1 of 9 | 12.2% |
| abs >= 800 | 424 (9.7%) | 62 (11.3%) | 14.6% | 1.16 | 0 of 9 | 12.4% |
| abs >= 1000 | 292 (6.7%) | 45 (8.2%) | 15.4% | 1.22 | 0 of 9 | 12.4% |

| AUC | Pooled | Within folder (held out) | Pairs within folder |
| --- | ---: | ---: | ---: |
| Pick vs non-pick | 0.570 | 0.579 | 133761 |
| Pick vs reject | 0.663 | 0.622 | 209 |

#### All face-free frames: per folder

| Folder | Face-free | Picks (base) | Rejects | AUC pick vs non-pick | AUC pick vs reject | Precision / coverage at 200 | Precision / coverage at 400 | Precision / coverage at 800 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-01-08` | 3 | 2 (66.7%) | 0 | 0.000 | - | 0.0% / 33.3% | 0.0% / 33.3% | - / 0.0% |
| `2026-02-14` | 1 | 0 (0.0%) | 0 | - | - | 0.0% / 100.0% | 0.0% / 100.0% | 0.0% / 100.0% |
| `2026-03-29` | 38 | 6 (15.8%) | 0 | 0.802 | - | 20.0% / 78.9% | 21.4% / 73.7% | 22.2% / 71.1% |
| `2026-04-12` | 6 | 3 (50.0%) | 0 | 0.778 | - | 50.0% / 100.0% | 50.0% / 100.0% | 50.0% / 100.0% |
| `2026-04-18` | 3 | 1 (33.3%) | 0 | 1.000 | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-05-22` | 37 | 19 (51.4%) | 0 | 0.661 | - | 71.4% / 18.9% | 0.0% / 2.7% | - / 0.0% |
| `2026-05-26` | 33 | 0 (0.0%) | 0 | - | - | 0.0% / 81.8% | 0.0% / 45.5% | 0.0% / 18.2% |
| `2026-06-05` | 429 | 44 (10.3%) | 0 | 0.456 | - | 8.4% / 44.3% | 5.2% / 22.4% | 3.3% / 7.0% |
| `2026-06-13` | 161 | 7 (4.3%) | 0 | 0.757 | - | 8.3% / 37.3% | 12.5% / 14.9% | 12.5% / 5.0% |
| `2026-06-14` | 160 | 10 (6.2%) | 0 | 0.727 | - | 7.8% / 71.9% | 10.4% / 41.9% | 22.2% / 11.2% |
| `2026-06-20` | 102 | 15 (14.7%) | 0 | 0.568 | - | 19.6% / 45.1% | 13.6% / 21.6% | 0.0% / 9.8% |
| `2026-07-05` | 184 | 37 (20.1%) | 0 | 0.559 | - | 22.9% / 64.1% | 21.4% / 30.4% | 23.5% / 9.2% |
| `2026-07-11` | 27 | 2 (7.4%) | 0 | 0.380 | - | 8.3% / 44.4% | 0.0% / 18.5% | 0.0% / 3.7% |
| `2026-07-18` | 349 | 41 (11.7%) | 0 | 0.559 | - | 13.0% / 39.5% | 7.8% / 18.3% | 5.6% / 5.2% |
| `2026-07-21` | 2 | 0 (0.0%) | 0 | - | - | 0.0% / 50.0% | 0.0% / 50.0% | - / 0.0% |
| `2026-07-22` | 13 | 2 (15.4%) | 0 | 0.409 | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-07-23` | 1 | 0 (0.0%) | 0 | - | - | 0.0% / 100.0% | 0.0% / 100.0% | - / 0.0% |
| `2026-07-24` | 45 | 5 (11.1%) | 0 | 0.190 | - | 11.9% / 93.3% | 8.3% / 80.0% | 0.0% / 62.2% |
| `2026-07-26` | 12 | 1 (8.3%) | 0 | 0.818 | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-07-30` | 76 | 3 (3.9%) | 0 | 0.626 | - | 4.0% / 65.8% | 7.4% / 35.5% | 0.0% / 7.9% |
| `2026-07-31` | 60 | 9 (15.0%) | 0 | 0.464 | - | 15.7% / 85.0% | 9.1% / 73.3% | 14.3% / 46.7% |
| `2026-08-01` | 533 | 47 (8.8%) | 0 | 0.578 | - | 9.8% / 61.0% | 13.4% / 25.1% | 10.5% / 3.6% |
| `2026-08-02` | 21 | 3 (14.3%) | 0 | 0.389 | - | 0.0% / 38.1% | 0.0% / 9.5% | - / 0.0% |
| `2026-08-08` | 48 | 12 (25.0%) | 0 | 0.468 | - | 25.0% / 66.7% | 23.8% / 43.8% | 22.2% / 18.8% |
| `2026-08-15` | 36 | 4 (11.1%) | 0 | 0.547 | - | 13.3% / 41.7% | 0.0% / 5.6% | - / 0.0% |
| `2026-08-22` | 196 | 22 (11.2%) | 0 | 0.590 | - | 13.0% / 62.8% | 13.6% / 30.1% | 18.2% / 5.6% |
| `2026-08-29-l` | 11 | 0 (0.0%) | 0 | - | - | 0.0% / 63.6% | 0.0% / 9.1% | 0.0% / 9.1% |
| `2026-08-29` | 294 | 36 (12.2%) | 0 | 0.585 | - | 15.3% / 51.0% | 10.3% / 26.5% | 0.0% / 5.4% |
| `2026-09-05` | 207 | 35 (16.9%) | 0 | 0.635 | - | 18.8% / 79.7% | 20.7% / 70.0% | 22.9% / 57.0% |
| `2026-09-13-b` | 282 | 31 (11.0%) | 2 | 0.521 | 0.484 | 12.4% / 45.7% | 8.5% / 20.9% | 0.0% / 4.6% |
| `2026-09-19` | 182 | 21 (11.5%) | 7 | 0.616 | 0.680 | 14.9% / 40.7% | 15.8% / 20.9% | 6.7% / 8.2% |
| `2026-09-27-a` | 463 | 88 (19.0%) | 0 | 0.614 | - | 24.1% / 46.7% | 24.4% / 17.7% | 25.0% / 2.6% |
| `2026-09-27-b` | 158 | 17 (10.8%) | 0 | 0.772 | - | 21.2% / 32.9% | 28.6% / 13.3% | 33.3% / 1.9% |
| `2026-10-03` | 188 | 26 (13.8%) | 0 | 0.675 | - | 27.1% / 25.5% | 5.3% / 10.1% | 0.0% / 1.6% |

#### All face-free frames: the cut held out by folder

| Cuts chosen (folders) | Marked (coverage) | Precision | Base | Lift | Passes (>= 2x base, >= 20%) |
| --- | ---: | ---: | ---: | ---: | --- |
| abs >= 200 (34) | 2240 (51.4%) | 14.7% | 12.6% | 1.17 | no |

#### Face-free singles: pooled

391 face-free frames, 53 picks (base 13.6%), 0 rejects.

| Cut | Marked (coverage) | Picks marked (recall) | Precision | Lift | Rejects marked | Below the cut: pick share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| abs >= 100 | 316 (80.8%) | 42 (79.2%) | 13.3% | 0.98 | 0 of 0 | 14.7% |
| abs >= 200 | 247 (63.2%) | 30 (56.6%) | 12.1% | 0.90 | 0 of 0 | 16.0% |
| abs >= 300 | 200 (51.2%) | 20 (37.7%) | 10.0% | 0.74 | 0 of 0 | 17.3% |
| abs >= 400 | 164 (41.9%) | 15 (28.3%) | 9.1% | 0.67 | 0 of 0 | 16.7% |
| abs >= 600 | 128 (32.7%) | 11 (20.8%) | 8.6% | 0.63 | 0 of 0 | 16.0% |
| abs >= 800 | 99 (25.3%) | 8 (15.1%) | 8.1% | 0.60 | 0 of 0 | 15.4% |
| abs >= 1000 | 77 (19.7%) | 8 (15.1%) | 10.4% | 0.77 | 0 of 0 | 14.3% |

| AUC | Pooled | Within folder (held out) | Pairs within folder |
| --- | ---: | ---: | ---: |
| Pick vs non-pick | 0.436 | 0.489 | 1184 |
| Pick vs reject | - | - | 0 |

#### Face-free singles: per folder

| Folder | Face-free | Picks (base) | Rejects | AUC pick vs non-pick | AUC pick vs reject | Precision / coverage at 200 | Precision / coverage at 400 | Precision / coverage at 800 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-01-08` | 3 | 2 (66.7%) | 0 | 0.000 | - | 0.0% / 33.3% | 0.0% / 33.3% | - / 0.0% |
| `2026-02-14` | 1 | 0 (0.0%) | 0 | - | - | 0.0% / 100.0% | 0.0% / 100.0% | 0.0% / 100.0% |
| `2026-03-29` | 23 | 0 (0.0%) | 0 | - | - | 0.0% / 65.2% | 0.0% / 56.5% | 0.0% / 52.2% |
| `2026-04-12` | 5 | 2 (40.0%) | 0 | 0.667 | - | 40.0% / 100.0% | 40.0% / 100.0% | 40.0% / 100.0% |
| `2026-04-18` | 3 | 1 (33.3%) | 0 | 1.000 | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-05-22` | 33 | 19 (57.6%) | 0 | 0.605 | - | 71.4% / 21.2% | 0.0% / 3.0% | - / 0.0% |
| `2026-05-26` | 1 | 0 (0.0%) | 0 | - | - | 0.0% / 100.0% | - / 0.0% | - / 0.0% |
| `2026-06-05` | 2 | 0 (0.0%) | 0 | - | - | 0.0% / 50.0% | - / 0.0% | - / 0.0% |
| `2026-06-14` | 15 | 0 (0.0%) | 0 | - | - | 0.0% / 93.3% | 0.0% / 60.0% | 0.0% / 13.3% |
| `2026-06-20` | 5 | 3 (60.0%) | 0 | 0.500 | - | 60.0% / 100.0% | 50.0% / 40.0% | 0.0% / 20.0% |
| `2026-07-05` | 1 | 0 (0.0%) | 0 | - | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-07-11` | 21 | 1 (4.8%) | 0 | 0.050 | - | 0.0% / 52.4% | 0.0% / 23.8% | 0.0% / 4.8% |
| `2026-07-18` | 8 | 1 (12.5%) | 0 | 0.286 | - | 16.7% / 75.0% | 0.0% / 25.0% | 0.0% / 12.5% |
| `2026-07-21` | 2 | 0 (0.0%) | 0 | - | - | 0.0% / 50.0% | 0.0% / 50.0% | - / 0.0% |
| `2026-07-22` | 6 | 0 (0.0%) | 0 | - | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-07-23` | 1 | 0 (0.0%) | 0 | - | - | 0.0% / 100.0% | 0.0% / 100.0% | - / 0.0% |
| `2026-07-24` | 27 | 4 (14.8%) | 0 | 0.217 | - | 15.4% / 96.3% | 10.0% / 74.1% | 0.0% / 55.6% |
| `2026-07-26` | 1 | 0 (0.0%) | 0 | - | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-07-30` | 47 | 1 (2.1%) | 0 | 0.196 | - | 0.0% / 53.2% | 0.0% / 31.9% | 0.0% / 8.5% |
| `2026-07-31` | 26 | 3 (11.5%) | 0 | 0.203 | - | 10.0% / 76.9% | 0.0% / 61.5% | 0.0% / 34.6% |
| `2026-08-01` | 12 | 1 (8.3%) | 0 | 0.727 | - | 8.3% / 100.0% | 12.5% / 66.7% | 0.0% / 8.3% |
| `2026-08-02` | 6 | 0 (0.0%) | 0 | - | - | 0.0% / 33.3% | - / 0.0% | - / 0.0% |
| `2026-08-08` | 22 | 5 (22.7%) | 0 | 0.506 | - | 26.7% / 68.2% | 25.0% / 54.5% | 16.7% / 27.3% |
| `2026-08-15` | 16 | 2 (12.5%) | 0 | 0.536 | - | 14.3% / 43.8% | - / 0.0% | - / 0.0% |
| `2026-08-22` | 5 | 0 (0.0%) | 0 | - | - | 0.0% / 40.0% | - / 0.0% | - / 0.0% |
| `2026-08-29-l` | 9 | 0 (0.0%) | 0 | - | - | 0.0% / 55.6% | 0.0% / 11.1% | 0.0% / 11.1% |
| `2026-08-29` | 1 | 0 (0.0%) | 0 | - | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-09-05` | 76 | 8 (10.5%) | 0 | 0.546 | - | 11.9% / 77.6% | 12.2% / 64.5% | 12.8% / 51.3% |
| `2026-09-13-b` | 3 | 0 (0.0%) | 0 | - | - | 0.0% / 66.7% | 0.0% / 33.3% | - / 0.0% |
| `2026-09-19` | 5 | 0 (0.0%) | 0 | - | - | 0.0% / 60.0% | 0.0% / 20.0% | 0.0% / 20.0% |
| `2026-09-27-a` | 4 | 0 (0.0%) | 0 | - | - | - / 0.0% | - / 0.0% | - / 0.0% |
| `2026-09-27-b` | 1 | 0 (0.0%) | 0 | - | - | - / 0.0% | - / 0.0% | - / 0.0% |

#### Face-free singles: the cut held out by folder

| Cuts chosen (folders) | Marked (coverage) | Precision | Base | Lift | Passes (>= 2x base, >= 20%) |
| --- | ---: | ---: | ---: | ---: | --- |
| abs >= 100 (31), abs >= 200 (1) | 302 (77.2%) | 11.3% | 13.6% | 0.83 | no |

### Within-burst relative

#### Face-free bursts

244 face-free bursts of two or more frames (703 face-free frames, 53 picks, base 7.5%, 0 rejects; 49 bursts hold a face-free pick) in 26 folders. Face-free frames in bursts that also hold a faced frame: 3267. Face-free singles: 391.

| Rule | Marked (coverage) | Picks marked (share of picks) | Precision | Lift | Rejects marked |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sharpest frame | 244 (34.7%) | 24 (45.3%) | 9.8% | 1.30 | 0 of 0 |
| Top half | 404 (57.5%) | 35 (66.0%) | 8.7% | 1.15 | 0 of 0 |
| rel >= 0.5 | 560 (79.7%) | 45 (84.9%) | 8.0% | 1.07 | 0 of 0 |
| rel >= 0.7 | 469 (66.7%) | 39 (73.6%) | 8.3% | 1.10 | 0 of 0 |
| rel >= 0.8 | 415 (59.0%) | 38 (71.7%) | 9.2% | 1.21 | 0 of 0 |
| rel >= 0.9 | 349 (49.6%) | 33 (62.3%) | 9.5% | 1.25 | 0 of 0 |

| Pairwise AUC within a burst | AUC | Pairs |
| --- | ---: | ---: |
| Pick vs non-pick | 0.536 | 110 |
| Pick vs reject | - | 0 |

| Folder | Bursts | Frames | Picks | Rejects | Within-burst AUC pick vs non-pick | Pick share at the sharpest frame |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-03-29` | 2 | 4 | 0 | 0 | - | - |
| `2026-05-22` | 2 | 4 | 0 | 0 | - | - |
| `2026-05-26` | 4 | 10 | 0 | 0 | - | - |
| `2026-06-05` | 33 | 105 | 9 | 0 | 0.789 | 55.6% |
| `2026-06-13` | 12 | 36 | 1 | 0 | 0.667 | 0.0% |
| `2026-06-14` | 13 | 27 | 3 | 0 | 1.000 | 100.0% |
| `2026-06-20` | 8 | 21 | 2 | 0 | 0.750 | 50.0% |
| `2026-07-05` | 3 | 9 | 2 | 0 | 0.500 | 50.0% |
| `2026-07-18` | 20 | 69 | 2 | 0 | 1.000 | 100.0% |
| `2026-07-22` | 2 | 4 | 2 | 0 | 0.500 | 50.0% |
| `2026-07-24` | 3 | 6 | 0 | 0 | - | - |
| `2026-07-26` | 1 | 10 | 1 | 0 | 0.778 | 0.0% |
| `2026-07-30` | 3 | 7 | 0 | 0 | - | - |
| `2026-07-31` | 5 | 14 | 3 | 0 | 0.333 | 33.3% |
| `2026-08-01` | 23 | 60 | 2 | 0 | 0.500 | 50.0% |
| `2026-08-02` | 2 | 4 | 0 | 0 | - | - |
| `2026-08-08` | 8 | 19 | 4 | 0 | 0.000 | 0.0% |
| `2026-08-15` | 4 | 11 | 2 | 0 | 0.400 | 0.0% |
| `2026-08-22` | 6 | 20 | 1 | 0 | 0.167 | 0.0% |
| `2026-08-29` | 15 | 35 | 0 | 0 | - | - |
| `2026-09-05` | 40 | 102 | 17 | 0 | 0.467 | 47.1% |
| `2026-09-13-b` | 11 | 42 | 0 | 0 | - | - |
| `2026-09-19` | 3 | 10 | 0 | 0 | - | - |
| `2026-09-27-a` | 12 | 32 | 0 | 0 | - | - |
| `2026-09-27-b` | 3 | 6 | 1 | 0 | 1.000 | 100.0% |
| `2026-10-03` | 6 | 36 | 1 | 0 | 0.250 | 0.0% |

#### Face-free bursts: the cut held out by folder

| Cuts chosen (folders) | Marked (coverage) | Precision | Base | Lift | Passes (>= 2x base, >= 20%) |
| --- | ---: | ---: | ---: | ---: | --- |
| rel >= 0.9 (26) | 349 (49.6%) | 9.5% | 7.5% | 1.25 | no |

#### Face-free frames of every burst, against the whole burst

1588 bursts of two or more frames with a face-free frame (3970 face-free frames, 496 picks, base 12.5%, 9 rejects; 342 bursts hold a face-free pick) in 29 folders. Face-free frames in bursts that also hold a faced frame: 3267. Face-free singles: 391.

| Rule | Marked (coverage) | Picks marked (share of picks) | Precision | Lift | Rejects marked |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sharpest frame | 640 (16.1%) | 86 (17.3%) | 13.4% | 1.08 | 3 of 9 |
| Top half | 1986 (50.0%) | 309 (62.3%) | 15.6% | 1.25 | 4 of 9 |
| rel >= 0.5 | 1958 (49.3%) | 269 (54.2%) | 13.7% | 1.10 | 4 of 9 |
| rel >= 0.7 | 1392 (35.1%) | 191 (38.5%) | 13.7% | 1.10 | 3 of 9 |
| rel >= 0.8 | 1129 (28.4%) | 151 (30.4%) | 13.4% | 1.07 | 3 of 9 |
| rel >= 0.9 | 880 (22.2%) | 116 (23.4%) | 13.2% | 1.06 | 3 of 9 |

| Pairwise AUC within a burst | AUC | Pairs |
| --- | ---: | ---: |
| Pick vs non-pick | 0.598 | 1100 |
| Pick vs reject | - | 0 |

| Folder | Bursts | Frames | Picks | Rejects | Within-burst AUC pick vs non-pick | Pick share at the sharpest frame |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-03-29` | 4 | 15 | 6 | 0 | 0.520 | 33.3% |
| `2026-04-12` | 1 | 1 | 1 | 0 | - | 100.0% |
| `2026-05-22` | 2 | 4 | 0 | 0 | - | - |
| `2026-05-26` | 11 | 32 | 0 | 0 | - | - |
| `2026-06-05` | 155 | 427 | 44 | 0 | 0.538 | 18.2% |
| `2026-06-13` | 72 | 161 | 7 | 0 | 0.667 | 42.9% |
| `2026-06-14` | 69 | 145 | 10 | 0 | 0.933 | 40.0% |
| `2026-06-20` | 54 | 97 | 12 | 0 | 0.700 | 33.3% |
| `2026-07-05` | 60 | 183 | 37 | 0 | 0.579 | 10.8% |
| `2026-07-11` | 5 | 6 | 1 | 0 | 1.000 | 0.0% |
| `2026-07-18` | 136 | 341 | 40 | 0 | 0.667 | 12.5% |
| `2026-07-22` | 5 | 7 | 2 | 0 | 0.500 | 50.0% |
| `2026-07-24` | 13 | 18 | 1 | 0 | 0.000 | 0.0% |
| `2026-07-26` | 2 | 11 | 1 | 0 | 0.778 | 0.0% |
| `2026-07-30` | 18 | 29 | 2 | 0 | 1.000 | 50.0% |
| `2026-07-31` | 20 | 34 | 6 | 0 | 0.429 | 50.0% |
| `2026-08-01` | 164 | 521 | 46 | 0 | 0.581 | 15.2% |
| `2026-08-02` | 11 | 15 | 3 | 0 | 0.000 | 0.0% |
| `2026-08-08` | 15 | 26 | 7 | 0 | 0.000 | 42.9% |
| `2026-08-15` | 12 | 20 | 2 | 0 | 0.400 | 0.0% |
| `2026-08-22` | 94 | 191 | 22 | 0 | 0.595 | 18.2% |
| `2026-08-29-l` | 2 | 2 | 0 | 0 | - | - |
| `2026-08-29` | 134 | 293 | 36 | 0 | 0.719 | 8.3% |
| `2026-09-05` | 52 | 131 | 27 | 0 | 0.510 | 44.4% |
| `2026-09-13-b` | 83 | 279 | 31 | 2 | 0.611 | 9.7% |
| `2026-09-19` | 76 | 177 | 21 | 7 | 0.410 | 19.0% |
| `2026-09-27-a` | 186 | 459 | 88 | 0 | 0.567 | 6.8% |
| `2026-09-27-b` | 60 | 157 | 17 | 0 | 0.839 | 11.8% |
| `2026-10-03` | 72 | 188 | 26 | 0 | 0.697 | 23.1% |

#### Every burst: the cut held out by folder

| Cuts chosen (folders) | Marked (coverage) | Precision | Base | Lift | Passes (>= 2x base, >= 20%) |
| --- | ---: | ---: | ---: | ---: | --- |
| rel >= 0.5 (18), rel >= 0.7 (10), rel >= 0.9 (1) | 1647 (41.5%) | 11.9% | 12.5% | 0.95 | no |

### Against the criterion

| Form | Held-out AUC pick vs non-pick (>= 0.70) | Cut at >= 2x base, >= 20% marked, held out |
| --- | ---: | --- |
| Absolute | 0.579 | no |
| Within-burst relative, face-free bursts | 0.536 | no |
| Within-burst relative, every burst | 0.598 | no |

Folders with 30 or more face-free rejects: none (most: 7).
