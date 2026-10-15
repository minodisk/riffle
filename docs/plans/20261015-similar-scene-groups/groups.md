# Step 1: similar-scene groups evaluation

## Pre-registration

Written 2026-10-15T08:55+09:00, before `scene_pairs.py` or `scene_groups.py`
exists, before any pair is drawn and before any label is given, and merged on
its own (the merge commit of the PR that adds this section, without the draw
and the results below, is the record, as `ce0ea14` was for the in-burst
[gate](../_archived/20261012-image-embeddings/gate.md)). Nothing in this
section changes after the labels are in; a deviation found while implementing
Steps 2 and 3 is recorded in `learnings.md` with its reason and does not touch
the criterion.

The question: does the SigLIP 2 cosine plus capture-time proximity group
same-scene frames of the user's non-sports folders precisely enough, and
often enough, to rank and order frames over scene groups instead of 1 s
bursts?

### Definition of "same scene" (non-sports)

"Same scene" means interchangeable candidates of which you would keep one:

- the same place and composition: a landscape or a street reframed slightly,
  re-exposed, or re-shot a moment later; or
- the same subject in the same pose or activity: a portrait sitting, a child
  at the same spot doing the same thing.

"Different" means another composition of the same place (a wide shot and a
detail), another pose or activity, another subject, or another moment of a
moving scene of which you would keep both.

"Unsure" is allowed; an unsure pair is dropped from every count and its
number is reported. The labeling page carries this definition in Japanese.

### Data

- **Pool**: the 13 non-sports ARW folders of the app dump in
  `D:\Photos\tests\2026-10-12-rank-app\` (`embed.tsv`, 768-d L2-normalized
  SigLIP 2 at `--near 404`, `EMBED_VERSION` 1; `features/<folder>.tsv`;
  `features/<folder>.output.txt`; `genres.tsv`): `2026-08-16`, `2026-08-15`,
  `2026-08-08`, `2026-08-02`, `2026-07-31`, `2026-07-30`, `2026-07-26`,
  `2026-07-24`, `2026-07-23`, `2026-07-22`, `2026-07-21`, `2026-07-11`,
  `2026-07-02`; plus the DNG folders `2026-09-05` and `2026-08-29-l` from the
  fresh dump of Step 2 in `D:\Photos\tests\2026-10-15-scene-embed\` (same
  format, same model, `--near 404`). If Step 2 cannot dump the DNG folders,
  the pool is the 13 ARW folders and the reason is recorded.
- **Excluded**: from `2026-08-08`, the 24 files of its 12 pairs in the sports
  round's `D:\Photos\tests\2026-10-12-similar-pairs\pairs.tsv` (labeled under
  the "same play" definition); they are neither drawn nor counted in
  coverage. `2026-06-14` (91% sports frames) is excluded as a sports folder.
  `2026-05-22` (37 frames, no sidecars) is dropped.
- **No genre filter**: every other frame of the pool folders stays, sports
  frames included, because the app would not filter by genre at run time. The
  sports share of the drawn pairs is reported: a pair counts as sports when
  either frame's top-level genre (the argmax of the `softmax` columns of
  `genres.tsv`, as the plan's folder table) is sports; a frame without genre
  scores (the DNG folders) counts as unknown and is reported apart.
- **Capture time**: the dump's capture time and subsec as
  `pairs.capture_ms` reads them (subsec right-padded to milliseconds). A
  frame without a capture time is not drawn and stays its own group; a
  frame without a vector is not drawn but keeps its burst (it joins or opens
  a group with its burst), and a burst whose boundary frame (the first frame
  when joining, the open group's last frame when being joined) lacks a vector
  does not chain: it opens a new group. The number of such frames is
  reported.
- **Bursts**: `rank::bursts`, which is `burst.ts` `groupBursts` at gap
  1000 ms (capture time, subsec, file name; a gap equal to 1000 ms stays in
  the burst). A **single** is a burst of one frame.
- **Picks** (for the interaction check only): the in-burst gate's rule, via
  the archived [`gate.py`](../_archived/20261012-image-embeddings/gate.py)'s
  `load`: a frame is a pick when its `.dop` flag is Pick, or it has a virtual
  copy in `Output/` (`<stem>_<n>`), or it has no `.dop` and its XMP flag is
  Pick; every other frame is a non-pick.

### Grouping rule candidates

- **Chained rule (primary)**, the one the criterion decides on: within one
  folder, frames in capture order (as `rank::bursts` orders them); a frame of
  the same 1 s burst as the open group's last frame always joins (the burst
  is the atom and is never split); otherwise it joins the open group when its
  cosine to the group's last frame is >= `t` and its capture gap to that
  frame is <= `W`, else it opens a new group.
- **Grid**: `t` in 0.80, 0.81, …, 0.98 (19 values, step 0.01) crossed with
  `W` in {unbounded, 60 s, 300 s, 1800 s}: 76 rules.
- **Fallbacks**, evaluated only if at least one chained rule meets the
  precision bar and none of those meets the recall bar (to limit the
  search), on the same grid:
  - single linkage: a burst joins the open group when the cosine between any
    frame of the burst and any member of the group is >= `t` and the capture
    gap to the group's last frame is <= `W`;
  - centroid: a burst joins the open group when the cosine between its first
    frame and the L2-normalized mean vector of the group's members is >= `t`
    and the capture gap to the group's last frame is <= `W`.

  A fallback that is selected passes or fails on the same bars; on a tie in
  recall between the two, single linkage is taken.

### Pair sampling

Label-free sizing, counted before the pre-registration over the 13 ARW pool
folders (3304 frames, the 24 excluded files left out), as candidate pairs /
distinct files per cell of within-folder pairs of frames from different
bursts (the DNG folders add to these when dumped):

| cosine \ gap | <= 60 s | 60–300 s | 300–1800 s | > 1800 s |
|---|---:|---:|---:|---:|
| 0.70–0.80 | 7719 / 2149 | 25370 / 2599 | 51470 / 2525 | 156062 / 2974 |
| 0.80–0.85 | 8883 / 2435 | 18962 / 2309 | 18231 / 2001 | 12603 / 1820 |
| 0.85–0.90 | 10037 / 2647 | 11044 / 1724 | 6208 / 1144 | 2075 / 717 |
| 0.90–0.93 | 5592 / 2204 | 1652 / 742 | 375 / 236 | 158 / 111 |
| 0.93–0.96 | 3480 / 1512 | 572 / 225 | 10 / 18 | 76 / 42 |
| 0.96–1.00 | 593 / 420 | 33 / 29 | 0 / 0 | 1 / 2 |

- **Candidates**: within-folder pairs of two frames of *different* bursts
  of the pool, the cosine of their `embed.tsv` vectors in a cosine bin
  `[lo, hi)` (the top bin `[0.96, 1.01)`) and their capture gap in a gap bin
  (`<= 60 s`, `(60 s, 300 s]`, `(300 s, 1800 s]`, `> 1800 s`). Pairs below
  cosine 0.70 are not drawn.
- **Counts per cell** (fixed here; 185 cross-burst pairs, 105 of them at
  cosine >= 0.90):

  | cosine \ gap | <= 60 s | 60–300 s | 300–1800 s | > 1800 s | total |
  |---|---:|---:|---:|---:|---:|
  | 0.70–0.80 | 4 | 4 | 4 | 4 | 16 |
  | 0.80–0.85 | 6 | 6 | 6 | 6 | 24 |
  | 0.85–0.90 | 10 | 10 | 10 | 10 | 40 |
  | 0.90–0.93 | 16 | 12 | 8 | 4 | 40 |
  | 0.93–0.96 | 18 | 12 | 5 | 5 | 40 |
  | 0.96–1.00 | 18 | 6 | 0 | 1 | 25 |

- **Anchors**: 15 within-burst pairs (two frames of the same 1 s burst of
  2+ frames, any cosine), as a check on the definition. **Total: 200 pairs.**
- **Draw**: seed `20261015` (Python `random.Random`). Cells are drawn from
  the highest cosine bin down and, within a bin, from the longest gap bin
  down (the scarce cells first), then the anchors. For each pair a folder is
  chosen uniformly among the pool folders that still have a candidate in the
  cell whose two files are both unused, then one such candidate uniformly
  (adapted from the archived
  [`pairs.py`](../_archived/20261012-image-embeddings/pairs.py) `draw`, which
  differs in detail). **No file is reused** across the whole draw, anchors included.
  A cell that runs out of usable candidates contributes what it has; the
  shortfall is not refilled from another cell and is reported under "Draw".
- **Page**: ids given after a seeded shuffle, the sides of each pair swapped
  at random; the page shows neither the cosine, the gap, the bin nor the file
  names; S / D / U keys as `pairs.py`'s `PAGE`; `labels.tsv` (id, label:
  same / different / unsure) exported beside `pairs.tsv`.

### Precision bar

- A non-anchor labeled pair is **predicted same** at `(t, W)` when its cosine
  is >= `t` and its capture gap is <= `W` (the pairwise form of the chained
  rule's join test). For a fallback, a pair is predicted same when the rule,
  applied to the pair's folder, puts both frames in one group.
- Anchors are left out of precision and recall: every rule groups them
  unconditionally, so they test the definition, not the rule. Their same /
  different counts are reported apart.
- **Precision** at `(t, W)`: same / (same + different) among the predicted
  same pairs, unsure dropped. The bar: the **one-sided 95% Clopper-Pearson
  lower bound** on that precision is >= **0.95**, computed as
  `scipy.stats.beta.ppf(0.05, k, n - k + 1)` for `k` same of `n` predicted
  same (`0.05 ** (1 / n)` when `k = n`, 0 when `k = 0`). This needs at least
  59 predicted-same pairs with no "different", or at least 93 with one. The
  point estimate is reported beside the bound.
- Rejected alternative: the point estimate >= 0.95 alone. Over the 20–40
  predicted-same pairs a high threshold leaves, a point estimate of 1.0 is
  compatible with a true precision well under 0.90 (with 20 of 20, the
  one-sided 95% bound is 0.86); the sports round showed same and different
  interleaving above cosine 0.93, so a lucky high-threshold cell is the
  failure this bar exists to catch.
- The labeled set is stratified, not drawn in the folders' proportions, so
  its precision is not a folder-level estimate; it is weighted toward the
  high cosines where the predicted-same pairs lie, which is where the bar is
  decided.

### Minimum recall worth shipping

Both must hold at the selected `(t, W)`:

1. **Recall** >= **0.30**: predicted same / all labeled same pairs, anchors
   excluded, unsure dropped (for a fallback, predicted same as defined above).
2. **Coverage** >= **20%**: the rule applied to every frame of the pool
   folders (the 24 excluded files left out) puts at least 20% of today's
   singles, pooled over the folders, into a group of two or more frames.

Also reported, not deciding: the recall reweighted to the folders'
proportions (each labeled pair weighted by its cell's candidate count over
the cell's drawn count); for the selected chained rule, the precision and
recall when "predicted same" means "in one group" under the chained rule;
the number of groups, the largest group's size and span, and the share of
frames in groups of two or more, per folder.

### Selection rule

Among the 76 chained rules that meet the precision bar, the one with the
highest recall; on a tie, the larger `W` (unbounded largest), then the lower
`t`. The **evaluation passes** when the selected rule meets both the recall
and the coverage bar. If no chained rule meets the precision bar, it fails.
If the selected chained rule meets the recall bar but not the coverage bar,
it fails. If it fails the recall bar (every chained rule that meets the
precision bar has recall below 0.30), the fallbacks are evaluated under the same
precision bar, selection rule and recall and coverage bars, and the
evaluation passes only if the selected fallback meets all three.

### Interaction check (reported, not deciding)

Only when a rule is selected: on the pool folders, with the shipped prior
weights of `crates/core/ranking/siglip2.json` (as of the commit Step 3 runs
on), the Top-1 within scene groups of two or more frames holding at least one
pick and one non-pick (the share of such groups whose top-scored frame is a
pick, ties split) against the first-frame baseline (the share whose first
frame in capture order is a pick), pooled over the groups, with the number of
groups. The `Burst position` columns were fit on 1 s bursts; if Top-1 reads
below the first-frame baseline, the question of zeroing them inside non-burst
groups goes to the user before Step 5.

### What happens on fail

Steps 4–6 are dropped; Decision 1 in [plan.md](plan.md) records the tables
and the numbers that decided; the plan ends with the wrap-up.

## Draw

Drawn 2026-10-15 by [`scene_pairs.py`](scene_pairs.py) (seed `20261015`)
from the app dump `D:\Photos\tests\2026-10-12-rank-app\` and the fresh DNG
dump `D:\Photos\tests\2026-10-15-scene-embed\` into
`D:\Photos\tests\2026-10-15-scene-pairs\` (`pairs.tsv`, `img/`,
`label.html`), before any label:

```sh
python -I scene_pairs.py D:/Photos/tests/2026-10-12-rank-app \
  D:/Photos/tests/2026-10-15-scene-embed D:/Photos/tests/2026-10-15-scene-pairs
```

- **Pool**: 15 folders, 3926 frames (the 13 ARW folders, 3304 frames with
  the 24 excluded `2026-08-08` files left out, plus `2026-09-05` 411 and
  `2026-08-29-l` 211). Every frame of the dumps has a capture time and a
  vector, so none is left out of the draw.
- **Pairs drawn per cell**: every cell got its pre-registered count; no
  shortfall. 185 cross-burst pairs (105 at cosine >= 0.90), 15 anchors, 200
  pairs over 400 distinct files.

  | cosine \ gap | <= 60 s | 60–300 s | 300–1800 s | > 1800 s | total |
  |---|---:|---:|---:|---:|---:|
  | 0.70–0.80 | 4 | 4 | 4 | 4 | 16 |
  | 0.80–0.85 | 6 | 6 | 6 | 6 | 24 |
  | 0.85–0.90 | 10 | 10 | 10 | 10 | 40 |
  | 0.90–0.93 | 16 | 12 | 8 | 4 | 40 |
  | 0.93–0.96 | 18 | 12 | 5 | 5 | 40 |
  | 0.96–1.00 | 18 | 6 | 0 | 1 | 25 |

- **Candidate pairs per cell** with the DNG folders added (the ARW part
  matches the pre-registration's sizing table but for one pair at the
  cosine 0.85 edge, see `learnings.md`):

  | cosine \ gap | <= 60 s | 60–300 s | 300–1800 s | > 1800 s |
  |---|---:|---:|---:|---:|
  | 0.70–0.80 | 8510 | 27585 | 64321 | 182693 |
  | 0.80–0.85 | 9851 | 21003 | 24655 | 17346 |
  | 0.85–0.90 | 11174 | 12272 | 8341 | 3317 |
  | 0.90–0.93 | 6242 | 1855 | 531 | 214 |
  | 0.93–0.96 | 3868 | 612 | 17 | 76 |
  | 0.96–1.00 | 713 | 34 | 0 | 1 |

- **Pairs per folder**: `2026-08-16` 9, `2026-08-15` 16, `2026-08-08` 16,
  `2026-08-02` 12, `2026-07-31` 12, `2026-07-30` 18, `2026-07-26` 4,
  `2026-07-24` 10, `2026-07-23` 6, `2026-07-22` 16, `2026-07-21` 8,
  `2026-07-11` 18, `2026-07-02` 16, `2026-09-05` 28, `2026-08-29-l` 11.
- **Sports share**: 3 of the 200 pairs have a frame whose top-level genre is
  sports; 39 (the DNG folders') have no genre scores; 158 are neither.
