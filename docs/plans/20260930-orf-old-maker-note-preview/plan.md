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

# ORF: the preview of old Olympus bodies (`OLYMP\0` MakerNote)

## Purpose

The Olympus E-300 sample (`D:\Photos\samples\ORF\E-300_P1252148.ORF`)
gets no filmstrip thumbnail: the reader fails with
`no embedded preview`. `crates/core/src/orf.rs` only knows the
`OLYMPUS\0` / `OM SYSTEM\0\0\0` MakerNote headers, whose offsets are
note-relative and whose CameraSettings is a type 13 pointer. The 2003-2006
Four Thirds DSLRs write the older header, and the parser gives up on it.

Measured at planning time (2026-09-30, Python TIFF dump of every file in
`D:\Photos\samples\ORF\`):

- The E-300's MakerNote (Exif IFD 0x927c, file offset 788, 3290 bytes)
  opens with `OLYMP\0` + a 2-byte version (`02 00`); its IFD starts at
  note + 8. It has no byte-order field: the note is in the file's byte
  order, and every offset in it is **absolute** (relative to the TIFF
  header, like IFD0's), not note-relative (ExifTool's `Olympus.pm`: the
  `OLYMP\0` note has `Start => '$valuePtr + 8'` and no `Base`).
- Its main IFD carries 0x2020 CameraSettings as **type 7 (undefined),
  count 384**: the sub-IFD is written inline at the entry's absolute value
  offset (1384). That IFD holds 0x0100 `PreviewImageValid` 1, 0x0101
  `PreviewImageStart` 24576 (absolute), 0x0102 `PreviewImageLength`
  414528; a JPEG SOI + SOF 1600x1200 sits at offset 24576. The only other
  JPEGs in the file are the 160x120 thumbnail (IFD1 0x0201 and the note's
  0x0100), so this preview is `full` too.
- The same layout (`OLYMP\0` `02 00`, inline type 7 CameraSettings,
  absolute offsets, preview inside the 1 MiB prefix) is on the E-1
  (1280x960 preview), E-330, E-400 and E-500 (1600x1200) samples, so the
  fix covers those too.
- Every other sample (E-3, E-410 to E-520, E-30, E-5, E-600 / E-620, the
  PEN / OM-D / OM / TG / XZ / Stylus / SH-2 bodies) uses `OLYMPUS\0`
  `03 00` or `OM SYSTEM` `04 00` with a type 13 CameraSettings and works
  today; nothing about that path may change.
- The compacts with `OLYMP\0` version `01 00` / `02 01` (C5050Z, C5060WZ,
  C7070WZ, C8080WZ, E-10, E-20, SP-350 / 500UZ / 510UZ / 550UZ / 565UZ /
  570UZ) have no 0x2020 at all and only a 160x120 thumbnail; they stay
  "no embedded preview" and are out of scope.
- `crates/app/src/index.rs` writes `EXTRACTOR_VERSION` (10) into error
  rows too, so the rows that recorded the E-300 failure are only
  re-extracted after a bump.

Once done, the E-300 (and E-1 / E-330 / E-400 / E-500) files get a
thumbnail, a preview and a 1:1 view of the 1600x1200 (1280x960) embedded
JPEG, and the meta pane rows they already had.

## Steps

- [x] Step 1: Read the preview of an `OLYMP\0` MakerNote with an inline CameraSettings IFD
  - Done when:
    - `orf::parse` returns the preview `Embedded` of a MakerNote that opens
      with `OLYMP\0` + 2-byte version: the IFD at note + 8, walked in the
      **file's** byte order with **absolute** offsets (the file's own
      `Tiff` walker, base 0), so `PreviewImageStart` is used as is, not
      added to the note start.
    - CameraSettings 0x2020 written as type 7 (`undefined`, count > 4) is
      accepted: the IFD is at the entry's value offset. The existing type
      13 / 4 pointer path keeps working for both header kinds (the
      `OLYMPUS\0` / `OM SYSTEM` path stays note-relative and unchanged in
      behavior).
    - The bounded-prefix rules keep their shape: a note wholly inside
      `buf` that does not read gives no preview; a note cut by the prefix
      before its IFDs end is an error so `reader` retries with the whole
      file (`maker_note_preview`'s `cut` logic applies as is).
    - `reader::read_preview` / `read_full` on
      `D:\Photos\samples\ORF\E-300_P1252148.ORF` return the 1600x1200
      JPEG at offset 24576, length 414528 (check with `riffle-cli bench`
      through `mise exec -- cargo run -p riffle-cli --release -- bench
      <path>`; it decodes preview and 1:1). Also checked on the E-1,
      E-330, E-400 and E-500 samples, and `riffle-cli scan
      D:\Photos\samples\ORF` reports no more errors than before minus
      those five (the `OLYMP\0` compacts without 0x2020 still fail, as
      before). A few `OLYMPUS\0` / `OM SYSTEM` samples (E-30, E-M1 Mark
      III, OM-1, XZ-10) still give the same `Embedded` as before the
      change. Record the per-body result in `learnings.md`.
    - `EXTRACTOR_VERSION` in `crates/app/src/index.rs` is bumped to 11 and
      the doc comment above it gains "`11` reads the preview of the old
      `OLYMP\0` Olympus MakerNote" in the style of the existing entries.
    - Unit tests in `orf.rs` with synthetic bytes (no sample file): a test
      builder for the old note (`OLYMP\0` + version, IFD at +8, a type 7
      0x2020 entry whose value area is the CameraSettings IFD, all offsets
      absolute) in both file byte orders, asserting `preview` / `full` =
      `(start, length)` **without** the note start added; a
      `PreviewImageValid` 0 old note giving none; an old note with a type
      13 / 4 CameraSettings pointer (absolute) also read; the existing
      tests untouched and passing. `reader.rs` needs no new test unless
      the prefix behavior changes.
    - The module doc of `orf.rs` and the ORF section of
      `docs/agents/raw-metadata-parsing.md` ("Two MakerNote headers,
      note-relative offsets", which currently says the old-style form is
      not supported, and "The preview is the only JPEG") describe the
      third header: absolute offsets, the inline type 7 sub-IFD, the
      1600x1200 / 1280x960 previews, and that the `OLYMP\0` compacts
      without 0x2020 have no preview.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the change inside `crates/core/src/orf.rs` (plus the
      `index.rs` bump and the docs). Minimal shape: `header()` learns a
      third variant (`OLYMP\0` -> IFD at +8, "absolute" flag, no byte
      order of its own); `maker_note_preview` takes the file's `tiff`
      (or `buf` + file byte order) and, for the old header, walks the note
      through a walker with base 0 (`Tiff::with_order(buf, 0, buf.len(),
      file_little_endian)` or the existing `tiff` itself) and uses
      `start` as the absolute offset; for the new headers it keeps the
      note-relative walker and `at + start`. Alternatively express both as
      one code path with a `base` (0 or `at`) and an `ifd` offset; pick
      whichever reads shorter.
    - In `camera_settings_preview`, replace the "type 7 gives none" early
      return with: type 7 and `count > 4` -> the CameraSettings IFD is at
      the entry's value offset (`note.u32(e.value_field)`), the same
      `ifd_entries` walk after; type 13 / 4 count 1 -> as today. Drop the
      "none of the supported ones does" comment.
    - For the synthetic fixture, the existing `maker_note` builder emits
      note-relative offsets. The old-note builder needs absolute offsets,
      so build the file twice: once to find where `OLYMP\0` lands
      (`W::tiff`'s layout is deterministic for the same field lengths),
      then again with the absolute offsets patched in; or compute the
      offset the way `a_note_cut_after_the_camera_settings_ifd_still_gives_the_preview`
      computes `count_at`. Generalize `note_at()` to also find `OLYMP\0`
      if the reader tests want it.
    - Do not add the E-1 / E-300 / E-330 / E-400 / E-500 to `README.md` /
      `docs/cameras.md` (decided at approval, see Trade-offs).

## Trade-offs and risks

- **Listing the old bodies in the README / `docs/cameras.md`.** Decided
  at approval (2026-09-30): fix the parser only, do not list the E-1 /
  E-300 / E-330 / E-400 / E-500; record their results in `learnings.md`.
- **The `OLYMP\0` compacts without 0x2020.** C5050Z, C5060WZ, C7070WZ,
  C8080WZ, E-10, E-20 and the SP-series carry only a 160x120 thumbnail
  (IFD1 0x0201 / note 0x0100); the SP570UZ also has an untagged 640x480
  JPEG at the file's end. Showing a 160x120 thumbnail as the preview would
  be a poor 1:1 view, so they stay "no embedded preview". Opening them
  anyway (IFD1's 0x0201 / 0x0202 as a last resort) is a separate task.
- **`EXTRACTOR_VERSION` bump cost.** Every indexed row of every folder is
  re-extracted once after the bump. The precedent (`8`, an error-row text
  change) accepted that; not bumping would leave the E-300 rows failed
  until the file's mtime changes. Bump.
- **Old note in a big-endian file.** No `MMOR` sample exists locally; the
  fixture covers both byte orders synthetically since the old note simply
  follows the file's order.
- **Prefix behavior.** The old notes are ~3 KB and their previews end
  below 450 KB, so nothing new crosses `HEAD_LIMIT`; the `cut` logic in
  `maker_note_preview` is kept rather than special-cased.

## Progress

- Step 1 done (2026-09-30): the `OLYMP\0` note is read with absolute offsets and an inline type 7 CameraSettings; the E-1 / E-300 / E-330 / E-400 / E-500 samples now give their preview; see learnings.md
