# Head-pose measurement (Step 2)

The angles of `pose::head_pose` against the labels of
[`pose-truth.md`](pose-truth.md): 190 faces drawn, 159 readable. Every
number below is on the full pipeline with the `MAX_ROLL` guard this step
adds (it changes none of the 159). Raw outputs and the aggregation are in
`D:\Photos\tests\2026-10-07-head-pose\` (`results.txt` is the full output of
`aggregate.py`).

Class thresholds on the angle, the same as the labels': yaw `frontal` under
15 deg, `oblique` 15-50, `profile` over 50; pitch `up` over +15, `down`
under -15; roll `right` over +15, `left` under -15. Signs as `Pose`
documents them: yaw positive toward the image's right, pitch positive up,
roll positive clockwise.

## Variants

- **63 deg**: MediaPipe's default camera (`VERTICAL_FOV`), what Step 1
  ships.
- **EXIF FOV**: the vertical FOV of the file, `2 atan(e / 2f)` with `f` the
  EXIF focal length and `e` the full-frame side along the upright preview's
  height (24 mm landscape, 36 mm portrait). Both bodies are full frame, so
  the focal length is the 35 mm-equivalent one; `Shot` has no
  35 mm-equivalent tag for other sensors. These files run 14-25 deg
  (85-150 mm), all of them with a focal length.
- **Weak perspective**: the same pipeline with a 0.05 deg FOV, which makes
  the unprojection a uniform scale, so it is Procrustes on the pixel points
  (no new code).

## Agreement with the labels

| | 63 deg | EXIF FOV | Weak |
|---|---|---|---|
| Yaw sign (oblique + profile) | 76/81 (93.8%) | 76/81 | 76/81 |
| - oblique | 43/44 | 43/44 | 43/44 |
| - profile | 33/37 | 33/37 | 33/37 |
| Pitch sign (up + down) | 73/77 (94.8%) | 73/77 | 73/77 |
| - up | 60/63 | 60/63 | 60/63 |
| - down | 13/14 | 13/14 | 13/14 |
| Roll sign (left + right) | 15/18 (83.3%) | 15/18 | 15/18 |
| Yaw class | 108/159 (67.9%) | 107/159 (67.3%) | 105/159 (66.0%) |
| Pitch class | 125/159 (78.6%) | 124/159 (78.0%) | 122/159 (76.7%) |
| Roll class | 142/159 (89.3%) | 145/159 (91.2%) | 145/159 (91.2%) |

Over all 2024 judged faces the EXIF FOV moves an angle by a median of
1.1 deg (yaw), 0.8 (pitch), 0.9 (roll), p95 6.9 / 3.3 / 5.2; the weak
variant by 1.5 / 1.2 / 1.2, p95 9.0 / 4.4 / 7.2. The labels cannot tell them
apart: on the labeled faces the variants change a sign only of an angle
within about 8 deg of zero, on an axis labeled `level` or `frontal`.

## The angle per label (63 deg)

Median [quartiles] (n):

| Label | Angle |
|---|---|
| Yaw `frontal` | \|yaw\| 10 [5, 19] (78) |
| Yaw `oblique` | \|yaw\| 41 [28, 50] (44) |
| Yaw `profile` | \|yaw\| 65 [54, 73] (37) |
| Pitch `up` | +24 [+17, +31] (63) |
| Pitch `level` | +1 [-6, +7] (82) |
| Pitch `down` | -19 [-28, -10] (14) |
| Roll `left` | -18 [-20, +16] (10) |
| Roll `level` | 0 [-5, +7] (141) |
| Roll `right` | +17 [+16, +20] (8) |

## Confusion (63 deg; rows the label, columns the class the angle implies)

Yaw:

| | frontal | oblique | profile |
|---|---|---|---|
| frontal | 50 | 28 | 0 |
| oblique | 3 | 30 | 11 |
| profile | 2 | 7 | 28 |

Pitch:

| | up | level | down |
|---|---|---|---|
| up | 49 | 11 | 3 |
| level | 7 | 67 | 8 |
| down | 0 | 5 | 9 |

Roll:

| | left | level | right |
|---|---|---|---|
| left | 6 | 1 | 3 |
| level | 6 | 128 | 7 |
| right | 0 | 0 | 8 |

The yaw class is where the angle and the eye disagree most, and almost all
of it is at the frontal / oblique boundary: 28 faces labeled `frontal` read
15-42 deg. A face turned 20 deg still looks frontal to a reader; with the
boundary at 20 deg instead of 15 the agreement is 118/159 (74%), at 25 / 60
deg 120/159. The angle orders the classes correctly (the medians 10, 41,
65 are well apart) and no `frontal` face reads as a profile.

## Where it fails

- **Far profiles and sunglasses.** The 5 yaw sign misses: 4 labeled
  `profile` (two in sports sunglasses, one man turned past 90 deg with
  glasses, one man in a cap looking up) and 1 `oblique`
  boy whose eyes look at the camera while his head turns. Of the 4 pitch
  sign misses, 3 are profiles looking up that read -28 to -30 deg: on a
  far profile the mesh has only one side of the face.
- **Upside-down fits.** 15 of the 2024 judged faces fitted a roll past
  90 deg (pitch mostly -40 to -85, the yaw anywhere): every one is the back
  of a head, an ear, a blur or a far profile looking up, and all three
  angles of them are meaningless. `pose.rs` now returns `None` past
  `MAX_ROLL` = 90 deg (none of the 159 labeled faces is affected).
- **Yaw past 90.** 21 other faces read |yaw| over 90 deg with a sane roll:
  18 are readable far profiles and the sign is right on 16 of them, so they
  are kept; the magnitude overshoots there.
- **Roll.** 3 of the 18 labeled leans read the other way (+16 to +20 against
  a `left` label), two of them DNG portraits turned about 30 deg, where the
  line through the eyes the label is read from tilts with the turn.

## By face side (63 deg)

Yaw class agreement, yaw sign, pitch sign, by the face box's longer side in
stored preview pixels:

| Side | Yaw class | Yaw sign | Pitch sign | Unreadable (`x`) |
|---|---|---|---|---|
| < 60 px | 5/8 | 3/3 | 4/4 | 16/24 |
| 60-79 | 42/58 | 30/32 | 37/38 | 9/67 |
| 80-99 | 28/41 | 24/26 | 18/20 | 5/46 |
| >= 100 | 33/52 | 19/20 | 14/15 | 1/53 |

From `EYES_MIN_FACE` (60 px) up, the signs are as reliable at 60-79 px as at
100 px and over; the faces under it are mostly unreadable by eye, and
`Judged.pose` already has none there. **No floor beyond `EYES_MIN_FACE` is
needed.** ARW and DNG agree alike (yaw class 86/127 and 22/32). No judged
face was cut at the image edge (every face crop was square), so the
edge-clamp caveat did not occur on these folders.

## Solve time

`head_pose` on the 72 judged DNG faces, one thread, release build, 200
repetitions per face: median 8.2 µs, p95 18 µs, max 26 µs per face. The
face mesh model run it follows is about 50 ms.
