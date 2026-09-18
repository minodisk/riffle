# Learnings

## Step 1

- The MakerNote gate in the plan ("parse only when the note starts with `SONY`
  or `Make` starts with `SONY`") would break the existing untouched tests,
  whose synthetic files have a headerless Sony5-style note and no `Make`. The
  gate was implemented as "skip when the note lacks the `SONY` header and a
  `Make` is present that does not start with `SONY`", so a missing `Make` still
  gets the Sony parse. The non-Sony test therefore sets `Make` to Leica.
- A single `SubIFDs` entry (`count == 1`) holds the IFD offset inline, not an
  offset to an array; the synthetic-TIFF test helper for SubIFDs needs at least
  two SubIFDs to exercise the array path.
- Whole-file fallback check: with a temporary (uncommitted) `eprintln!` in
  `reader::read_embedded`, `riffle-cli crop` (read_full) on every sample DNG and
  `riffle-cli scan` (read_preview) over `.ARW`-named symlinks to the DNGs never
  hit the fallback. The scan saw 32 files, 0 errors, thumbnails ~30KB mean.
- `riffle-cli info` on `_DSC6978.ARW` still prints preview 204962/337198, full
  544768/5761112, focus 7008 4672 3613 1732.

## Step 2

- `riffle_core::scan::is_raw_file` is the one shared extension predicate (ARW/DNG, case-insensitive); the app's `list_arw_in` and `riffle-cli scan` both use it.
- The sample folder holds 32 DNGs, not 31 as the plan says. `riffle-cli scan` on it (release): 32 files, 0 errors, 0.09s, mean 30.5ms/file, thumbnails 30231 bytes mean (967419 total).
- The GUI checks of Step 2 need the user's confirmation; the step stays unchecked until then.
