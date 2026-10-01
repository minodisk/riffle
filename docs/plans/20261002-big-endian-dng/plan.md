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

# Open big-endian (`MM`) DNGs

## Purpose

`riffle-cli check D:\Photos\samples` (the sample run recorded in
`../_archived/20261001-cli-check-samples/learnings.md`) fails 42 DNGs at every
stage with `not a little-endian TIFF/ARW`: the Pentax K-series and 645D,
Samsung GX10 / GX20, Ricoh GR / GR II / GR DIGITAL 2 and 4 / GXR / GX200,
Leica M (Typ 240) and M Monochrom (Typ 246), the iPhone 6s Plus through
13 Pro Max, the Canon EOS 350D DNG, the OpticFilm 8100 / 8200i scans, the
Blackmagic Pocket Cinema Camera and `One_IMG_20150729_201116.dng`. All 42
start with `MM` (checked by reading the first two bytes of every file in
the folder; exactly 42 do). A DNG goes through `crates/core/src/arw.rs`,
whose `parse` rejects anything but `II` and whose every multi-byte read
(`u16le`, `u32le`, `value.to_le_bytes()`, `value & 0xFFFF`, `value as u8`)
assumes little-endian. The other TIFF-shaped parsers (`nef.rs`, `orf.rs`,
`jpeg.rs` via `exif.rs`) already read both orders through
`sequence::Tiff`. Once `arw.rs` follows the header's byte order, these
cameras' DNGs open in Riffle (or fail for a concrete, file-specific reason
such as `no embedded preview`), and the todo.md item
"Core: big-endian DNGs do not open" can be closed.

## Steps

- [x] Step 1: Make `arw::parse` follow the TIFF header's byte order, bump
      `EXTRACTOR_VERSION`, record the finding, clear the todo
  - Done when:
    - `cargo test -p riffle-core` and `mise run ci` pass.
    - `arw::parse` accepts an `MM` header (magic 42) and reads every IFD
      entry, inline value, offset value, SubIFD offset array, RATIONAL,
      `SHORT[N]` and MakerNote IFD in the header's byte order; an `II`
      file reads exactly as before.
    - Unit tests in `arw.rs` cover a big-endian container: IFD0
      orientation (an inline `SHORT` sits in the **high** two bytes of a
      big-endian value field, so the `& 0xFFFF` mask of
      `short_entries_ignore_the_padding_in_their_high_half` becomes
      `>> 16` for `MM`; same for an inline `BYTE`, which is the top byte),
      an inline ASCII of 4 bytes or fewer (`SubSecTimeOriginal`), an
      ASCII / RATIONAL at an offset, the DNG strip-JPEG selection over a
      `SubIFDs` array, and the Leica note read through a big-endian IFD.
      `rejects_big_endian_header` is replaced (not just deleted) by the
      positive test.
    - The `inv.DNG` case of
      `reader::tests::every_format_hands_out_jpegs_without_an_exif_rotation`
      is a real `MM` container (today `arw_with_both` always writes `II`;
      only its embedded JPEGs' Exif is `le`-parametrized).
    - `crates/app/src/index.rs` `EXTRACTOR_VERSION` is bumped from 11 to
      12, because the index has cached these files' parse errors and only
      retries a row when the version changes (see
      `docs/agents/tauri-app.md`, "Bump `EXTRACTOR_VERSION`, not
      `SCHEMA_VERSION`").
    - `mise x -- cargo run -q -p riffle-cli --release -- check D:\Photos\samples`
      was run before and after the change on the same tree, both outputs
      saved in this plan folder (`check-before.txt`, `check-after.txt`),
      and the diff shows: no `not a little-endian TIFF/ARW` line remains;
      every one of the 42 either disappears from the failure list or
      fails with a different, concrete message; and the non-DNG lines
      (arw / cr3 / jpg / jpeg / nef / orf / raf) are identical, as are the
      DNG lines of files that already opened. The `learnings.md` lists
      which of the 42 now open at which stages and the new message of
      each that still fails.
    - ExifTool (13.59, installed at
      `C:\Users\daisu\AppData\Local\Programs\ExifTool\ExifTool.exe`; open a
      fresh shell or refresh `PATH` if `exiftool` is not found) cross-checks
      the fields Riffle reads (orientation, `DateTimeOriginal`, f-number,
      exposure time, ISO, focal length, make / model, and the M240
      `FocusDistance`) on at least one Pentax, one iPhone and the Leica
      M (Typ 240) `MM` sample; the comparison is recorded in `learnings.md`.
    - `docs/agents/raw-metadata-parsing.md` gets a new "TIFF structure"
      entry (tagged Hit, in the guide's existing format: what breaks, the
      rule, the source link to this plan's `learnings.md`) saying that
      `arw.rs` reads both byte orders, which bodies write `MM` DNGs, and
      the inline-value rule for `MM` (SHORT in the high half, BYTE in the
      top byte, ASCII bytes in file order). The intro paragraph's
      description of `arw.rs` and the Leica entry ("a plain little-endian
      IFD at note offset 8") are corrected: the M (Typ 240) / M Monochrom
      (Typ 246) note is `LEICA\0 02 ff` followed by a big-endian IFD in
      a big-endian DNG (bytes at the note start of both samples:
      `4c45 4943 4100 02ff 001f 0300 0007 ...`), while the M10 / Q2 note
      is `LEICA\0 02 00` + little-endian in a little-endian DNG, so the
      note follows the container's order and `leica_focus_distance` must
      read it that way.
    - The "Core: big-endian DNGs do not open" heading and its body are
      removed from `todo.md`.
  - Implementation approach:
    - Keep the change inside `arw.rs` (plus the fixture in `reader.rs`
      and the version bump): no new module, no change to `reader.rs`'s
      dispatch, no change to the `Arw` / `Shot` / `Embedded` types or to
      `nef.rs` / `orf.rs` / `exif.rs`.
    - Mechanism (decided): build a `sequence::Tiff::new(buf, 0, buf.len())`
      at the top of `parse` (it already accepts `II` / `MM`, checks magic
      42 and range-checks every read; with base 0 its relative offsets
      are `arw.rs`'s absolute ones) and read through its `u16` / `u32` /
      `bytes` in place of `u16le` / `u32le`. The `Entry` tuple
      `(tag, value, typ, count)` can stay: `value` is the value field
      decoded in the file's order, and the three places that reinterpret
      it inline (`ascii` for `count <= 4`, `integer` for a `SHORT`,
      `byte`, and `sigma_af_point`'s `SHORT[2]`) need the order to pick
      the right bytes. The `anyhow` error strings of the existing
      range checks (`IFD offset out of range`, `SubIFD offset array out
      of range`, ...) should stay as they are, since
      `reader::read_embedded` retries a bounded prefix on any error and
      the sample `check` output is compared on them.
    - Change the header check's message so it no longer says
      "little-endian"; `Tiff::new` already yields `invalid TIFF byte
      order` / `invalid TIFF magic number`.
    - `maker_note_ifd` (Sony), `leica_focus_distance` and `sigma_af_point`
      read the note's IFD in the container's order. Sony ARWs and Sigma
      DNGs are `II` so nothing changes for them; the Leica M240 / M246
      samples are the only big-endian notes and are big-endian inside
      (see above). Do not add a per-note byte-order sniff.
    - Test fixtures: the existing `arw.rs` helpers (`ifd`, `tiff`,
      `tiff_with_exif`, ...) write `II` with `to_le_bytes`; leave them
      and add the `MM` cases with `crate::jpeg::tests::W(false)` (its
      `tiff(&[fields], &[exif fields])`, `short`, `long`, `ascii`,
      `rational` already produce either order and are what `exif.rs`,
      `nef.rs`, `orf.rs` tests use), or with a small order-parametrized
      variant of the local helpers if `W` cannot express a SubIFD / strip
      layout. Do not rewrite every existing test to loop over both orders.
    - The DNG preview / full selection (`strip_jpeg`, the size-based pick
      in `parse`) is unchanged in logic; the after-run `check` output is
      what verifies it on real `MM` files. Record in `learnings.md` which
      of the 42 still fail and why (expected: some Pentax / scanner /
      Blackmagic files carry no JPEG strip and report `no embedded
      preview`, which is in scope of the camera-support rule in auto
      memory, not of this plan).

## Trade-offs and risks

- Mechanism: adopting `sequence::Tiff` inside `arw.rs` reuses the
  existing walker (consistent with `nef.rs` / `orf.rs`) but touches most
  function signatures in the file (`buf: &[u8]` becomes `tiff: &Tiff`
  or both). Chosen over a local `Order` helper, which would add a second
  byte-order reader to the crate. Neither changes behaviour for `II` files.
- A full rewrite of the DNG path as its own `dng.rs` over `Tiff` +
  `exif.rs` (as `nef.rs` is) was not chosen: it would duplicate the DNG
  strip selection, the Leica and Sigma MakerNote reads and the
  `PREVIEW_MIN_WIDTH` logic, and `reader.rs` cannot tell a Sony ARW
  from a DNG by anything but extension, so `arw.rs` would still need to
  stay for `.arw`.
- One PR: the guide entry depends on the after-run numbers from the same
  change, so the docs / todo edits ride with the parser change.
- Bumping `EXTRACTOR_VERSION` re-extracts every indexed folder's first
  pass (thumbnails and metadata) on the next open; this is the documented
  cost of any extraction change and is unavoidable, since the failed
  rows for these DNGs are otherwise never retried.

## Progress
