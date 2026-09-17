# Learnings

## Step 1: Extract `crates/core`

- The move was purely mechanical: `arw.rs` / `partial.rs` went over with
  `git mv` untouched, and `decode_rgb` / `apply_orientation` were lifted out of
  `crates/cli/src/main.rs` into `crates/core/src/decode.rs` with `pub` added.
  `focus_location` and `draw_rect` stayed in the CLI as planned.
- `mozjpeg` / `mozjpeg-sys` / `anyhow` moved to `crates/core`; `image` and
  `anyhow` stay in `crates/cli`. `anyhow` is needed in both because the CLI's
  own functions still return `Result`.
- `cargo clippy -D warnings` did not complain about missing docs on the newly
  `pub` items, so no extra doc comments were needed beyond the module `//!`.
- Verified `riffle-cli info ~/Downloads/_DSC6978.ARW` still prints
  `orientation: 8`, `preview: Some(Embedded { offset: 204962, length: 337198 })`,
  `full: Some(Embedded { offset: 544768, length: 5761112 })`.
- The hand-built TIFF fixture for the `arw::parse` tests only needs the 8-byte
  header plus IFD0; `next_ifd = 0` keeps the chain walk from running, so no
  SubIFD data has to be faked.
- `mise run fmt` + `mise run ci` passed on the first attempt.

## Deferred issues (todo candidates)

- (none)
