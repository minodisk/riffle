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

## Step 2: the app's `focus_crop` command

- The payload header is 24 bytes: kind, orientation, crop width, crop height,
  point x, point y, and 4 reserved bytes. The existing 8-byte `payload()`
  header and kinds 1/2 are untouched; kind 3 is a separate builder
  (`crop_payload`) because the field set has nothing in common with them.
- `crop_size()` holds both the cap (`CROP_MAX = 1024`) and the Orientation 6/8
  swap, so it is unit-testable without a file. The synthetic TIFF the
  round-trip test builds has no Orientation tag (so orientation 1), which is
  why the swap is covered through `crop_size` rather than through the ARW.
- The crop's origin snaps **down** to the MCU boundary and the returned width
  is not necessarily widened: for a 64px-wide request centred at x=200 on a
  400x300 gradient, `jpeg_crop_scanline` gave width 64 at x=160, so the point
  of interest landed at 40, not 32. The test asserts
  `point_x == centre - crop.x` rather than a fixed number; an earlier guess
  that the width grows to absorb the snap was wrong (it does on the 4:2:2 test
  file, not here).
- `mozjpeg` was already a dev-dependency of `crates/app`, so the gradient
  encoder used by `index.rs`'s tests could be mirrored in `commands.rs`
  without touching `Cargo.toml`.

## Step 3: the frontend's 1:1 view

- The crop header carries no full-JPEG size, so `drawZoom()` cannot get the
  placeholder's scale from it. It uses the index row's `sensor_w` as the full
  JPEG's width (`scale = focus.sensor_w / bitmap.width`), which is exact on
  the test file (JPEG 7008x4672 = sensor) but is an approximation on a body
  whose JpgFromRaw is smaller than the sensor: the placeholder around the
  crop would then be slightly off-scale. The crop itself is unaffected (it is
  placed by its own point of interest). Adding the JPEG size to the header's
  reserved word would remove the approximation.
- With no index row (scan not there yet) or no `FocusLocation`, no placeholder
  is drawn; only the crop, centred by its point of interest.
- `draw()` applies `context.scale(dpr, dpr)`; `drawZoom()` deliberately does
  not, so everything in it is device pixels and the crop is 1:1 by
  construction.
- `putImageData` ignores the transform, so the RGBA bytes go through
  `createImageBitmap(new ImageData(...))`. No worker is involved because
  nothing has to be JPEG-decoded.
- `ZOOM_TIMING` is left in the code set to `false`. Flip it to `true` in
  `crates/app/ui/src/main.ts` to get the three `console.debug` marks
  (keypress, invoke resolved, bitmap ready).
- The (manual) checks of Step 3 are **awaiting the user's confirmation**: GUI
  automation does not work on this machine, so `Space` toggling, the upright
  Orientation 8 placement, paging while zoomed, the manual-focus centre
  fallback and the three timings are all unverified.

## Step 4: documentation and measurements

- The measurements were re-taken, not copied from the plan: a temporary
  `#[ignore]`d integration test under `crates/core/tests/` called
  `reader::read_full`, `partial::decode_focus_crop` and `partial::decode_crop`
  n=20 each in one release process, and was deleted before committing. The
  numbers moved a little against the planning ones (read 0.7ms vs 1.7ms, crop
  1024 at row 4400 44.1ms vs 42.7ms) but the shape is the same: the row
  dominates, the size barely matters.
- `riffle-cli crop` cannot produce the by-row column on its own (it always
  crops at the focus point and prints one run), which is why the harness called
  the core functions directly. The CLI's own output is quoted in the README's
  "verified without a GUI" paragraph instead.
- `CLAUDE.md`'s "Layout" section names crates and `src/index.rs` but does not
  list commands, so per the step's own condition it is left unchanged.
- Nothing about the 50ms budget is claimed as met: the README says so
  explicitly and points at the 44.1ms bottom-of-frame row, with the IPC hop and
  `createImageBitmap` still excluded. All Step 3 manual checks are listed under
  "Awaiting the user's confirmation (Phase 5)".

## Deferred issues (todo candidates)

- `riffle-cli bench`'s crop row still feeds `FocusLocation` coordinates
  straight into `decode_crop` without the sensor→JPEG scaling
  (`crates/cli/src/main.rs`, `bench()`). The plan explicitly scopes Step 1 to
  fixing `crop` only ("`riffle-cli crop` and `bench` currently skip the
  scaling; Step 1 fixes `crop`"), so `bench` was left as is. It is correct on
  the test file (sensor and JPEG are both 7008 wide) but wrong on any body
  where they differ.
- The `focus_crop` header has a reserved 4-byte word but no full-JPEG size,
  so the frontend's placeholder scale in `drawZoom()`
  (`crates/app/ui/src/main.ts`) falls back to `FocusLocation`'s sensor width.
  Carrying the JPEG width/height in the header (`crates/app/src/commands.rs`,
  `crop_payload`) would make it exact. Out of scope for Step 3, which is
  scoped to the frontend.
- The crop bitmap held for a file is only `close()`d when a new crop replaces
  it, so paging with the 1:1 view off leaves one stale bitmap alive until the
  next `Space` (`crates/app/ui/src/main.ts`). Dropping it in `show()` would
  need more than the one line Step 3 allows there.
