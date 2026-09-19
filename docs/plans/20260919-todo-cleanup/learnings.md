# Learnings

## Step 1

- The `riffle-cli bench` crop-scaling todo item was stale: both `bench()` and
  `crop()` in `crates/cli/src/main.rs` call `partial::decode_focus_crop(.., a.shot.focus, ..)`,
  which scales sensor coordinates via `focus_point`. `partial.rs` already had a
  test for a JPEG smaller than the sensor (3504x2336 vs 7008x4672), so no new
  test was added.
- mozjpeg panics on both non-JPEG bytes and a JPEG truncated to half its
  length; `decode_rgb` now catches the panic and returns `Err`.
