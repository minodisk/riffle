# Learnings

## Step 1: Sony face tracking docs

- Verified samples in `D:\photos\samples\ARW` (`AFTracking` 1, valid
  `FocusFrameSize`, frame drawn on the preview), frame on a face or eye:
  - α7 IV: `ILCE-7M4_valeqvisuals-1-signature_edits_free_raw_photos.ARW` (eye)
  - α7R V: `ILCE-7RM5_sony_a7r_v_37.arw` and `ILCE-7RM5_sony_a7r_v_69.arw` (eyes)
  - α7S III: `ILCE-7SM3_tag_signatureeditsco_DSC02439.ARW` and
    `ILCE-7SM3_tag_signatureeditsco_DSC04841.ARW` (face)
  - α6700: `ILCE-6700_sony_a6700_01.arw` (head)
  - ZV-E1: `ZV-E1_sony_zv_e1_01.arw` (eye of a face on a poster)
- Off-face with `AFTracking` 1 (the reason for the caveat in `docs/cameras.md`):
  `ILCE-6700_sony_a6700_70.arw` (empty background) and
  `ILCE-6700_sony_a6700_71.arw` (bread on a market stall).
- Older Sony bodies with `AFTracking` 1 but no `FocusFrameSize` were left out
  of the table, since the plan adds no rows and they gain nothing from face
  tracking without a frame.
- The exact-sensor-center rejection of `eye_af_frame` was kept in the
  cameras.md paragraph as one sentence.
