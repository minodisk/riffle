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

# Canon CR3 and Nikon NEF support

## Purpose

Riffle reads only Sony ARW and DNG (`README.md` "RAW formats and cameras").
It is about to be promoted on Reddit (r/sportsphotography and the like),
where most shooters use Canon and Nikon bodies, so most of that audience
cannot open a single folder today. This is tier 3 of the `todo.md` item
"Core: widen camera support from public sample RAW files" for the two
containers that matter most: CR3 (ISOBMFF) and NEF (big-endian TIFF).

Once done, a folder of `.CR3` / `.NEF` files opens like an ARW folder:
strip thumbnails, the embedded preview turned by its Orientation, the 1:1
view on the larger embedded JPEG, the meta pane's EXIF rows, bursts grouped
by capture time (sub-second where the body records it), XMP / `.dop`
sidecars, folder RAW counts, watchers and Move Rejected to Trash. Without an
AF point the focus mark and the sharpness cue fall back the way they already
do for the Leica M11-P and SIGMA fp L (`docs/cameras.md`).

The user owns no Canon or Nikon body: verification is on public samples
(raw.pixls.us, CC0) and, locally only, review-site galleries; none of those
files is committed. Tests use synthetic bytes as `crates/core/src/arw.rs`
does.

What the repository already gives us (investigated 2026-09-29):

- `crates/core/src/arw.rs` walks little-endian TIFF only (`parse` bails on
  anything but `II`) with its own `read_ifd` / `ascii` / `rational` helpers.
  Its `Arw { preview, full, orientation, shot }` and `Shot` are the shape
  the scan, the index, the meta pane and the CLI consume; `jpeg.rs` already
  fills the same `Arw` for a plain JPEG, so no new output type is needed.
- `crates/core/src/sequence.rs` has a byte-order-aware walker,
  `Tiff::new(buf, base, end)` with `u16` / `u32` / `bytes` / `ifd_entries`
  and `Entry { tag, typ, count, value_field }`, and `jpeg.rs::exif` reads
  IFD0 + ExifIFD through it into `(orientation, Shot)`. A NEF is a big-endian
  TIFF and a CR3's `CMT1` / `CMT2` boxes are TIFF blobs, so this walker, not
  `arw.rs`, is the base for both parsers.
- `crates/core/src/reader.rs` is the only place that picks bytes: a 1 MiB
  prefix (`HEAD_LIMIT`), `arw::parse` on it, a ranged read of the
  `Embedded { offset, length }` the parser returned, and a whole-file retry
  when the prefix fails to parse. `read_preview` / `read_full` /
  `read_metadata` dispatch on `scan::is_jpeg_file`; adding a container means
  adding a dispatch there and nothing in the app.
- Every extension check in the app goes through
  `riffle_core::scan::is_raw_file` (`folders::Media::add` for the strip and
  the tree counts, `commands::raw_only`, `sidecar::SidecarFormat::matches`
  for `*.<raw>.dop`, `rename.rs`, `trash.rs`, `watch.rs`, the CLI). Adding
  `.cr3` / `.nef` there lists, counts, sidecars and trashes them. The UI's
  `viewonly.ts` tests only for `.jpe?g`, so it needs no change; the
  user-facing strings are `crates/app/ui/src/empty.ts` (`NO_FILES_TEXT`),
  the CLI's bench error text, `rename.rs`'s "not a RAW file name" error,
  `commands.rs`'s `list_arw_in` doc comment, and the docs. The two error /
  doc strings already use generic "RAW" wording (fixed in Round 1 review of
  Step 2), so only `NO_FILES_TEXT`, the CLI bench error and the docs remain
  for Step 4.
- The index (`crates/app/src/index.rs`, `EXTRACTOR_VERSION = 6`) keys rows
  by path; `.CR3` / `.NEF` files were never listed, so they have no rows and
  no bump is needed as long as ARW / DNG / JPEG output is unchanged.

## Steps

- [x] Step 1: Share the Exif `Shot` reader between `jpeg.rs` and the new parsers
  - Done when:
    - A crate-internal module (e.g. `crates/core/src/exif.rs`) exposes the
      IFD0 + ExifIFD reading `jpeg.rs::exif` does today over
      `sequence::Tiff`: given a `Tiff` and its IFD0 offset (or given IFD0
      entries and ExifIFD entries separately, since a CR3 keeps them in two
      TIFF blobs), it returns the orientation and a `Shot` with `make`,
      `model`, `capture_time`, `subsec`, `lens_model`, `exposure_time`,
      `f_number`, `estimated_f_number` (cleared when `f_number` is present),
      `focal_length`, `exposure_bias`, `iso`, and the `ascii` / `rational` /
      `integer` helpers. `jpeg.rs` calls it and its tests are unchanged.
    - `arw.rs` is not touched (ARW / DNG output identical; no
      `EXTRACTOR_VERSION` bump).
    - `mise run ci` passes.
  - Implementation approach:
    - Pure move plus a split into "IFD0 part" and "ExifIFD part" so a CR3
      can feed `CMT1`'s IFD0 and `CMT2`'s IFD0 (which *is* the ExifIFD) in.
      Keep `jpeg.rs`'s leniency (an unreadable entry leaves its own field
      `None`, no error) since NEF and CR3 want the same.
    - `jpeg.rs::tests` already has a `W(little_endian)` TIFF writer,
      `plain_jpeg` and `with_exif`; make what the next steps' synthetic
      fixtures need `pub(crate)` here rather than duplicating them.
    - Add the new module to `docs/agents/raw-metadata-parsing.md` in the
      step that first uses it for a RAW (Step 2), not here.

- [x] Step 2: NEF parser (`crates/core/src/nef.rs`) wired into the reader and the listing
  - Done when:
    - `nef::parse(buf) -> Result<Arw>` reads a big-endian (or little-endian)
      TIFF: IFD0's `Make`, `Model`, `Orientation`, the ExifIFD through
      Step 1's reader (`DateTimeOriginal`, `SubSecTimeOriginal`, ...), and
      the embedded JPEGs: `full` = the JpgFromRaw in the SubIFD
      (`SubIFDs` 0x014a; the one with `NewSubFileType = 1`, `Compression =
      6`, `JPEGInterchangeFormat` 0x0201 / `JPEGInterchangeFormatLength`
      0x0202), `preview` = the Nikon MakerNote's `PreviewIFD` (tag 0x0011)
      JPEG when it is at least `PREVIEW_MIN_WIDTH` (1600) wide, else the
      full JPEG (the DNG rule in `arw::parse`). IFD0's own 0x0201 / 0x0202
      is Nikon's 160x120 thumbnail and must not be taken as the preview.
    - `scan::is_raw_file` accepts `.nef`; `reader::read_preview`,
      `read_full` and `read_metadata` dispatch to `nef::parse` for a `.nef`
      (by extension, like `is_jpeg_file`), keeping the bounded-prefix,
      ranged-read and whole-file-retry behavior.
    - Unit tests with synthetic bytes (a big-endian TIFF via Step 1's
      writer): the SubIFD full JPEG, the MakerNote preview (offsets
      relative to the note's own TIFF header), the small-preview fallback
      to the full JPEG, IFD0's thumbnail ignored, the ISO fallback,
      orientation, sub-second, an out-of-range offset is an error, and
      `reader` tests for a `.nef` whose full JPEG lies past `HEAD_LIMIT`.
    - Verified locally (not committed) on raw.pixls.us NEFs of the Z bodies
      available there, aiming at Z 9, Z 8, Z 6III, Z 6II, Z 7II, Z f, Z 50
      (and one older DSLR such as D850 / D500 if handy): preview, 1:1,
      orientation on a portrait frame, meta pane rows against `exiftool`,
      capture time with sub-second. Record which bodies passed and each
      file's preview / full sizes in `learnings.md`; the README list is
      updated in Step 4 from that record.
    - `docs/agents/raw-metadata-parsing.md` gains a NEF section (the
      MakerNote header and relative offsets, the thumbnail-vs-preview
      pitfall) and `CLAUDE.md`'s Layout paragraph names `nef.rs` (and
      Step 1's module).
    - `mise run ci` passes.
  - Implementation approach:
    - The Nikon MakerNote (0x927c in the ExifIFD) is `Nikon\0` + 2 bytes
      version + 2 bytes, then a full TIFF header at note offset 10; every
      offset inside the note, including `PreviewIFD` and its 0x0201, is
      relative to that inner header. Build a second `Tiff::new(buf,
      note + 10, note_end)` for it, then convert the preview's offset to
      an absolute `Embedded`. Follow `raw-metadata-parsing.md`'s
      "inline value first, then range check" order for the note's size.
    - `ISO`: read ExifIFD 0x8827 first; only if the Z samples lack it,
      fall back to the MakerNote `ISO` tag 0x0002 (`int16u[2]`, the second
      value). Do not add fields the meta pane does not show.
    - The `Shot`'s Sony-only fields stay `None`; `sharpness::trusted_focus`
      and the face fallback already handle a body without an AF point.
    - `read_metadata` must succeed on the 1 MiB prefix for a typical NEF
      (IFD0, SubIFDs, ExifIFD and the MakerNote sit at the front); measure
      with `riffle-cli bench` / `scan` on the samples and note it.
    - Reader dispatch: a `fn parse_raw(path, buf)` (or an enum) in
      `reader.rs` chosen by extension, so Step 3 adds one arm.
  - Changed during implementation (measured on 24 raw.pixls.us bodies,
    see `learnings.md`): the MakerNote `PreviewIFD` JPEG is 640x424 on
    every body (570x375 on older ones), so it would never clear
    `PREVIEW_MIN_WIDTH`. Every body since about the D800 instead carries a
    second JPEG SubIFD of 1620x1080 after the JpgFromRaw one. `preview` is
    therefore that later JPEG SubIFD (else `full`) and the MakerNote is not
    read; its preview test and the ISO fallback were dropped (every sample
    has ExifIFD 0x8827). Most samples are little-endian; older bodies are
    big-endian, and both are tested.

- [x] Step 3: CR3 parser (`crates/core/src/cr3.rs`) wired into the reader and the listing
  - Done when:
    - `cr3::parse(buf) -> Result<Arw>` walks the ISOBMFF top-level boxes
      (`ftyp` with brand `crx `, `moov`, the `uuid` boxes, stopping at
      `mdat`; 64-bit `largesize` when `size == 1`) and reads:
      - inside `moov`, the Canon `uuid` box
        (`85c0b687-820f-11e0-8111-f4ce462b6a48`): `CMT1` (a little-endian
        TIFF whose IFD0 holds `Make`, `Model`, `Orientation`) and `CMT2`
        (a TIFF whose IFD0 *is* the ExifIFD: `DateTimeOriginal`,
        `SubSecTimeOriginal`, `ExposureTime`, `FNumber`, `ISO`,
        `LensModel`, ...), through Step 1's reader over
        `Tiff::new(buf, box_payload_start, box_end)`; `CMT3` (the Canon
        MakerNote TIFF) is located but not read here.
      - `full` = the full-size JPEG track: the `trak` whose `stsd` sample
        entry (`CRAW`) carries a `JPEG` sub-box (the raw tracks carry
        `CMP1`), taking `stsz` sample size and `co64` / `stco` offset as
        `Embedded`.
      - `preview` = the `PRVW` box inside the top-level `uuid`
        `eaf42b5e-1c98-4b88-b9fb-b7dc406e4d16` (a 1620x1080 JPEG: header
        of unknown u32, u16 1, u16 width, u16 height, u16, u32 jpeg
        length, then the JPEG), as `Embedded` offsets; fall back to the
        `THMB` box in the Canon uuid only if no `PRVW` is found, else to
        `full`.
    - `scan::is_raw_file` accepts `.cr3`; `reader` dispatches to
      `cr3::parse`. A prefix that holds `moov` and the `PRVW` header but
      not the preview bytes parses fine and the reader range-reads the
      JPEG; a prefix cut inside `moov` is an error (the existing whole-file
      retry then runs).
    - Unit tests with synthetic boxes (a small box builder in the test
      module, like `arw.rs`'s `ifd` / `tiff` helpers): the PRVW preview and
      the JPEG track, box sizes out of range, `largesize`, the JPEG track
      picked over the raw tracks, orientation / capture time / sub-second
      from CMT1 / CMT2, and `reader` tests for a `.cr3` whose preview lies
      past `HEAD_LIMIT`.
    - Verified locally on raw.pixls.us CR3s of the EOS R bodies available
      there, aiming at R5, R5 Mark II, R6 Mark II, R3, R1, R7, R8, R10
      (whichever exist): preview, 1:1, orientation on a portrait frame,
      meta pane rows against `exiftool`, capture time with sub-second;
      results and the offset of the end of `PRVW` per body recorded in
      `learnings.md`.
    - `docs/agents/raw-metadata-parsing.md` gains a CR3 section and
      `CLAUDE.md`'s Layout paragraph names `cr3.rs`.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged (the reader dispatch and the shared
      Exif reader).
    - Measure on the samples whether `ftyp` + `moov` + the XMP `uuid` +
      `PRVW` fit in `HEAD_LIMIT` (1 MiB). If the preview JPEG regularly
      ends past it, that only costs the ranged read; if `moov` itself ends
      past it, either raise `HEAD_LIMIT` (documented in `reader.rs` and
      `docs/performance.md`) or accept the whole-file retry; record the
      numbers and the choice.
    - `thumbnail_jpeg` scales at a fixed 2/8, so the 1620x1080 PRVW gives
      405x270 thumbnails, the same as ARW; nothing to change.
    - Also check the Canon `CTBO` box as a cross-check of the PRVW offset,
      but do not depend on it.
  - Changed during implementation (measured on 14 raw.pixls.us bodies,
    see `learnings.md`): files shot with HDR PQ (HEIF) carry HEVC, not
    JPEG, in `PRVW`, `THMB` and the first track (every R8 sample, two of the
    four R5 Mark II ones). `PRVW` / `THMB` are therefore taken only when
    their data starts with a JPEG SOI, so such a file parses with no
    preview and no full JPEG and the reader reports "no embedded preview";
    HEVC decoding is left to `todo.md`. `moov` + `PRVW` end by ~940 KB on
    every sample, so `HEAD_LIMIT` stays 1 MiB. The `CRAW` sample entry's
    sub-boxes start 82 bytes into its payload.

- [x] Step 4: App strings, CLI text, docs and the compatibility lists
  - Done when:
    - `crates/app/ui/src/empty.ts` `NO_FILES_TEXT` and the CLI bench error
      in `crates/cli/src/main.rs` name the four formats (or say "RAW
      files"); any test asserting the old string is updated.
    - `README.md` "RAW formats and cameras" and `README.ja.md` "RAW 形式と
      カメラ" add `CR3` and `NEF` entries listing only the bodies Steps 2
      and 3 verified on samples (checked) and the JPEG-only sentence names
      the RAW formats generically or lists all four; both files change in
      the same PR.
    - `docs/cameras.md` gains a row per verified body (AF point `–`, AF
      frame `–`, face tracking `–`, sub-second `✓` / `–` as measured) and
      `docs/usage.md` lines that say "ARW and DNG" (the Features intro, the
      JPEG-only folder bullet, the sidecar examples where the extension
      matters) cover the new formats.
    - `todo.md` "Core: widen camera support from public sample RAW files":
      the Tier 3 checkbox records CR3 and NEF as done (RAF and the rest
      still open) and the Tier 2 checkbox names Canon `AFInfo2` / Nikon
      `AFInfo2` for these bodies as the follow-up unless Step 5 is taken.
    - `docs/performance.md` gets a short section only if Steps 2 / 3
      measured something worth recording (prefix fit, per-page read).
    - A new user-facing `docs/raw-formats.md` (added at the user's request,
      2026-09-29) explains, with Mermaid diagrams and tables, the layers of
      a RAW file and what each depends on: the container (TIFF / ISOBMFF,
      depends on the format), the standard Exif (shared across makers), where
      the embedded previews live (depends on the maker; sizes vary by body)
      and the MakerNote (depends on the maker, with per-generation / per-body
      versions, e.g. Nikon `AFInfo2` versions, Canon `AFInfo2` vs `AFInfo3`,
      Sony per-model offsets, SIGMA BF vs fp L within DNG). It covers how
      ARW, DNG, NEF and CR3 differ at each layer (from what Steps 2 / 3
      found: NEF's 1620x1080 preview SubIFD vs the small MakerNote preview,
      CR3's PRVW and JPEG track, HEIF CR3s), which layers Riffle reads for
      which feature (preview / 1:1 / meta pane / bursts vs the AF point), and
      why the compatibility list is per body rather than per format or
      maker. `README.md` / `README.ja.md` (Compatibility) and
      `docs/cameras.md` link to it.
    - `mise run ci` passes (lychee checks the links).
  - Implementation approach:
    - Assumes Steps 2 and 3 are merged. Keep the compatibility list
      honest: a body not opened on a real sample stays unlisted.
    - `docs/raw-formats.md` is for readers, not agents: no pitfall list
      (that stays in `docs/agents/raw-metadata-parsing.md`), and GitHub
      renders the Mermaid blocks.

- [ ] Step 5 (optional, see Trade-offs): AF point from Nikon `AFInfo2` and Canon `AFInfo2`
  - Done when:
    - NEF: the Nikon MakerNote tag 0x00b7 `AFInfo2` (UNDEFINED) is read for
      the versions the Z bodies write (`0300` / `0301`: `AFImageWidth` /
      `AFImageHeight` at note-relative byte 16 / 18, `AFAreaXPosition` /
      `AFAreaYPosition` at 20 / 22, `AFAreaWidth` / `AFAreaHeight` at
      24 / 26, all `int16u`, verify against exiftool's `Nikon.pm`), mapped
      to `Shot.focus = FocusLocation { sensor_w: AFImageWidth, sensor_h:
      AFImageHeight, x, y }` and, when the area size is nonzero, to
      `Shot.focus_frame` so the eye-AF window path can use it. A zero
      position or an unknown version gives `None`.
    - CR3: `CMT3`'s IFD0 tag 0x0026 `AFInfo2` (`int16u[]`: `AFInfoSize`,
      `AFAreaMode`, `NumAFPoints`, `ValidAFPoints`, `CanonImageWidth` /
      `Height`, `AFImageWidth` / `Height`, then `AFAreaWidths[N]`,
      `AFAreaHeights[N]`, `AFAreaXPositions[N]`, `AFAreaYPositions[N]`
      (signed, relative to the image center), then the `AFPointsInFocus`
      and `AFPointsSelected` bitmasks) is read; the first in-focus point
      (else the first selected) is converted from center-relative to
      top-left `FocusLocation` in the `AFImage` frame. The Y axis direction
      (exiftool's tables say positive Y is up; confirm on an off-center
      sample) and whether `AFImage` is the sensor or a smaller frame are
      settled on samples and written down in `raw-metadata-parsing.md`.
    - Synthetic-byte tests for both, including a portrait frame, and
      `EXTRACTOR_VERSION` bumped to 7 (rows written by Steps 2 / 3 builds
      lack the focus); `docs/cameras.md` rows flip to `✓` for the bodies
      confirmed; the Tier 2 `todo.md` checkbox is updated.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 2 and 3 are merged. Sample coverage is the risk:
      raw.pixls.us scenes are mostly centered, so an off-center check needs
      the local review-site galleries. If a maker's mapping cannot be
      confirmed on at least one off-center landscape and one portrait
      sample, ship the other maker only and leave that one in `todo.md`.
    - The consumers (`sharpness.rs`, `partial.rs`, the focus mark) treat
      `FocusLocation` as unrotated sensor coordinates and clamp into the
      JPEG, like the Sigma BF grid; no consumer change is expected.

## Trade-offs and risks

- **Step 5 in or out.** The requirement says "only if it fits". The
  container work (Steps 1-4) is the promotion blocker; the AF point is
  what makes the focus mark and the sharpness cue as good as on Sony. Taking
  Step 5 adds one PR and depends on off-center samples the CC0 set may not
  offer. Decision (user, 2026-09-29): run Steps 1-4 first, then decide Step 5
  from the samples found; Step 4 already records the follow-up either way.
- **Refactor first (Step 1) vs folding it into Step 2.** A separate pure
  move keeps the NEF PR about NEF and lets `jpeg.rs`'s tests prove nothing
  changed. Cost: one small extra PR.
- **Generalizing `arw.rs` to big-endian instead of a new module.** Rejected:
  `arw.rs` carries Sony / Leica / Sigma MakerNote logic and a LE-only helper
  set; making it order-aware touches every helper and every ARW test for no
  gain, while `sequence::Tiff` already handles both orders.
- **Keeping the `Arw` type name for the shared output.** `jpeg.rs` set the
  precedent; renaming it (`Raw`, `Parsed`) would touch the app, the index
  and the CLI. Left as is; a rename can be its own chore later.
- **NEF preview size is unverified.** If the Z bodies' `PreviewIFD` JPEG is
  small (older Nikons wrote 570x375), the preview falls back to the
  full-size JPEG: correct, but each page turn then decodes an 8000-px JPEG
  (the Leica M11-P 1:1 decode measured ~105 ms) and the fixed 2/8 thumbnail
  scale gives large thumbnails, the same issue `todo.md` records for the
  SIGMA fp L. Step 2 measures this; if it bites, `thumbnail_jpeg_near` or a
  scaled preview decode is a follow-up, recorded in `todo.md`.
- **CR3 prefix fit.** If `moov` + `PRVW` routinely exceed 1 MiB, every CR3
  open pays the whole-file retry (20-40 MB). Step 3 measures and either
  raises `HEAD_LIMIT` for all formats (more bytes read per ARW too) or makes
  the limit per container; the plan leaves that choice to the measurement.
- **Compatibility claims.** Only bodies whose samples were opened get listed
  in the README; the Reddit audience will have R6 / Z 6-class bodies whose
  files may differ in details (older `AFInfo2` versions, different track
  order). The camera issue template already asks for a sample.
- **No committed fixtures.** CC0 would allow a small raw.pixls.us file, but
  the repository's rule is synthetic bytes; real-file checks live in
  `learnings.md` only.

## Progress

- (2026-09-29) Step 1 complete
- (2026-09-29) Step 2 complete
- (2026-09-29) Step 3 complete
- (2026-09-29) Step 4 complete
