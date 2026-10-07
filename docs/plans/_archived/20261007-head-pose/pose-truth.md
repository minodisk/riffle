# Head-pose truth set (Step 2)

Hand labels of the head pose of the face each file is judged on, read from
the face crop the face mesh runs on (`eyes::face_square` cut from the
full-size upright preview by `faces::crop_rgb`, resized to 200x200, no mesh
drawn). The agent labeled every face; **the user's review of the labels is
pending** (see `learnings.md`).

Raw outputs are in `D:\Photos\tests\2026-10-07-head-pose\`, out of the
repository: the crops (`crops\<stem>.png`), the sheets the labels were read
from (`sheets\s00.png`-`s16.png`, 4 tiles across, read row by row in the
`#` order below), the per-face angles of every variant (`dump-arw.tsv`,
`dump-dng.tsv`), the draw (`sample.py`, `sample2.py`, `sample.tsv`), the
labels (`labels.tsv`), the aggregation (`load.py`, `aggregate.py`,
`results.txt`) and the throwaway `riffle-cli posedump` / `sheet`
subcommands they came from (`posedump-cli.patch`, not merged).

## Which faces

The judged face of each file as `riffle-cli eyes` picks it (the face nearest
the AF point on the scan's detection, else the largest face at or above
`sharpness::FACE_CONFIDENCE`), so the same face as `eyes-truth.md` of the
closed-eyes plan, on all of it, not only on the faces at or above
`EYES_MIN_FACE`:

- `D:\photos\2026\2026-09-19`: 1952 of the 2134 α7 V ARWs have a judged
  face. `D:\photos\2026\2026-02-01`: 72 of the 146 M11-P DNGs.
- 190 faces drawn, then shuffled together so the source and the stratum were
  hidden while labeling:
  - 30 ARW at random and 30 DNG at random (seed 20261007): the unbiased part.
  - 30 ARW from each Step 1 |yaw| bucket at the 63 deg default (`frontal`
    under 15, `oblique` 15-50, `profile` over 50), so oblique and profile
    faces are not swamped by frontal ones.
  - 40 ARW and DNG added after the first 150 were labeled (seed 202610072,
    labeled the same blind way): 15 with roll over +15, 15 under -15 and 10
    within, because the first 150 had only 3 faces with a visible roll.

## Labels

- **Yaw class**: `frontal` (|yaw| under ~15 deg), `oblique` (~15-50),
  `profile` (over ~50, one eye hidden or nearly), with the **direction**:
  the image side the nose points to (`left` / `right`; empty for frontal).
- **Pitch**: `up` / `level` / `down` (chin raised or lowered by more than
  ~15 deg).
- **Roll**: `left` / `level` / `right`, the side the top of the head leans
  to on screen; `right` is clockwise, read off the line through the eyes
  (the image-right eye lower). Only a clear lean (~15 deg or more) is
  labeled; a slight one is `level`.
- `x` on every axis when the face cannot be read: the back of a head, an
  ear, a face too blurred to see, or an occluded one (a hand over the eyes).

159 of the 190 are labeled: 78 frontal, 44 oblique, 37 profile; pitch 63 up,
82 level, 14 down; roll 10 left, 141 level, 8 right. The 31 `x` faces are
16 of the 24 faces under 60 px, 9 of 67 at 60-79 px, 5 of 46 at 80-99 px
and 1 of 53 at 100 px and over.

## How YuNet limits the profile class

YuNet does return side views on these folders (children turned toward the
ball in a gym): 414 of the 1952 judged ARW faces read over 50 deg of yaw.
But what it returns past about 70 deg is mostly not a face the mesh can
fit: of the 30 faces drawn from that bucket, 5 were backs of heads, ears or
blurs (`x`), 8 of the 25 readable ones were `oblique` by eye, and the
labeled profiles read a median |yaw| of 65 deg (upper quartile 73). So the
`profile` class here means "about 50-75 deg"; a true 90 deg profile is
either not detected, not readable, or fitted wrong (`pose-results.md`), and
the measured range is honest only up to there.

## Faces

`#` is the position in the sheets, `side` the face box's longer side in
stored preview pixels.

| # | File | Stratum | Side | Yaw | Direction | Pitch | Roll |
|---|------|---------|------|-----|-----------|-------|------|
| 0 | `_DSC2036.ARW` | arw-random | 58 | profile | left | level | level |
| 1 | `_DSC2316.ARW` | arw-random | 67 | x | x | x | x |
| 2 | `L1005175.DNG` | dng-random | 99 | frontal |  | level | level |
| 3 | `_DSC2235.ARW` | arw-profile | 100 | profile | left | up | level |
| 4 | `_DSC1975.ARW` | arw-random | 58 | profile | left | level | level |
| 5 | `_DSC2040.ARW` | arw-frontal | 66 | oblique | left | up | left |
| 6 | `_DSC2910.ARW` | arw-profile | 73 | profile | left | up | level |
| 7 | `_DSC2045.ARW` | arw-frontal | 84 | oblique | left | level | level |
| 8 | `L1005161.DNG` | dng-random | 90 | frontal |  | down | level |
| 9 | `_DSC2888.ARW` | arw-profile | 83 | profile | left | up | level |
| 10 | `L1005567.DNG` | dng-random | 349 | frontal |  | level | level |
| 11 | `_DSC3346.ARW` | arw-profile | 89 | oblique | right | level | level |
| 12 | `L1005709.DNG` | dng-random | 125 | frontal |  | level | left |
| 13 | `L1005295.DNG` | dng-random | 128 | frontal |  | level | level |
| 14 | `L1005669.DNG` | dng-random | 155 | frontal |  | level | level |
| 15 | `_DSC2973.ARW` | arw-profile | 90 | profile | left | up | level |
| 16 | `_DSC3404.ARW` | arw-oblique | 74 | oblique | left | level | level |
| 17 | `L1005146.DNG` | dng-random | 110 | frontal |  | level | level |
| 18 | `_DSC2448.ARW` | arw-random | 70 | oblique | left | down | level |
| 19 | `_DSC2363.ARW` | arw-profile | 84 | profile | left | level | level |
| 20 | `_DSC1798.ARW` | arw-frontal | 51 | frontal |  | level | level |
| 21 | `_DSC3846.ARW` | arw-random | 99 | profile | left | level | level |
| 22 | `_DSC2576.ARW` | arw-frontal | 85 | frontal |  | up | level |
| 23 | `_DSC2475.ARW` | arw-profile | 74 | profile | left | down | level |
| 24 | `L1005722.DNG` | dng-random | 227 | frontal |  | level | level |
| 25 | `_DSC1846.ARW` | arw-oblique | 79 | oblique | right | level | level |
| 26 | `L1005227.DNG` | dng-random | 160 | frontal |  | level | level |
| 27 | `_DSC2415.ARW` | arw-profile | 99 | profile | left | level | level |
| 28 | `L1005643.DNG` | dng-random | 235 | frontal |  | level | left |
| 29 | `_DSC1803.ARW` | arw-profile | 70 | x | x | x | x |
| 30 | `_DSC2319.ARW` | arw-oblique | 73 | frontal |  | down | level |
| 31 | `_DSC2750.ARW` | arw-oblique | 76 | oblique | right | up | level |
| 32 | `L1005576.DNG` | dng-random | 372 | oblique | right | level | level |
| 33 | `_DSC2863.ARW` | arw-oblique | 51 | x | x | x | x |
| 34 | `_DSC3126.ARW` | arw-oblique | 49 | frontal |  | up | level |
| 35 | `_DSC3078.ARW` | arw-oblique | 71 | oblique | right | level | level |
| 36 | `_DSC3403.ARW` | arw-oblique | 74 | oblique | left | level | level |
| 37 | `_DSC2237.ARW` | arw-random | 120 | profile | left | level | level |
| 38 | `_DSC2796.ARW` | arw-oblique | 83 | oblique | left | up | level |
| 39 | `_DSC2032.ARW` | arw-random | 35 | x | x | x | x |
| 40 | `_DSC3254.ARW` | arw-profile | 77 | oblique | right | up | level |
| 41 | `_DSC3770.ARW` | arw-random | 74 | frontal |  | level | level |
| 42 | `_DSC2058.ARW` | arw-profile | 82 | oblique | right | up | level |
| 43 | `_DSC3074.ARW` | arw-oblique | 71 | oblique | right | level | level |
| 44 | `_DSC3145.ARW` | arw-random | 56 | frontal |  | up | level |
| 45 | `_DSC2293.ARW` | arw-profile | 82 | profile | left | level | level |
| 46 | `_DSC2143.ARW` | arw-profile | 78 | profile | left | level | level |
| 47 | `_DSC2041.ARW` | arw-frontal | 75 | frontal |  | level | level |
| 48 | `_DSC1982.ARW` | arw-frontal | 68 | frontal |  | up | level |
| 49 | `_DSC3630.ARW` | arw-random | 104 | frontal |  | down | level |
| 50 | `L1005648.DNG` | dng-random | 132 | frontal |  | level | level |
| 51 | `_DSC2557.ARW` | arw-frontal | 62 | frontal |  | up | level |
| 52 | `_DSC2649.ARW` | arw-frontal | 77 | frontal |  | up | level |
| 53 | `_DSC2460.ARW` | arw-oblique | 46 | x | x | x | x |
| 54 | `_DSC2836.ARW` | arw-frontal | 74 | frontal |  | up | level |
| 55 | `_DSC3212.ARW` | arw-oblique | 78 | oblique | right | up | level |
| 56 | `L1005739.DNG` | dng-random | 116 | frontal |  | level | level |
| 57 | `_DSC2520.ARW` | arw-frontal | 69 | frontal |  | up | level |
| 58 | `_DSC3811.ARW` | arw-frontal | 76 | frontal |  | level | level |
| 59 | `L1005572.DNG` | dng-random | 371 | oblique | right | down | level |
| 60 | `L1005362.DNG` | dng-random | 203 | frontal |  | level | level |
| 61 | `_DSC3004.ARW` | arw-oblique | 82 | oblique | left | up | level |
| 62 | `_DSC3430.ARW` | arw-random | 121 | profile | right | up | level |
| 63 | `_DSC2425.ARW` | arw-random | 80 | x | x | x | x |
| 64 | `_DSC1822.ARW` | arw-profile | 81 | x | x | x | x |
| 65 | `_DSC3222.ARW` | arw-profile | 73 | oblique | right | up | level |
| 66 | `_DSC3250.ARW` | arw-oblique | 77 | oblique | right | up | level |
| 67 | `L1005288.DNG` | dng-random | 279 | frontal |  | level | level |
| 68 | `_DSC3597.ARW` | arw-frontal | 74 | frontal |  | up | level |
| 69 | `_DSC3451.ARW` | arw-random | 79 | frontal |  | level | level |
| 70 | `_DSC2789.ARW` | arw-frontal | 73 | frontal |  | up | level |
| 71 | `_DSC1808.ARW` | arw-random | 106 | oblique | right | up | level |
| 72 | `L1005654.DNG` | dng-random | 109 | frontal |  | level | level |
| 73 | `_DSC1885.ARW` | arw-frontal | 85 | frontal |  | up | level |
| 74 | `_DSC3636.ARW` | arw-random | 70 | x | x | x | x |
| 75 | `L1005179.DNG` | dng-random | 85 | frontal |  | level | level |
| 76 | `_DSC2882.ARW` | arw-oblique | 85 | oblique | left | level | level |
| 77 | `_DSC3613.ARW` | arw-random | 71 | frontal |  | level | level |
| 78 | `_DSC2200.ARW` | arw-profile | 220 | oblique | left | down | level |
| 79 | `L1005640.DNG` | dng-random | 225 | frontal |  | level | level |
| 80 | `_DSC2121.ARW` | arw-profile | 101 | profile | right | level | level |
| 81 | `_DSC3192.ARW` | arw-profile | 65 | profile | right | up | level |
| 82 | `_DSC2323.ARW` | arw-frontal | 70 | profile | left | level | level |
| 83 | `_DSC2181.ARW` | arw-profile | 77 | x | x | x | x |
| 84 | `_DSC3626.ARW` | arw-random | 95 | frontal |  | level | level |
| 85 | `_DSC3670.ARW` | arw-frontal | 86 | frontal |  | down | level |
| 86 | `L1005657.DNG` | dng-random | 126 | frontal |  | level | level |
| 87 | `_DSC2826.ARW` | arw-oblique | 38 | x | x | x | x |
| 88 | `L1005762.DNG` | dng-random | 347 | frontal |  | down | level |
| 89 | `_DSC2090.ARW` | arw-random | 65 | oblique | right | up | level |
| 90 | `_DSC2852.ARW` | arw-oblique | 103 | oblique | left | up | level |
| 91 | `_DSC3053.ARW` | arw-random | 86 | oblique | right | level | level |
| 92 | `L1005559.DNG` | dng-random | 399 | frontal |  | level | level |
| 93 | `_DSC3172.ARW` | arw-random | 65 | oblique | right | up | level |
| 94 | `_DSC2548.ARW` | arw-frontal | 74 | frontal |  | up | level |
| 95 | `_DSC1962.ARW` | arw-oblique | 23 | x | x | x | x |
| 96 | `_DSC3114.ARW` | arw-oblique | 59 | oblique | left | up | level |
| 97 | `_DSC2033.ARW` | arw-random | 32 | x | x | x | x |
| 98 | `_DSC3193.ARW` | arw-random | 66 | profile | right | up | level |
| 99 | `_DSC2285.ARW` | arw-profile | 129 | profile | left | down | level |
| 100 | `_DSC1909.ARW` | arw-oblique | 75 | profile | left | up | level |
| 101 | `_DSC2080.ARW` | arw-profile | 68 | profile | left | level | level |
| 102 | `_DSC3103.ARW` | arw-profile | 59 | x | x | x | x |
| 103 | `L1005300.DNG` | dng-random | 151 | frontal |  | level | level |
| 104 | `_DSC2027.ARW` | arw-profile | 98 | profile | left | up | level |
| 105 | `_DSC3344.ARW` | arw-random | 131 | oblique | left | level | level |
| 106 | `_DSC2053.ARW` | arw-frontal | 36 | x | x | x | x |
| 107 | `_DSC2979.ARW` | arw-profile | 96 | profile | left | up | level |
| 108 | `L1005240.DNG` | dng-random | 230 | frontal |  | level | level |
| 109 | `_DSC3881.ARW` | arw-oblique | 112 | profile | right | up | level |
| 110 | `_DSC2733.ARW` | arw-oblique | 86 | x | x | x | x |
| 111 | `_DSC2054.ARW` | arw-oblique | 105 | frontal |  | up | level |
| 112 | `_DSC2530.ARW` | arw-frontal | 79 | frontal |  | level | level |
| 113 | `_DSC3533.ARW` | arw-profile | 127 | profile | right | level | level |
| 114 | `_DSC2906.ARW` | arw-random | 85 | profile | left | up | level |
| 115 | `_DSC2167.ARW` | arw-frontal | 22 | x | x | x | x |
| 116 | `_DSC1965.ARW` | arw-frontal | 74 | frontal |  | up | level |
| 117 | `L1005155.DNG` | dng-random | 105 | frontal |  | level | level |
| 118 | `_DSC3527.ARW` | arw-oblique | 138 | frontal |  | down | level |
| 119 | `L1005205.DNG` | dng-random | 174 | frontal |  | level | level |
| 120 | `_DSC2241.ARW` | arw-frontal | 72 | profile | right | level | level |
| 121 | `_DSC2673.ARW` | arw-frontal | 75 | frontal |  | up | level |
| 122 | `_DSC1924.ARW` | arw-frontal | 94 | oblique | right | level | level |
| 123 | `_DSC2375.ARW` | arw-profile | 98 | oblique | left | level | level |
| 124 | `_DSC2950.ARW` | arw-oblique | 87 | oblique | left | up | level |
| 125 | `_DSC2725.ARW` | arw-frontal | 80 | frontal |  | up | level |
| 126 | `_DSC2755.ARW` | arw-oblique | 99 | x | x | x | x |
| 127 | `_DSC2747.ARW` | arw-frontal | 63 | x | x | x | x |
| 128 | `_DSC3563.ARW` | arw-frontal | 72 | frontal |  | up | level |
| 129 | `_DSC2580.ARW` | arw-random | 95 | frontal |  | up | level |
| 130 | `_DSC3035.ARW` | arw-random | 104 | oblique | right | level | level |
| 131 | `L1005164.DNG` | dng-random | 100 | frontal |  | level | level |
| 132 | `_DSC3608.ARW` | arw-random | 79 | profile | left | up | level |
| 133 | `_DSC3137.ARW` | arw-oblique | 62 | frontal |  | level | level |
| 134 | `_DSC3909.ARW` | arw-oblique | 79 | oblique | right | down | level |
| 135 | `_DSC3334.ARW` | arw-profile | 66 | profile | right | level | level |
| 136 | `_DSC3244.ARW` | arw-oblique | 78 | oblique | right | up | level |
| 137 | `_DSC2616.ARW` | arw-frontal | 46 | frontal |  | up | level |
| 138 | `L1005568.DNG` | dng-random | 266 | oblique | right | level | level |
| 139 | `L1005188.DNG` | dng-random | 227 | oblique | right | down | level |
| 140 | `L1005563.DNG` | dng-random | 262 | frontal |  | level | level |
| 141 | `_DSC2253.ARW` | arw-profile | 106 | oblique | left | level | level |
| 142 | `_DSC3607.ARW` | arw-profile | 74 | oblique | left | up | level |
| 143 | `_DSC2037.ARW` | arw-oblique | 64 | profile | left | level | level |
| 144 | `_DSC2845.ARW` | arw-random | 77 | frontal |  | up | level |
| 145 | `_DSC2332.ARW` | arw-profile | 41 | x | x | x | x |
| 146 | `_DSC3612.ARW` | arw-random | 97 | frontal |  | level | level |
| 147 | `_DSC3721.ARW` | arw-random | 199 | oblique | left | level | level |
| 148 | `_DSC2274.ARW` | arw-frontal | 66 | x | x | x | x |
| 149 | `_DSC2067.ARW` | arw-frontal | 77 | frontal |  | up | level |
| 150 | `_DSC2739.ARW` | roll-neg | 55 | x | x | x | x |
| 151 | `_DSC3524.ARW` | roll-pos | 101 | frontal |  | level | right |
| 152 | `_DSC2194.ARW` | roll-neg | 89 | frontal |  | level | left |
| 153 | `_DSC2439.ARW` | roll-level | 32 | x | x | x | x |
| 154 | `_DSC3170.ARW` | roll-neg | 60 | x | x | x | x |
| 155 | `_DSC2138.ARW` | roll-pos | 83 | oblique | right | level | right |
| 156 | `_DSC3562.ARW` | roll-pos | 69 | frontal |  | up | left |
| 157 | `_DSC1887.ARW` | roll-neg | 80 | frontal |  | up | left |
| 158 | `_DSC3908.ARW` | roll-neg | 81 | x | x | x | x |
| 159 | `_DSC3446.ARW` | roll-neg | 47 | x | x | x | x |
| 160 | `_DSC3890.ARW` | roll-pos | 156 | frontal |  | up | right |
| 161 | `_DSC3299.ARW` | roll-neg | 72 | oblique | right | level | level |
| 162 | `_DSC3790.ARW` | roll-neg | 133 | profile | left | level | level |
| 163 | `_DSC2331.ARW` | roll-neg | 121 | profile | left | level | level |
| 164 | `_DSC1988.ARW` | roll-pos | 70 | frontal |  | up | right |
| 165 | `_DSC2856.ARW` | roll-level | 49 | x | x | x | x |
| 166 | `_DSC2622.ARW` | roll-level | 80 | frontal |  | level | level |
| 167 | `_DSC3650.ARW` | roll-pos | 61 | x | x | x | x |
| 168 | `_DSC3311.ARW` | roll-pos | 81 | profile | right | level | level |
| 169 | `_DSC2830.ARW` | roll-pos | 76 | frontal |  | up | right |
| 170 | `_DSC3541.ARW` | roll-pos | 105 | x | x | x | x |
| 171 | `L1005603.DNG` | roll-neg | 135 | frontal |  | level | left |
| 172 | `_DSC2467.ARW` | roll-level | 42 | x | x | x | x |
| 173 | `_DSC2354.ARW` | roll-neg | 86 | profile | right | down | level |
| 174 | `_DSC2125.ARW` | roll-level | 73 | profile | left | level | level |
| 175 | `_DSC3141.ARW` | roll-level | 55 | frontal |  | level | level |
| 176 | `_DSC2047.ARW` | roll-pos | 81 | frontal |  | up | right |
| 177 | `_DSC2579.ARW` | roll-pos | 66 | x | x | x | x |
| 178 | `_DSC2594.ARW` | roll-neg | 40 | x | x | x | x |
| 179 | `_DSC2556.ARW` | roll-level | 71 | frontal |  | up | level |
| 180 | `_DSC3611.ARW` | roll-neg | 100 | frontal |  | level | level |
| 181 | `_DSC2048.ARW` | roll-pos | 81 | frontal |  | up | right |
| 182 | `_DSC2693.ARW` | roll-level | 74 | frontal |  | up | level |
| 183 | `L1005708.DNG` | roll-pos | 121 | frontal |  | level | left |
| 184 | `_DSC2350.ARW` | roll-level | 93 | profile | right | level | level |
| 185 | `_DSC1860.ARW` | roll-pos | 110 | frontal |  | up | right |
| 186 | `_DSC2405.ARW` | roll-pos | 89 | oblique | left | level | level |
| 187 | `_DSC3381.ARW` | roll-neg | 88 | oblique | right | up | left |
| 188 | `_DSC3216.ARW` | roll-level | 73 | profile | right | up | level |
| 189 | `_DSC3378.ARW` | roll-neg | 94 | oblique | right | level | left |
