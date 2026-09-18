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

# Leica DNG support (LEICA M11-P)

## Purpose

Riffle only lists and parses Sony ARW files. The user has a folder of LEICA
M11-P DNGs (`/Users/mino/Downloads/leica raw files/*.DNG`, 31 files, 60-78MB)
and wants to cull them the same way. A DNG is a little-endian TIFF like an ARW,
and the M11-P embeds baseline JPEGs at four sizes, so every tier the app has —
filmstrip thumbnail, preview, 1:1 focus check — can come from embedded JPEGs
without running a RAW decoder. Once done, a folder of `.DNG` files behaves like
a folder of `.ARW` files: indexed, paged, focus-checked (centre fallback, the
M-lens is manual focus) and rated via XMP sidecars.

## What the sample files look like (investigated during planning)

Measured on `L1005200.DNG` with exiftool and a hand-written IFD walk; the other
30 files have the same layout with different JPEG byte lengths.

- Header `II`, IFD0 at offset 12 with 52 entries. `Make` = `Leica Camera AG`,
  `Model` = `LEICA M11-P`, `DNGVersion` 1.4.0.0, `Orientation` 1 on 30 files
  and **6 on `L1005231.DNG`** (use it for the orientation check).
- IFD0 is the raw: 9536x6336, `Compression=7` (lossless JPEG),
  `PhotometricInterpretation=32803` (CFA), `BitsPerSample=16`,
  `StripOffsets`=2265534, `StripByteCounts`=57907200. **It must never be
  selected as a JPEG**: it shares `Compression=7` with the previews.
- `SubIFDs` (0x014a, LONG[4]) at 642 / 792 / 942 / 1092, each with
  `NewSubfileType=1`, `Compression=7`, `PhotometricInterpretation=6`,
  `BitsPerSample=8 8 8`, and the JPEG located by **`StripOffsets` (0x0111) /
  `StripByteCounts` (0x0117)**, not by `0x0201/0x0202` as in an ARW:
  - 160x120 at 14336 (10752 bytes)
  - **9504x6320 at 25088 (1.9-5.5MB across the samples): the 1:1 tier**
  - **2112x1408 (272-717KB): the preview tier**
  - 720x480 (60KB)
- Both large JPEGs are **baseline DCT, 4:2:2**, so the partial decode in
  `partial.rs` applies.
- ExifIFD at 1242: `ExposureTime`, `ISO`, `DateTimeOriginal`, `ExposureBias`,
  `FocalLength`, `LensModel` (`Summicron-M 1:2/50`). **No `FNumber`, no
  `SubSecTimeOriginal`.** MakerNote (0x927c, 4096 bytes at 6144) starts with
  `LEICA\0`, not `SONY`.
- Everything the parser needs sits in the first ~14KB, so `HEAD_LIMIT` (1MiB)
  covers the metadata; both the preview and the 1:1 JPEG end past 1MiB, so
  they go through the ranged read `reader.rs` already has.
- `.DNG.dop` files (DxO PhotoLab) sit next to some DNGs; their extension is
  `dop`, so the extension filter ignores them by itself.

## Steps

- [x] Step 1: Core: find the embedded JPEGs of a DNG in `arw::parse` and gate the Sony MakerNote
  - Done when:
    - `riffle_core::arw::parse` on any of the 31 sample DNGs returns a
      `preview` of the 2112x1408 JPEG and a `full` of the 9504x6320 JPEG,
      `orientation` (6 on `L1005231.DNG`), `make`/`model`, `capture_time`,
      `lens_model`, `exposure_time`, `iso`, `focal_length`, `exposure_bias`,
      and `focus: None`, `subsec: None`, `f_number: None`.
    - `riffle-cli info <file.DNG>` prints those locations; `riffle-cli crop
      <file.DNG> out.png` writes a centred crop; `riffle-cli focusbox` may
      still fail with "no FocusLocation" (that is the existing behaviour on
      manual-focus ARWs).
    - ARW behaviour is unchanged: the existing tests pass untouched, and
      `riffle-cli info` on an α7 V ARW prints the same offsets as before.
    - Unit tests, built with the same synthetic-TIFF helpers the file already
      has, cover: (a) a DNG-style file whose SubIFDs carry strip JPEGs of
      several sizes in a non-sorted order — the test must assert the selection
      is by size, not by SubIFD index (put the 1:1 JPEG in a middle slot and
      the preview-sized one last); (b) an IFD0 with `Compression=7` but
      `PhotometricInterpretation=32803` is not picked; (c) a MakerNote with a
      non-Sony header yields `focus: None` and does not error; (d) a file with
      neither `0x0201` nor strip JPEGs still parses with both `None`.
  - Implementation approach:
    - Keep the module name `arw` and the type `Arw`; this step is about
      parsing, not renaming (see Trade-offs). Update the module doc comment to
      say the parser also covers DNG.
    - In `crates/core/src/arw.rs`, extend the IFD walk that already visits the
      IFD chain and the `SubIFDs`: for each IFD, besides `embedded()` (the
      `0x0201/0x0202` pair), also recognise a **strip JPEG**: `Compression`
      (0x0103) == 7, `PhotometricInterpretation` (0x0106) == 6, `StripOffsets`
      (0x0111) and `StripByteCounts` (0x0117) both with `count == 1`, plus
      `ImageWidth` (0x0100) / `ImageHeight` (0x0101). The width/height are
      needed for the size-based choice; the existing `integer()` helper reads
      SHORT/LONG values.
    - Selection rule when the file has strip JPEGs and no IFD0 `0x0201`
      preview: `full` = the candidate with the largest pixel area; `preview` =
      the smallest candidate whose width is at least 1600 (yields 2112x1408;
      falls back to `full` when nothing else qualifies). Write the reason in
      the code comment.
    - The ARW path stays as it is: IFD0 `0x0201` preview, and `full` = the
      largest `0x0201` JPEG by byte length. Do not change how ARW picks, only
      add the strip-JPEG candidates to the same walk.
    - Gate `maker_note_ifd` so the Sony IFD parse only runs when the MakerNote
      starts with `SONY` or the `Make` read from IFD0 starts with `SONY`
      (planning found that on the Leica note it reads the `LE` bytes as an
      entry count and walks garbage; it happens to stay inside the 1MiB prefix
      on the sample file, but it can bail on another and force the whole-file
      fallback, or return a bogus `FocusLocation`). `Shot.make` is already read
      before the ExifIFD, so it is available.
    - Adding `width`/`height` to `Embedded` is acceptable if it keeps the
      selection code simple, but keep `Embedded` `Copy` and keep the field
      set minimal; `reader.rs` and `commands.rs` only use `offset`/`length`.
    - `Kind::name()` in `reader.rs` returns "JpgFromRaw" for error messages;
      leave it.
    - Verify with the real files (read-only) that `read_preview` and
      `read_full` succeed on all 31 without hitting the whole-file fallback
      (e.g. a quick `riffle-cli info` loop; the fallback is silent, so check
      by timing or by a temporary `eprintln!` that is not committed).

- [x] Step 2: App and CLI list `.DNG` next to `.ARW`; end-to-end on the sample folder
  - Done when:
    - `crates/app/src/commands.rs::is_arw` (rename to something like
      `is_raw`) accepts `arw` and `dng` case-insensitively; the
      `lists_only_arw_files_sorted_by_name` test grows a `.DNG`/`.dng` case and
      a `.dop` negative case.
    - `riffle-cli scan` uses the same extension rule and its "no ARW files"
      message says RAW/ARW/DNG.
    - `crates/app/ui/src/main.ts` user-facing strings ("No ARW files in that
      folder.", "Drop a single folder or ARW file.") no longer say only ARW.
    - Opening `/Users/mino/Downloads/leica raw files` in the release build
      (`mise run tauri:release:devtools`) is **confirmed by the user** for:
      all 31 files listed, the scan completing with 0 errors, filmstrip
      thumbnails, the preview, `Space` showing a centred 1:1 crop,
      `L1005231.DNG` displayed upright (Orientation 6) in the strip, preview
      and crop, the meta pane showing camera/lens/shutter/ISO/focal
      length with a blank aperture, and `1`-`5`/`x`/`u`/`0` producing
      `L100xxxx.xmp` next to the DNG (the `.dop` files untouched). GUI
      automation does not work on this Mac (see `docs/agents/tauri-app.md`),
      so list exactly these checks for the user and report anything not
      checked as "not verified".
  - Implementation approach:
    - Keep the Tauri command name `list_arw` and the `arw` module/type names
      unchanged in this step. Only the extension predicate and the strings
      change.
    - Share one predicate: a small `pub fn is_raw_file(path: &Path) -> bool`
      in `riffle_core` (e.g. in `scan.rs`) used by both `commands.rs` and
      `crates/cli/src/main.rs::scan_dir`. Do not add more than that.
    - The index (`index.rs`), the sidecar writer (`sidecar.rs`) and
      `xmp::sidecar_path` are extension-agnostic; nothing to change there
      beyond doc comments on lines the change actually touches.
    - Thumbnails: `decode::thumbnail_jpeg` scales by 2/8, so a DNG thumbnail
      is 528x352 instead of 404x270. Leave the scale alone unless the
      measured thumbnail bytes in Step 3 are out of line.

- [ ] Step 3: CLI benchmark on the DNG folder and the measured numbers
  - Done when:
    - `riffle-cli bench` times the 1:1 crop on a file without `FocusLocation`
      too, by using `partial::decode_focus_crop(jpeg, a.shot.focus, ...)` (the
      centre fallback the app uses) instead of skipping when `focus` is
      `None`; the ARW numbers it prints stay comparable (same crop size).
    - `riffle-cli bench` over the 31 DNGs and `riffle-cli scan` over the
      folder (at least 1 thread and the machine's core count) have been run
      on release builds, and the numbers are in README under
      "Measurements" as a new DNG subsection, with the measurement
      conditions stated (distinct real files, page cache state, file sizes,
      preview 2112x1408, 1:1 JPEG 9504x6320, thumbnail bytes per file).
    - The app-side 1:1 cost on a DNG is either recorded from a user-run
      session or listed explicitly as "awaiting the user's confirmation".
  - Implementation approach:
    - The DNG's 1:1 JPEG is 9504 wide with 6320 rows, and the cost of a crop
      is set by its row, so expect the centre crop (row ~3160) to cost more
      than the α7 V's typical focus row. Report the number against the 50ms
      budget plainly, rather than as a pass.
    - Preview cost: a 2112x1408 decode versus 1616x1080; record it.

- [ ] Step 4: README: supported products and the "Sony ARW only" wording
  - Done when:
    - `README.md` ticks `- [x] M11-P` under Supported products > Cameras >
      Leica.
    - The intro and the Status paragraph describe Sony ARW and Leica DNG; the
      "1:1 focus check" and "The index" sections mention the DNG tiers
      (2112x1408 / 9504x6320, thumbnail 528x352); the CLI usage block accepts
      `<file.ARW|file.DNG>`; the sidecar section's `FOO.ARW` -> `FOO.xmp`
      example gains the DNG equivalent.
    - `CLAUDE.md`'s first line and the `crates/core` description are updated
      the same way.
    - Measurement tables taken on ARWs are left as they are.
  - Implementation approach:
    - Only wording; no code. Keep the "confirmed / verified without a GUI /
      awaiting the user's confirmation" split README already uses.

## Trade-offs and risks

- **Preview selection rule**: "smallest candidate with width >= 1600" (chosen)
  degrades to the full JPEG on a body with only thumbnails plus one big JPEG;
  "largest below `full`" would pick a 160x120 thumbnail on such a body.
- **Naming**: `arw` module, `Arw` type, `list_arw` command are kept. A rename
  would be its own chore PR.
- **Thumbnail size**: 2/8 scaling gives 528x352 (vs 404x270); kept unless
  Step 3's bytes-per-thumbnail is clearly out of line with the ~19KB ARW figure.
- **1:1 cost on a 6320-row JPEG**: may exceed the 50ms budget; Step 3 measures
  it and README reports it. No mitigation planned.
- **Camera label**: "Leica Camera AG LEICA M11-P" in the meta pane; accepted.
- **Aperture**: no Exif `FNumber` on M-mount lenses; the Leica MakerNote is not
  parsed, so aperture is blank.
- **Scope of "DNG"**: only the M11-P layout (little-endian, strip-based SubIFD
  JPEGs, baseline previews). Other DNGs come back with `full`/`preview` `None`
  and show the existing "no embedded preview" error.
- **MakerNote gating** changes ARW behaviour only for a file whose MakerNote
  lacks the `SONY` header and whose `Make` is not Sony.
- **Whole-file fallback**: Step 1 must check the DNGs never take that path.

## Progress

- (2026-09-18) Step 1 complete
- (2026-09-18) Step 2 complete (GUI checks 1-8 confirmed by the user on the 32-file sample folder)
