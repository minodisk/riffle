# Eye-state truth set (Step 1)

Hand labels of the eye state of the face each file is judged on, read from
the eye-band tiles of the survey (the band through both eye points of the
face, cut from the full-size upright preview and upscaled to 288x128) and,
where a tile was unclear, from the `riffle-cli eyecrops` crops. The agent
labeled every face; **the user's review of the `closed` faces below is
pending** (see `learnings.md`).

Raw outputs (crops, tiles, sheets, the per-face model outputs) are in
`D:\Photos\tests\2026-10-07-closed-eyes\`; the sheets the labels were read
from are `sheets\s00.png`-`s17.png` (ARW), `d0.png`-`d2.png` (DNG) and
`e0.png` (DNG at the 3/8 decode), each 4 tiles across, read row by row.

## Which faces

- **ARW**: `D:\photos\2026\2026-09-19` (2134 α7 V ARWs, AF point on every
  file). The face judged is the one nearest the AF point
  (`candidate::nearest_face` on the `faces::detect_around` crop path),
  1952 files. Labeled: 432 of them, shuffled together so the source was not
  visible while labeling:
  - 240 drawn at random (seed 20261007): the unbiased part, used for the
    base rate and the labelable fraction.
  - 192 enriched toward closed eyes (blinks are rare): the 72 highest
    closed probabilities of the eye-crop classifier, then the 48 lowest
    face-mesh v1 EARs, the 24 lowest face-mesh v2 EARs and the 48 lowest
    iris-model openness values not already drawn (see `model-survey.md`).
    No folder with known blinks was named, and the focus cue's `Soft`
    frames were not used: soft focus says nothing about the eyelids.
- **DNG**: `D:\photos\2026\2026-02-01` (146 M11-P DNGs, no AF point). The
  face judged is the largest face scoring at least 0.8 on the whole-image
  search (the faces the sharpness score uses), 72 files; all labeled, at full
  size, and the first 24 also on the 3/8 decode the whole-image search uses.

## Labels

Per eye, image-left eye first:

- `o` open: the iris or pupil is visible.
- `c` closed: no iris or pupil visible, the lids meet or nearly. This
  includes eyes cast down so far that the lid covers the iris (most of the
  ARW `c` faces are children looking down, not mid-blink; the DNG ones are
  mostly smiles and squints): a single frame cannot tell them apart, and on
  the strip both read as closed eyes.
- `u` unsure: the eye is there but too small, blurred, dark or half-lidded to
  call.
- `x` no eye to judge: occluded (hand, ball, glasses glare), turned away,
  cut by the frame, or not a face (false detections on hair, ears, blur).

For the scores, an eye counts when it is `o` or `c`; a face is closed when
it has a `c` eye and no `o` eye, open when it has an `o` eye and no `c` eye.

## Labelable fraction by face side (the preview-resolution floor)

Faces with at least one eye labeled `o` or `c`, by the face box side (the
longer of width and height) in stored preview px:

| Face side | ARW random sample | ARW all labeled | DNG |
|-----------|-------------------|-----------------|-----|
| < 40 px | 0 / 5 | 0 / 26 | - |
| 40-59 px | 6 / 31 | 7 / 63 | - |
| 60-79 px | 72 / 105 | 101 / 165 | 0 / 3 |
| 80-99 px | 39 / 58 | 60 / 108 | 4 / 7 |
| 100-149 px | 23 / 35 | 32 / 55 | 13 / 30 |
| >= 150 px | 4 / 6 | 9 / 15 | 27 / 32 |
| All | 144 / 240 (60%) | 209 / 432 | 44 / 72 |

Of the 240 random ARW faces, 59 have no eye to judge (`x x`: mostly false
detections, heads turned away and occlusions, which the face side does not
predict) and 37 only `u` eyes. Below 60 px a human can call the eye state of
about one face in five (6 / 31) on the 1616 px preview; from 60 px up about
two in three. The floor for a model is therefore a face side of about 60 px.
On the DNG folder the 3/8 decode lost a fifth of the labelable faces of the
first 24 (16 against 20 faces with an `o` or `c` eye, 31 against 38 eyes).

Closed faces among the labelable: 15 of 144 in the random ARW sample (10%,
nearly all looking down), 49 of 209 in all labeled ARW faces, 8 of 44 DNG
faces.

## Faces with a closed eye (for the user's review)

File, face side, left / right eye:

```text
L1005161.DNG  90px  c c
L1005164.DNG  100px  c c
L1005188.DNG  227px  c u
L1005548.DNG  124px  c c
L1005579.DNG  188px  c c
L1005693.DNG  173px  c u
L1005722.DNG  227px  c c
L1005734.DNG  146px  c u
_DSC1889.ARW  88px  c c
_DSC1890.ARW  87px  c c
_DSC1891.ARW  112px  c c
_DSC1892.ARW  117px  c c
_DSC1893.ARW  120px  c c
_DSC1926.ARW  94px  c c
_DSC1927.ARW  95px  c c
_DSC1930.ARW  77px  c c
_DSC1932.ARW  76px  c c
_DSC1933.ARW  99px  c c
_DSC2006.ARW  68px  c c
_DSC2008.ARW  76px  c c
_DSC2013.ARW  90px  c c
_DSC2083.ARW  84px  c c
_DSC2109.ARW  64px  c c
_DSC2110.ARW  67px  c c
_DSC2111.ARW  72px  c c
_DSC2182.ARW  111px  c c
_DSC2183.ARW  113px  c c
_DSC2184.ARW  121px  c c
_DSC2199.ARW  223px  c c
_DSC2207.ARW  77px  c c
_DSC2208.ARW  80px  c c
_DSC2248.ARW  200px  c c
_DSC2250.ARW  261px  c c
_DSC2329.ARW  114px  x c
_DSC2368.ARW  71px  c c
_DSC2369.ARW  66px  c c
_DSC2370.ARW  75px  c c
_DSC2371.ARW  88px  c c
_DSC2372.ARW  88px  c c
_DSC2398.ARW  84px  c c
_DSC2408.ARW  85px  c c
_DSC3200.ARW  71px  c c
_DSC3202.ARW  72px  c c
_DSC3250.ARW  77px  c u
_DSC3528.ARW  135px  c c
_DSC3529.ARW  135px  c c
_DSC3530.ARW  133px  c c
_DSC3622.ARW  90px  c c
_DSC3630.ARW  104px  c c
_DSC3670.ARW  86px  c c
_DSC3683.ARW  62px  c c
_DSC3688.ARW  68px  c c
_DSC3729.ARW  70px  c c
_DSC3770.ARW  74px  c c
_DSC3795.ARW  105px  c c
_DSC3885.ARW  194px  c c
_DSC3909.ARW  79px  c c
```

## All labeled faces

File, face side, detection score, left / right eye, sample source (`random`,
or the model whose ranking drew it; `dng` for the DNG folder) and, for the
first 24 DNGs, the labels read on the 3/8 decode:

```text
L1005146.DNG  110px  0.90  o o  dng  3/8: o o
L1005148.DNG  99px  0.91  o o  dng  3/8: o o
L1005155.DNG  105px  0.87  o o  dng  3/8: o o
L1005161.DNG  90px  0.89  c c  dng  3/8: c c
L1005164.DNG  100px  0.84  c c  dng  3/8: c c
L1005172.DNG  101px  0.90  o o  dng  3/8: u u
L1005173.DNG  95px  0.85  u u  dng  3/8: u u
L1005175.DNG  99px  0.91  o o  dng  3/8: o o
L1005178.DNG  90px  0.81  o o  dng  3/8: u u
L1005179.DNG  85px  0.86  u u  dng  3/8: u u
L1005188.DNG  227px  0.87  c u  dng  3/8: c u
L1005191.DNG  105px  0.82  o o  dng  3/8: o o
L1005194.DNG  177px  0.88  u u  dng  3/8: u u
L1005199.DNG  164px  0.89  o o  dng  3/8: o o
L1005200.DNG  163px  0.90  o o  dng  3/8: o o
L1005205.DNG  174px  0.92  o o  dng  3/8: o o
L1005215.DNG  107px  0.86  u o  dng  3/8: u u
L1005227.DNG  160px  0.90  o o  dng  3/8: o o
L1005240.DNG  230px  0.92  o o  dng  3/8: o o
L1005246.DNG  166px  0.92  o o  dng  3/8: o o
L1005255.DNG  230px  0.89  o o  dng  3/8: o o
L1005259.DNG  128px  0.88  o o  dng  3/8: u u
L1005268.DNG  112px  0.84  x x  dng  3/8: x x
L1005288.DNG  279px  0.92  o o  dng  3/8: o o
L1005294.DNG  127px  0.89  u u  dng
L1005295.DNG  128px  0.89  u u  dng
L1005300.DNG  151px  0.89  o o  dng
L1005329.DNG  76px  0.83  u u  dng
L1005362.DNG  203px  0.90  o o  dng
L1005373.DNG  623px  0.91  o o  dng
L1005377.DNG  246px  0.87  o o  dng
L1005408.DNG  76px  0.85  u u  dng
L1005493.DNG  144px  0.87  u u  dng
L1005548.DNG  124px  0.81  c c  dng
L1005553.DNG  403px  0.91  o o  dng
L1005559.DNG  399px  0.92  u u  dng
L1005563.DNG  262px  0.91  o o  dng
L1005567.DNG  349px  0.86  o o  dng
L1005568.DNG  266px  0.87  o o  dng
L1005572.DNG  371px  0.85  x x  dng
L1005575.DNG  337px  0.88  o o  dng
L1005576.DNG  372px  0.89  o o  dng
L1005579.DNG  188px  0.86  c c  dng
L1005591.DNG  101px  0.84  x x  dng
L1005603.DNG  135px  0.86  o o  dng
L1005618.DNG  112px  0.82  u u  dng
L1005640.DNG  225px  0.93  o o  dng
L1005643.DNG  235px  0.89  o o  dng
L1005648.DNG  132px  0.83  u u  dng
L1005651.DNG  116px  0.90  u u  dng
L1005654.DNG  109px  0.84  u u  dng
L1005656.DNG  119px  0.82  o o  dng
L1005657.DNG  126px  0.88  o o  dng
L1005669.DNG  155px  0.90  u u  dng
L1005672.DNG  157px  0.89  o o  dng
L1005693.DNG  173px  0.86  c u  dng
L1005705.DNG  130px  0.84  u u  dng
L1005708.DNG  121px  0.84  u u  dng
L1005709.DNG  125px  0.82  u u  dng
L1005714.DNG  120px  0.83  u u  dng
L1005719.DNG  124px  0.83  u u  dng
L1005721.DNG  113px  0.88  u u  dng
L1005722.DNG  227px  0.89  c c  dng
L1005734.DNG  146px  0.86  c u  dng
L1005739.DNG  116px  0.87  u o  dng
L1005740.DNG  117px  0.80  u u  dng
L1005747.DNG  82px  0.87  x x  dng
L1005749.DNG  79px  0.84  x x  dng
L1005755.DNG  115px  0.83  x x  dng
L1005758.DNG  172px  0.87  o o  dng
L1005761.DNG  320px  0.84  x x  dng
L1005762.DNG  347px  0.81  o o  dng
_DSC1799.ARW  50px  0.83  u u  random
_DSC1802.ARW  86px  0.77  x o  random
_DSC1808.ARW  106px  0.88  o o  random
_DSC1826.ARW  115px  0.89  o o  random
_DSC1827.ARW  118px  0.91  o o  random
_DSC1834.ARW  111px  0.91  o o  random
_DSC1856.ARW  111px  0.64  x x  random
_DSC1858.ARW  124px  0.74  x o  random
_DSC1867.ARW  71px  0.90  o o  random
_DSC1868.ARW  71px  0.90  o o  random
_DSC1869.ARW  72px  0.90  o o  occ
_DSC1878.ARW  34px  0.81  x x  random
_DSC1881.ARW  95px  0.90  o o  random
_DSC1884.ARW  83px  0.85  u u  random
_DSC1886.ARW  74px  0.76  u u  fm1
_DSC1889.ARW  88px  0.92  c c  fm2
_DSC1890.ARW  87px  0.92  c c  fm2
_DSC1891.ARW  112px  0.92  c c  occ
_DSC1892.ARW  117px  0.92  c c  occ
_DSC1893.ARW  120px  0.92  c c  occ
_DSC1899.ARW  104px  0.60  x x  occ
_DSC1901.ARW  91px  0.65  x x  occ
_DSC1902.ARW  87px  0.69  x x  random
_DSC1904.ARW  89px  0.89  x u  random
_DSC1905.ARW  90px  0.89  x o  random
_DSC1906.ARW  91px  0.87  x u  random
_DSC1910.ARW  65px  0.82  x x  random
_DSC1915.ARW  108px  0.79  x x  random
_DSC1922.ARW  80px  0.83  x x  occ
_DSC1923.ARW  90px  0.62  x x  occ
_DSC1926.ARW  94px  0.87  c c  random
_DSC1927.ARW  95px  0.90  c c  occ
_DSC1929.ARW  104px  0.86  x u  random
_DSC1930.ARW  77px  0.92  c c  random
_DSC1932.ARW  76px  0.91  c c  iris
_DSC1933.ARW  99px  0.90  c c  fm2
_DSC1935.ARW  92px  0.91  u u  random
_DSC1942.ARW  95px  0.76  x x  occ
_DSC1943.ARW  55px  0.83  x x  random
_DSC1944.ARW  57px  0.79  x x  fm1
_DSC1946.ARW  54px  0.66  u u  random
_DSC1947.ARW  66px  0.73  x o  iris
_DSC1948.ARW  37px  0.86  u u  occ
_DSC1949.ARW  39px  0.86  u u  occ
_DSC1954.ARW  40px  0.88  u u  occ
_DSC1957.ARW  36px  0.88  u u  iris
_DSC1964.ARW  37px  0.77  x x  random
_DSC1968.ARW  89px  0.62  x u  fm1
_DSC1972.ARW  116px  0.67  x x  random
_DSC1974.ARW  51px  0.78  x x  iris
_DSC1975.ARW  58px  0.83  x u  random
_DSC1976.ARW  51px  0.74  x x  occ
_DSC1988.ARW  70px  0.79  u u  iris
_DSC1991.ARW  96px  0.81  x u  occ
_DSC2003.ARW  69px  0.78  o o  random
_DSC2006.ARW  68px  0.92  c c  random
_DSC2008.ARW  76px  0.91  c c  fm2
_DSC2009.ARW  70px  0.92  u u  occ
_DSC2013.ARW  90px  0.93  c c  fm2
_DSC2014.ARW  89px  0.92  o o  random
_DSC2022.ARW  116px  0.86  o o  random
_DSC2027.ARW  98px  0.72  x x  occ
_DSC2031.ARW  34px  0.72  x x  fm1
_DSC2032.ARW  35px  0.79  x x  random
_DSC2033.ARW  32px  0.73  x x  random
_DSC2034.ARW  43px  0.72  x x  occ
_DSC2035.ARW  51px  0.73  x x  occ
_DSC2036.ARW  58px  0.80  x x  random
_DSC2042.ARW  68px  0.84  u o  fm1
_DSC2043.ARW  78px  0.87  x x  occ
_DSC2045.ARW  84px  0.60  u o  random
_DSC2050.ARW  83px  0.74  o o  random
_DSC2059.ARW  78px  0.74  o u  random
_DSC2065.ARW  79px  0.92  o o  random
_DSC2072.ARW  215px  0.74  x x  random
_DSC2079.ARW  72px  0.84  x o  occ
_DSC2080.ARW  68px  0.79  x u  occ
_DSC2081.ARW  79px  0.77  x x  occ
_DSC2083.ARW  84px  0.92  c c  occ
_DSC2090.ARW  65px  0.82  u u  random
_DSC2097.ARW  42px  0.91  u u  random
_DSC2103.ARW  100px  0.84  o x  random
_DSC2108.ARW  75px  0.76  x x  occ
_DSC2109.ARW  64px  0.91  c c  random
_DSC2110.ARW  67px  0.92  c c  fm2
_DSC2111.ARW  72px  0.89  c c  random
_DSC2114.ARW  70px  0.67  u o  random
_DSC2118.ARW  93px  0.87  o o  random
_DSC2122.ARW  100px  0.88  x u  fm1
_DSC2127.ARW  82px  0.63  x x  random
_DSC2134.ARW  77px  0.87  o o  iris
_DSC2135.ARW  79px  0.90  o o  random
_DSC2137.ARW  87px  0.90  u u  iris
_DSC2150.ARW  34px  0.81  x x  occ
_DSC2151.ARW  28px  0.79  x x  occ
_DSC2152.ARW  29px  0.82  x x  occ
_DSC2153.ARW  29px  0.76  x x  occ
_DSC2167.ARW  22px  0.77  x x  occ
_DSC2168.ARW  23px  0.80  x x  occ
_DSC2177.ARW  25px  0.61  x x  fm1
_DSC2178.ARW  50px  0.65  x x  random
_DSC2182.ARW  111px  0.93  c c  iris
_DSC2183.ARW  113px  0.93  c c  random
_DSC2184.ARW  121px  0.94  c c  fm2
_DSC2185.ARW  85px  0.91  x x  occ
_DSC2189.ARW  44px  0.83  x x  random
_DSC2190.ARW  37px  0.88  u u  occ
_DSC2193.ARW  92px  0.89  x u  random
_DSC2195.ARW  86px  0.85  o u  random
_DSC2198.ARW  138px  0.93  u u  random
_DSC2199.ARW  223px  0.93  c c  occ
_DSC2200.ARW  220px  0.94  x u  occ
_DSC2207.ARW  77px  0.89  c c  iris
_DSC2208.ARW  80px  0.91  c c  iris
_DSC2212.ARW  93px  0.82  x x  fm1
_DSC2228.ARW  116px  0.63  x x  random
_DSC2234.ARW  105px  0.85  x x  random
_DSC2237.ARW  120px  0.87  x o  random
_DSC2241.ARW  72px  0.69  x x  random
_DSC2246.ARW  183px  0.92  u u  occ
_DSC2247.ARW  189px  0.91  o o  random
_DSC2248.ARW  200px  0.92  c c  occ
_DSC2250.ARW  261px  0.94  c c  random
_DSC2251.ARW  302px  0.84  x o  iris
_DSC2263.ARW  95px  0.78  x x  occ
_DSC2270.ARW  77px  0.88  x o  random
_DSC2279.ARW  74px  0.89  x u  random
_DSC2280.ARW  88px  0.69  x u  occ
_DSC2281.ARW  85px  0.84  x x  occ
_DSC2285.ARW  129px  0.90  x o  random
_DSC2289.ARW  68px  0.78  x x  random
_DSC2297.ARW  70px  0.79  x x  occ
_DSC2304.ARW  63px  0.88  x u  random
_DSC2316.ARW  67px  0.74  x x  random
_DSC2321.ARW  35px  0.78  x x  fm1
_DSC2325.ARW  75px  0.89  u u  random
_DSC2326.ARW  84px  0.89  x x  iris
_DSC2329.ARW  114px  0.86  x c  random
_DSC2333.ARW  64px  0.79  x x  fm1
_DSC2338.ARW  30px  0.80  x x  occ
_DSC2339.ARW  71px  0.66  x x  occ
_DSC2342.ARW  70px  0.87  x x  random
_DSC2346.ARW  73px  0.89  x x  iris
_DSC2356.ARW  85px  0.91  u u  random
_DSC2360.ARW  119px  0.79  x x  iris
_DSC2361.ARW  111px  0.86  x x  occ
_DSC2362.ARW  109px  0.85  x x  occ
_DSC2365.ARW  78px  0.90  x o  random
_DSC2366.ARW  82px  0.87  x u  occ
_DSC2367.ARW  44px  0.66  x x  occ
_DSC2368.ARW  71px  0.88  c c  fm2
_DSC2369.ARW  66px  0.85  c c  occ
_DSC2370.ARW  75px  0.87  c c  fm2
_DSC2371.ARW  88px  0.90  c c  iris
_DSC2372.ARW  88px  0.92  c c  iris
_DSC2381.ARW  82px  0.92  o o  random
_DSC2385.ARW  84px  0.91  o o  random
_DSC2388.ARW  43px  0.87  x x  random
_DSC2392.ARW  110px  0.92  o o  random
_DSC2394.ARW  96px  0.92  u u  random
_DSC2398.ARW  84px  0.91  c c  iris
_DSC2400.ARW  67px  0.73  u u  iris
_DSC2401.ARW  95px  0.92  u u  random
_DSC2408.ARW  85px  0.92  c c  iris
_DSC2411.ARW  142px  0.80  x x  random
_DSC2414.ARW  104px  0.92  u u  iris
_DSC2424.ARW  88px  0.80  x x  occ
_DSC2425.ARW  80px  0.71  x x  random
_DSC2428.ARW  128px  0.94  o o  occ
_DSC2431.ARW  128px  0.77  x x  fm1
_DSC2434.ARW  49px  0.90  o o  random
_DSC2443.ARW  75px  0.66  x x  random
_DSC2444.ARW  32px  0.65  x x  random
_DSC2448.ARW  70px  0.91  u u  random
_DSC2451.ARW  38px  0.73  x x  fm1
_DSC2453.ARW  75px  0.88  x x  occ
_DSC2454.ARW  75px  0.85  x x  random
_DSC2456.ARW  39px  0.77  x x  fm1
_DSC2457.ARW  69px  0.65  x x  random
_DSC2458.ARW  48px  0.72  x x  random
_DSC2459.ARW  49px  0.73  x x  random
_DSC2468.ARW  70px  0.63  x x  occ
_DSC2487.ARW  49px  0.74  x x  occ
_DSC2490.ARW  51px  0.62  x x  occ
_DSC2498.ARW  53px  0.68  x x  random
_DSC2514.ARW  67px  0.88  o o  random
_DSC2523.ARW  72px  0.91  o o  random
_DSC2538.ARW  76px  0.91  o o  random
_DSC2544.ARW  64px  0.90  o o  random
_DSC2548.ARW  74px  0.91  o o  occ
_DSC2550.ARW  73px  0.90  o o  occ
_DSC2552.ARW  78px  0.90  o o  occ
_DSC2554.ARW  73px  0.91  o o  iris
_DSC2557.ARW  62px  0.89  x x  iris
_DSC2559.ARW  75px  0.92  o o  random
_DSC2565.ARW  87px  0.87  o u  random
_DSC2567.ARW  78px  0.91  o o  random
_DSC2569.ARW  84px  0.92  o o  random
_DSC2579.ARW  66px  0.77  x x  random
_DSC2580.ARW  95px  0.86  x o  random
_DSC2592.ARW  43px  0.66  x x  occ
_DSC2594.ARW  40px  0.60  x x  random
_DSC2596.ARW  41px  0.79  x x  occ
_DSC2599.ARW  47px  0.90  u u  random
_DSC2605.ARW  72px  0.92  o o  random
_DSC2617.ARW  59px  0.75  o o  occ
_DSC2619.ARW  68px  0.91  o o  random
_DSC2620.ARW  71px  0.90  o o  random
_DSC2629.ARW  78px  0.93  o o  occ
_DSC2632.ARW  80px  0.89  o o  occ
_DSC2635.ARW  82px  0.93  o o  occ
_DSC2637.ARW  84px  0.91  o o  random
_DSC2639.ARW  76px  0.92  o o  occ
_DSC2642.ARW  85px  0.92  o o  random
_DSC2644.ARW  82px  0.90  o o  occ
_DSC2647.ARW  80px  0.86  x x  occ
_DSC2657.ARW  96px  0.92  u u  random
_DSC2664.ARW  76px  0.92  o o  random
_DSC2665.ARW  77px  0.90  o o  random
_DSC2667.ARW  77px  0.92  o o  random
_DSC2672.ARW  76px  0.92  o o  random
_DSC2676.ARW  61px  0.66  x o  random
_DSC2678.ARW  85px  0.72  o o  random
_DSC2679.ARW  68px  0.71  o o  random
_DSC2691.ARW  73px  0.92  o o  random
_DSC2700.ARW  86px  0.82  x x  occ
_DSC2702.ARW  95px  0.80  x x  random
_DSC2709.ARW  42px  0.82  u u  random
_DSC2717.ARW  82px  0.92  o o  random
_DSC2724.ARW  86px  0.81  x x  iris
_DSC2727.ARW  99px  0.80  x x  occ
_DSC2729.ARW  95px  0.87  u u  iris
_DSC2730.ARW  99px  0.82  x x  random
_DSC2739.ARW  55px  0.67  x x  random
_DSC2750.ARW  76px  0.81  x x  fm1
_DSC2758.ARW  97px  0.84  u x  occ
_DSC2769.ARW  76px  0.92  o o  random
_DSC2771.ARW  74px  0.93  o o  random
_DSC2789.ARW  73px  0.90  o o  occ
_DSC2795.ARW  78px  0.91  o o  occ
_DSC2797.ARW  74px  0.90  o o  occ
_DSC2802.ARW  77px  0.88  x x  random
_DSC2810.ARW  69px  0.77  x x  fm1
_DSC2811.ARW  35px  0.65  x x  fm1
_DSC2813.ARW  69px  0.93  o o  random
_DSC2816.ARW  74px  0.93  o o  random
_DSC2819.ARW  72px  0.92  o o  random
_DSC2828.ARW  70px  0.91  o o  random
_DSC2840.ARW  43px  0.69  x x  fm1
_DSC2841.ARW  45px  0.72  x x  fm1
_DSC2844.ARW  76px  0.92  o o  random
_DSC2845.ARW  77px  0.93  o o  random
_DSC2864.ARW  48px  0.82  x x  fm1
_DSC2865.ARW  56px  0.82  x x  fm1
_DSC2866.ARW  62px  0.81  x x  random
_DSC2887.ARW  86px  0.90  x o  random
_DSC2899.ARW  60px  0.64  x x  fm2
_DSC2906.ARW  85px  0.88  o o  random
_DSC2911.ARW  47px  0.61  x x  random
_DSC2912.ARW  78px  0.83  x o  random
_DSC2914.ARW  42px  0.80  x x  fm1
_DSC2916.ARW  44px  0.75  x x  fm1
_DSC2918.ARW  68px  0.62  x x  random
_DSC2919.ARW  58px  0.62  x x  random
_DSC2921.ARW  61px  0.77  x x  random
_DSC2922.ARW  59px  0.74  x x  random
_DSC2928.ARW  77px  0.91  o o  random
_DSC2930.ARW  78px  0.90  o o  random
_DSC2935.ARW  82px  0.92  o o  random
_DSC2957.ARW  76px  0.68  x x  random
_DSC2958.ARW  54px  0.65  x x  fm1
_DSC2981.ARW  94px  0.82  x o  iris
_DSC2987.ARW  76px  0.67  x x  random
_DSC2991.ARW  47px  0.79  x x  fm1
_DSC2992.ARW  44px  0.77  x x  random
_DSC2993.ARW  56px  0.67  x x  fm1
_DSC3001.ARW  75px  0.90  o o  random
_DSC3008.ARW  89px  0.90  o o  random
_DSC3010.ARW  85px  0.87  x o  random
_DSC3014.ARW  88px  0.83  x o  random
_DSC3020.ARW  93px  0.90  o o  iris
_DSC3025.ARW  109px  0.72  x x  fm1
_DSC3030.ARW  57px  0.88  u u  fm1
_DSC3031.ARW  60px  0.90  u u  fm1
_DSC3032.ARW  67px  0.89  u u  random
_DSC3033.ARW  60px  0.86  u u  random
_DSC3035.ARW  104px  0.93  o o  random
_DSC3043.ARW  76px  0.92  o o  random
_DSC3045.ARW  44px  0.78  x x  fm2
_DSC3046.ARW  40px  0.67  x x  fm1
_DSC3050.ARW  44px  0.79  x x  fm1
_DSC3051.ARW  39px  0.64  x x  fm1
_DSC3053.ARW  86px  0.91  o o  random
_DSC3055.ARW  87px  0.90  o o  random
_DSC3066.ARW  86px  0.78  x x  iris
_DSC3072.ARW  69px  0.91  o o  random
_DSC3102.ARW  57px  0.83  x x  fm1
_DSC3103.ARW  59px  0.78  x x  fm1
_DSC3112.ARW  58px  0.91  o o  random
_DSC3118.ARW  55px  0.89  o o  random
_DSC3127.ARW  50px  0.86  o o  random
_DSC3139.ARW  58px  0.86  u u  fm2
_DSC3143.ARW  57px  0.90  o o  random
_DSC3145.ARW  56px  0.92  o o  random
_DSC3161.ARW  73px  0.68  o o  random
_DSC3163.ARW  62px  0.82  o o  random
_DSC3169.ARW  63px  0.84  u u  fm1
_DSC3170.ARW  60px  0.61  x x  random
_DSC3172.ARW  65px  0.82  u u  random
_DSC3185.ARW  63px  0.88  u u  random
_DSC3187.ARW  52px  0.75  x x  random
_DSC3190.ARW  56px  0.70  x x  random
_DSC3191.ARW  22px  0.75  x x  fm1
_DSC3193.ARW  66px  0.80  x u  random
_DSC3194.ARW  69px  0.85  u u  random
_DSC3200.ARW  71px  0.91  c c  random
_DSC3201.ARW  68px  0.91  u u  iris
_DSC3202.ARW  72px  0.91  c c  iris
_DSC3204.ARW  80px  0.89  x u  iris
_DSC3208.ARW  77px  0.92  o o  random
_DSC3210.ARW  74px  0.92  o o  random
_DSC3214.ARW  73px  0.90  o o  random
_DSC3227.ARW  78px  0.92  o o  random
_DSC3243.ARW  81px  0.91  o o  random
_DSC3250.ARW  77px  0.88  c u  random
_DSC3263.ARW  75px  0.92  o o  random
_DSC3275.ARW  72px  0.83  u x  iris
_DSC3293.ARW  62px  0.82  u o  random
_DSC3298.ARW  76px  0.87  o o  random
_DSC3307.ARW  69px  0.86  o o  random
_DSC3310.ARW  87px  0.88  x u  fm1
_DSC3314.ARW  85px  0.90  x u  fm2
_DSC3326.ARW  65px  0.86  o o  random
_DSC3328.ARW  67px  0.86  o o  random
_DSC3336.ARW  71px  0.86  x x  random
_DSC3339.ARW  73px  0.90  o o  random
_DSC3344.ARW  131px  0.87  u o  random
_DSC3367.ARW  93px  0.90  o o  random
_DSC3382.ARW  99px  0.88  o o  random
_DSC3386.ARW  90px  0.89  o o  random
_DSC3394.ARW  82px  0.92  o o  random
_DSC3395.ARW  80px  0.91  o o  random
_DSC3398.ARW  77px  0.93  o o  random
_DSC3403.ARW  74px  0.92  u o  iris
_DSC3405.ARW  76px  0.92  o o  random
_DSC3406.ARW  72px  0.92  o o  iris
_DSC3407.ARW  72px  0.92  o o  random
_DSC3408.ARW  74px  0.92  o o  iris
_DSC3410.ARW  73px  0.92  o o  iris
_DSC3415.ARW  61px  0.88  o x  random
_DSC3419.ARW  85px  0.91  o o  random
_DSC3428.ARW  133px  0.94  o o  random
_DSC3430.ARW  121px  0.78  x o  random
_DSC3451.ARW  79px  0.93  o o  random
_DSC3454.ARW  84px  0.92  u u  random
_DSC3455.ARW  81px  0.93  u u  iris
_DSC3456.ARW  79px  0.92  o o  iris
_DSC3457.ARW  80px  0.91  o o  iris
_DSC3458.ARW  81px  0.92  o o  iris
_DSC3479.ARW  70px  0.80  x x  fm1
_DSC3480.ARW  64px  0.81  x x  fm1
_DSC3495.ARW  114px  0.92  o o  random
_DSC3496.ARW  118px  0.92  o o  random
_DSC3506.ARW  53px  0.82  x x  fm1
_DSC3507.ARW  56px  0.83  x x  fm1
_DSC3508.ARW  44px  0.75  x x  random
_DSC3512.ARW  41px  0.73  x x  fm1
_DSC3513.ARW  42px  0.70  x x  fm1
_DSC3528.ARW  135px  0.93  c c  fm2
_DSC3529.ARW  135px  0.93  c c  fm2
_DSC3530.ARW  133px  0.94  c c  random
_DSC3536.ARW  110px  0.74  x u  fm1
_DSC3573.ARW  82px  0.90  x o  random
_DSC3578.ARW  45px  0.91  u u  random
_DSC3587.ARW  72px  0.78  x x  random
_DSC3592.ARW  60px  0.85  x x  iris
_DSC3602.ARW  107px  0.87  x o  random
_DSC3606.ARW  71px  0.91  x o  random
_DSC3608.ARW  79px  0.84  x o  random
_DSC3612.ARW  97px  0.92  u u  random
_DSC3613.ARW  71px  0.94  o o  random
_DSC3622.ARW  90px  0.93  c c  fm2
_DSC3626.ARW  95px  0.92  o o  random
_DSC3627.ARW  111px  0.91  o o  random
_DSC3629.ARW  165px  0.77  x u  random
_DSC3630.ARW  104px  0.92  c c  random
_DSC3636.ARW  70px  0.70  x x  random
_DSC3638.ARW  52px  0.90  u u  random
_DSC3642.ARW  81px  0.82  x o  iris
_DSC3649.ARW  67px  0.92  o o  random
_DSC3663.ARW  81px  0.89  x x  random
_DSC3670.ARW  86px  0.93  c c  fm2
_DSC3679.ARW  113px  0.87  x o  random
_DSC3683.ARW  62px  0.92  c c  random
_DSC3688.ARW  68px  0.93  c c  iris
_DSC3692.ARW  69px  0.90  x x  random
_DSC3699.ARW  72px  0.94  o o  random
_DSC3721.ARW  199px  0.95  o o  random
_DSC3727.ARW  74px  0.92  o o  random
_DSC3729.ARW  70px  0.91  c c  random
_DSC3730.ARW  85px  0.90  u u  fm2
_DSC3743.ARW  123px  0.86  x x  random
_DSC3755.ARW  81px  0.92  o o  random
_DSC3764.ARW  60px  0.68  x x  fm2
_DSC3766.ARW  22px  0.78  x x  fm1
_DSC3770.ARW  74px  0.89  c c  random
_DSC3784.ARW  154px  0.91  o o  random
_DSC3791.ARW  118px  0.76  x x  random
_DSC3795.ARW  105px  0.89  c c  fm2
_DSC3799.ARW  24px  0.64  x x  fm1
_DSC3803.ARW  71px  0.86  u u  random
_DSC3805.ARW  73px  0.88  u u  fm2
_DSC3806.ARW  82px  0.89  u u  fm2
_DSC3808.ARW  68px  0.93  u u  fm2
_DSC3810.ARW  80px  0.93  u u  random
_DSC3823.ARW  66px  0.83  x x  fm1
_DSC3837.ARW  74px  0.87  u u  iris
_DSC3843.ARW  85px  0.90  o o  random
_DSC3844.ARW  87px  0.93  o o  random
_DSC3846.ARW  99px  0.91  x o  random
_DSC3858.ARW  88px  0.89  x u  random
_DSC3866.ARW  113px  0.71  x x  fm1
_DSC3870.ARW  105px  0.80  x x  fm1
_DSC3871.ARW  108px  0.81  x x  random
_DSC3873.ARW  111px  0.83  x x  random
_DSC3885.ARW  194px  0.87  c c  iris
_DSC3886.ARW  187px  0.89  u u  iris
_DSC3888.ARW  203px  0.89  u u  iris
_DSC3891.ARW  187px  0.86  o o  iris
_DSC3904.ARW  74px  0.85  u u  random
_DSC3909.ARW  79px  0.86  c c  fm2
_DSC3910.ARW  66px  0.67  x x  iris
```
