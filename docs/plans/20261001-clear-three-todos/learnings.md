# Learnings

## Step 1: Trim trailing spaces from the Exif `Make` / `Model`

- The trim lives in `exif::read_ifd0` for `Make` and `Model` only, not in
  `exif::ascii`, so `DateTimeOriginal`, `SubSecTimeOriginal` and
  `LensModel` keep their bytes.
- Covered: every parser that calls `exif::read_ifd0`, i.e. `jpeg.rs` (plain
  JPEG, and RAF through `jpeg::read_exif`), `nef.rs`, `cr3.rs` and `orf.rs`.
  The `orf.rs` local trim ("Olympus pads Make and Model with spaces") was
  removed as redundant; its test still asserts `OM Digital Solutions` /
  `OM-1` from padded input.
- Not covered: `arw.rs` (ARW and DNG) has its own `ascii`, which trims NULs
  only. Left alone since no padded Sony / Leica / Sigma `Model` is known.

## Deferred issues (todo candidates)

- Pending manual check (Step 1): open a folder of old FinePix RAFs (e.g.
  `FinePix E550` / `FinePix S5000` from `D:\photos\samples\RAF\`) in the app
  on any platform and confirm the meta pane shows `Model` without trailing
  spaces. Already carried in `todo.md` under `### Core: old FinePix bodies
  show Exif `Model` with trailing spaces`. The step's checkbox was ticked on
  the automated criteria (unit tests in `exif.rs` and `orf.rs`).
