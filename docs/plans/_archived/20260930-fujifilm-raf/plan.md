<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Fujifilm RAF support

## Purpose

Riffle reads ARW, DNG, NEF and CR3 (`README.md` "RAW formats and cameras").
Before the Reddit announcement (r/sportsphotography and the like) it should
read as many current formats as possible, and Fujifilm RAF has a large share.
This closes the RAF entry left open in Tier 3 of `todo.md` "Core: widen
camera support from public sample RAW files". Sony ARW verification on other
bodies is being done in parallel by another session, so `crates/core/src/arw.rs`
is not touched here, and the CR3 / NEF parsers keep their behavior.

Once done, a folder of `.RAF` files opens like an ARW folder: strip
thumbnails, the embedded JPEG turned by its Orientation, the 1:1 view, the
meta pane's EXIF rows, bursts by capture time (sub-second where recorded),
XMP / `.dop` sidecars, folder RAW counts, watchers and Move Rejected to
Trash. Without an AF point the focus mark and the sharpness cue fall back the
way they do for the Leica M11-P and SIGMA fp L; Step 3 tries to read the
Fujifilm `FocusPixel` so the Fujifilm bodies get the focus mark too.

Precedent: the CR3 / NEF work in
[`../20260929-canon-nikon-raw/plan.md`](../20260929-canon-nikon-raw/plan.md)
and its `learnings.md` (PRs #555, #559, #564, #566, #569, #575, #577).

What the repository already gives us (investigated 2026-09-30):

- `crates/core/src/nef.rs` and `cr3.rs` are the template: a `parse(buf) ->
  Result<Arw>` over `sequence::Tiff` (both byte orders) and the crate-private
  `exif.rs` (`read(tiff, ifd0)` returns `(orientation, Shot)`), strict on any
  structure that runs past `buf` (so a short prefix triggers the reader's
  whole-file retry), lenient on individual Exif fields.
- `crates/core/src/reader.rs::parse_raw` dispatches by extension (`nef`,
  `cr3`, else `arw`); one more arm is all the reader needs. `HEAD_LIMIT` is
  1 MiB; a preview or full JPEG whose bytes lie past the prefix is a ranged
  read, a prefix that fails to parse is a whole-file retry.
- `crates/core/src/scan.rs::is_raw_file` (`["arw", "cr3", "dng", "nef"]`) is
  the only extension gate: the strip listing and tree counts
  (`folders::Media::add`), `commands::raw_only`, `SidecarFormat::matches`
  (`*.RAF.dop`), `rename.rs`, `trash.rs`, `watch.rs` and the CLI all go
  through it. Adding `raf` there lists, counts, sidecars and trashes RAFs.
  The UI's `viewonly.ts` only tests `.jpe?g`, so no UI change is needed;
  `NO_FILES_TEXT` already says "RAW or JPEG files".
- `crates/core/src/jpeg.rs::parse` reads a JPEG's APP1 Exif (via
  `sequence::find_exif_tiff` + `exif::read`) into `(orientation, Shot)`. A
  RAF's Exif lives inside its embedded JPEG, so the RAF parser reuses exactly
  this path on the JPEG slice rather than walking a TIFF of its own.
- The index (`crates/app/src/index.rs`, `EXTRACTOR_VERSION = 7`) keys rows by
  path; `.RAF` files were never listed, so Steps 1 and 2 need no bump. Only
  Step 3 (a new AF-point source for rows Step 1 builds may have written)
  bumps it.

RAF layout, confirmed against ExifTool's `FujiFilm.pm` `ProcessRAF` /
`RAFHeader` (fetched 2026-09-30; all header words big-endian `int32u`):

- `0x00` magic `FUJIFILMCCD-RAW ` + version (e.g. `0201`), `0x14` camera ID,
  `0x1c` model name (32 bytes), `0x3c` firmware version (4 bytes).
- `0x48` / `0x4c` M-RAW header offset / length (multi-image RAFs only, else 0;
  such files carry one embedded JPEG like any other, so nothing special is
  needed beyond reading the JPEG). `0x54` / `0x58` **embedded JPEG offset /
  length** (ExifTool: `unpack('x84NN')`; it also rejects `jpos & 0x8000`).
  `0x5c` / `0x60` RAF directory (the `RAF` tag table: `0x100`
  `RawImageFullSize`, `0x111` `RawImageCroppedSize`, ...), `0x64` / `0x68`
  FujiIFD (a TIFF on some models), `0x78` / `0x7c` and `0x80` / `0x84` a
  second RAF / FujiIFD pair, `0x94` typically where the JPEG starts.
- The embedded JPEG is a normal Exif JPEG (`ProcessJPEG`): IFD0 (`Make`,
  `Model`, `Orientation`), the Exif IFD (`DateTimeOriginal`,
  `SubSecTimeOriginal`, exposure, `LensModel`, ...) and the Fujifilm
  MakerNote (`FUJIFILM` 8 bytes, then a little-endian `int32u` offset of the
  IFD relative to the note start; every offset in the note is relative to the
  note start, little-endian, no TIFF header; `MakerNotes.pm`
  `MakerNoteFujiFilm`). Its size varies by body and is not known to be full
  resolution on any body: measure it (see Step 1 and the Trade-offs).
- Fujifilm MakerNote tags of interest (`FujiFilm::Main`): `0x1021 FocusMode`
  (`int16u`: 0 Auto, 1 Manual, 65535 Movie), `0x1022 AFMode` (0 No, 1 Single
  Point, 256 Zone, 512 Wide/Tracking), `0x1023 FocusPixel` (`int16u[2]`, the
  focus coordinates; ExifTool documents no frame for them), `0x102d
  FocusSettings` (AF area point / zone size bit fields, X-T3 on).

raw.pixls.us (CC0) Fujifilm RAF samples available as of 2026-09-30 (from
`json/getrepository.php?set=all`), current bodies first: X-T5 (4, one 16:9),
X-H2 (3), X-H2S (3), X-S20 (3), X100VI (3), X-T50 (3), X-M5 (4, one 16:9),
X-T30 III (3), X-E5 (3), X-T4 (3), X-T30 II (3), X-E4 (3), X-S10 (3),
X-Pro3 (2), X-T3 (2), X100V (2), GFX100 II (5), GFX100S II (6), GFX100S (6),
GFX100RF (6), GFX50S II (3), GFX 100 (4), GFX 50R / 50S (2 each), plus older
X-T1 / X-T2 / X-T20 / X-T30 / X-E1..E3 / X-Pro1 / X-Pro2 / X100F / X-A
bodies and many FinePix compacts. Most bodies have one compressed and one
uncompressed sample, so the compression mode is covered per body.

**Where the samples go (user instruction, 2026-09-30):** download the RAF
samples to `D:\photos\samples\RAF\` (one folder per format under
`D:\photos\samples\`; CR3 and NEF samples already live in
`D:\photos\samples\CR3\` and `D:\photos\samples\NEF\`), not the OS temp
directory. Anything already downloaded to a temp directory is copied there
too. The samples are still never committed to the repository.

## Steps

- [x] Step 1: RAF parser (`crates/core/src/raf.rs`) wired into the reader and the listing
  - Done when:
    - `raf::parse(buf) -> Result<Arw>` checks the `FUJIFILMCCD-RAW` magic,
      reads the JPEG offset / length at `0x54` / `0x58` (big-endian), errors
      on a zero or out-of-file-range pair, and reads the embedded JPEG's Exif
      through `jpeg::parse`-equivalent code on the slice
      `buf[jpos..min(jpos + jlen, buf.len())]`: the SOI check plus
      `find_exif_tiff` + `exif::read`. A JPEG whose APP1 segment runs past
      `buf` (i.e. a prefix cut inside the Exif) must be an error, not an
      orientation-1 default, so the reader's whole-file retry runs; a JPEG
      fully inside `buf` with no Exif at all is lenient like `jpeg.rs`.
      `preview` and `full` are both the `Embedded { offset: jpos, length:
      jlen }` unless the sample survey below finds a second, larger JPEG (see
      the implementation approach); the `Shot`'s Sony-only fields stay `None`.
    - `scan::is_raw_file` accepts `.raf` (doc comment and its
      `jpeg_and_raw_extensions_are_told_apart_in_any_case` test updated);
      `reader::parse_raw` gains a `raf` arm; the CLI bench error string in
      `crates/cli/src/main.rs` names `.RAF`; `lib.rs` adds `pub mod raf;`.
    - Unit tests with synthetic bytes (a `raf::tests::raf(header fields,
      jpeg)` builder around `jpeg::tests::{W, plain_jpeg, with_exif}`): the
      JPEG offsets and orientation / capture time / sub-second read from the
      embedded JPEG's Exif, a bad magic, a zero JPEG offset, a JPEG range past
      the buffer, a prefix cut inside the APP1 segment is an error, and
      `reader` tests for a `.RAF` whose JPEG runs past `HEAD_LIMIT` (ranged
      read) and one whose Exif segment itself lies past it (whole-file retry).
    - Verified locally (not committed) on raw.pixls.us RAFs saved under
      `D:\photos\samples\RAF\`, aiming at X-T5, X-H2, X-H2S, X-S20, X100VI,
      X-T50, X-M5, X-T30 III, X-E5, X-T4, X-T30 II, X-E4, X-S10, X-Pro3,
      X-T3, X100V, GFX100 II, GFX100S II, GFX100S, GFX100RF, GFX50S II,
      GFX 100 (both the compressed and the uncompressed sample where both
      exist): preview decoded, 1:1 decoded, orientation on a portrait frame if
      any sample is one, meta pane rows (Model, Orientation,
      DateTimeOriginal, SubSecTimeOriginal, LensModel, ExposureTime, FNumber,
      FocalLength, ExposureBiasValue, ISO) cross-checked against an
      independent dump as the NEF / CR3 steps did (exiftool is not installed;
      raw.pixls.us also publishes an `exifdata` text per sample that can serve
      as the reference), capture time with sub-second, and `riffle-cli bench`
      / `scan` / `focusbox` run. Per body, `learnings.md` records the
      embedded JPEG's pixel size and byte length, its offset, whether it (and
      its Exif) fits in the 1 MiB prefix, the decode times, and whether the
      file is M-RAW (`0x48` nonzero). It also records whether the
      `FocusPixel` survey (off-center AF points, portrait, 16:9 samples)
      makes Step 3 viable.
    - `docs/agents/raw-metadata-parsing.md` gains a RAF section (the header
      words, the JPEG-carries-Exif layout, the MakerNote header and
      note-relative offsets, and the prefix / truncation rule) and
      `CLAUDE.md`'s Layout paragraph names `raf.rs`.
    - `mise run ci` passes.
  - Implementation approach:
    - Model on `nef.rs` / `cr3.rs` for structure and error style; the parser
      itself is small (a fixed header, then the JPEG path `jpeg.rs` already
      has). Consider exposing the `(orientation, Shot)` Exif read of a JPEG
      slice from `jpeg.rs` as a `pub(crate)` function (with the
      "truncated APP1 is an error" variant) instead of duplicating it; keep
      `jpeg::parse`'s public behavior and tests unchanged.
    - Only ever slice within `buf`: the prefix normally holds the header and
      the JPEG's Exif (the JPEG starts around `0x94`), while the JPEG bytes
      may run past it; the reader's `embedded_from` already range-reads the
      rest. Do not read the RAF directory / FujiIFD in this step.
    - Measure on the samples whether any body carries more than one JPEG
      (ExifTool reads exactly one; if the survey finds none, `preview ==
      full`). Measure the embedded JPEG's dimensions per body: if it is far
      below the sensor size on every body, the 1:1 view is effectively that
      JPEG (record it; Step 2 states it in the README). If it is large on
      some bodies, the fixed 2/8 thumbnail scale gives big thumbnails (the
      SIGMA fp L / pre-2012 NEF issue in `todo.md`); decide then whether the
      RAF arm should use `thumbnail_jpeg_near` with
      `JPEG_THUMBNAIL_LONG_EDGE` as the JPEG-file path does, and record the
      decision in `learnings.md`.
    - Check `0x48` on every sample so M-RAW files (if any on raw.pixls.us)
      are known to parse; do not implement the M-RAW header.

- [x] Step 2: Docs, the compatibility lists and `todo.md`
  - Done when:
    - `README.md` "RAW formats and cameras" and `README.ja.md` "RAW 形式と
      カメラ" add an `RAF` entry listing only the bodies Step 1 opened on a
      whole sample (preview and 1:1 decoded), in the same PR, plus one
      sentence on what the 1:1 view is on RAF if the survey found the
      embedded JPEG is below full resolution (next to the CR3 HEIF sentence).
    - `docs/cameras.md` gains a row per verified body (AF point `–` unless
      Step 3 lands first, AF frame `–`, face tracking `–`, sub-second `✓` /
      `–` as measured).
    - `docs/raw-formats.md`: the intro lists RAF among the formats, the
      container node / table row mention the RAF header, "How the four
      formats differ" becomes five with an `RAF (Fujifilm)` column (container:
      a fixed `FUJIFILMCCD-RAW` header with offsets, then a JPEG and the
      sensor data; Exif: inside the embedded JPEG; MakerNote: Exif IFD tag
      0x927c of that JPEG, `FUJIFILM` header, note-relative offsets; preview /
      full: the one embedded JPEG, its size per body from Step 1; AF point:
      `FocusPixel` if Step 3 is done, else "not read"), a short "RAF: the Exif
      rides inside the embedded JPEG" subsection like the NEF / CR3 ones, and
      a Fujifilm bullet under "Why the MakerNote varies by body".
    - `docs/usage.md` lines that enumerate the formats ("ARW, CR3, DNG and
      NEF" in the Features intro; the `.dop` sidecar example) include RAF;
      `docs/performance.md` "Nikon NEF and Canon CR3" section gains a RAF
      column or a short RAF paragraph from Step 1's numbers (preview size,
      decode time, prefix fit).
    - `todo.md` "Core: widen camera support from public sample RAW files":
      the Tier 3 checkbox records RAF as done (naming `crates/core/src/raf.rs`)
      and, unless Step 3 is taken, the Tier 2 checkbox names Fujifilm
      `FocusPixel` as the follow-up. If Step 1 deferred anything (large
      embedded JPEG thumbnails, M-RAW), add it as its own `todo.md` section.
    - `mise run ci` passes (lychee checks the links).
  - Implementation approach:
    - Assumes Step 1 is merged. Keep the list honest: a body whose sample was
      only parsed from a prefix, or not decoded, stays unlisted, as the NEF
      step did.
    - `docs/raw-formats.md` is for readers: no pitfalls (those stay in
      `docs/agents/raw-metadata-parsing.md`).

- [x] Step 3 (decided after Step 1's survey, see Trade-offs): AF point from the Fujifilm MakerNote `FocusPixel`
  - Done when:
    - `raf.rs` reads the embedded JPEG's Exif IFD tag 0x927c: after the
      `FUJIFILM` 8-byte header, a little-endian `int32u` IFD offset relative
      to the note start, then a plain IFD whose offsets are also relative to
      the note start (no TIFF header, so `sequence::Tiff::new` cannot be used
      as is; add a crate-private constructor that takes the byte order
      without a header, or walk the IFD with the same entry layout). From it:
      `0x1023 FocusPixel` (`int16u[2]`) into `Shot.focus = FocusLocation {
      sensor_w, sensor_h, x, y }`, and `0x1021 FocusMode` so a manual-focus
      frame (`1`) yields no AF point (do not put Fujifilm values into
      `Shot.focus_mode`, whose documented meaning is Sony's).
    - The frame `FocusPixel` is measured in (`sensor_w` / `sensor_h`) is
      settled on samples and written in `raw-metadata-parsing.md`: candidates
      are the RAF directory's `RawImageCroppedSize` / `RawImageFullSize`
      (`0x5c` directory, tags `0x111` / `0x100`, big-endian, height-first) or
      the embedded JPEG's `PixelXDimension` / `PixelYDimension`, scaled. Also
      settle whether the coordinates are unrotated (portrait sample) and what
      a 16:9 sample (X-T5, X-M5) records.
    - Synthetic-byte tests (a MakerNote builder in `raf::tests`, incl. a
      note running past the prefix being an error, like `nef.rs`'s
      `af_point`), `EXTRACTOR_VERSION` bumped to 9 in
      `crates/app/src/index.rs` (main was already at 8 when this step ran), `docs/cameras.md` rows flip to `✓` for the
      bodies confirmed with the point on the subject, `docs/raw-formats.md`'s
      AF-point row and the Tier 2 `todo.md` checkbox updated.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged. As in the Canon / Nikon step, draw
      the point on the preview with a scratch script and then confirm
      `riffle-cli focusbox` reproduces it; require at least one off-center
      landscape and, if available, one portrait sample before shipping. If
      the frame cannot be pinned down, ship nothing here and leave the note
      in `todo.md` Tier 2.
    - The consumers (`sharpness.rs`, `partial.rs`, the focus mark) treat
      `FocusLocation` as unrotated sensor coordinates and clamp into the
      JPEG; no consumer change is expected.

## Trade-offs and risks

- **Step 3 in or out.** Steps 1–2 are the announcement blocker; the AF point
  is what makes the focus mark and the sharpness cue as good as on Sony /
  Canon / Nikon. `FocusPixel`'s frame is undocumented in ExifTool, so Step 3
  depends on the samples having off-center points (raw.pixls.us scenes are
  often centered). Decided with the user: run Steps 1–2, then decide from the
  survey recorded in `learnings.md`.
- **1:1 view quality on RAF.** A RAF carries one embedded JPEG (ExifTool reads
  one), and on the X bodies it is widely reported to be a reduced-size
  preview (around 1920x1280) rather than full resolution. If Step 1 confirms
  this, Riffle's 1:1 view on RAF is that JPEG, i.e. it cannot show sensor
  pixels the way ARW / NEF / CR3 do. Options: (a) ship as is and say so in
  the README (chosen by this plan; it matches "Riffle never decodes the
  sensor data" in `docs/raw-formats.md`), or (b) treat RAF as
  preview-only and add a `todo.md` item for decoding the raw. If some bodies
  do embed a large JPEG, the opposite issue appears (large thumbnails, slow
  page turns); Step 1 measures and decides on `thumbnail_jpeg_near`.
- **Prefix fit.** The header and the JPEG's Exif sit at the front, so
  `read_metadata` should always succeed on the 1 MiB prefix; the JPEG bytes
  themselves may exceed it and then cost one ranged read per open (same as
  the Z 6 / Z 50 NEFs). If a body's JPEG is regularly over 1 MiB the
  `preview` read is the whole JPEG anyway; `HEAD_LIMIT` is not expected to
  change. Record the numbers.
- **Where the JPEG-Exif reading lives.** Reusing `jpeg.rs`'s Exif path on the
  slice keeps `raf.rs` small but needs a stricter "truncated APP1 is an
  error" variant than `jpeg::parse`'s lenient one. A `pub(crate)` helper in
  `jpeg.rs` is preferred over a copy in `raf.rs`; the implementer decides on
  the exact signature.
- **Compressed vs uncompressed RAF, M-RAW, FinePix.** Compression only changes
  the sensor data, not the header or the JPEG, but both samples per body are
  checked anyway. M-RAW (multi-exposure) files and old FinePix bodies (which
  may lack `0x54`-style headers on very old versions) are not targets; if a
  header word looks off (`jpos & 0x8000`, zero offset) the file is an error,
  not a guess. Only bodies opened on a real sample get listed.
- **Sony ARW / CR3 / NEF are untouched.** Any helper added for RAF (e.g. a
  header-less `Tiff` constructor in `sequence.rs`) must leave the existing
  parsers' output identical; no `EXTRACTOR_VERSION` bump in Steps 1–2.
- **No committed fixtures.** CC0 would allow one, but the repository rule is
  synthetic bytes; real-file checks live in `learnings.md` only, and the
  samples stay in `D:\photos\samples\RAF\`.

## Progress

- (2026-09-30) Step 1 complete
- (2026-09-30) Step 2 complete
- (2026-09-30) Step 3 complete
