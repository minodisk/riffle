# Learnings

## Step 1: core ranged read + focus-point RGBA crop

- `reader::read_preview` and the new `read_full` share `read_embedded` /
  `embedded_from`, parameterised by a small `Kind` enum that picks
  `arw.preview` or `arw.full` and names itself in the error messages.
  `read_preview`'s behaviour is unchanged.
- `partial::decode_crop` and the new `decode_focus_crop` share
  `decode_region`, which takes the colour space, the bytes per pixel and a
  closure handed the JPEG's size that returns the wanted centre and size.
  `Crop::rgb` became `Crop::pixels` because the RGBA variant fills the same
  field; `bench` does not read the field, so it stayed untouched.
- `FocusCrop` carries `point_x` / `point_y` in crop coordinates. They come out
  of the crop's actual origin, which `jpeg_crop_scanline` snaps down to an MCU
  boundary (16px here), so the point is not the crop's centre: on the test
  file 3613 - 3344 = 269 against a centre of 262.
- A crop decoded with `jpeg_crop_scanline` is **not** bit-identical to the
  same region of a full `decode_rgb` near its left and right edges: chroma
  upsampling has one fewer neighbour there. Measured differences up to 2 in
  the first columns and 1 in the last. The unit test therefore asserts
  equality only for `8..width - 8` and a tolerance of 4 on the edge columns.
  Tripped this over twice before narrowing it down.
- CLI output on `~/Downloads/_DSC6978.ARW` (release build, second run, warm
  page cache, n=1 per line; the PNG goes to the scratchpad and is never
  committed):

  ```
  read JpgFromRaw (5761112 bytes) in 887.125µs
  crop 525x512 at (3344,1476) point (269,256) in 19.052042ms
  wrote ".../crop.png" (512x525)
  ```

  The first run of a fresh process reads in 9.5ms and crops in 19.3ms. These
  are single runs, not the n=20 medians from the plan; Step 4 re-measures.
- `image::save_buffer` and `decode::apply_orientation` are RGB-only, so the
  CLI drops the alpha channel before rotating. Not worth generalising
  `apply_orientation` for one caller.

## Deferred issues (todo candidates)

- `riffle-cli bench`'s crop row still feeds `FocusLocation` coordinates
  straight into `decode_crop` without the sensor→JPEG scaling
  (`crates/cli/src/main.rs`, `bench()`). The plan explicitly scopes Step 1 to
  fixing `crop` only ("`riffle-cli crop` and `bench` currently skip the
  scaling; Step 1 fixes `crop`"), so `bench` was left as is. It is correct on
  the test file (sensor and JPEG are both 7008 wide) but wrong on any body
  where they differ.
