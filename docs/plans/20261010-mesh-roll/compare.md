# Before / after the eye-line roll correction (Step 2)

`compare.py` (pure Python over the Step 1 `meshfit` TSVs) prints the tables
below (the numbers marked "ad-hoc query" in the findings and in the Decision
come from separate queries over the same TSVs, not from the script); its output is pasted unedited under "Output". A variant picks, per
face, the `after` (rotated) fit or the `before` one from the same TSV row, so
no variant needed a re-run.

## Inputs

- `D:\Photos\tests\2026-10-09-mesh-roll\dump\*.tsv`: `riffle-cli meshfit`
  over the six folders, the nine `*-focus-sample` folders, `2026-02-01` and
  `good-mark-2026-10-09`, re-dumped in Step 2 after `meshfit` gained three
  columns (`yunet_eye_dist`, `yunet_mid_dx`, `yunet_mid_dy`: YuNet's eye
  distance and eye midpoint over the box's longer side, for the guards).
  Every other column is byte-identical to the Step 1 dump (266256 cells
  compared, the mesh times aside; ad-hoc query).
- The misfit truth set (`misfit-truth.md`, labels pending the user's review).
- AF eye: the XMP flags of the focus-sample folders, training `2026-06-05`,
  `2026-07-31`, `2026-09-13-a`, `2026-09-19`, `2026-09-19-focus-sample-2`
  (406 frames) and held-out `2026-06-14`, `2026-07-18`, `2026-08-01`,
  `2026-08-22` (400): Pick or Reject with a logit, as `fit.py` counted them.
- Closed eyes: the 253 open / closed faces of
  [eyes-truth.md](../_archived/20261007-closed-eyes-detection/eyes-truth.md)
  (57 closed); all 253 are in the dump with the face side of the label.
- Head pose: the 159 labeled faces of
  [pose-truth.md](../_archived/20261007-head-pose/pose-truth.md) (labels
  unreviewed).
- Stars: the 60 frames of `samples-scored.tsv` (the 60 the user starred, of
  the 180 now in the good-mark sample folder).

## Variants

- `before`: no rotation (the code on `main`); `always`: MediaPipe's step on
  every face.
- `band 10` / `band 20`: rotate only when |`yunet_roll`| is at least 10 / 20
  deg; `band 10-25`: only between 10 and 25 (added after the truth set's
  per-cell counts hinted that the harm sits above 25).
- `wide T`: rotate only when YuNet's eye distance is at least T of the box
  side (a guard for eye points that are not on two eyes), alone and with
  `band 10`.
- `band 10 + yaw 45 (2 pass)`: rotate only when the unrotated mesh's |yaw| is
  under 45 (a second mesh run on the faces past the band).

The guard threshold could not be chosen on the AF-eye training rows as
planned: `band 10 + wide T` rotates 13 of the 406 training frames at T =
0.10, 3 at 0.15 and none from 0.20 up, and the training AUC is 0.882 at every
T. T = 0.20 is taken instead from the eye-distance distribution of the six
folders (frontal faces sit at 0.30-0.40; the profiles whose eye points sit
together under 0.10) and from the truth set, where every frame whose label
the rotation changes has an eye distance of 0.14 or less; it is therefore
not a held-out choice, which does not matter for the Decision since the
guarded variant changes no labeled outcome.

## Findings

1. **The frozen numbers reproduce exactly from the dump** (`before`): AF eye
   held-out AUC 0.800, precision 88.6% (328 / 370), coverage 95.9%
   (328 / 342), training 0.882 / 93.9% / 91.4%; closed eyes AUC 0.974,
   accuracy 0.957 at `EYES_CLOSED_EAR` 0.137; head pose yaw sign 76 / 81
   (94%), pitch sign 73 / 77 (95%), roll sign 15 / 18, yaw class 108 / 159
   (68%).
2. **Always rotating loses on every labeled set but the pose.** AF eye
   held-out AUC 0.800 -> 0.783, precision 88.3%, coverage 95.3% (two picks
   lost), training 0.882 -> 0.875; the drop sits on the tilted frames
   (held-out |`yunet_roll`| 10-25: 0.716 -> 0.595, over 25: 0.770 -> 0.620;
   under 10: 0.787 -> 0.791). Closed eyes: AUC 0.974 -> 0.973, accuracy
   0.957 -> 0.949 (13 wrong against 11). Head pose: yaw sign 77 / 81, yaw
   class 109 / 159 (one face each). On the truth set it turns 7 `off` meshes
   `on` and 15 `on` meshes `off` (86 `on` of 131 against 94). The 1.7-point
   held-out AUC drop is at the edge of the noise `fit.md` named (0.01-0.02),
   but nothing improves to offset it.
3. **The bands do not help.** `band 10` matches `always` on the truth
   set and the pose, ties `before` on closed eyes (0.976 / 0.957 / 11 wrong)
   and is the worst variant on held-out AF AUC (0.779; `always` 0.783); `band 20` is worse on the
   truth set (83 `on`, 3 fixed, 14 broken). `band 10-25`, the only variant
   with a net gain on the truth set (97 `on`, 4 fixed, 1 broken, chosen
   after seeing the per-cell counts, so in-sample), still costs held-out AF
   AUC (0.787) and moves little else (yaw sign 76 -> 77, closed-eyes AUC 0.974 -> 0.976,
   six-folder count 2083 -> 2081).
4. **The eye-distance guard is neutral, because it rotates only faces that
   already fit.** `band 10 + wide 0.20` rotates 131 of the 7267 six-folder
   frames (1.8%), 0 of the training focus frames, 5 held-out, 3 closed-eyes
   and 1 head-pose face (ad-hoc query for the held-out, closed-eyes and pose counts); no truth-set label changes, every frozen number is
   identical but the yaw class (+1 face), the share over 0.10 is 2084
   against 2083. The 12 `tilted-wide` truth frames (tilted frontal faces,
   10-18 deg) are all `on` before the rotation: the mesh already copes with
   that roll. And it is not harmless: on the 4-star
   `2026-10-03__DSC3537` (a dark, blurred three-quarter face) it moves the
   EAR from 0.350 to 0.055, which would mark the eyes closed.
5. **Nothing of the 28.6% is recovered.** (28.6%, 2079 of 7261, is the
   Step 5 re-dump's index rows; this dump's population is 7267 meshed
   frames, so the shares below read 28.7%.) Over 0.10: 28.7% before, 28.7%
   always (178 frames fall under, 177 rise over), 28.9% `band 10`, 28.7%
   guarded. The reason is in the truth set: weighted back to the six
   folders, about 27% of the frames over 0.10 have a mesh off the face and
   about 65% an `on` mesh with YuNet's eye points off the eyes, so the
   offset over 0.10 is mostly a comparison artifact. YuNet's eye distance
   says where: the frames under 0.20 of the box side are 33.5% of the six
   folders (2435 of 7267) but 66% of the frames over 0.10 (1378 of 2083,
   56.6% of them over), against 14.6% over at 0.20 and up.
6. **What the real misfits are.** Weighted, `side` is 78% of the off
   meshes, then `cut`, `rotated`, `other` (6-7% each) and `notface`; under
   the 60 px floor nothing is measured here and at or above it no frame was
   off for being `small`. Rotation cannot fix `side`: on a profile YuNet's
   two eye points sit together on the visible eye or the nose, the eye-line
   angle is noise, and rotating by it is what breaks the 15 `on` meshes.
7. **The motivating frame is not an eye-line case.** `2026-07-11__DSC2638`
   (1 star; EAR 0.360 -> 0.260, mesh roll +12 -> +5, `eye_offset` 0.125 ->
   0.092 with the rotation, `yunet_roll` -11) is a baby lying with the head
   toward the bottom of the frame. YuNet orders its two eye points by image
   x (`faces.rs`), so the eye-line angle folds every roll into -90..+90 and
   a face past 90 deg reads as nearly level; no eye-line rotation can turn
   it upright.
8. **Misjudgments on the frames whose `before` mesh is off** (truth-set
   `off` or `before eye_offset` over 0.10): AF eye state against the flag
   wrong on 68 of 297 before, 71 always, 68 guarded; closed-eyes class
   2 / 47 in every variant; yaw sign 4 / 40 before, 3 always; yaw class 22 /
   56 before, 20 always; roll sign 2 / 9 throughout. The AF eye errors do
   concentrate there: 68 of 297 (23%) against 37 of the other 509 (7%); at
   or above the 60 px floor 35 of 190 (18%) against 16 of 408 (4%) (ad-hoc
   query). But the
   rotation does not reduce them (71 always, 68 guarded), because the off
   meshes behind them are side faces.
9. **Starred frames** (information only): of the 60, the guard rotates 6
   (5, 5, 4, 4, 3 and 1 star); the frames over 0.10 per star level are 0 / 2
   / 5 / 5 / 5 (5 to 1 stars) before and 0 / 1 / 5 / 5 / 3 always, 0 / 2 /
   5 / 5 / 4 guarded.

No number fell beyond noise under a variant worth adopting, so no re-fit
was run and there is no `frozen.json`.

## Output

### Misfit truth set (140 frames)

Strata: `off0.06-0.10|roll10-25` 9, `off0.06-0.10|roll<10` 9, `off0.06-0.10|roll>25` 9, `off0.10-0.20|roll10-25` 9, `off0.10-0.20|roll<10` 9, `off0.10-0.20|roll>25` 9, `off<0.06|roll10-25` 9, `off<0.06|roll<10` 9, `off<0.06|roll>25` 9, `off>0.20|roll10-25` 9, `off>0.20|roll<10` 9, `off>0.20|roll>25` 9, `random` 20, `tilted-wide` 12

| label | before | after (always) |
|---|---|---|
| on | 94 | 86 |
| rotated | 2 | 1 |
| side | 28 | 37 |
| small | 0 | 0 |
| occluded | 1 | 2 |
| cut | 1 | 1 |
| notface | 3 | 3 |
| other | 2 | 1 |
| x | 9 | 9 |

YuNet's eye points: `off` 85, `ok` 45, `x` 10

Share of `off` among the 131 readable frames, `before` mesh:

| bucket | n | off | on with `yunet off` |
|---|---|---|---|
| offset >0.20 | 30 | 24 (80%) | 6 |
| offset 0.10-0.20 | 27 | 9 (33%) | 17 |
| offset <0.06 | 37 | 2 (5%) | 11 |
| offset 0.06-0.10 | 37 | 2 (5%) | 18 |
| side >=200 | 31 | 7 (23%) | 5 |
| side 60-99 | 77 | 21 (27%) | 37 |
| side 100-199 | 23 | 9 (39%) | 10 |
| roll <10 | 47 | 6 (13%) | 17 |
| roll 10-25 | 47 | 14 (30%) | 15 |
| roll >25 | 37 | 17 (46%) | 20 |
| eye dist >=0.20 | 49 | 5 (10%) | 8 |
| eye dist 0.10-0.20 | 23 | 6 (26%) | 12 |
| eye dist <0.10 | 59 | 26 (44%) | 32 |

Over 0.10: 57 readable, 24 `on` (23 of them with `yunet off`); at or under 0.10: 74, 4 `off`.

Per variant over the readable frames (label of the fit the variant keeps):

| variant | on | off -> on | on -> off |
|---|---|---|---|
| before | 94 (72%) | 0 | 0 |
| always | 86 (66%) | 7 | 15 |
| band 10 | 86 (66%) | 7 | 15 |
| band 20 | 83 (63%) | 3 | 14 |
| band 10-25 | 97 (74%) | 4 | 1 |
| wide 0.15 | 94 (72%) | 0 | 0 |
| wide 0.20 | 94 (72%) | 0 | 0 |
| wide 0.25 | 94 (72%) | 0 | 0 |
| band 10 + wide 0.20 | 94 (72%) | 0 | 0 |
| band 10 + wide 0.25 | 94 (72%) | 0 | 0 |
| band 10 + yaw 45 (2 pass) | 93 (71%) | 0 | 1 |

Weighted to the six folders (the twelve strata only, readable frames):

- `before` off the face: 8.3% of the meshed AF frames
- of the frames over 0.10: 27% off, 65% on with `yunet off`
- off frames by kind: side 78%, cut 7%, rotated 6%, other 6%, notface 2%
- `always`: 1.7% of the frames off -> on, 1.3% on -> off

Frames whose label changed under `always`:

| frame | before | after | YuNet | yunet_roll | eye dist | before yaw | eye_offset |
|---|---|---|---|---|---|---|---|
| `2026-09-27-a__DSC5621` | off:side | on | off | +34 | 0.03 | -94 | 0.138 -> 0.094 |
| `2026-07-11__DSC2589` | off:side | on | ok | -27 | 0.14 | -72 | 0.044 -> 0.061 |
| `2026-09-19__DSC2309` | on | off:side | off | -40 | 0.04 | +77 | 0.041 -> 0.148 |
| `2026-09-19__DSC3744` | on | off:side | off | -53 | 0.03 | +63 | 0.104 -> 0.315 |
| `2026-09-27-a__DSC4415` | on | off:side | off | +71 | 0.04 | -77 | 0.122 -> 0.193 |
| `2026-09-27-a__DSC7045` | on | off:side | off | +49 | 0.03 | +90 | 0.023 -> 0.316 |
| `2026-09-27-a__DSC5587` | off:side | on | off | +18 | 0.05 | -113 | 0.073 -> 0.066 |
| `2026-09-19__DSC3334` | on | off:occluded | off | -30 | 0.11 | +74 | 0.055 -> 0.141 |
| `2026-10-03__DSC3054` | off:rotated | on | ok | +12 | 0.06 | - | 0.267 -> 0.051 |
| `2026-09-19__DSC2044` | on | off:side | off | +67 | 0.03 | -95 | 0.072 -> 0.223 |
| `2026-09-27-a__DSC7275` | on | off:side | off | +11 | 0.11 | -105 | 0.044 -> 0.043 |
| `2026-07-24__DSC4875` | on | off:side | off | +55 | 0.02 | -100 | 0.056 -> 0.132 |
| `2026-09-27-a__DSC7512` | off:side | on | off | +26 | 0.05 | -79 | 0.227 -> 0.123 |
| `2026-09-19__DSC3432` | on | off:side | off | -69 | 0.02 | +65 | 0.128 -> 0.223 |
| `2026-09-19__DSC2122` | on | off:side | off | -87 | 0.03 | +81 | 0.035 -> 0.274 |
| `2026-07-11__DSC2763` | off:side | on | off | -11 | 0.10 | +65 | 0.118 -> 0.087 |
| `2026-09-19__DSC2251` | on | off:side | off | -53 | 0.06 | -38 | 0.096 -> 0.331 |
| `2026-09-27-a__DSC7101` | on | off:side | off | -63 | 0.03 | +73 | 0.101 -> 0.107 |
| `2026-09-27-a__DSC7262` | on | off:side | off | -26 | 0.04 | +59 | 0.079 -> 0.143 |
| `2026-09-19__DSC3679` | on | off:side | off | +64 | 0.03 | -73 | 0.044 -> 0.120 |
| `2026-09-27-a__DSC7772` | on | off:side | off | -73 | 0.03 | +74 | 0.096 -> 0.268 |
| `2026-09-19__DSC3819` | off:other | on | off | +10 | 0.06 | -57 | 0.238 -> 0.093 |

### Six folders: share over MAX_EYE_OFFSET (AF, side >= 60, meshed)

| variant | over 0.10 of 7267 | fell under | rose over | net recovered of the before share | rotated |
|---|---|---|---|---|---|
| before | 2083 (28.7%) | 0 | 0 | +0.0% | 0 |
| always | 2082 (28.7%) | 178 | 177 | +0.0% | 7267 |
| band 10 | 2100 (28.9%) | 85 | 102 | -0.8% | 1018 |
| band 20 | 2108 (29.0%) | 33 | 58 | -1.2% | 428 |
| band 10-25 | 2081 (28.6%) | 59 | 57 | +0.1% | 704 |
| wide 0.15 | 2062 (28.4%) | 71 | 50 | +1.0% | 5514 |
| wide 0.20 | 2059 (28.3%) | 56 | 32 | +1.2% | 4832 |
| wide 0.25 | 2063 (28.4%) | 44 | 24 | +1.0% | 4170 |
| band 10 + wide 0.20 | 2084 (28.7%) | 5 | 6 | -0.0% | 131 |
| band 10 + wide 0.25 | 2087 (28.7%) | 1 | 5 | -0.2% | 108 |
| band 10 + yaw 45 (2 pass) | 2072 (28.5%) | 23 | 12 | +0.5% | 421 |

By `yunet_roll` and by YuNet eye distance (before -> always / band 10 + wide 0.20):

| bucket | n | before | always | band 10 + wide 0.20 |
|---|---|---|---|---|
| roll 10-25 | 704 | 63.1% | 62.8% | 63.4% |
| roll <10 | 6249 | 22.3% | 22.0% | 22.3% |
| roll >25 | 314 | 79.0% | 85.0% | 78.7% |
| eye dist 0.10-0.20 | 1412 | 47.6% | 47.3% | 47.6% |
| eye dist <0.10 | 1023 | 69.0% | 71.7% | 69.0% |
| eye dist >=0.20 | 4832 | 14.6% | 14.1% | 14.6% |

### AF eye (406 training / 400 held-out labeled frames)

| variant | train AUC | train prec | train cov | held AUC | held prec | held cov |
|---|---|---|---|---|---|---|
| before | 0.882 | 93.9% | 91.4% | 0.800 | 88.6% (328/370) | 95.9% (328/342) |
| always | 0.875 | 93.7% | 91.4% | 0.783 | 88.3% (326/369) | 95.3% (326/342) |
| band 10 | 0.881 | 93.9% | 91.2% | 0.779 | 88.3% (326/369) | 95.3% (326/342) |
| band 20 | 0.881 | 93.9% | 91.2% | 0.782 | 88.3% (326/369) | 95.3% (326/342) |
| band 10-25 | 0.882 | 93.9% | 91.4% | 0.787 | 88.6% (327/369) | 95.6% (327/342) |
| wide 0.15 | 0.871 | 93.7% | 91.4% | 0.803 | 88.6% (328/370) | 95.9% (328/342) |
| wide 0.20 | 0.882 | 93.9% | 91.4% | 0.801 | 88.6% (328/370) | 95.9% (328/342) |
| wide 0.25 | 0.882 | 93.9% | 91.4% | 0.801 | 88.6% (328/370) | 95.9% (328/342) |
| band 10 + wide 0.20 | 0.882 | 93.9% | 91.4% | 0.800 | 88.6% (328/370) | 95.9% (328/342) |
| band 10 + wide 0.25 | 0.882 | 93.9% | 91.4% | 0.800 | 88.6% (328/370) | 95.9% (328/342) |
| band 10 + yaw 45 (2 pass) | 0.883 | 93.9% | 91.4% | 0.780 | 88.3% (326/369) | 95.3% (326/342) |

Guard grid on the training rows (`band 10 + wide T`): train AUC, frames rotated

| T | train AUC | train prec | train cov | rotated |
|---|---|---|---|---|
| 0.10 | 0.882 | 93.9% | 91.4% | 13 |
| 0.15 | 0.882 | 93.9% | 91.4% | 3 |
| 0.20 | 0.882 | 93.9% | 91.4% | 0 |
| 0.25 | 0.882 | 93.9% | 91.4% | 0 |
| 0.30 | 0.882 | 93.9% | 91.4% | 0 |

Held-out split (AUC / precision / coverage; before -> always -> band 10 + wide 0.20):

| bucket | n | picks | before | always | band 10 + wide 0.20 |
|---|---|---|---|---|---|
| roll 10-25 | 35 | 24 | 0.716 / 69% / 92% | 0.595 / 68% / 88% | 0.716 / 69% / 92% |
| roll <10 | 345 | 308 | 0.787 / 92% / 96% | 0.791 / 92% / 96% | 0.787 / 92% / 96% |
| roll >25 | 20 | 10 | 0.770 / 59% / 100% | 0.620 / 53% / 90% | 0.770 / 59% / 100% |
| yaw 30-60 | 128 | 112 | 0.788 / 93% / 100% | 0.777 / 92% / 98% | 0.788 / 93% / 100% |
| yaw <30 | 224 | 189 | 0.829 / 87% / 93% | 0.814 / 86% / 93% | 0.829 / 87% / 93% |
| yaw >=60 | 45 | 39 | 0.667 / 88% / 97% | 0.671 / 88% / 97% | 0.667 / 88% / 97% |
| yaw none | 3 | 2 | 0.500 / 67% / 100% | 0.500 / 67% / 100% | 0.500 / 67% / 100% |
| side 100-199 | 104 | 97 | 0.928 / 94% / 100% | 0.886 / 94% / 98% | 0.928 / 94% / 100% |
| side 60-99 | 170 | 148 | 0.726 / 90% / 97% | 0.715 / 89% / 97% | 0.726 / 90% / 97% |
| side <60 | 103 | 74 | 0.688 / 76% / 88% | 0.688 / 76% / 88% | 0.688 / 76% / 88% |
| side >=200 | 23 | 23 | - / 100% / 100% | - / 100% / 100% | - / 100% / 100% |

Training split (same columns):

| bucket | n | picks | before | always | band 10 + wide 0.20 |
|---|---|---|---|---|---|
| roll 10-25 | 31 | 25 | 0.680 / 88% / 88% | 0.660 / 88% / 88% | 0.680 / 88% / 88% |
| roll <10 | 361 | 305 | 0.899 / 95% / 92% | 0.893 / 95% / 92% | 0.899 / 95% / 92% |
| roll >25 | 14 | 9 | 0.778 / 80% / 89% | 0.689 / 78% / 78% | 0.778 / 80% / 89% |
| yaw 30-60 | 146 | 121 | 0.920 / 93% / 95% | 0.902 / 94% / 96% | 0.920 / 93% / 95% |
| yaw <30 | 207 | 171 | 0.885 / 94% / 89% | 0.892 / 94% / 89% | 0.885 / 94% / 89% |
| yaw >=60 | 53 | 47 | 0.730 / 93% / 89% | 0.706 / 93% / 87% | 0.730 / 93% / 89% |
| side 100-199 | 91 | 81 | 0.900 / 99% / 94% | 0.949 / 97% / 94% | 0.900 / 99% / 94% |
| side 60-99 | 207 | 181 | 0.897 / 95% / 96% | 0.875 / 95% / 96% | 0.897 / 95% / 96% |
| side <60 | 105 | 74 | 0.768 / 87% / 78% | 0.768 / 87% / 78% | 0.768 / 87% / 78% |
| side >=200 | 3 | 3 | - / 100% / 100% | - / 100% / 100% | - / 100% / 100% |

### Closed eyes (253 labeled faces, 57 closed; 253 found in the dump, 0 with a face side more than 2 px off the label's)

| variant | AUC | accuracy | precision | recall | n | wrong |
|---|---|---|---|---|---|---|
| before | 0.974 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| always | 0.973 | 0.949 | 0.923 | 0.842 | 253 | 13 |
| band 10 | 0.976 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| band 20 | 0.974 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| band 10-25 | 0.976 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| wide 0.15 | 0.971 | 0.949 | 0.923 | 0.842 | 253 | 13 |
| wide 0.20 | 0.971 | 0.953 | 0.941 | 0.842 | 253 | 12 |
| wide 0.25 | 0.971 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| band 10 + wide 0.20 | 0.974 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| band 10 + wide 0.25 | 0.974 | 0.957 | 0.942 | 0.860 | 253 | 11 |
| band 10 + yaw 45 (2 pass) | 0.975 | 0.957 | 0.942 | 0.860 | 253 | 11 |

Split (AUC / accuracy; before -> always -> band 10 + wide 0.20):

| bucket | n | closed | before | always | band 10 + wide 0.20 |
|---|---|---|---|---|---|
| roll 10-25 | 10 | 3 | 0.571 / 0.700 | 0.714 / 0.700 | 0.571 / 0.700 |
| roll <10 | 240 | 54 | 0.994 / 0.967 | 0.991 / 0.958 | 0.994 / 0.967 |
| roll >25 | 3 | 0 | - / 1.000 | - / 1.000 | - / 1.000 |
| yaw 30-60 | 67 | 18 | 0.978 / 0.925 | 0.989 / 0.896 | 0.978 / 0.925 |
| yaw <30 | 170 | 38 | 0.996 / 0.971 | 0.989 / 0.971 | 0.995 / 0.971 |
| yaw >=60 | 16 | 1 | 0.000 / 0.938 | 0.067 / 0.938 | 0.000 / 0.938 |
| side 100-199 | 58 | 17 | 0.937 / 0.948 | 0.935 / 0.948 | 0.937 / 0.948 |
| side 60-99 | 167 | 35 | 0.991 / 0.958 | 0.991 / 0.952 | 0.991 / 0.958 |
| side <60 | 7 | 0 | - / 1.000 | - / 1.000 | - / 1.000 |
| side >=200 | 21 | 5 | 0.975 / 0.952 | 0.975 / 0.905 | 0.975 / 0.952 |

### Head pose (159 labeled faces, 159 found in the dump)

| variant | yaw sign | pitch sign | roll sign | yaw class | no pose |
|---|---|---|---|---|---|
| before | 76/81 (94%) | 73/77 (95%) | 15/18 | 108/159 (68%) | 0 |
| always | 77/81 (95%) | 73/77 (95%) | 15/18 | 109/159 (69%) | 0 |
| band 10 | 77/81 (95%) | 73/77 (95%) | 15/18 | 109/159 (69%) | 0 |
| band 20 | 76/81 (94%) | 73/77 (95%) | 15/18 | 110/159 (69%) | 0 |
| band 10-25 | 77/81 (95%) | 73/77 (95%) | 15/18 | 108/159 (68%) | 0 |
| wide 0.15 | 76/81 (94%) | 73/77 (95%) | 15/18 | 110/159 (69%) | 0 |
| wide 0.20 | 76/81 (94%) | 73/77 (95%) | 15/18 | 109/159 (69%) | 0 |
| wide 0.25 | 76/81 (94%) | 73/77 (95%) | 15/18 | 109/159 (69%) | 0 |
| band 10 + wide 0.20 | 76/81 (94%) | 73/77 (95%) | 15/18 | 109/159 (69%) | 0 |
| band 10 + wide 0.25 | 76/81 (94%) | 73/77 (95%) | 15/18 | 109/159 (69%) | 0 |
| band 10 + yaw 45 (2 pass) | 77/81 (95%) | 73/77 (95%) | 15/18 | 111/159 (70%) | 0 |

Split (yaw sign / yaw class agreeing; before -> always -> band 10 + wide 0.20):

| bucket | n | before | always | band 10 + wide 0.20 |
|---|---|---|---|---|
| roll 10-25 | 17 | 16/17 / 11/17 | 17/17 / 11/17 | 16/17 / 12/17 |
| roll <10 | 138 | 56/60 / 94/138 | 56/60 / 94/138 | 56/60 / 94/138 |
| roll >25 | 4 | 4/4 / 3/4 | 4/4 / 4/4 | 4/4 / 3/4 |
| yaw 30-60 | 43 | 38/40 / 26/43 | 38/40 / 25/43 | 38/40 / 26/43 |
| yaw <30 | 90 | 12/15 / 59/90 | 13/15 / 62/90 | 12/15 / 60/90 |
| yaw >=60 | 26 | 26/26 / 23/26 | 26/26 / 22/26 | 26/26 / 23/26 |
| side 100-199 | 35 | 14/15 / 23/35 | 14/15 / 23/35 | 14/15 / 23/35 |
| side 60-99 | 101 | 54/58 / 70/101 | 55/58 / 72/101 | 54/58 / 71/101 |
| side <60 | 8 | 3/3 / 5/8 | 3/3 / 4/8 | 3/3 / 5/8 |
| side >=200 | 15 | 5/5 / 10/15 | 5/5 / 10/15 | 5/5 / 10/15 |

### Misjudgments on the frames whose `before` mesh is off

The truth set's `before off` frames (37) come from the six folders; only `2026-09-19` overlaps a labeled set (the closed-eyes / head-pose faces), so the restriction below is `before eye_offset` > 0.10 or a `before off` label.

| variant | AF eye state wrong | eyes class wrong | yaw sign wrong | yaw class wrong | roll sign wrong | no pose |
|---|---|---|---|---|---|---|
| before | 68/297 | 2/47 | 4/40 | 22/56 | 2/9 | 0 |
| always | 71/297 | 2/47 | 3/40 | 20/56 | 2/9 | 0 |
| band 10 | 71/297 | 2/47 | 3/40 | 21/56 | 2/9 | 0 |
| band 20 | 71/297 | 2/47 | 4/40 | 20/56 | 2/9 | 0 |
| band 10 + wide 0.20 | 68/297 | 2/47 | 4/40 | 22/56 | 2/9 | 0 |
| band 10 + wide 0.25 | 68/297 | 2/47 | 4/40 | 22/56 | 2/9 | 0 |

### The 60 starred frames of `samples-scored.tsv` (information only)

| frame | stars | yunet_roll | eye dist | guard rotates | eye_focus | EAR | yaw | roll | eye_offset | moved |
|---|---|---|---|---|---|---|---|---|---|---|
| `2026-07-24__DSC4825` | 5 | -23 | 0.36 | yes | 1.00 -> 1.00 | 0.295 -> 0.346 | -6 -> -5 | -24 -> -33 | 0.064 -> 0.054 | * |
| `2026-07-24__DSC4850` | 5 | -0 | 0.38 |  | 1.00 -> 1.00 | 0.297 -> 0.302 | +2 -> +2 | +1 -> +1 | 0.016 -> 0.016 |  |
| `2026-07-24__DSC4984` | 5 | -5 | 0.38 |  | 1.00 -> 1.00 | 0.345 -> 0.347 | -6 -> -6 | -7 -> -8 | 0.029 -> 0.031 |  |
| `2026-08-08__DSC0009` | 5 | +2 | 0.37 |  | 1.00 -> 1.00 | 0.331 -> 0.335 | +6 -> +6 | +1 -> +1 | 0.049 -> 0.050 |  |
| `2026-08-08__DSC0117` | 5 | +8 | 0.40 |  | 0.99 -> 0.99 | 0.266 -> 0.251 | +14 -> +14 | +8 -> +8 | 0.020 -> 0.023 | * |
| `2026-08-08__DSC0165` | 5 | +0 | 0.36 |  | 1.00 -> 1.00 | 0.390 -> 0.387 | -2 -> -2 | +2 -> +2 | 0.013 -> 0.013 |  |
| `2026-08-08__DSC0212` | 5 | -14 | 0.38 | yes | 1.00 -> 1.00 | 0.309 -> 0.257 | -15 -> -14 | -23 -> -26 | 0.075 -> 0.073 | * |
| `2026-07-24__DSC4868` | 4 | +3 | 0.36 |  | 1.00 -> 1.00 | 0.294 -> 0.291 | -25 -> -25 | +11 -> +11 | 0.045 -> 0.045 |  |
| `2026-07-24__DSC4884` | 4 | -11 | 0.30 | yes | 1.00 -> 1.00 | 0.379 -> 0.356 | -31 -> -32 | -9 -> -12 | 0.046 -> 0.061 | * |
| `2026-07-24__DSC4988` | 4 | -39 | 0.20 |  | 0.99 -> 1.00 | 0.409 -> 0.378 | -54 -> -57 | -12 -> -19 | 0.178 -> 0.067 | * |
| `2026-08-08__DSC0148` | 4 | -4 | 0.30 |  | 0.99 -> 0.99 | 0.325 -> 0.314 | -34 -> -35 | -2 -> -3 | 0.041 -> 0.045 | * |
| `2026-08-08__DSC0173` | 4 | -5 | 0.29 |  | 1.00 -> 1.00 | 0.418 -> 0.427 | +39 -> +39 | -2 -> -2 | 0.034 -> 0.031 |  |
| `2026-09-27-a__DSC7351` | 4 | +0 | 0.36 |  | 1.00 -> 1.00 | 0.368 -> 0.365 | -2 -> -2 | +7 -> +7 | 0.022 -> 0.022 |  |
| `2026-10-03__DSC2142` | 4 | -1 | 0.38 |  | 1.00 -> 1.00 | 0.318 -> 0.321 | +3 -> +3 | -2 -> -2 | 0.019 -> 0.020 |  |
| `2026-10-03__DSC2538` | 4 | -0 | 0.37 |  | 0.99 -> 0.99 | 0.276 -> 0.275 | +14 -> +14 | +6 -> +7 | 0.020 -> 0.019 |  |
| `2026-10-03__DSC3186` | 4 | -1 | 0.39 |  | 1.00 -> 1.00 | 0.324 -> 0.322 | +11 -> +10 | +7 -> +7 | 0.058 -> 0.058 |  |
| `2026-10-03__DSC3537` | 4 | -24 | 0.21 | yes | 0.68 -> 0.68 | 0.350 -> 0.055 | +33 -> +16 | +1 -> -19 | 0.121 -> 0.161 | * |
| `2026-10-03__DSC3553` | 4 | +0 | 0.28 |  | 0.81 -> 0.81 | 0.236 -> 0.241 | -36 -> -36 | -4 -> -4 | 0.093 -> 0.092 |  |
| `2026-07-11__DSC2588` | 3 | +2 | 0.37 |  | 1.00 -> 1.00 | 0.301 -> 0.288 | +2 -> +1 | +0 -> +0 | 0.027 -> 0.026 | * |
| `2026-07-11__DSC2625` | 3 | -16 | 0.33 | yes | 0.99 -> 0.99 | 0.335 -> 0.355 | +32 -> +40 | -21 -> -23 | 0.035 -> 0.038 | * |
| `2026-07-11__DSC2636` | 3 | -2 | 0.22 |  | 0.98 -> 1.00 | 0.182 -> 0.203 | +18 -> +17 | -16 -> -19 | 0.117 -> 0.155 | * |
| `2026-07-11__DSC2663` | 3 | -0 | 0.16 |  | 1.00 -> 1.00 | 0.495 -> 0.516 | +54 -> +58 | +5 -> +5 | 0.054 -> 0.047 | * |
| `2026-07-11__DSC2850` | 3 | +52 | 0.15 |  | 1.00 -> 1.00 | 0.249 -> 0.146 | -34 -> -25 | +18 -> +35 | 0.306 -> 0.289 | * |
| `2026-07-11__DSC2899` | 3 | -0 | 0.36 |  | 1.00 -> 1.00 | 0.307 -> 0.307 | -3 -> -3 | +13 -> +13 | 0.067 -> 0.067 |  |
| `2026-07-11__DSC2957` | 3 | +2 | 0.27 |  | 1.00 -> 1.00 | 0.342 -> 0.349 | -40 -> -40 | -2 -> -2 | 0.043 -> 0.041 |  |
| `2026-08-08__DSC0279` | 3 | -1 | 0.41 |  | 1.00 -> 1.00 | 0.089 -> 0.083 | -19 -> -20 | +2 -> +2 | 0.031 -> 0.032 |  |
| `2026-09-19__DSC2686` | 3 | -2 | 0.36 |  | 1.00 -> 1.00 | 0.345 -> 0.347 | +15 -> +14 | +7 -> +7 | 0.047 -> 0.046 |  |
| `2026-09-19__DSC2948` | 3 | +1 | 0.15 |  | 0.94 -> 1.00 | 0.351 -> 0.348 | -46 -> -46 | +6 -> +6 | 0.108 -> 0.112 |  |
| `2026-09-19__DSC3039` | 3 | -4 | 0.23 |  | 1.00 -> 1.00 | 0.383 -> 0.367 | +38 -> +38 | -5 -> -6 | 0.057 -> 0.056 | * |
| `2026-09-19__DSC3676` | 3 | -0 | 0.31 |  | 1.00 -> 1.00 | 0.277 -> 0.273 | -24 -> -23 | -3 -> -3 | 0.053 -> 0.053 |  |
| `2026-09-19__DSC3914` | 3 | +3 | 0.39 |  | 1.00 -> 1.00 | 0.378 -> 0.389 | +16 -> +16 | +12 -> +14 | 0.080 -> 0.084 | * |
| `2026-09-27-a__DSC4366` | 3 | +2 | 0.13 |  | 1.00 -> 1.00 | 0.316 -> 0.318 | -57 -> -58 | +2 -> +2 | 0.075 -> 0.075 |  |
| `2026-09-27-a__DSC4673` | 3 | +2 | 0.39 |  | 1.00 -> 1.00 | 0.298 -> 0.262 | -8 -> -7 | -10 -> -13 | 0.329 -> 0.332 | * |
| `2026-09-27-a__DSC4955` | 3 | +37 | 0.04 |  | 0.94 -> 0.94 | 0.363 -> 0.222 | -66 -> -66 | +1 -> +13 | 0.121 -> 0.218 | * |
| `2026-09-27-a__DSC4973` | 3 | +3 | 0.29 |  | 1.00 -> 1.00 | 0.293 -> 0.317 | -37 -> -35 | +19 -> +20 | 0.042 -> 0.042 | * |
| `2026-09-27-a__DSC6746` | 3 | -0 | 0.35 |  | 1.00 -> 1.00 | 0.365 -> 0.370 | +20 -> +20 | +6 -> +6 | 0.037 -> 0.036 |  |
| `2026-09-27-a__DSC8083` | 3 | -0 | 0.31 |  | 0.99 -> 0.99 | 0.241 -> 0.235 | -12 -> -12 | +1 -> +1 | 0.029 -> 0.029 |  |
| `2026-10-03__DSC2586` | 3 | -1 | 0.38 |  | 1.00 -> 1.00 | 0.295 -> 0.305 | -5 -> -4 | -2 -> -2 | 0.040 -> 0.040 |  |
| `2026-10-03__DSC2748` | 3 | +1 | 0.38 |  | 1.00 -> 1.00 | 0.303 -> 0.297 | -3 -> -3 | -2 -> -2 | 0.048 -> 0.047 |  |
| `2026-10-03__DSC2772` | 3 | -1 | 0.37 |  | 1.00 -> 1.00 | 0.309 -> 0.313 | +2 -> +1 | +2 -> +1 | 0.025 -> 0.024 |  |
| `2026-10-03__DSC3254` | 3 | -2 | 0.35 |  | 1.00 -> 1.00 | 0.278 -> 0.281 | -10 -> -10 | +11 -> +10 | 0.086 -> 0.080 |  |
| `2026-07-24__DSC4998` | 2 | -2 | 0.31 |  | 1.00 -> 0.99 | 0.170 -> 0.125 | +41 -> +37 | +4 -> +4 | 0.147 -> 0.177 | * |
| `2026-07-24__DSC5035` | 2 | -35 | 0.03 |  | 1.00 -> 1.00 | 0.343 -> 0.341 | +19 -> +19 | -1 -> -36 | 0.220 -> 0.367 | * |
| `2026-08-08__DSC9962` | 2 | +1 | 0.34 |  | 0.90 -> 0.90 | 0.106 -> 0.108 | +3 -> +3 | +1 -> +1 | 0.041 -> 0.042 |  |
| `2026-09-19__DSC1805` | 2 | +1 | 0.35 |  | 1.00 -> 1.00 | 0.256 -> 0.254 | +18 -> +18 | -2 -> -1 | 0.037 -> 0.037 |  |
| `2026-09-19__DSC2373` | 2 | -3 | 0.32 |  | 1.00 -> 1.00 | 0.323 -> 0.317 | -31 -> -32 | -4 -> -4 | 0.026 -> 0.026 |  |
| `2026-09-19__DSC2376` | 2 | +7 | 0.13 |  | 0.95 -> 0.95 | 0.253 -> 0.259 | -70 -> -68 | +4 -> +3 | 0.032 -> 0.034 |  |
| `2026-09-19__DSC3385` | 2 | -17 | 0.09 |  | 0.88 -> 0.88 | 0.495 -> 0.409 | +10 -> +16 | -8 -> -27 | 0.130 -> 0.151 | * |
| `2026-09-27-a__DSC6692` | 2 | -2 | 0.28 |  | 0.22 -> 0.22 | 0.244 -> 0.232 | -17 -> -16 | -3 -> -3 | 0.072 -> 0.079 | * |
| `2026-09-27-a__DSC7404` | 2 | +3 | 0.42 |  | 0.93 -> 0.92 | 0.264 -> 0.259 | -0 -> -9 | -4 -> +0 | 0.138 -> 0.169 | * |
| `2026-10-03__DSC2541` | 2 | -3 | 0.37 |  | 0.95 -> 0.95 | 0.187 -> 0.165 | -13 -> -11 | +2 -> +3 | 0.138 -> 0.140 | * |
| `2026-07-11__DSC2638` | 1 | -11 | 0.25 | yes | 1.00 -> 1.00 | 0.360 -> 0.260 | +3 -> +11 | +12 -> +5 | 0.125 -> 0.092 | * |
| `2026-07-11__DSC2827` | 1 | +9 | 0.12 |  | 1.00 -> 0.95 | 0.303 -> 0.479 | -49 -> -44 | +9 -> +7 | 0.087 -> 0.092 | * |
| `2026-07-24__DSC4867` | 1 | +3 | 0.35 |  | 0.99 -> 0.99 | 0.021 -> 0.012 | -20 -> -19 | +9 -> +9 | 0.045 -> 0.046 |  |
| `2026-09-19__DSC3345` | 1 | +4 | 0.24 |  | 1.00 -> 1.00 | 0.314 -> 0.323 | -15 -> -24 | -4 -> -3 | 0.352 -> 0.306 | * |
| `2026-09-19__DSC3512` | 1 | -4 | 0.32 |  | 0.18 -> 0.18 | 0.362 -> 0.396 | +18 -> +4 | -9 -> -11 | 0.139 -> 0.189 | * |
| `2026-09-27-a__DSC6920` | 1 | -17 | 0.06 |  | 0.99 -> 0.91 | 0.405 -> 0.253 | +27 -> +53 | +4 -> -11 | 0.177 -> 0.067 | * |
| `2026-09-27-a__DSC7307` | 1 | -4 | 0.37 |  | 0.23 -> 0.23 | 0.257 -> 0.256 | -7 -> -4 | +4 -> +0 | 0.089 -> 0.088 |  |
| `2026-10-03__DSC2050` | 1 | +4 | 0.21 |  | 1.00 -> 1.00 | 0.323 -> 0.328 | -37 -> -36 | +5 -> +5 | 0.024 -> 0.028 |  |
| `2026-10-03__DSC3508` | 1 | -33 | 0.09 |  | 0.65 -> 0.65 | 0.130 -> 0.553 | -65 -> +89 | -8 -> -5 | 0.360 -> 0.203 | * |

- 5 stars: 7 frames, over 0.10 before 0, always 0, guard 0
- 4 stars: 11 frames, over 0.10 before 2, always 1, guard 2
- 3 stars: 23 frames, over 0.10 before 5, always 5, guard 5
- 2 stars: 10 frames, over 0.10 before 5, always 5, guard 5
- 1 stars: 9 frames, over 0.10 before 5, always 3, guard 4

