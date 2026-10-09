# Misfit truth set (Step 2)

Hand labels of whether the face mesh sits on the eyes, before and after the
eye-line roll correction, on 140 frames of the six folders of the
burst-keep-score Step 5 re-dump. The agent labeled every frame; **the user's
review of the labels is pending**.

Raw outputs are in `D:\Photos\tests\2026-10-09-mesh-roll\`, out of the
repository: the `riffle-cli meshfit` dump and its overlays (`dump\`), and in
`sheets\` the draw and sheet script (`draw.py`, Pillow), the draw
(`sample.tsv`: number, folder, file, stratum), the labels (`labels.tsv`:
number, before, after, YuNet) and the sheets they were read from
(`s00.png`-`s15.png`, 3 tiles across, read row by row in number order).

## Which frames

The frames of the six folders (`2026-07-11`, `2026-07-24`, `2026-08-08`,
`2026-09-19`, `2026-09-27-a`, `2026-10-03`) the scan meshes and Step 5 cuts
on: an AF point, the face nearest it at or above `EYES_MIN_FACE` = 60 px,
a `before eye_offset`. 7267 frames, 2083 (28.7%) over `MAX_EYE_OFFSET` =
0.10. Drawn with seed 20261010, then shuffled together so the stratum, the
offsets and the file were hidden while labeling (a tile shows only its
number):

- 9 from each cell of `before eye_offset` (under 0.06, 0.06-0.10, 0.10-0.20,
  over 0.20) x |`yunet_roll`| (under 10, 10-25, over 25): 108.
- 12 `tilted-wide`: |`yunet_roll`| >= 10 with YuNet's eye distance at least
  0.25 of the box side, the tilted faces whose eye line is likely a real
  one (the first look of Step 1 showed most large `yunet_roll` values come
  from profiles whose two YuNet eye points sit together).
- 20 at random from the rest.

## Labels

Each tile is the Step 1 overlay: the `eyes::face_square` window of the
upright preview, YuNet's box (yellow) and two eye points (cyan), the
`before` mesh's two eyelid contours (red) and the `after` mesh's (green,
drawn over the red, so a fit that did not move shows green only).

- `before` / `after`: `on` when the eyelid contours sit on the eyes. On a
  profile, where one eye is hidden, `on` means the visible eye's contour is
  on it and the other sits behind the nose bridge where the hidden eye is; a
  contour on the cheek, the temple, the ear or the hair makes it `off`.
  `off` carries the kind: `rotated` (in-plane rotated face), `side` (profile
  or far turn: the contours land on the cheek / ear / hair of a turned
  head), `small`, `occluded` (hand, hair, glasses, goggles), `cut` (by the
  frame), `notface` (back of a head, two faces merged in one box), `other`
  (e.g. a face seen from straight above). `x` when the frame cannot be read
  (a heavy blur or a dark face).
- `YuNet`: `ok` when the two cyan points sit on two distinct eyes, `off`
  when they do not (most often both on the one visible eye or the nose of a
  profile, or one on the nose bridge or a brow), `x` unreadable. A large
  `eye_offset` with an `on` mesh and `YuNet off` is a comparison artifact,
  not a misfit.

## Results (acceptance criterion 1)

From `compare.py` (the full tables are in `compare.md`):

| Label | before | after (always rotate) |
|-------|--------|-----------------------|
| on | 94 | 86 |
| off: side | 28 | 37 |
| off: notface | 3 | 3 |
| off: rotated | 2 | 1 |
| off: other | 2 | 1 |
| off: occluded | 1 | 2 |
| off: cut | 1 | 1 |
| off: small | 0 | 0 |
| x | 9 | 9 |

YuNet's eye points: 45 `ok`, 85 `off`, 10 `x`.

Share of `off` among the 131 readable frames (`before` mesh), with the
frames whose large offset is a comparison artifact:

| Bucket | n | off | on with `YuNet off` |
|--------|---|-----|---------------------|
| offset < 0.06 | 37 | 2 (5%) | 11 |
| offset 0.06-0.10 | 37 | 2 (5%) | 18 |
| offset 0.10-0.20 | 27 | 9 (33%) | 17 |
| offset > 0.20 | 30 | 24 (80%) | 6 |
| side 60-99 px | 77 | 21 (27%) | 37 |
| side 100-199 px | 23 | 9 (39%) | 10 |
| side >= 200 px | 31 | 7 (23%) | 5 |
| \|yunet_roll\| < 10 | 47 | 6 (13%) | 17 |
| \|yunet_roll\| 10-25 | 47 | 14 (30%) | 15 |
| \|yunet_roll\| > 25 | 37 | 17 (46%) | 20 |
| YuNet eye distance < 0.10 | 59 | 26 (44%) | 32 |
| YuNet eye distance 0.10-0.20 | 23 | 6 (26%) | 12 |
| YuNet eye distance >= 0.20 | 49 | 5 (10%) | 8 |

- **0.10 does not read as "off the face".** Of the 57 readable frames over
  0.10, 24 have an `on` mesh, 23 of them with YuNet's points off the eyes;
  in the 0.10-0.20 bucket two frames in three are `on`. Under 0.10, 4 of 74
  are `off`. Weighted back to the six folders by the twelve cells, about
  8% of the meshed AF frames have a mesh off the face, and of the 28.7% over
  0.10 about 27% are off and 65% are `on` with `YuNet off`: the offset
  measures YuNet's eye points about as much as the mesh.
- **The misfits are side faces.** Weighted, `side` is 78% of the `off`
  frames, then `cut` 7%, `rotated` 6%, `other` 6%, `notface` 2%; no frame
  at or above 60 px was off for being `small`. Only two frames are a
  `rotated` misfit: `2026-10-03/_DSC3054.ARW` (#42, a head thrown back,
  fixed by the rotation) and `2026-07-11/_DSC2629.ARW` (#120, a baby lying
  on its side, still off after it: YuNet's eye points are on the cheeks,
  not the eyes, so the eye line is wrong). The tilted frontal faces of the
  `tilted-wide` slice are all `on` before the rotation already.
- **Where the rotation changes a label, YuNet's eye line is not a real one.**
  All 22 frames whose label changed under always-rotate have a YuNet eye
  distance of 0.14 of the box side or less (the frontal faces sit at
  0.30-0.40) and 21 of them `YuNet off`: 7 went `off` -> `on`, 15 `on` ->
  `off`. No frame at an eye distance of 0.20 or more changed its label.

## Frames

`Side` is the face box's longer side in stored preview pixels, `Eye dist`
YuNet's eye distance over the box's longer side.

| # | Frame | Stratum | Side | yunet_roll | Eye dist | eye_offset before / after | Before | After | YuNet |
|---|-------|---------|------|------------|----------|---------------------------|--------|-------|-------|
| 0 | `2026-08-08/_DSC0186.ARW` | offset >0.20 / roll<10 | 415 | -2 | 0.43 | 0.504 / 0.500 | off:cut | off:cut | x |
| 1 | `2026-09-19/_DSC3196.ARW` | offset 0.10-0.20 / roll10-25 | 73 | -19 | 0.10 | 0.135 / 0.145 | on | on | off |
| 2 | `2026-10-03/_DSC1959.ARW` | random | 69 | +3 | 0.25 | 0.031 / 0.031 | on | on | ok |
| 3 | `2026-09-27-a/_DSC5621.ARW` | offset 0.10-0.20 / roll>25 | 72 | +34 | 0.03 | 0.138 / 0.094 | off:side | on | off |
| 4 | `2026-07-11/_DSC2627.ARW` | tilted-wide | 356 | +12 | 0.32 | 0.012 / 0.013 | on | on | ok |
| 5 | `2026-07-11/_DSC2589.ARW` | offset <0.06 / roll>25 | 226 | -27 | 0.14 | 0.044 / 0.061 | off:side | on | ok |
| 6 | `2026-09-27-a/_DSC4644.ARW` | offset >0.20 / roll<10 | 87 | +5 | 0.14 | 0.247 / 0.298 | off:side | off:side | off |
| 7 | `2026-09-27-a/_DSC4948.ARW` | offset 0.10-0.20 / roll<10 | 82 | +6 | 0.33 | 0.156 / 0.211 | x | x | x |
| 8 | `2026-07-11/_DSC2886.ARW` | offset <0.06 / roll10-25 | 92 | -18 | 0.08 | 0.053 / 0.065 | on | on | ok |
| 9 | `2026-08-08/_DSC0275.ARW` | offset <0.06 / roll10-25 | 292 | -15 | 0.37 | 0.042 / 0.043 | on | on | ok |
| 10 | `2026-10-03/_DSC1965.ARW` | offset 0.10-0.20 / roll10-25 | 81 | -25 | 0.03 | 0.164 / 0.168 | on | on | off |
| 11 | `2026-07-11/_DSC2804.ARW` | offset 0.10-0.20 / roll<10 | 90 | -2 | 0.18 | 0.152 / 0.142 | x | x | x |
| 12 | `2026-07-11/_DSC2731.ARW` | random | 154 | -3 | 0.20 | 0.074 / 0.074 | on | on | ok |
| 13 | `2026-09-19/_DSC2309.ARW` | offset <0.06 / roll>25 | 131 | -40 | 0.04 | 0.041 / 0.148 | on | off:side | off |
| 14 | `2026-09-27-a/_DSC7104.ARW` | offset 0.10-0.20 / roll>25 | 72 | -34 | 0.06 | 0.190 / 0.349 | off:side | off:side | off |
| 15 | `2026-10-03/_DSC2090.ARW` | offset >0.20 / roll>25 | 123 | -48 | 0.03 | 0.216 / 0.281 | off:side | off:side | off |
| 16 | `2026-09-19/_DSC3744.ARW` | offset 0.10-0.20 / roll>25 | 131 | -53 | 0.03 | 0.104 / 0.315 | on | off:side | off |
| 17 | `2026-09-27-a/_DSC4137.ARW` | random | 139 | -1 | 0.32 | 0.016 / 0.017 | on | on | ok |
| 18 | `2026-09-27-a/_DSC4415.ARW` | offset 0.10-0.20 / roll>25 | 93 | +71 | 0.04 | 0.122 / 0.193 | on | off:side | off |
| 19 | `2026-09-27-a/_DSC5986.ARW` | offset 0.10-0.20 / roll>25 | 77 | +30 | 0.04 | 0.165 / 0.177 | off:side | off:side | off |
| 20 | `2026-07-11/_DSC2838.ARW` | tilted-wide | 378 | +12 | 0.41 | 0.049 / 0.050 | on | on | ok |
| 21 | `2026-09-27-a/_DSC5393.ARW` | offset <0.06 / roll<10 | 86 | +4 | 0.23 | 0.045 / 0.044 | on | on | ok |
| 22 | `2026-10-03/_DSC2610.ARW` | random | 79 | +0 | 0.38 | 0.062 / 0.062 | on | on | ok |
| 23 | `2026-07-24/_DSC5015.ARW` | offset <0.06 / roll10-25 | 355 | +17 | 0.31 | 0.030 / 0.050 | on | on | ok |
| 24 | `2026-09-27-a/_DSC6909.ARW` | random | 94 | +9 | 0.17 | 0.103 / 0.074 | x | x | x |
| 25 | `2026-09-27-a/_DSC7045.ARW` | offset <0.06 / roll>25 | 86 | +49 | 0.03 | 0.023 / 0.316 | on | off:side | off |
| 26 | `2026-07-11/_DSC2996.ARW` | offset <0.06 / roll<10 | 264 | -0 | 0.18 | 0.036 / 0.033 | on | on | ok |
| 27 | `2026-09-19/_DSC2127.ARW` | random | 82 | -4 | 0.28 | 0.109 / 0.114 | off:occluded | off:occluded | off |
| 28 | `2026-09-19/_DSC2051.ARW` | offset >0.20 / roll<10 | 70 | +1 | 0.24 | 0.203 / 0.217 | on | on | off |
| 29 | `2026-10-03/_DSC2343.ARW` | offset 0.10-0.20 / roll<10 | 68 | -3 | 0.34 | 0.129 / 0.119 | on | on | off |
| 30 | `2026-10-03/_DSC2982.ARW` | offset 0.06-0.10 / roll>25 | 70 | -44 | 0.06 | 0.093 / 0.085 | on | on | off |
| 31 | `2026-09-27-a/_DSC5587.ARW` | offset 0.06-0.10 / roll10-25 | 121 | +18 | 0.05 | 0.073 / 0.066 | off:side | on | off |
| 32 | `2026-09-19/_DSC3759.ARW` | offset 0.06-0.10 / roll10-25 | 90 | +12 | 0.09 | 0.066 / 0.105 | on | on | off |
| 33 | `2026-09-19/_DSC3334.ARW` | offset <0.06 / roll>25 | 66 | -30 | 0.11 | 0.055 / 0.141 | on | off:occluded | off |
| 34 | `2026-09-27-a/_DSC7506.ARW` | offset 0.10-0.20 / roll<10 | 75 | -4 | 0.15 | 0.131 / 0.166 | on | on | off |
| 35 | `2026-07-11/_DSC2852.ARW` | tilted-wide | 219 | +13 | 0.33 | 0.061 / 0.059 | on | on | ok |
| 36 | `2026-10-03/_DSC1966.ARW` | offset 0.10-0.20 / roll10-25 | 86 | -15 | 0.11 | 0.115 / 0.114 | on | on | off |
| 37 | `2026-09-27-a/_DSC6018.ARW` | offset >0.20 / roll>25 | 160 | -41 | 0.02 | 0.224 / 0.364 | off:notface | off:notface | off |
| 38 | `2026-09-27-a/_DSC6970.ARW` | offset 0.10-0.20 / roll<10 | 61 | +5 | 0.14 | 0.172 / 0.143 | on | on | off |
| 39 | `2026-10-03/_DSC3044.ARW` | offset >0.20 / roll<10 | 102 | -1 | 0.29 | 0.200 / 0.196 | x | x | x |
| 40 | `2026-09-27-a/_DSC7183.ARW` | offset 0.10-0.20 / roll>25 | 126 | -69 | 0.02 | 0.169 / 0.171 | off:side | off:side | off |
| 41 | `2026-09-27-a/_DSC8378.ARW` | offset >0.20 / roll10-25 | 91 | +18 | 0.06 | 0.345 / 0.356 | off:side | off:side | off |
| 42 | `2026-10-03/_DSC3054.ARW` | offset >0.20 / roll10-25 | 90 | +12 | 0.06 | 0.267 / 0.051 | off:rotated | on | ok |
| 43 | `2026-09-27-a/_DSC8046.ARW` | offset 0.10-0.20 / roll10-25 | 63 | -11 | 0.15 | 0.181 / 0.162 | x | x | x |
| 44 | `2026-09-27-a/_DSC4482.ARW` | offset <0.06 / roll<10 | 71 | +4 | 0.09 | 0.057 / 0.045 | on | on | off |
| 45 | `2026-09-19/_DSC2172.ARW` | random | 83 | -4 | 0.29 | 0.107 / 0.117 | x | x | x |
| 46 | `2026-09-19/_DSC2044.ARW` | offset 0.06-0.10 / roll>25 | 78 | +67 | 0.03 | 0.072 / 0.223 | on | off:side | off |
| 47 | `2026-09-27-a/_DSC8725.ARW` | offset >0.20 / roll>25 | 89 | +26 | 0.08 | 0.262 / 0.267 | off:side | off:side | off |
| 48 | `2026-09-27-a/_DSC5388.ARW` | offset 0.06-0.10 / roll10-25 | 76 | -10 | 0.10 | 0.074 / 0.113 | on | on | off |
| 49 | `2026-08-08/_DSC0139.ARW` | tilted-wide | 446 | +12 | 0.27 | 0.068 / 0.055 | on | on | ok |
| 50 | `2026-09-27-a/_DSC4733.ARW` | offset 0.10-0.20 / roll>25 | 69 | +39 | 0.07 | 0.176 / 0.123 | off:side | off:side | off |
| 51 | `2026-09-19/_DSC1972.ARW` | offset >0.20 / roll<10 | 116 | -2 | 0.13 | 0.377 / 0.389 | off:side | off:side | off |
| 52 | `2026-09-27-a/_DSC6857.ARW` | offset 0.06-0.10 / roll10-25 | 61 | +22 | 0.04 | 0.097 / 0.197 | off:side | off:side | off |
| 53 | `2026-09-19/_DSC1960.ARW` | offset 0.06-0.10 / roll<10 | 62 | -9 | 0.22 | 0.064 / 0.061 | on | on | ok |
| 54 | `2026-09-27-a/_DSC7187.ARW` | offset <0.06 / roll>25 | 78 | +44 | 0.04 | 0.059 / 0.069 | on | on | off |
| 55 | `2026-07-11/_DSC2744.ARW` | tilted-wide | 200 | +12 | 0.31 | 0.065 / 0.064 | on | on | ok |
| 56 | `2026-09-27-a/_DSC7275.ARW` | offset <0.06 / roll10-25 | 71 | +11 | 0.11 | 0.044 / 0.043 | on | off:side | off |
| 57 | `2026-09-19/_DSC2062.ARW` | offset 0.06-0.10 / roll<10 | 76 | -6 | 0.21 | 0.097 / 0.086 | on | on | ok |
| 58 | `2026-09-27-a/_DSC7132.ARW` | offset 0.10-0.20 / roll<10 | 79 | +2 | 0.41 | 0.107 / 0.111 | on | on | ok |
| 59 | `2026-07-24/_DSC4875.ARW` | offset <0.06 / roll>25 | 278 | +55 | 0.02 | 0.056 / 0.132 | on | off:side | off |
| 60 | `2026-09-27-a/_DSC4257.ARW` | offset <0.06 / roll10-25 | 72 | -11 | 0.18 | 0.052 / 0.075 | x | x | x |
| 61 | `2026-09-27-a/_DSC4724.ARW` | offset <0.06 / roll<10 | 100 | -2 | 0.32 | 0.042 / 0.043 | on | on | ok |
| 62 | `2026-09-19/_DSC2173.ARW` | offset >0.20 / roll>25 | 87 | +30 | 0.07 | 0.231 / 0.086 | off:side | off:side | off |
| 63 | `2026-09-27-a/_DSC7512.ARW` | offset >0.20 / roll>25 | 78 | +26 | 0.05 | 0.227 / 0.123 | off:side | on | off |
| 64 | `2026-07-24/_DSC4857.ARW` | tilted-wide | 367 | -10 | 0.32 | 0.019 / 0.018 | on | on | ok |
| 65 | `2026-09-19/_DSC3341.ARW` | offset 0.06-0.10 / roll10-25 | 129 | +11 | 0.06 | 0.077 / 0.103 | on | on | off |
| 66 | `2026-09-27-a/_DSC7858.ARW` | offset <0.06 / roll>25 | 84 | -29 | 0.03 | 0.056 / 0.265 | off:side | off:side | off |
| 67 | `2026-08-08/_DSC0156.ARW` | offset 0.06-0.10 / roll<10 | 362 | -8 | 0.33 | 0.078 / 0.082 | on | on | ok |
| 68 | `2026-09-27-a/_DSC5686.ARW` | offset 0.10-0.20 / roll10-25 | 96 | +10 | 0.09 | 0.194 / 0.190 | off:side | off:side | off |
| 69 | `2026-10-03/_DSC3478.ARW` | offset <0.06 / roll<10 | 86 | +0 | 0.26 | 0.046 / 0.046 | on | on | ok |
| 70 | `2026-09-27-a/_DSC8592.ARW` | random | 114 | -13 | 0.04 | 0.272 / 0.312 | off:side | off:side | off |
| 71 | `2026-07-11/_DSC2884.ARW` | offset <0.06 / roll10-25 | 88 | -22 | 0.06 | 0.040 / 0.045 | on | on | off |
| 72 | `2026-08-08/_DSC9979.ARW` | random | 415 | -6 | 0.21 | 0.035 / 0.037 | on | on | ok |
| 73 | `2026-09-19/_DSC2269.ARW` | offset 0.06-0.10 / roll<10 | 82 | +8 | 0.08 | 0.071 / 0.090 | on | on | off |
| 74 | `2026-07-11/_DSC2783.ARW` | tilted-wide | 253 | +11 | 0.34 | 0.037 / 0.037 | on | on | ok |
| 75 | `2026-10-03/_DSC2432.ARW` | random | 72 | +1 | 0.18 | 0.191 / 0.192 | on | on | off |
| 76 | `2026-09-27-a/_DSC7863.ARW` | offset >0.20 / roll>25 | 114 | -76 | 0.04 | 0.313 / 0.421 | off:side | off:side | off |
| 77 | `2026-09-19/_DSC2735.ARW` | offset 0.10-0.20 / roll<10 | 66 | -4 | 0.10 | 0.194 / 0.202 | x | x | x |
| 78 | `2026-10-03/_DSC3019.ARW` | offset 0.06-0.10 / roll<10 | 88 | +2 | 0.35 | 0.064 / 0.063 | on | on | ok |
| 79 | `2026-08-08/_DSC0137.ARW` | offset 0.06-0.10 / roll10-25 | 451 | +15 | 0.25 | 0.072 / 0.073 | on | on | off |
| 80 | `2026-09-19/_DSC3321.ARW` | offset 0.06-0.10 / roll<10 | 74 | -4 | 0.23 | 0.079 / 0.082 | on | on | ok |
| 81 | `2026-09-19/_DSC3432.ARW` | offset 0.10-0.20 / roll>25 | 130 | -69 | 0.02 | 0.128 / 0.223 | on | off:side | off |
| 82 | `2026-10-03/_DSC3586.ARW` | offset 0.10-0.20 / roll10-25 | 69 | -17 | 0.11 | 0.167 / 0.141 | on | on | off |
| 83 | `2026-09-27-a/_DSC6697.ARW` | offset >0.20 / roll>25 | 90 | -46 | 0.02 | 0.345 / 0.232 | off:side | off:side | off |
| 84 | `2026-07-11/_DSC2626.ARW` | tilted-wide | 418 | -14 | 0.34 | 0.071 / 0.064 | on | on | ok |
| 85 | `2026-07-11/_DSC2989.ARW` | offset <0.06 / roll10-25 | 333 | -14 | 0.13 | 0.041 / 0.038 | on | on | ok |
| 86 | `2026-10-03/_DSC2669.ARW` | offset <0.06 / roll<10 | 74 | +1 | 0.38 | 0.025 / 0.021 | on | on | ok |
| 87 | `2026-10-03/_DSC2237.ARW` | offset 0.10-0.20 / roll<10 | 70 | -6 | 0.34 | 0.138 / 0.129 | on | on | off |
| 88 | `2026-10-03/_DSC3321.ARW` | offset >0.20 / roll<10 | 120 | -7 | 0.14 | 0.328 / 0.317 | on | on | off |
| 89 | `2026-09-19/_DSC1910.ARW` | offset 0.10-0.20 / roll10-25 | 65 | +12 | 0.11 | 0.135 / 0.135 | x | x | x |
| 90 | `2026-09-27-a/_DSC6006.ARW` | offset >0.20 / roll>25 | 104 | +74 | 0.04 | 0.325 / 0.310 | off:side | off:side | off |
| 91 | `2026-09-19/_DSC3538.ARW` | offset >0.20 / roll10-25 | 103 | +10 | 0.10 | 0.401 / 0.279 | off:side | off:side | off |
| 92 | `2026-10-03/_DSC1894.ARW` | offset <0.06 / roll<10 | 102 | -0 | 0.35 | 0.039 / 0.039 | on | on | ok |
| 93 | `2026-09-27-a/_DSC6429.ARW` | offset 0.06-0.10 / roll>25 | 111 | +30 | 0.05 | 0.096 / 0.060 | on | on | off |
| 94 | `2026-09-27-a/_DSC6550.ARW` | tilted-wide | 168 | +18 | 0.29 | 0.142 / 0.121 | on | on | off |
| 95 | `2026-09-19/_DSC2122.ARW` | offset <0.06 / roll>25 | 100 | -87 | 0.03 | 0.035 / 0.274 | on | off:side | off |
| 96 | `2026-09-19/_DSC3353.ARW` | offset <0.06 / roll<10 | 85 | -3 | 0.19 | 0.040 / 0.046 | on | on | ok |
| 97 | `2026-07-11/_DSC2892.ARW` | offset 0.06-0.10 / roll>25 | 90 | -28 | 0.06 | 0.068 / 0.062 | on | on | off |
| 98 | `2026-07-11/_DSC3013.ARW` | offset 0.06-0.10 / roll>25 | 314 | -29 | 0.08 | 0.082 / 0.114 | on | on | off |
| 99 | `2026-07-11/_DSC2889.ARW` | offset 0.06-0.10 / roll10-25 | 89 | -17 | 0.08 | 0.065 / 0.031 | on | on | off |
| 100 | `2026-07-11/_DSC2763.ARW` | offset 0.10-0.20 / roll10-25 | 223 | -11 | 0.10 | 0.118 / 0.087 | off:side | on | off |
| 101 | `2026-09-19/_DSC3148.ARW` | offset 0.10-0.20 / roll10-25 | 67 | -13 | 0.10 | 0.125 / 0.143 | on | on | off |
| 102 | `2026-09-19/_DSC2317.ARW` | random | 70 | +12 | 0.03 | 0.365 / 0.337 | off:side | off:side | off |
| 103 | `2026-07-24/_DSC5026.ARW` | offset >0.20 / roll10-25 | 277 | +10 | 0.23 | 0.358 / 0.439 | off:other | off:other | ok |
| 104 | `2026-09-27-a/_DSC4612.ARW` | offset >0.20 / roll<10 | 68 | -1 | 0.10 | 0.282 / 0.292 | on | on | off |
| 105 | `2026-09-19/_DSC2251.ARW` | offset 0.06-0.10 / roll>25 | 302 | -53 | 0.06 | 0.096 / 0.331 | on | off:side | off |
| 106 | `2026-09-27-a/_DSC4603.ARW` | offset 0.06-0.10 / roll<10 | 68 | -1 | 0.32 | 0.062 / 0.059 | on | on | ok |
| 107 | `2026-09-27-a/_DSC5050.ARW` | random | 106 | +2 | 0.07 | 0.047 / 0.049 | on | on | off |
| 108 | `2026-09-27-a/_DSC6288.ARW` | random | 67 | -6 | 0.24 | 0.091 / 0.099 | on | on | off |
| 109 | `2026-09-27-a/_DSC4515.ARW` | offset 0.06-0.10 / roll<10 | 66 | +1 | 0.38 | 0.061 / 0.060 | on | on | ok |
| 110 | `2026-09-19/_DSC2297.ARW` | offset >0.20 / roll10-25 | 70 | +16 | 0.12 | 0.206 / 0.149 | off:side | off:side | off |
| 111 | `2026-08-08/_DSC0138.ARW` | offset <0.06 / roll10-25 | 433 | +10 | 0.28 | 0.056 / 0.064 | on | on | ok |
| 112 | `2026-09-27-a/_DSC7101.ARW` | random | 84 | -63 | 0.03 | 0.101 / 0.107 | on | off:side | off |
| 113 | `2026-09-27-a/_DSC5915.ARW` | random | 96 | -9 | 0.19 | 0.272 / 0.179 | off:notface | off:notface | off |
| 114 | `2026-09-27-a/_DSC7262.ARW` | offset 0.06-0.10 / roll>25 | 94 | -26 | 0.04 | 0.079 / 0.143 | on | off:side | off |
| 115 | `2026-09-27-a/_DSC5585.ARW` | offset 0.06-0.10 / roll10-25 | 284 | -13 | 0.33 | 0.081 / 0.093 | on | on | ok |
| 116 | `2026-09-19/_DSC3620.ARW` | offset <0.06 / roll<10 | 71 | +0 | 0.35 | 0.018 / 0.018 | on | on | ok |
| 117 | `2026-10-03/_DSC1828.ARW` | offset 0.10-0.20 / roll>25 | 88 | +85 | 0.02 | 0.146 / 0.226 | off:side | off:side | off |
| 118 | `2026-09-19/_DSC2299.ARW` | offset >0.20 / roll10-25 | 84 | +19 | 0.07 | 0.224 / 0.153 | off:side | off:side | off |
| 119 | `2026-09-19/_DSC3679.ARW` | offset <0.06 / roll>25 | 113 | +64 | 0.03 | 0.044 / 0.120 | on | off:side | off |
| 120 | `2026-07-11/_DSC2629.ARW` | offset >0.20 / roll10-25 | 420 | +10 | 0.32 | 0.213 / 0.177 | off:rotated | off:rotated | off |
| 121 | `2026-09-27-a/_DSC7772.ARW` | offset 0.06-0.10 / roll>25 | 100 | -73 | 0.03 | 0.096 / 0.268 | on | off:side | off |
| 122 | `2026-07-11/_DSC2703.ARW` | tilted-wide | 197 | -10 | 0.28 | 0.066 / 0.083 | on | on | ok |
| 123 | `2026-07-11/_DSC2594.ARW` | random | 378 | -6 | 0.38 | 0.039 / 0.046 | on | on | ok |
| 124 | `2026-10-03/_DSC2588.ARW` | random | 76 | +2 | 0.42 | 0.070 / 0.068 | on | on | ok |
| 125 | `2026-09-19/_DSC3819.ARW` | offset >0.20 / roll10-25 | 91 | +10 | 0.06 | 0.238 / 0.093 | off:other | on | off |
| 126 | `2026-09-27-a/_DSC4797.ARW` | offset >0.20 / roll<10 | 85 | -4 | 0.17 | 0.212 / 0.230 | on | on | off |
| 127 | `2026-10-03/_DSC3442.ARW` | random | 92 | +8 | 0.14 | 0.047 / 0.047 | on | on | ok |
| 128 | `2026-09-27-a/_DSC4552.ARW` | offset 0.06-0.10 / roll>25 | 95 | +35 | 0.06 | 0.068 / 0.096 | on | on | off |
| 129 | `2026-09-19/_DSC3254.ARW` | offset 0.06-0.10 / roll10-25 | 77 | -16 | 0.12 | 0.086 / 0.091 | on | on | ok |
| 130 | `2026-09-27-a/_DSC5390.ARW` | offset 0.06-0.10 / roll<10 | 71 | -4 | 0.09 | 0.086 / 0.089 | on | on | off |
| 131 | `2026-09-27-a/_DSC8292.ARW` | offset >0.20 / roll<10 | 94 | +1 | 0.18 | 0.208 / 0.209 | on | on | off |
| 132 | `2026-07-11/_DSC2657.ARW` | random | 399 | -8 | 0.23 | 0.335 / 0.339 | off:notface | off:notface | off |
| 133 | `2026-08-08/_DSC0276.ARW` | tilted-wide | 292 | -13 | 0.35 | 0.058 / 0.053 | on | on | ok |
| 134 | `2026-09-19/_DSC3397.ARW` | offset >0.20 / roll10-25 | 92 | -18 | 0.05 | 0.215 / 0.188 | on | on | off |
| 135 | `2026-08-08/_DSC0145.ARW` | tilted-wide | 410 | -11 | 0.26 | 0.076 / 0.088 | on | on | off |
| 136 | `2026-10-03/_DSC1878.ARW` | offset 0.10-0.20 / roll<10 | 137 | +1 | 0.31 | 0.182 / 0.186 | on | on | off |
| 137 | `2026-10-03/_DSC2981.ARW` | random | 68 | -4 | 0.14 | 0.162 / 0.167 | on | on | off |
| 138 | `2026-07-24/_DSC4884.ARW` | offset <0.06 / roll10-25 | 409 | -11 | 0.30 | 0.046 / 0.061 | on | on | ok |
| 139 | `2026-07-11/_DSC2972.ARW` | offset >0.20 / roll>25 | 242 | +31 | 0.01 | 0.331 / 0.295 | off:side | off:side | off |
