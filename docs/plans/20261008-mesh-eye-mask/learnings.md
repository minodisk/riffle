# Learnings: mesh-eye-mask

## Step 1: mask primitives and the mask dump

- The primitives in `crates/core/src/candidate.rs`: `eye_mask` (scanline
  even-odd fill at pixel centers, then a morphological dilation by a disk of
  radius `dilation` x the contour box's longer side, evaluated at pixel
  centers as "within the radius of a polygon edge"; the window is the tight
  bounding box of the masked pixels, `EYE_MASK_MIN` = 3),
  `masked_laplacian_variance`, `masked_edge_width`, `masked_eye_measures`.
  Both masked measures take every masked pixel whose neighbors lie in the
  **image** (not the window interior), since the mask window is tight and its
  border pixels are masked ones. With a mask covering the whole image the
  masked edge width equals `edge_width` over that window, and with a mask over
  a window's interior the masked Laplacian equals `laplacian_variance` (both
  tested). The edge-width walk stays bounded by the mask window, so the
  clipped-walk variant of the Trade-offs is not in the dump.
- The fill is half-open along a row (a pixel whose center lies exactly on the
  right crossing is out), so a symmetric diamond's mask is 19 columns by 20
  rows; its pixel count still equals its area (200).
- The dump: [`maskdump.patch`](maskdump.patch), applied on this step's
  commit, built in release, `riffle-cli maskdump <dir>... [threads]`, then
  reverted. Dilations `[0, 0.1, 0.25, 0.5, 1.0]`. It applies the face gate
  (`candidate::meshes_face`) like the scan: dlg-mesh-face-gate (#737,
  `ac6f6ade`) was already on `main` when the dump ran, so no rebase was
  needed. Data in `D:\Photos\tests\2026-10-08-mesh-eye-mask\`:
  `training-mask.jsonl` (406 faced lines, all labeled), `heldout-mask.jsonl`
  (400), `training-mask-1-thread.jsonl` (the same training dump at 1 thread,
  for the timing), `run.txt` (commit hash, gate status).
- Check: the dump's rectangle columns (margin 0.5, floor 24, current
  `MESH_LOGIT_*`, window fallback) reproduce the stored `p` of all 806 faced
  frames exactly. Meshed frames: 300 / 406 training, 297 / 400 held-out (the
  gate sends faces under 60 px straight to the window); the archived
  0.882 / 0.800 predate the gate, which changes one counting training frame,
  so Step 2's baseline may differ from them by that frame.
- Sizes, per dilation, over the meshed eyes (600 training, 594 held-out;
  "masks" is how many eyes have a mask of at least 3 px, "side" the mask
  window's longer side in px, "no edge" the masks without a masked edge
  width, ">= 24" the masks whose side reaches the current 24 px floor):

  | set | d | masks | pixels p10 / p50 / p90 | side p10 / p50 / p90 | no edge | >= 24 |
  | --- | --- | --- | --- | --- | --- | --- |
  | training | 0 | 562 | 7 / 23 / 61 | 5 / 10 / 16 | 89 | 3 |
  | training | 0.1 | 593 | 11 / 49 / 124 | 5 / 12 / 20 | 36 | 19 |
  | training | 0.25 | 600 | 20 / 102 / 263 | 6 / 15 / 25 | 16 | 72 |
  | training | 0.5 | 600 | 42 / 221 / 584 | 9 / 20 / 34 | 13 | 198 |
  | training | 1.0 | 600 | 108 / 584 / 1541 | 13 / 31 / 51 | 1 | 417 |
  | held-out | 0 | 567 | 7 / 26 / 120 | 4 / 10 / 23 | 70 | 55 |
  | held-out | 0.1 | 591 | 13 / 58 / 258 | 5 / 13 / 29 | 19 | 82 |
  | held-out | 0.25 | 594 | 25 / 117 / 582 | 7 / 16 / 37 | 16 | 131 |
  | held-out | 0.5 | 594 | 53 / 256 / 1317 | 9 / 22 / 49 | 7 | 252 |
  | held-out | 1.0 | 594 | 129 / 657 / 3458 | 14 / 33 / 74 | 0 | 440 |

  At dilation 0 the mask window's longer side is about the rectangle's at
  margin 0 (archived: median 11 px), and the 24 px floor would keep almost no
  training eye, so Step 2's floor grid (0-24) matters most at small
  dilations.
- Per-eye cost of the mask raster plus the masked measures (training, 1
  thread, 600 eyes, `Instant` around `eye_mask` + `masked_eye_measures`), in
  microseconds, p10 / p50 / p90 / max (mean): d 0: 1.4 / 3.5 / 6.6 / 140
  (4.1); d 0.1: 3.2 / 8.2 / 17.6 / 62 (9.5); d 0.25: 4.7 / 12.8 / 28.5 / 95
  (15.2); d 0.5: 6.9 / 22.2 / 49.7 / 179 (26.3); d 1.0: 13.1 / 46.5 / 110 /
  413 (55.5). Two eyes at any dilation stay well under 1 ms per frame,
  negligible against the ~100 ms mesh.
