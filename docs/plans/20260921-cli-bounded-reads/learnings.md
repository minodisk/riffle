# Learnings

## Step 1

- `info` / `focusbox` / `bench` now go through `riffle_core::reader`; `riffle_core::arw` became
  unused in `crates/cli/src/main.rs` and the import was removed. `std::fs::read` is gone (only
  `std::fs::read_dir` in `scan_dir` remains).
- `bench` reads metadata first (`reader::read_metadata`) and gates the preview timing on
  `a.preview.is_some()` before calling `reader::read_preview`, and the full/crop timings on
  `a.full.is_some()` before calling `reader::read_full`, so a file missing either kind of
  embedded JPEG still contributes whichever timings it can, instead of aborting the whole batch
  (round 1 review fix: the first draft called `read_preview` unconditionally, which errored out
  on a preview-less file and discarded all timings collected so far).

### Output verification

Local samples: `/Users/mino/Downloads/_DSC6978.ARW` (Sony ARW) and
`/Users/mino/Downloads/leica raw files/L1005206.DNG` (Leica DNG). Ran each command on the
pre-change `main.rs` (restored with `git checkout`) and again after the change:

- `cargo run -p riffle-cli -- info <ARW>` / `<DNG>`: stdout byte-identical (`cmp` passed).
- `cargo run -p riffle-cli -- focusbox <ARW> out.png`: stdout differs only in the decode-time
  line (98.9ms before, 84.6ms after) and the output path in the `wrote` line; the PNG is
  byte-identical (`cmp before.png after.png` passed).
- `focusbox <DNG>`: fails both before and after with `no FocusLocation in ...` (the DNG carries
  no FocusLocation), so the reworded `no preview` path was not exercised.
- `cargo run -p riffle-cli -- bench <ARW> <DNG>`: the same three stat lines with numbers in the
  same range (preview mean 119.1 -> 130.0ms, full 2171.3 -> 2170.5ms, crop 46.5 -> 46.9ms).

### Removed todo item

The `todo.md` section "App: `riffle-cli info`/`focusbox`/`bench` read whole files instead of the
bounded prefix" was deleted; `git diff --numstat origin/main -- todo.md` showed `0 8 todo.md`,
i.e. the deletion only.
