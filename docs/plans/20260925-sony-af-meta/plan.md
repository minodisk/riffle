<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Sony MakerNote fields in the meta pane

## Purpose

The meta pane's `Maker note` section shows only the Sony shutter type and the
Leica focus distance. Sony bodies record much more of the shooting setup in
the plain (unenciphered) MakerNote IFD: the AF setup (`AFTracking` 0x2021,
`FocusMode` 0x201b, `AFAreaModeSetting` 0x201c), the drive (`ReleaseMode`
0xb049 and `SequenceNumber` 0xb04a), `ImageStabilization` 0xb026, and the
exposure and picture settings (`ExposureMode` 0xb041, `MeteringMode2` 0x202c,
`CreativeStyle` 0xb020, `DynamicRangeOptimizer` 0xb025, `RAWFileType`
0x2029). `FocusMode` and `AFTracking` are already parsed into
`riffle_core::arw::Shot` (they drive the sharpness score and the face-catch
state) but never shown; the rest is not parsed. Showing them as rows lets the
user tell, while culling, how the camera was set up for a frame. Sony ARW
only; Leica DNG and SIGMA files show none of these rows.

Not shown: enciphered values (`ShutterCount`, `PictureProfile`, Tag9050 /
Tag94xx) and, because the MakerNote does not record them (checked with
ExifTool 13.59), the recognized subject type (human / animal / bird /
vehicle) and whether the AF locked on an eye or a face.

Value-to-label mappings, from ExifTool's Sony.pm (`%Sony::Main`). Besides
`AFAreaModeSetting`, `FocusMode` and `AFTracking` also carry a model
condition: `($$self{Model} !~ /^DSC-/) or ($$self{Model} =~
/^DSC-(RX10M4|RX100M6|RX100M7|RX100M5A|HX95|HX99|RX0M2|RX1RM3)/)`, so on
other `DSC-` bodies (older RX / HX compacts) both rows are `None`:

- `FocusMode` (0x201b, int8u): 0 Manual, 2 AF-S, 3 AF-C, 4 AF-A, 6 DMF,
  7 AF-D.
- `AFTracking` (0x2021, int8u): 0 Off, 1 Face tracking, 2 Lock On AF.
- `AFAreaModeSetting` (0x201c, int8u), NEX/ILCE/ZV table only: 0 Wide,
  1 Center, 3 Flexible Spot, 4 Flexible Spot (LA-EA4), 9 Center (LA-EA4),
  11 Zone, 12 Expanded Flexible Spot, 13 Custom AF Area. ExifTool's
  condition for this table is
  `^(NEX-|ILCE-|ILME-|ZV-|DSC-(RX10M4|RX100M6|RX100M7|RX100M5A|HX95|HX99|RX0M2|RX1RM3))`;
  Riffle deliberately narrows it to `ILCE-`, `NEX-` and `ZV-` (decided with
  the user). SLT/HV and ILCA bodies use different tables and get no row.
- `ReleaseMode` (0xb049, int16u): 0 Normal, 2 Continuous, 5 Exposure
  Bracketing, 6 White Balance Bracketing, 8 DRO Bracketing; 65535 n/a.
- `SequenceNumber` (0xb04a, int16u): "shot number in continuous burst";
  0 Single, 65535 n/a, any other number is the frame number. On five
  ILCE-7M5 files it read 2, 1, 2, 3, 1 with `ReleaseMode` 2: it restarts at
  1 per burst.
- `ImageStabilization` (0xb026, int32u): 0 Off, 1 On; 0xffffffff n/a.
- `ExposureMode` (0xb041, int16u): 0 Program AE, 1 Portrait, 2 Beach,
  3 Sports, 4 Snow, 5 Landscape, 6 Auto, 7 Aperture-priority AE, 8 Shutter
  speed priority AE, 9 Night Scene / Twilight, 10 Hi-Speed Shutter,
  11 Twilight Portrait, 12 Soft Snap/Portrait, 13 Fireworks, 14 Smile
  Shutter, 15 Manual, 18 High Sensitivity, 19 Macro, 20 Advanced Sports
  Shooting, 29 Underwater, 33 Food, 34 Sweep Panorama, 35 Handheld Night
  Shot, 36 Anti Motion Blur, 37 Pet, 38 Backlight Correction HDR,
  39 Superior Auto, 40 Background Defocus, 41 Soft Skin, 42 3D Image;
  65535 n/a.
- `MeteringMode2` (0x202c, int16u): 0x100 Multi-segment, 0x200
  Center-weighted average, 0x301 Spot (Standard), 0x302 Spot (Large),
  0x400 Average, 0x500 Highlight.
- `CreativeStyle` (0xb020, string, always English): shown as recorded
  except ExifTool's normalizations: AdobeRGB -> Adobe RGB, Nightview ->
  Night View/Portrait, BW -> B&W, Autumnleaves -> Autumn Leaves, VV2 ->
  Vivid 2. ("Creative Look" on current bodies; the tag name is historical.)
- `DynamicRangeOptimizer` (0xb025, int32u): 0 Off, 1 Standard, 2 Advanced
  Auto, 3 Auto, 8-12 Advanced Lv1-Lv5, 16-23 Lv1-Lv8 (confirm the exact
  table in Sony.pm when implementing). Chosen with the user over 0xb04f
  (0 Off, 1 Standard, 2 Plus), which ExifTool marks `Priority => 0` and which
  read `Standard` on the sample files where 0xb025 read `Auto`, the value the
  body's menu offers.
- `RAWFileType` (0x2029, int16u): 0 Compressed RAW, 1 Uncompressed RAW,
  2 Lossless Compressed RAW, 3 Compressed RAW 2 (the ILCE-7M5 writes 3 for
  both "Compressed RAW" and "Compressed (HQ) RAW"); 65535 n/a.

Standard-EXIF overlap: Riffle reads neither EXIF `ExposureProgram` (0x8822)
nor `MeteringMode` (0x9207), so `ExposureMode` and `MeteringMode2` do not
duplicate an existing row; they are shown under the `Maker note` divider and
the EXIF twins stay unread.

## Steps

- [x] Step 1: Read the new Sony MakerNote tags in core and bump `EXTRACTOR_VERSION`
  - Done when:
    - `riffle_core::arw::Shot` gains raw fields, `None` when the MakerNote
      is absent, non-Sony, or lacks the tag, each with a doc comment
      listing the meanings (as `focus_mode` does):
      `af_area_mode: Option<u8>`, `release_mode: Option<u32>`,
      `sequence_number: Option<u32>`, `image_stabilization: Option<u32>`,
      `exposure_mode: Option<u32>`, `metering_mode: Option<u32>`,
      `creative_style: Option<String>`, `dynamic_range_optimizer: Option<u32>`,
      `raw_file_type: Option<u32>`.
    - Unit tests on synthetic bytes in `crates/core/src/arw.rs`: one Sony
      note carrying all the new tags yields their values (including the
      out-of-line `CreativeStyle` string); a Sony note without them yields
      `None` for each; the Leica and Sigma notes yield `None` (extend the
      existing `a_non_sony_maker_note_is_skipped` / Sigma assertions that
      already list `focus_mode`).
    - `EXTRACTOR_VERSION` in `crates/app/src/index.rs` is bumped by one (the
      only bump in this plan), with the doc comment's history extended
      ("reads the Sony AF area, drive, stabilization and picture settings").
      `SCHEMA_VERSION` is unchanged (no column added). The existing
      re-extraction tests still pass.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/core/src/arw.rs`: add the tag consts next to `TAG_FOCUS_MODE`
      / `TAG_AF_TRACKING` with the ExifTool name and type in the doc comment:
      `TAG_AF_AREA_MODE_SETTING = 0x201c` (BYTE, read with `byte()`),
      `TAG_RELEASE_MODE = 0xb049`, `TAG_SEQUENCE_NUMBER = 0xb04a`,
      `TAG_IMAGE_STABILIZATION = 0xb026`, `TAG_EXPOSURE_MODE = 0xb041`,
      `TAG_METERING_MODE2 = 0x202c`, `TAG_DYNAMIC_RANGE_OPTIMIZER = 0xb025`,
      `TAG_RAW_FILE_TYPE = 0x2029` (all read with `integer()`, which takes
      SHORT or LONG count 1 so int16u / int32u both work and a body writing
      the other width still reads), `TAG_CREATIVE_STYLE = 0xb020` (read with
      `ascii(buf, e)?`, which follows the TIFF-relative offset the Sony IFD
      uses; `"Standard\0"` is 9 bytes, so it is out of line).
    - Populate in `exif()` from the `maker` IFD exactly like `focus_mode` /
      `af_tracking`. With nine lookups, a small local closure
      `find(tag) -> Option<&Entry>` over `maker` keeps it readable; do not
      restructure the existing reads. Name it `maker_entry` (not `find`), since the
module already has a top-level `find(entries, tag)` used a few lines above.
    - Keep core a raw dump: no `n/a` (65535 / 0xffffffff) filtering and no
      model gate here; those are display concerns (Steps 2-3). The CLI in
      `crates/cli/src/main.rs` prints `focus_mode`; extending its dump is
      optional.
    - Tests: `tiff_with_sony_note` takes a closure returning MakerNote
      entries and a trailing `data` slice for out-of-line values (the
      closure receives `data_at`); use it for one note with all the new
      tags, with `CreativeStyle` pointing into `data`. Absent case: reuse
      `tiff_with_exif(true, None, false)` as `reads_the_sony_focus_mode`
      does.
    - Verify on a real ILCE-7M5 ARW (a throwaway test calling
      `riffle_core::reader::read_metadata`, as the shutter-type work did;
      samples: `C:\Users\daisu\OneDrive\Desktop\sony flags\_DSC000{1..5}.ARW`)
      that every tag reads: expected 0x201c as BYTE (0x201a turned out to be
      LONG contrary to ExifTool, so if 0x201c is not BYTE accept `integer()`
      too), `release_mode` 2, `sequence_number` 1..3, `image_stabilization`
      1, `exposure_mode` 15, `metering_mode` 256, `creative_style`
      "Standard", `dynamic_range_optimizer` 3, `raw_file_type` 2. Also
      confirm the MakerNote lies inside the bounded prefix
      `reader::read_metadata` reads. Record the observed types in
      `learnings.md`.

- [x] Step 2: Format the AF fields in `exif.rs` and show them in the meta pane
  - Done when:
    - `crates/app/src/exif.rs::Exif` gains `focus_mode: Option<String>`,
      `af_tracking: Option<String>` and `af_area: Option<String>`, filled by
      `exif(&Shot)` from `shot.focus_mode`, `shot.af_tracking`,
      `shot.af_area_mode` and `shot.model` with the mappings in Purpose;
      unlisted values are `None`; `af_area` is `None` unless `Model` starts
      with `ILCE-`, `NEX-` or `ZV-`.
    - Unit tests in `exif.rs` `mod tests`: every listed value of each field
      maps to its label; an unlisted value (e.g. 1, 5, 255 for focus mode;
      3 for tracking; 2, 8, 255 for AF area) is `None`; AF area is `None` for
      a non-family model (`ILME-FX3`, `ILCA-99M2`, `SLT-A99V`, `DSC-RX100M7`,
      `LEICA M11-P`, `None`) even for value 0; `Shot::default()` still
      formats to `Exif::default()`.
    - `Metadata` in `crates/app/src/commands.rs` gains the three
      `Option<String>` fields, copied from `Exif` in `read_metadata` (next to
      `shutter_type`).
    - `crates/app/ui/src/meta.ts`: the `Metadata` interface mirrors them and
      `metaGroups` adds `["Focus mode", meta.focus_mode]`,
      `["AF area", meta.af_area]`, `["AF tracking", meta.af_tracking]` to the
      `MAKER_NOTE_LABEL` section after `Shutter type` and before
      `Focus distance` (Leica's row stays last). `meta.test.ts` fixtures
      gain the fields; one test asserts the three rows with sample labels
      and the existing "no maker note field" test is extended so a Sony
      fixture with all Maker note fields `null` still drops the section.
    - Opening a Leica DNG or a SIGMA fp L / BF DNG shows none of the rows
      (their `Shot` fields are `None`, covered by Step 1's core tests).
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (needs `Shot::af_area_mode`).
    - Keep the mappings as plain `match` arms in `exif.rs` (one small
      function per field returning `Option<&'static str>`, e.g.
      `fn focus_mode(v: u8)`, `fn af_tracking(v: u8)`,
      `fn af_area(model: Option<&str>, v: u8)`), as strings, not `Labeled`
      (they are not sortable / filterable). Cite ExifTool's Sony.pm in a
      comment on each; on `af_area` note that SLT/HV and ILCA tables are
      intentionally not mapped and that `ILME-` / the RX/HX `DSC-` bodies
      are deliberately left out.
    - Model test: `starts_with` over the prefixes, as `sigma_af_point` in
      core compares `Model` with a plain string.
    - No change to `crates/app/src/index.rs` (the `files` table caches only
      what the filter menu needs) or `crates/app/ui/src/filter.ts`.
    - Label wording follows ExifTool ("Face tracking", "Lock On AF",
      "Manual", "AF-C", "Flexible Spot", ...).

- [x] Step 3: Format the drive, stabilization and picture settings in `exif.rs` and show them in the meta pane
  - Done when:
    - `Exif` gains `drive: Option<String>`, `stabilization: Option<String>`,
      `exposure_mode: Option<String>`, `metering: Option<String>`,
      `creative_style: Option<String>`, `dro: Option<String>` and
      `raw_type: Option<String>`, filled from the Step 1 fields with the
      mappings in Purpose; unlisted values and the `n/a` sentinels (65535,
      0xffffffff) are `None`.
    - `drive` is one row combining `ReleaseMode` and `SequenceNumber`
      (decided with the user): the release-mode label alone when the
      sequence number is absent or 0 (`"Normal"`, `"Continuous"`), and
      `"Continuous, frame 2"` when the sequence number is 1 or more. A
      sequence number without a known release mode shows nothing.
    - `creative_style` passes the recorded string through, with ExifTool's
      five normalizations applied, and is `None` for an empty string.
    - Unit tests in `exif.rs`: every listed value of each field maps to its
      label; unlisted values and the sentinels are `None`; the drive row's
      three shapes (release mode only, with frame number, sequence number
      without release mode); creative style passthrough, each normalization
      and the empty string; `Shot::default()` still formats to
      `Exif::default()`.
    - `Metadata` in `commands.rs` gains the seven fields; `meta.ts` mirrors
      them and `metaGroups` adds, after `AF tracking` and before
      `Focus distance`: `["Drive", meta.drive]`,
      `["Stabilization", meta.stabilization]`,
      `["Exposure mode", meta.exposure_mode]`, `["Metering", meta.metering]`,
      `["Creative style", meta.creative_style]`, `["DRO", meta.dro]`,
      `["RAW type", meta.raw_type]`. `meta.test.ts` fixtures and the
      row-order assertion are extended.
    - Leica / SIGMA files show none of the rows; `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged (same `Exif` / `Metadata` /
      `metaGroups` shape).
    - Same style as Step 2: one `match` function per field in `exif.rs`
      with the ExifTool table cited. `exposure_mode` maps the full ExifTool
      table (a flat `match`, so completeness costs only lines).
    - `metering` matches on the hex values (`0x100`, ...); a comment says
      the tag is `MeteringMode2`, finer than EXIF `MeteringMode`, which
      Riffle does not read.
    - `dro` reads `shot.dynamic_range_optimizer` (0xb025); a comment notes
      why 0xb04f is not used.
    - Row labels (decided with the user): "Drive", "Stabilization",
      "Exposure mode", "Metering", "Creative style", "DRO", "RAW type".
    - No index / filter change.

- [ ] Step 4: Document the new rows in `docs/usage.md`, `docs/cameras.md`, `README.md` and `README.ja.md`
  - Done when:
    - `docs/usage.md` (Meta pane bullet) lists the Sony rows next to the
      shutter type: AF tracking, focus mode, AF area (ILCE/NEX/ZV bodies
      only; other Sony families use a different table), drive (release mode
      with the frame number within a burst), stabilization, exposure mode,
      metering, creative style, DRO and RAW type; and notes that the
      MakerNote does not record the recognized subject type or whether the
      AF sat on an eye or a face, so Riffle does not show them, and that
      enciphered values (shutter count, picture profile) are not read.
    - `docs/cameras.md` (the "AF frame size and face tracking" bullet or a
      new sentence) notes that on the α7 V `Face tracking` is also recorded
      when the AF sits on the back of a head: it means the camera recognized
      a person's head, not strictly a face or an eye. The meta pane's
      `AF tracking` row shows this value.
    - `README.md` and `README.ja.md` are updated together: a sentence where
      the meta pane is mentioned says the Maker note section shows the Sony
      AF, drive, stabilization and picture settings. The Japanese body
      mirrors the English.
    - `mise run ci` passes.
  - Implementation approach:
    - Docs-only PR; assumes Step 3 is merged so the described behavior exists.
    - Keep the wording consistent with the row labels used in `meta.ts`.

## Trade-offs and risks

- Drive row shape: one combined row (decided with the user); the sequence
  number is not used to change burst grouping (out of scope).
- DRO tag: 0xb025 (decided with the user), not 0xb04f.
- `RAWFileType` on the ILCE-7M5: value 3 is written for both "Compressed
  RAW" and "Compressed (HQ) RAW", so the row cannot tell them apart; the
  label follows ExifTool ("Compressed RAW 2").
- `CreativeStyle` on current bodies is the "Creative Look" (ST, PT, NT, VV,
  VV2, FL, IN, SH, ...); ExifTool passes unknown strings through, so the row
  may show a two-letter code. Accepted; the docs step can mention it.
- AF area family gate: strict `ILCE-` / `NEX-` / `ZV-` (decided with the
  user); an ILME-FX3 or an RX100M7 shows no row.
- `EXTRACTOR_VERSION` bump: no index column is added, so the bump changes
  nothing visible; it is done because `docs/agents/tauri-app.md` requires it
  on any `arw.rs` parsing change. One bump total, in Step 1. Cost: every
  folder is re-extracted once on its next scan.
- `FocusMode` label set includes 7 = AF-D; only 0 and 3 were observed on the
  α7 V.
- Where the mapping lives: `exif.rs` (keeps core's `Shot` a raw dump and
  the CLI keeps raw values) rather than core.
- Not filterable: none of the Maker note rows is in the filter menu. Out of
  scope.
- Tag types on real bodies: `integer()` accepts SHORT and LONG, so only
  0x201c (BYTE) is at risk; Step 1 verifies on a real file.
- Row count: the `Maker note` section grows to 11 rows on Sony files
  (accepted by the user).
- "Face tracking" wording: the value is recorded on the back of a head too;
  the docs (Step 4) say so rather than renaming the label away from
  ExifTool's.

## Progress

- 2026-09-25: Step 1 landed (`feat(core): read the Sony AF area, drive,
  stabilization and picture settings`). `riffle_core::arw::Shot` gained the
  nine raw fields and `EXTRACTOR_VERSION` was bumped by one. See
  `learnings.md` for the tag types observed on the ILCE-7M5 samples.
- (2026-09-25) Step 2 complete
