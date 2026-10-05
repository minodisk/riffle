# Face ground truth: 36 M11-P DNGs plus `L1005161.DNG`

Counted by eye on 2026-10-06 on the `riffle-cli faces` PNG of each file (the
upright 2112x1408 or 1408x2112 preview, YuNet's boxes drawn in red), read with
the agent's image viewer. The PNGs are in `D:\Photos\tests\2026-10-06-face-recall\`,
not in the repository.

## Sample

`D:\photos\2026\2026-02-01` holds 146 DNGs. The 2026-09-22 measurement
sampled 36 of them "evenly from the folder listing" without recording which,
so the same rule was applied again: the sorted listing's entries at index
`floor(i * 146 / 36)` for `i` in 0..36. `L1005161.DNG` (the group of seven on
a swing) is not among them and is added as the 37th file. The file list is
also saved as `D:\Photos\tests\2026-10-06-face-recall\baseline\dng36.txt`.

On this sample the detector finds a face in 8 of the 36 files (the
2026-09-22 sample had 7 of 36), so the two samples overlap but are not the
same files.

## Counting rule

- A face counts when the eyes or at least one eye and the face around it are
  visible: frontal, three-quarter and profile, with a hat, goggles pushed up
  or a neck warmer over the mouth. Backs of heads, faces turned fully away and
  faces hidden behind a hand or snow do not count.
- Faces below about 40 preview pixels, partly cut off by the frame, or too
  occluded to be sure of are listed as **marginal** and kept out of the count
  and of the recall. Background crowds at 20-30 px are not listed at all.
- Sizes are the approximate face side (the longer of width and height of the
  head without the hat) in stored preview pixels, read off the image; they are
  good to about +-15%.
- **Found** is what `riffle-cli detect` reports on `main` (`6da10d28`, the
  whole-image path at the 320 input, faces at `CATCH_CONFIDENCE` 0.6). Every
  face found sits on a counted face; there is no false positive in the sample.

## Counts

| File | Faces | Marginal | Found | Smallest counted face (px) | Note |
|---|---|---|---|---|---|
| `L1005146.DNG` | 2 | 0 | 1 | ~95 | boy on the swing in three-quarter missed |
| `L1005164.DNG` | 7 | 0 | 0 | ~80 | same group as `L1005161`, none found |
| `L1005178.DNG` | 2 | 0 | 0 | ~85 | two adults with their backs turned not counted |
| `L1005194.DNG` | 2 | 0 | 0 | ~180 | two boys, large faces, motion blur |
| `L1005205.DNG` | 1 | 0 | 1 | ~170 | |
| `L1005233.DNG` | 2 | 0 | 0 | ~45 | two girls on an inflatable slide (portrait); the elephant's painted face not counted |
| `L1005259.DNG` | 1 | 1 | 0 | ~110 | marginal: child in the back, ~35 px, hand at the face |
| `L1005294.DNG` | 2 | 1 | 0 | ~110 | marginal: skier with goggles, ~50 px; crowd behind |
| `L1005300.DNG` | 1 | 1 | 0 | ~120 | marginal: child with goggles looking down, ~45 px |
| `L1005333.DNG` | 2 | 0 | 0 | ~85 | backlit; a child with the back turned not counted |
| `L1005357.DNG` | 1 | 0 | 0 | ~80 | profile, neck warmer over the chin |
| `L1005370.DNG` | 3 | 0 | 0 | ~110 | two adults (one masked, one looking down), a toddler |
| `L1005393.DNG` | 0 | 1 | 0 | - | marginal: child face down in the snow, face under the hat |
| `L1005408.DNG` | 2 | 0 | 0 | ~55 | girl ~85 px, man ~55 px; crowd at ~20 px |
| `L1005425.DNG` | 2 | 1 | 0 | ~50 | girl looking down, adult in a neck warmer; marginal: small child ~35 px |
| `L1005438.DNG` | 2 | 2 | 0 | ~55 | adult on the zip line, masked girl; marginal: small boy ~35 px, man cut by the frame |
| `L1005454.DNG` | 1 | 1 | 0 | ~55 | girl on the zip line, motion blur; marginal: walker in profile ~45 px |
| `L1005473.DNG` | 0 | 1 | 0 | - | marginal: girl on the zip line, face behind the arm |
| `L1005489.DNG` | 1 | 2 | 0 | ~50 | girl on a snow bike; marginal: two adults in profile ~40 px |
| `L1005506.DNG` | 2 | 0 | 0 | ~70 | boy with goggles, girl looking down; two backs not counted |
| `L1005512.DNG` | 1 | 2 | 0 | ~70 | girl with hands on her hat; marginal: boy behind the snow ~35 px, man ~45 px |
| `L1005538.DNG` | 3 | 0 | 0 | ~60 | girl in profile ~60 px; two backs and a silhouette not counted |
| `L1005553.DNG` | 1 | 0 | 1 | ~390 | |
| `L1005568.DNG` | 2 | 0 | 2 | ~230 | |
| `L1005576.DNG` | 2 | 0 | 2 | ~210 | |
| `L1005603.DNG` | 2 | 0 | 0 | ~70 | dark, against the snow; a child's back not counted |
| `L1005633.DNG` | 2 | 0 | 0 | ~75 | backlit against the sky |
| `L1005648.DNG` | 2 | 0 | 1 | ~90 | missed: goggles and a neck warmer over half the face |
| `L1005657.DNG` | 2 | 0 | 0 | ~85 | goggles and a hand over the face; hand over the mouth |
| `L1005672.DNG` | 1 | 0 | 1 | ~120 | |
| `L1005683.DNG` | 1 | 0 | 0 | ~70 | backlit, hair across the face |
| `L1005703.DNG` | 5 | 0 | 0 | ~75 | group of five, two with neck warmers or goggles |
| `L1005709.DNG` | 5 | 0 | 1 | ~75 | the same group; found the girl at the back |
| `L1005722.DNG` | 1 | 0 | 0 | ~250 | large laughing face under goggles, eyes squeezed shut |
| `L1005739.DNG` | 3 | 0 | 0 | ~65 | boy in profile ~65 px, child cut by the bottom edge |
| `L1005752.DNG` | 2 | 1 | 0 | ~60 | dark; marginal: girl lying sideways (face turned ~90 degrees) |
| `L1005161.DNG` | 7 | 0 | 2 | ~65 | the swing group; found the girl at the back left and the toddler (72 and 68 px boxes); missed the braided girl, the woman, the hooded boy, the boy looking down and the girl on the right |

## Totals

- 36 sampled files: 34 contain at least one counted face, 2 contain none
  (`L1005393`, `L1005473`, each with only an occluded face). 71 faces counted,
  14 marginal.
- Of the 28 files where the detector found no face, **26 contain at least one
  face** and 2 truly contain none. The todo item's "no face in 29 of 36"
  is therefore almost entirely misses, not empty frames.
- Recall on `main`: files 8 / 34 (23.5%), faces 10 / 71 (14.1%);
  `L1005161.DNG` 2 / 7.
- The smallest face that should count is about 45 preview px
  (`L1005233.DNG`); most missed faces are 55-120 px, i.e. 8-18 px at the
  320 input (6.6x shrink of the 2112 px long edge), at or below YuNet's
  stride-8 minimum. A few misses are large faces (`L1005194` 180 px with
  motion blur, `L1005722` 250 px under goggles with the eyes shut), which a
  larger input will not fix.
