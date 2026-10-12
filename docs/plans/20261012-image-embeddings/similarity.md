# Similarity threshold (Step 4)

Step 4's Done-when asked for a "Show Similar" threshold chosen from
hand-labeled same / different scene pairs that keeps different scenes apart
at >= 0.95 precision. No threshold does; Steps 4 and 6 were closed without
shipping (Decision 3 in [plan.md](plan.md)).

## Material

- [pairs.py](pairs.py) (`python -I pairs.py <gate-dir> <out-dir>`) drew 150
  within-folder pairs from 8 folders (`2026-06-05`, `2026-06-13`,
  `2026-06-14`, `2026-06-20`, `2026-08-08`, `2026-08-29`, `2026-09-27-a`,
  `2026-09-27-b`), stratified by the SigLIP 2 cosine of Step 2's
  `embed.tsv`: 135 pairs across six cosine bins with two frames of the same
  burst excluded, plus 15 within-burst anchors. No file is reused.
- The labeling page, the pairs and the labels are outside the repository, in
  `D:\Photos\tests\2026-10-12-similar-pairs\`: `label.html` (shows the pairs
  in shuffled id order without the cosine or the file names), `pairs.tsv`
  (id, folder, a, b, cosine, bin) and the user's `labels.tsv` (id, label).
- [similarity.py](similarity.py) (`python -I similarity.py <dir>`) joins the
  two TSVs and prints the tables below.

## Definition

"Same scene" means interchangeable candidates of which you would keep one:
the same player in the same play or the same kind of action. A different
play in the same game is "different". This was clarified to the user while
labeling.

## Labels

19 same, 131 different, 0 unsure.

| bin | same | different |
|---|---|---|
| 0.70-0.80 | 0 | 12 |
| 0.80-0.85 | 0 | 20 |
| 0.85-0.90 | 1 | 27 |
| 0.90-0.93 | 0 | 30 |
| 0.93-0.96 | 4 | 26 |
| 0.96-1.00 | 4 | 11 |
| within-burst anchors | 10 | 5 |

The 20 highest cosines, in order (S same, d different, * an anchor):

0.9740d 0.9731S 0.9724S* 0.9700S 0.9690d 0.9680d 0.9662d 0.9635S 0.9629d
0.9613d 0.9611d 0.9607d 0.9603S 0.9601d 0.9601d 0.9600d 0.9591S 0.9565d
0.9559d 0.9556d

The highest-cosine pair (0.974) is different. Above 0.93 same and different
interleave. The anchors range from 0.897 to 0.972, and 5 of the 15 frame
pairs inside one burst were labeled different (the play moved on within the
burst).

## Precision and recall at candidate thresholds

A pair is predicted "same" when its cosine is >= t. Precision is on this
labeled set, whose bins are stratified rather than drawn in a folder's
proportions, so it is not a folder-level estimate; it is enough to show that
no threshold approaches 0.95.

Stratified pairs (anchors excluded): 135 pairs, 9 same.

| t | predicted same | of them same | precision | recall |
|---|---|---|---|---|
| 0.800 | 123 | 9 | 0.073 | 1.000 |
| 0.850 | 103 | 9 | 0.087 | 1.000 |
| 0.900 | 75 | 8 | 0.107 | 0.889 |
| 0.920 | 54 | 8 | 0.148 | 0.889 |
| 0.930 | 45 | 8 | 0.178 | 0.889 |
| 0.940 | 27 | 5 | 0.185 | 0.556 |
| 0.950 | 20 | 5 | 0.250 | 0.556 |
| 0.960 | 15 | 4 | 0.267 | 0.444 |
| 0.965 | 6 | 2 | 0.333 | 0.222 |
| 0.970 | 3 | 2 | 0.667 | 0.222 |
| 0.975 | 0 | 0 | - | 0.000 |

All pairs (anchors included): 150 pairs, 19 same.

| t | predicted same | of them same | precision | recall |
|---|---|---|---|---|
| 0.800 | 138 | 19 | 0.138 | 1.000 |
| 0.850 | 118 | 19 | 0.161 | 1.000 |
| 0.900 | 89 | 17 | 0.191 | 0.895 |
| 0.920 | 65 | 16 | 0.246 | 0.842 |
| 0.930 | 54 | 15 | 0.278 | 0.789 |
| 0.940 | 34 | 10 | 0.294 | 0.526 |
| 0.950 | 21 | 6 | 0.286 | 0.316 |
| 0.960 | 16 | 5 | 0.312 | 0.263 |
| 0.965 | 7 | 3 | 0.429 | 0.158 |
| 0.970 | 4 | 3 | 0.750 | 0.158 |
| 0.975 | 0 | 0 | - | 0.000 |

Over the top cosine bin (0.96-1.00) precision is about 27%. The 0.970 row's
0.667 / 0.750 rest on 3-4 pairs and still miss the bar, and the next pair
above it (0.974) is different.

## Reading

SigLIP 2's global embedding captures the place, the sport and the
atmosphere, not which play within one game. A low cosine reliably means a
different scene (no "same" below 0.87), and ranking by cosine does
concentrate "same" at the top, but a threshold-based "Show Similar" or
connected-component visual groups would lump a whole game together.

## Outcome

The user chose (2026-10-12) to drop Steps 4 and 6 rather than ship a
threshold-free top-N similarity view or relabel with a looser definition.
Step 6's validation (same-scene pairs grouped at >= 0.9, different-scene at
<= 0.05) was not run; it rests on a threshold this data does not give.
