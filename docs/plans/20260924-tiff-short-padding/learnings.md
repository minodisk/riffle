# Learnings

## Step 1

- `Embedded` carries only offset and length, so the regression test proves
  the dimensions were read correctly indirectly: the size-based tier choice
  (preview = smallest strip JPEG at least 1600 wide, full = largest) only
  lands on the expected offsets when the widths were parsed and the padded
  Compression / Photometric entries were accepted.
- Confirmed the test fails before the `integer()` mask and passes after.
- Manual check with `riffle-cli info`: SDIM0521.DNG full = offset 60549448,
  length 28363616; SDIM0558.DNG preview and full are `Some` (offset 59293692,
  length 29708773). On both files preview and full are the same JPEG,
  because the only other strip JPEG is 640x480, below `PREVIEW_MIN_WIDTH`,
  so the preview tier falls back to the full-size JPEG.

## Deferred issues (todo candidates)

- SIGMA fp L DNGs have no strip JPEG between 640x480 and the 9520x6328
  full-size one, so the preview tier falls back to the full-size JPEG (about
  28 MB) and the strip decodes the full-size JPEG per thumbnail. It may be
  worth checking strip load time on SIGMA fp L folders. Basis: the Step 1
  manual `riffle-cli info` check. Files: `crates/core/src/arw.rs`
  (`PREVIEW_MIN_WIDTH`, tier selection).
