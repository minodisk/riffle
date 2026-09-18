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

## Step 3

- Plan change (user intervention, 2026-09-18): after Step 2 the user asked
  for the meta pane to show an aperture and the focus distance on the M11-P
  files. A new Step 3 was inserted and the old Steps 3/4 became 4/5.
- `ApertureValue` (0x9202) on the M11-P is RATIONAL, e.g. 297/100 on
  `L1005200.DNG`; `2^(2.97/2)` = 2.799 -> "f/2.8". Across the 31 samples the
  rounded values match exiftool's `ApertureValue` exactly (2.0 ... 11.0,
  including 9.5 / 4.8 / 3.4 / 6.8 / 5.6).
- Leica MakerNote layout (exiftool `MakerNoteLeica9`): `LEICA\0` + `02 00`,
  then a little-endian IFD at note offset 8 (32 entries). `FocusDistance` is
  tag **0x0304, LONG count 1, value inline** (921 on `L1005200.DNG`, 921
  read by our parser too). exiftool prints it as a bare integer; the unit is
  taken as **millimetres** (range 921-7529 over the samples, consistent with
  a 50mm Summicron's 0.7 m close focus). Shown as `0.92 m`.
- The Step 1 Sony gate only skips a non-`SONY` note when `Make` is present
  and non-Sony, so a synthetic Leica note in a test needs a `Make` entry;
  without it the Sony parse reads `LE` as an entry count and errors.
- The Step 1 test `a_non_sony_maker_note_is_skipped` used `0xffff` as the
  entry count after the `LEICA` header; with the Leica parse that is now an
  out-of-range IFD error, so the fixture became a valid empty Leica IFD.
- The meta pane reads `reader::read_metadata` directly, not the index, so no
  schema change was needed.

## Step 4: DNG benchmark

- The sample folder holds 32 DNGs (the plan said 31); all 32 were measured.
- `bench` now crops via `partial::decode_focus_crop` (RGBA, centre fallback)
  instead of `decode_crop` (RGB, skipped without focus). Crop size is still
  512x512, but ARW numbers now include RGBA output; the α7 V table in README
  was not re-measured.
- The bench's preview label no longer says "1616px" since DNG previews are
  2112 wide.
- Centre crop on the 9504x6320 JPEG: ~16.5ms median, well under 50ms, despite
  skipping ~2900 rows. Thumbnails average 30.2KB versus ~19KB on ARW.

## Step 5

- README had no dedicated meta pane section; the aperture `(est.)` and Leica
  focus distance rows are described in the Status paragraph, and the DNG
  confirmations get their own paragraph under "What has been confirmed".

## Deferred issues (todo candidates)

### Write a guide for RAW metadata parsing (`docs/agents/raw-metadata-parsing.md`)

- **Change**: create `docs/agents/raw-metadata-parsing.md` covering the
  MakerNote/TIFF parsing in `crates/core/src/{arw,reader}.rs`. Read it when
  touching the Sony/Leica MakerNote gate, adding another maker's MakerNote
  parser, or writing a synthetic-TIFF test fixture.
- **Points**: the Sony gate skips only when the note lacks `SONY` *and* `Make`
  is present and non-Sony, so non-Sony fixtures must set `Make`; `SubIFDs` with
  `count == 1` stores the IFD offset inline; the Leica note is `LEICA\0` +
  `02 00` then an IFD at note offset 8, `FocusDistance` = tag 0x0304 LONG in mm;
  `ApertureValue` (APEX) → `2^(AV/2)`; a "non-Sony note is skipped" test needs a
  valid IFD in the fixture, not a `0xffff` sentinel count.
- **Rationale**: these are non-obvious and cost time in Steps 1 and 3 (see the
  Step 1 and Step 3 sections above).
- **Done when**: the guide exists with the points above and links this archived
  `learnings.md` for the measurements instead of duplicating them.
