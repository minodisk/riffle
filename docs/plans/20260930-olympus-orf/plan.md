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

# OM System / Olympus ORF support

## Purpose

Riffle reads ARW, DNG, NEF and CR3 (`README.md` "RAW formats and cameras").
Before the Reddit announcement (r/sportsphotography and the like) it should
read as many recent formats as it can; OM-1-series bodies are common among
bird and sports shooters, and none of their `.ORF` files opens today. This
continues tier 3 of the `todo.md` item "Core: widen camera support from
public sample RAW files" after CR3 / NEF
(`../_archived/20260929-canon-nikon-raw/plan.md`); Fujifilm RAF and the
other Sony ARW bodies are handled in parallel sessions.

Once done, a folder of `.ORF` files opens like an ARW folder: strip
thumbnails, the embedded preview turned by its Orientation, the 1:1 view,
the meta pane's EXIF rows, bursts, XMP / `.dop` sidecars, folder RAW counts,
watchers and Move Rejected to Trash. Without an AF point the focus mark and
the sharpness cue fall back as on the Leica M11-P (`docs/cameras.md`).

Verification is on raw.pixls.us (CC0) samples. **Save every downloaded
sample under `D:\photos\samples\ORF\`** (the user's per-format sample
library, next to `D:\photos\samples\CR3\` and `D:\photos\samples\NEF\`), not
the OS temp directory, and never commit them; tests use synthetic bytes as
`nef.rs` does. exiftool is not installed locally, so EXIF rows are
cross-checked against an independent Python TIFF dump as in the CR3 / NEF
plan.

What was found at planning time (2026-09-30, on the raw.pixls.us OM-1 and
E-M1 Mark III samples, now at `D:\photos\samples\ORF\om1.orf` and
`D:\photos\samples\ORF\em1iii.orf`, plus exiftool's `Olympus.pm` /
`MakerNotes.pm`):

- An ORF is a little-endian TIFF whose header is `IIRO` (magic 0x4f52, IFD0
  at 8); `MMOR` (big-endian) and `IIRS` exist on older bodies.
  `crates/core/src/sequence.rs` `Tiff::new` rejects any magic but 42, so it
  cannot open an ORF as is. Everything else about the walker (`u16` /
  `u32` / `bytes` / `ifd_entries`, `exif::read_ifd0` / `read_exif_ifd`) fits.
- The Olympus MakerNote (Exif IFD 0x927c) has two headers on the candidate
  bodies: `OLYMPUS\0` + `II`/`MM` + u16 version, IFD at note+12 (Olympus
  bodies; E-M1 Mark III writes version `0300`), and `OM SYSTEM\0\0\0` +
  `II`/`MM` + version, IFD at note+16 (OM-1, version `0400`). Every offset
  in the note, including the sub-IFD pointers and `PreviewImageStart`, is
  relative to the **note start** (exiftool `Base => '$start - 12'` /
  `'$start - 16'`), not to the TIFF header. The note's byte order is its own
  (`II` on both samples). There is no TIFF header inside the note, so the
  walker needs a constructor that takes the byte order explicitly.
- The note's IFD holds 0x2010 Equipment, 0x2020 CameraSettings, 0x2040
  ImageProcessing, 0x2050 FocusInfo as type 13 (IFD) count 1 entries with a
  note-relative offset inline. All four sub-IFDs end within ~11 KB of the
  file. exiftool also documents an "old-style" form (type 7 `undef` with
  the IFD inline in the value area) on old bodies.
- The preview is CameraSettings 0x0100 `PreviewImageValid` (1), 0x0101
  `PreviewImageStart` (note-relative), 0x0102 `PreviewImageLength`:
  3200x2400 on both samples (OM-1: 962 KB ending at file offset 1,014,705;
  E-M1 Mark III: 1.24 MB ending at 1,295,242). It is the only JPEG in the
  file: IFD0 has no 0x0201 and the strips (0x0111) are the raw data. So
  `full` is the same 3200x2400 JPEG and the 1:1 view is limited to it.
- The MakerNote is 1.5-1.8 MB (the preview is inside it), so the note runs
  past the 1 MiB `HEAD_LIMIT` on every body. The parser must therefore not
  demand the whole note in the buffer (unlike `nef::af_point`); it needs
  only the sub-IFDs, which sit at the note's front, and returns the preview
  as an `Embedded` for the reader's ranged read.
- Neither sample writes `SubSecTimeOriginal` (0x9291), so bursts group by
  whole seconds; both are Orientation 1, so a portrait sample must be
  sought among the other models.
- AF: CameraSettings 0x0304 `AFAreas` (int32u[64], 0-255 coordinates) and
  0x0305 `AFPointSelected` (rational64s[5], percentages) are all zero on
  both samples. The OM-1 writes 0x030a `AFTargetInfo` (int16u[10]:
  `AFFrameSize` w/h, `AFFocusArea` x/y/w/h, `AFSelectedArea` x/y/w/h; the
  sample reads 640x480, 0,0,0,0, 312,231,16,18, i.e. a centered single
  point) and 0x030b `SubjectDetectInfo` (int16u[11], status 772 = no
  subject). A body without a readable AF point already works through the
  face / tile fallback.
- raw.pixls.us is a JS table, but its mirror lists directories at
  `https://raw.pixls.us/data/<Make>/<Model>/` (curl works; ranged requests
  do not). Makes / models there: `OM System/` OM-1, OM-1 Mark II, OM-5,
  TG-7; `OM Digital Solutions/` OM-3, OM-5MarkII; `Olympus/` E-M1MarkIII,
  E-M1X, E-M1MarkII, E-M5 Mark III, E-M10 Mark IV, E-P7, PEN-F, E-M1,
  E-M5 Mark II, E-M10 Mark II / III, and older bodies.
- `crates/core/src/reader.rs` `parse_raw` dispatches by extension and
  `crates/core/src/scan.rs` `is_raw_file` is the only extension list the app
  uses (`folders`, `commands::raw_only`, `sidecar::SidecarFormat::matches`,
  `rename.rs`, `trash.rs`, `watch.rs`, the CLI). The RAF session adds arms
  to the same two places; whichever merges later resolves the conflict.
- `scan.rs` thumbnails a RAW preview with `thumbnail_jpeg` at a fixed 2/8
  scale, so a 3200x2400 preview yields 800x600 thumbnails, the issue
  `todo.md` already records for the SIGMA fp L.

## Steps

- [x] Step 1: Let `sequence::Tiff` open an ORF header and a header-less MakerNote
  - Done when:
    - `crates/core/src/sequence.rs` gains crate-private constructors next
      to `Tiff::new`: one that opens a TIFF-shaped buffer whose magic is
      not 42 (either `new_with_magic(buf, base, end, magic: &[u16])` or a
      variant that skips the magic check and leaves the check to the
      caller; the ORF magics are 0x4f52 `RO` and 0x5352 `RS`), and one that
      builds a walker over a segment with an explicit byte order and no
      header (`with_order(buf, base, end, little_endian: bool)`) for the
      Olympus MakerNote, whose offsets are relative to the note start but
      which carries no TIFF header.
    - `Tiff::new`'s behavior is unchanged (the JPEG sequencer, `jpeg.rs`,
      `nef.rs` and `cr3.rs` tests pass untouched); ARW / DNG output is
      unchanged and `EXTRACTOR_VERSION` is not bumped.
    - Unit tests in `sequence.rs` (or `exif.rs`) cover the new constructors:
      the ORF magic accepted, 42 still accepted, another magic rejected, and
      a header-less segment read in both byte orders.
    - `mise run ci` passes.
  - Implementation approach:
    - Pure addition. Keep `Tiff`'s fields private; the new constructors live
      in the same `impl`. Do not touch `arw.rs`.

- [x] Step 2: ORF parser (`crates/core/src/orf.rs`) wired into the reader and the listing
  - Done when:
    - `orf::parse(buf) -> Result<Arw>` reads the ORF TIFF (both byte orders
      through Step 1's constructor): IFD0's `Make`, `Model`, `Orientation`
      and the Exif IFD through `exif::read_ifd0` / `read_exif_ifd`
      (`DateTimeOriginal`, `SubSecTimeOriginal` when present, `LensModel`,
      `ExposureTime`, `FNumber`, `ISO`, ...), with the NEF's rule that a
      prefix cut inside IFD0 or the Exif IFD is an error (read the next-IFD
      link) so the reader retries with the whole file.
    - The Olympus MakerNote is located from Exif IFD 0x927c; both headers
      (`OLYMPUS\0` + order + version, IFD at +12; `OM SYSTEM\0\0\0` + order +
      version, IFD at +16) are recognized, the byte order is taken from the
      `II` / `MM` in the header, and a `Tiff` is built with Step 1's
      header-less constructor with `base` = note start and `end` =
      `min(note start + count, buf.len())`, since the note is longer than
      the prefix on every body. A note with neither header gives no preview
      and no error.
    - `preview` = CameraSettings (main tag 0x2020, type 13 / 4 pointer,
      note-relative) 0x0101 / 0x0102 as an absolute `Embedded`, taken only
      when 0x0100 `PreviewImageValid` is 1 (or absent) and the length is
      nonzero; a preview offset that runs past the declared note or
      `buf.len()` is fine (the reader range-reads it), but a CameraSettings
      IFD cut by the prefix is an error. `full` = `preview` (no larger JPEG
      exists); document why in the module doc.
    - `scan::is_raw_file` accepts `.orf`; `reader::parse_raw` dispatches to
      `orf::parse` for `.orf`, keeping the bounded-prefix, ranged-read and
      whole-file-retry behavior. `reader.rs`'s module doc names ORF.
    - Unit tests with synthetic bytes (a writer built on
      `jpeg::tests::W`, which needs the ORF magic written in place of 42, so
      either a `W` option or a post-patch of bytes 2-3): both file byte
      orders, both MakerNote headers, a note byte order differing from the
      file's, the preview offset resolved relative to the note start, a
      `PreviewImageValid` 0 note giving no preview, a note cut off after the
      CameraSettings IFD still giving the preview, a prefix cut inside the
      CameraSettings IFD being an error, orientation / capture time /
      sub-second / ISO, and `reader` tests for an `.orf` whose preview lies
      past `HEAD_LIMIT` (the E-M1 Mark III case) and whose CameraSettings
      IFD lies past it (whole-file retry).
    - Verified locally (not committed) on raw.pixls.us ORFs of the
      candidate bodies available there, each saved under
      `D:\photos\samples\ORF\`: OM-1, OM-1 Mark II, OM-5, OM-3,
      OM-5 Mark II, E-M1 Mark III, E-M1X, E-M1 Mark II, E-M5 Mark III,
      E-M10 Mark IV, PEN E-P7 (and PEN-F if cheap). For each: preview
      decoded, 1:1 decoded, orientation on a portrait frame where one
      exists, meta pane rows against the Python dump, capture time and
      whether sub-seconds are recorded, the preview size, the offset where
      the preview ends versus `HEAD_LIMIT`, the MakerNote header and
      version, and whether the sub-IFDs are pointer (type 13 / 4) or
      old-style inline (type 7). Run `riffle-cli bench` / `scan` /
      `focusbox` and `read_metadata` on a 1 MiB truncated copy. Record the
      table in `learnings.md`; the compatibility lists in Step 3 come from
      it.
    - `docs/agents/raw-metadata-parsing.md` gains an ORF section (the
      `IIRO` magic, the two note headers, note-relative offsets, the note
      longer than the prefix, the single-JPEG layout) and `CLAUDE.md`'s
      Layout paragraph names `orf.rs`.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged. Model on `nef.rs`: same `Arw` output, same
      leniency for individual Exif fields, same "inline value first, then
      range check" order from `raw-metadata-parsing.md`.
    - Support old-style (type 7) sub-IFDs only if a candidate-body sample
      uses them; otherwise a body whose note is old-style stays unlisted
      and the gap goes to `learnings.md`.
    - Confirm on the OM-1 Mark II / OM-3 / OM-5 Mark II samples that the
      preview is still 3200x2400 and whether any body writes a second,
      larger JPEG (search the file for SOI + SOF as the planning dump did);
      if one exists, take it as `full` and say so.
    - The RAF session edits `parse_raw` and `is_raw_file` too; expect a
      trivial rebase.

- [ ] Step 3: App strings, CLI text, docs and the compatibility lists
  - Done when:
    - The CLI bench error in `crates/cli/src/main.rs` names `.ORF`
      (`crates/app/ui/src/empty.ts` `NO_FILES_TEXT` already says "RAW or
      JPEG files" and needs no change).
    - `README.md` "RAW formats and cameras" and `README.ja.md` "RAW 形式と
      カメラ" gain an `ORF` entry listing only the bodies Step 2 opened on a
      whole sample (checked), with a sentence that the 1:1 view of an ORF
      is the 3200x2400 embedded JPEG; both files change in the same PR.
    - `docs/cameras.md` gains a row per verified body (AF point `–`, AF
      frame `–`, face tracking `–`, sub-second as measured, expected `–`)
      and a sentence that Olympus bodies record no sub-second time so
      bursts group by whole seconds. `docs/usage.md` lines that enumerate
      the formats ("ARW, CR3, DNG and NEF") add ORF.
    - `docs/raw-formats.md` gains an ORF column in "How the four formats
      differ" (renamed to cover five: TIFF with a non-standard `IIRO`
      header, Exif in IFD0 / Exif IFD, MakerNote in 0x927c with note-
      relative offsets, the preview inside the MakerNote's CameraSettings,
      no full-size JPEG) and a short "ORF: the preview lives in the
      MakerNote" subsection next to the NEF and CR3 ones; the layer table's
      MakerNote row mentions ORF.
    - `docs/performance.md`'s NEF / CR3 section gains an ORF column (preview
      size, decode time, metadata in the prefix, preview in the prefix per
      body) from Step 2's numbers.
    - `todo.md`: the Tier 3 checkbox records ORF as done; the Tier 2
      checkbox names Olympus `AFTargetInfo` / `AFPointSelected` as the
      follow-up unless Step 4 is taken; the "Core: preview tier falls back
      ..." (SIGMA fp L 2/8 thumbnail) item notes that ORF's 3200x2400
      preview has the same 800x600 thumbnail cost.
    - `mise run ci` passes (lychee checks the links).
  - Implementation approach:
    - Assumes Step 2 is merged. Keep the lists honest: a body not opened on
      a real sample stays unlisted; list the Model string as the camera
      writes it, normalized like the existing rows ("OM System OM-1",
      "Olympus E-M1 Mark III").

- [ ] Step 4 (optional, see Trade-offs): AF point from the Olympus MakerNote
  - Done when:
    - `orf::parse` fills `Shot.focus` (and `Shot.focus_frame` when a size is
      given) from CameraSettings 0x030a `AFTargetInfo` on the OM bodies
      (`AFFrameSize` as `sensor_w` / `sensor_h`, the center of
      `AFFocusArea` when nonzero, else of `AFSelectedArea`, as `x` / `y`,
      the area as the frame), falling back to 0x0305 `AFPointSelected`
      (percent pairs scaled into a fixed frame such as 1000x1000 or the
      image size) on the Olympus bodies that write it. All-zero values give
      `None`, so sharpness keeps its face / tile fallback.
    - Synthetic-byte tests for both tags, including a portrait frame, and
      `EXTRACTOR_VERSION` bumped in `crates/app/src/index.rs` (rows
      written by the Step 2 build lack the focus). `docs/cameras.md` rows
      flip to `✓` only for bodies whose sample point landed on the subject;
      the Tier 2 `todo.md` checkbox is updated; `raw-metadata-parsing.md`
      records the coordinate frame and origin as measured.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 3 is merged, and is decided from Step 2's sample survey.
      The coordinate frame is the risk: `AFFrameSize` 640x480 is a 4:3
      frame matching the sensor, but whether the origin is top-left and
      whether the area is in that frame must be confirmed on at least one
      off-center landscape and one portrait sample (`riffle-cli focusbox`,
      and the Python dump drawing on the preview, as in the CR3 / NEF
      Step 5). If neither can be confirmed on the CC0 samples, ship nothing
      and leave it in `todo.md` with what was learned.
    - Consumers (`sharpness.rs`, `partial.rs`, the focus mark) treat
      `FocusLocation` as unrotated sensor coordinates and scale by
      `sensor_w`, so a 640x480 frame works like the SIGMA BF's 1000x667
      grid; no consumer change is expected.

## Trade-offs and risks

- **Thumbnail size of a 3200x2400 preview.** `scan.rs` scales every RAW
  preview at a fixed 2/8, giving 800x600 ORF thumbnails (about four times
  the bytes of the 405x270 ARW ones), the SIGMA fp L problem already in
  `todo.md`. Decided (user, 2026-09-30): option A, leave `scan.rs` alone in
  this plan and add ORF to that todo item (Step 3). Option B (switch RAW
  previews wider than ~2000 px to `thumbnail_jpeg_near`, with a
  thumbnail-cache invalidation) stays in `todo.md`.
- **The 1:1 view is only 3200x2400.** No ORF sample carries a larger JPEG.
  The README sentence in Step 3 says so rather than hiding it; decoding the
  raw data is out of scope.
- **Step 4 in or out.** Decided (user, 2026-09-30): run Steps 1-3, then
  decide Step 4 from the sample survey. Steps 1-3 are the announcement
  blocker; Step 4 depends on off-center samples the CC0 set may not offer.
- **Old-style MakerNote sub-IFDs.** exiftool warns that old bodies write
  the sub-IFDs inline with a broken size. The candidate list is 2016+ so
  the pointer form is expected; Step 2 supports the old form only if a
  candidate sample needs it.
- **`Tiff::new` change vs a new constructor.** Loosening `Tiff::new` to
  accept any magic would silently open corrupt JPEG Exif segments in the
  sequencer; a separate constructor keeps that check. Cost: one small
  extra PR (Step 1).
- **No sub-second time.** Olympus bodies record none, so a 50 fps OM-1
  burst is grouped by whole seconds, which merges consecutive bursts taken
  within the same second. Recorded in `docs/cameras.md`; no workaround is
  planned (the MakerNote has no known sub-second tag).
- **Concurrent RAF work.** Both plans add an arm to `parse_raw` and an
  extension to `is_raw_file`, and both add a README entry; the later PR
  rebases. Do not touch `arw.rs`, `cr3.rs`, `nef.rs` or the RAF parser.
- **Compatibility claims.** Only bodies opened on a whole sample get
  listed; bodies with only a truncated check or an old-style note stay
  off the list, as the CR3 / NEF plan did.

## Progress

- (2026-09-30) Step 1 complete
