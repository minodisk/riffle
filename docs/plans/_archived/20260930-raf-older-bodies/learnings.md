# Learnings: raf-older-bodies

## Step 1

- Re-checked the plan's per-body results before writing: joined the sweep's
  scratchpad `raf.tsv` (228 files, 87 bodies, every file `parse=prefix`,
  preview and full decode `ok`, `0x48` = `0x0` on all) with
  `D:\photos\samples\MANIFEST-pixls.tsv` on `file`, kept the
  "Creative Commons 0 - Public Domain" rows, and got exactly the plan's 57
  bodies with the plan's JPEG sizes and AF results. Files outside the
  raw.pixls.us manifest (Photography Blog, mirrorlesscomparison and similar)
  do not match the join and were ignored, as the plan says.
- Deviation from the plan's wording: the plan said the older bodies' JPEG is
  "1920x1280 or smaller", but the FinePix X100 (2176x1448) and the fifteen
  2048x1536 bodies are larger than 1920x1280. README / README.ja say
  "1920x1280 on most of them and at most 2176x1448", and
  `docs/raw-formats.md` names 1920x1280, 2048x1536 (small-sensor X
  compacts) and at most 2176x1448 (FinePix).
- The `focus_mode` column of `raf.tsv` is `-` on every file, so the "MF
  sample" reading of the AF `–` bodies comes from the plan (the sweep's
  harness notes), not from this join. The `–` itself is what the join shows:
  no `FocusPixel` on any CC0 sample of those bodies.
- The Exif `Model` has trailing spaces on 14 of the 87 bodies (FinePix E550,
  E900, F700, S100FS, S3Pro, S5000, S5200, S5500, S5600, S5Pro, S7000, S9500,
  S9600, SL1000); recorded as a `todo.md` section.
- The `todo.md` references point at the `_archived/` path this plan folder
  moves to on wrap-up.

### Per-body measurements (CC0 raw.pixls.us samples only)

JPEG size is the embedded JPEG (preview = 1:1 view). AF lists the
`FocusPixel` position per sample, `–` when absent. Sub-second is the Exif
`SubSecTimeOriginal`.

| Exif Model | CC0 sample files | JPEG size | AF (`FocusPixel`) | Sub-second |
|---|---|---|---|---|
| `DBP for GX680` | `DBP_for_GX680_DSCF0010.RAF` | 1344x960 | – | – |
| `FinePix F550EXR` | `FinePix_F550EXR_DSCF6714.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix F770EXR` | `FinePix_F770EXR_DSCF0686.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix HS30EXR` | `FinePix_HS30EXR_DSCF4080.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix HS33EXR` | `FinePix_HS33EXR_DSCF6464.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix HS50EXR` | `FinePix_HS50EXR_DSCF0016.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix S1` | `FinePix_S1_DSCF0001.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix S100FS` | `FinePix_S100FS_DSCF8410.RAF` | 1600x1200 | – | – |
| `FinePix S200EXR` | `FinePix_S200EXR_DSCF4754.RAF` | 2048x1536 | 1024,768 | – |
| `FinePix S3Pro` | `FinePix_S3Pro_DSCF3694.RAF` | 1440x960 | – | – |
| `FinePix S5000` | `FinePix_S5000_2007_0912Image0022.RAF` | 1280x960 | 640,480 | – |
| `FinePix S5200` | `FinePix_S5200_DSCF1523.RAF` | 1600x1200 | 800,600 | – |
| `FinePix S5500` | `FinePix_S5500_DSCF6206.RAF` | 1280x960 | 640,480 | – |
| `FinePix S5Pro` | `FinePix_S5Pro_2018_06150003.RAF` | 1440x960 | – | – |
| `FinePix S6000fd` | `FinePix_S6000fd_DSCF1973.RAF` | 1600x1200 | – | – |
| `FinePix S6500fd` | `FinePix_S6500fd_DSCF1333.RAF` | 1600x1200 | 800,600 | – |
| `FinePix S7000` | `FinePix_S7000_dscf3648.raf` | 1280x960 | 640,480 | – |
| `FinePix S9500` | `FinePix_S9500_20170430_0041.RAF` | 1600x1200 | 960,720 | – |
| `FinePix S9600` | `FinePix_S9600_2007-11-10--16-32-45-original.raf` | 1600x1200 | 800,600 | – |
| `FinePix SL1000` | `FinePix_SL1000_RAW_file_from_Fijifilm_Finepix_SL1000.RAF` | 2048x1536 | 1020,758 | – |
| `FinePix X100` | `FinePix_X100_DSCF1676.RAF` | 2176x1448 | 1088,724 | – |
| `FinePixS2Pro` | `FinePix_S2Pro_DSCF0002.RAF` | 1440x960 | – | – |
| `GFX 50R` | `GFX_50R_2019-01-24-14-02-50_DSCF1316_e819634e46ecdb8ea241012ee70ae11e5c220c48.raf`, `GFX_50R_2019-01-24-14-03-03_DSCF1317_c9ab436f1532b532976eec431f0e4db7e9f2b8e2.raf` | 4000x3000 | 1441,1781, 2001,941 | – |
| `GFX 50S` | `GFX_50S_20170525_0036TEST.RAF`, `GFX_50S_20170525_0037TEST.RAF` | 4000x3000 | –, – | – |
| `X-A1` | `X-A1_DSCF2482.RAF` | 1920x1280 | 1182,640 | – |
| `X-A10` | `X-A10_DSCF8207.RAF` | 1920x1280 | 957,637 | – |
| `X-A2` | `X-A2_DSCF2073.RAF` | 1920x1280 | – | – |
| `X-A3` | `X-A3_DSCF7241.RAF` | 1920x1280 | 301,776 | – |
| `X-A5` | `X-A5_DSCF0617.RAF` | 1920x1280 | 720,768 | – |
| `X-A7` | `X-A7_DSCF1595.RAF` | 1920x1280 | 1213,509 | – |
| `X-E1` | `X-E1_DSCF6490.RAF` | 1920x1280 | 960,640 | – |
| `X-E2` | `X-E2_DSCF9050.RAF` | 1920x1280 | 1195,641 | – |
| `X-E2S` | `X-E2S__DSF0540.RAF` | 1920x1280 | 960,640 | – |
| `X-E3` | `X-E3_DSCF2175.RAF`, `X-E3_DSCF2176.RAF` | 1920x1280 | –, – | – |
| `X-H1` | `X-H1_fuji_xh1_compressed.RAF`, `X-H1_fuji_xh1_uncompressed.RAF` | 1920x1280 | 961,775, 961,775 | – |
| `X-M1` | `X-M1_DSCF1442.RAF` | 1920x1280 | 960,640 | – |
| `X-Pro1` | `X-Pro1_DSCF2131.RAF` | 1920x1280 | 960,640 | – |
| `X-Pro2` | `X-Pro2__DSF3050.RAF`, `X-Pro2__DSF3051.RAF` | 1920x1280 | 961,641, 961,641 | – |
| `X-S1` | `X-S1_DSCF9185.RAF` | 2048x1536 | 1024,768 | – |
| `X-T1` | `X-T1_20171229_110916.RAF` | 1920x1280 | 961,641 | – |
| `X-T10` | `X-T10_DSCF8146.RAF` | 1920x1280 | 960,640 | – |
| `X-T100` | `X-T100_DSCF0627.RAF` | 1920x1280 | 850,637 | – |
| `X-T2` | `X-T2_20170114_173532_TFW04723.RAF`, `X-T2_20170114_174341_TFW04727.RAF` | 1920x1280 | 1363,775, 1229,775 | – |
| `X-T20` | `X-T20_DSCF0526.RAF`, `X-T20_DSCF0527.RAF` | 1920x1280 | 1162,574, 1162,574 | – |
| `X-T200` | `X-T200_DSCF0074.RAF` | 1920x1280 | 1264,485 | – |
| `X-T30` | `X-T30_DSCF0065.RAF`, `X-T30_DSCF0066.RAF` | 4416x2944 | 2208,1472, 2208,1472 | – |
| `X10` | `FinePix_X10_DSCF0514.RAF` | 2048x1536 | 787,946 | – |
| `X100F` | `X100F_DSCF5760_x100f_lossless_compressed_raw_Temple.RAF`, `X100F_DSCF5761_x100f_uncompressed_raw_Temple.RAF` | 1920x1280 | –, – | – |
| `X100S` | `X100S_fujifilm-x100s-daylight-DSCF9505.RAF` | 1920x1280 | 960,640 | – |
| `X100T` | `X100T_DSCF0442.RAF` | 1920x1280 | 961,641 | – |
| `X20` | `X20_DSCF7451.RAF` | 2048x1536 | 1024,768 | – |
| `X30` | `X30_20160604_0009.RAF` | 2048x1536 | 1023,768 | – |
| `X70` | `X70_DSCF8848.RAF` | 1920x1280 | 960,793 | – |
| `XF1` | `XF1_DSCF4005.RAF` | 2048x1536 | 1024,768 | – |
| `XF10` | `XF10_DSCF6466.RAF` | 1920x1280 | – | – |
| `XQ1` | `XQ1_DSCF2004.RAF` | 2048x1536 | 1023,768 | – |
| `XQ2` | `XQ2_DSCF4186.RAF` | 2048x1536 | 1023,768 | – |
