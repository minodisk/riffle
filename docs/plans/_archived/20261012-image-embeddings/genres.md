# Genre calibration (Step 5)

The thresholds of `crates/core/genres/siglip2.json` and what they give on
the hand labels. Everything here is recomputed by `genre_calibrate.py` from
the labeling folder `D:\Photos\tests\2026-10-12-genre-labels\` (`frames.tsv`,
`labels.tsv`) and the SigLIP 2 vectors of Step 2's `embed.tsv` plus
`extra\embed.tsv` (52 more folders), and the JSON by `genre_vectors.py` from
[genre_prompts.json](genre_prompts.json):

```sh
python -I genre_vectors.py <siglip2-b32-256-text> genre_prompts.json crates/core/genres/siglip2.json
python -I genre_calibrate.py --library crates/core/genres/siglip2.json <label-dir> <embed-gate>/embed.tsv <label-dir>/extra/embed.tsv
```

## Labels

300 frames from 56 folders, drawn by `genre_sample.py`: a quota per
provisional label from the frames that label scored highest on (Step 2's
prompts), 22 low-confidence frames, 45 random sports frames, at most one
frame per burst and no near duplicate within a folder. The user labeled each
frame with every genre that applies (a sub-genre brings its parent), `other`
for a clear subject outside the genres and `none` for a frame whose subject
cannot be told (dark, blurred, a misfire).

| Label | Frames |
|---|---|
| snap | 108 |
| sports | 63 |
| portrait | 62 |
| basketball | 59 |
| camping | 26 |
| other | 26 |
| landscape | 23 |
| birds / animals | 22 |
| group photo | 15 |
| none | 14 |
| pets | 11 |
| soccer | 0 |
| macro | 0 |

The sample is stratified by the provisional scores, so it over-represents
the frames each label already scores high on. The precision and recall below
are on this sample, not rates over a whole library; the library shares at the
end are the check against the real mix.

## Method

- A frame's score for a label is the cosine of its embedding with the
  label's text vector (the mean of its L2-normalized prompt vectors,
  normalized again). A label is on when its score reaches its threshold or
  one of its sub-genres is on; `Unknown` is on when no label is
  (`crates/core/src/genre.rs`).
- Each threshold maximizes F1 on the 300 frames (sub-genres first, then each
  parent given its sub-genres; ties take the higher threshold), rounded down
  to 4 decimals. F1 was chosen as the criterion before the first run.
- Leave one folder out (LOFO): each folder's frames are judged at the
  thresholds chosen on the other 55 folders. It shows how much choosing on the
  same 300 frames flatters the numbers. It does not cover the prompt choice
  below.
- The ground truth of `Unknown` is a frame none of the shipped labels applies
  to: `none`, plus `other` since `other` is not shipped (below).

## Prompts

Step 2's provisional prompts were compared against one revision written from
the labeling page's definitions (not from the misclassified frames): more
prompts for portrait, landscape, birds / animals, snap and camping, a
commemorative-photo prompt for group photo, concrete objects for other. LOFO
F1 per label (`Unknown` left out: this comparison predates the final
definition of its ground truth):

| Label | Provisional | Revision |
|---|---|---|
| sports | 0.90 | 0.90 |
| basketball | 0.93 | 0.93 |
| portrait | **0.54** | 0.51 |
| group photo | 0.36 | **0.64** |
| landscape | **0.56** | 0.50 |
| birds / animals | 0.59 | **0.63** |
| pets | 0.78 | 0.78 |
| snap | 0.51 | **0.64** |
| camping | 0.65 | **0.67** |
| other | **0.38** | 0.25 |

The shipped prompts take the better of the two per label (bold), so the
revised group photo, birds / animals, snap and camping prompts and the
provisional rest. This is a choice made on the same 300 frames, between two
options per label; the LOFO numbers below are slightly optimistic for it.

## Shipped labels at the chosen thresholds

| Label | Positives | Threshold | P | R | F1 | TP / FP | P (LOFO) | R (LOFO) | F1 (LOFO) |
|---|---|---|---|---|---|---|---|---|---|
| sports | 63 | 0.0958 | 0.92 | 0.94 | 0.93 | 59 / 5 | 0.90 | 0.90 | 0.90 |
| basketball | 59 | 0.1045 | 0.94 | 0.98 | 0.96 | 58 / 4 | 0.90 | 0.97 | 0.93 |
| portrait | 62 | 0.0544 | 0.51 | 0.66 | 0.57 | 41 / 40 | 0.49 | 0.63 | 0.55 |
| group photo | 15 | 0.0842 | 0.75 | 0.60 | 0.67 | 9 / 3 | 0.80 | 0.53 | 0.64 |
| landscape | 23 | 0.0652 | 0.50 | 0.74 | 0.60 | 17 / 17 | 0.48 | 0.65 | 0.56 |
| birds / animals | 22 | 0.0827 | 0.89 | 0.73 | 0.80 | 16 / 2 | 0.85 | 0.50 | 0.63 |
| pets | 11 | 0.0820 | 0.79 | 1.00 | 0.88 | 11 / 3 | 0.75 | 0.82 | 0.78 |
| snap | 108 | 0.0776 | 0.58 | 0.77 | 0.66 | 83 / 60 | 0.55 | 0.75 | 0.64 |
| camping | 26 | 0.0812 | 0.69 | 0.77 | 0.73 | 20 / 9 | 0.61 | 0.73 | 0.67 |
| Unknown | 38 | - | 0.29 | 0.29 | 0.29 | 11 / 27 | 0.26 | 0.29 | 0.27 |

Reading:

- **Sports and basketball work** (about 0.9 precision and recall held out).
  They are most of the author's library.
- **Birds / animals, pets and group photo are usable but thin** (11 to 22
  positives). Birds / animals keeps its precision held out (0.85) but its
  recall drops to 0.50.
- **Portrait, landscape and snap are weak: about half of what they let
  through is wrong** (held-out precision 0.49, 0.48, 0.55). No threshold
  fixes portrait: at 20% recall its precision is 0.65, at 40% 0.42. Most of
  the author's frames have a person in them, and a sports or snap frame with
  one prominent person scores as a portrait. Snap is a loose category (the
  user put 108 of 300 frames in it). They ship because the filter is a rough
  sort the user can ignore, and the usage docs say how often they are right;
  dropping them is the alternative if that is not good enough.
- **Unknown does not find unidentifiable frames.** The 14 `none` frames all
  clear some genre (a dark or blurred frame still lands near a genre's
  text), so `Unknown` (no label on) catches 11 of the 38 frames without a
  shipped genre and lets in 27 frames that have one. It means "no genre
  matched", not "nothing to see".

## Dropped labels

- **soccer and macro: dropped.** No labeled frame is either. The sampler
  drew 22 frames from the top of each one's ranking over the 37,779 library
  frames, and none was soccer or macro: the soccer stratum came back 17
  basketball, 3 snap / camping, 2 snap; the macro stratum 12 other, 4 landscape, 4
  none, 1 birds / animals + other, 1 snap / camping. So these labels fire on the wrong frames even at
  their most confident, and there is no positive to set a threshold on. A
  "conservative" threshold would be a guess that ships a filter item never
  seen to work; adding them back needs labeled positives (a soccer game, a
  macro session) and a re-run of both scripts.
- **other: dropped.** Precision 0.31 at its best F1, 0.27 held out, with
  either prompt set: generic prompts ("a photo", "a photo of an object")
  match nearly everything, and concrete-object prompts do worse. Its 26
  frames count as `Unknown` instead.

## Library shares

At the shipped thresholds, over the 37,779 frames of both embed.tsv files
(Step 2's 27 folders and the 52 extra folders):

| Label | Frames on | Share |
|---|---|---|
| sports | 29348 | 77.7% |
| basketball | 28945 | 76.6% |
| portrait | 1588 | 4.2% |
| group photo | 123 | 0.3% |
| landscape | 122 | 0.3% |
| birds / animals | 84 | 0.2% |
| pets | 69 | 0.2% |
| snap | 5435 | 14.4% |
| camping | 339 | 0.9% |
| Unknown | 2998 | 7.9% |

The library is mostly basketball, so the sports share matches what the
folders hold. The thresholds are one photographer's: on a library with
another mix (more landscapes, soccer, macro), they will fit worse, and the
data file is what to retune.
