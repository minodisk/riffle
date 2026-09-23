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

# Mask SHORT-typed TIFF entries to 16 bits

## Purpose

TIFF left-justifies a count-1 `SHORT` in the entry's 4-byte value field; the
remaining 2 bytes are padding with no meaning. SIGMA fp L DNGs write nonzero,
per-file garbage there (observed `Compression=0xFFFF0007`,
`PhotometricInterpretation=0x00300006` / `0x4D470006` / `0x2D470006`).
`integer()` in `crates/core/src/arw.rs` returns the raw `u32` for
`TYPE_SHORT`, so `strip_jpeg()` rejects the JPEG IFDs: SDIM0521.DNG picks the
640x480 JPEG instead of the 9520x6328 full-size one in SubIFD1, and
SDIM0558.DNG finds no JPEG at all (no preview in the app's strip).

Once fixed, SIGMA fp L DNGs get the right preview and full-size JPEG, and the
README can list SIGMA BF (already works: preview 1620x1080, full 6016x4012)
and SIGMA fp L as supported DNG cameras.

## Steps

- [x] Step 1: Mask count-1 `SHORT` values to 16 bits in the ARW/DNG parser, add a regression test, and list the SIGMA cameras in the README
  - Done when:
    - `integer()` in `crates/core/src/arw.rs` returns only the low 16 bits for
      `TYPE_SHORT` entries (the parser is little-endian only, so the SHORT is
      the low half of the raw `u32`); `TYPE_LONG` is unchanged.
    - A unit test in the `tests` module of `crates/core/src/arw.rs` builds a
      DNG-style TIFF (via `tiff_with_sub_ifds` / `strip_entries`) whose
      `Compression` and `PhotometricInterpretation` SHORT entries carry
      nonzero high-16-bit padding, and asserts the strip JPEGs (preview and
      full) are still found with the expected offsets/lengths/dimensions.
      The test must fail before the fix and pass after.
    - `README.md` "RAW formats and cameras" DNG list gains `- [x] SIGMA BF`
      and `- [x] SIGMA fp L` next to `Leica M11-P`.
    - `mise run ci` passes.
    - Manual check (sample files live outside the repo; do not commit them):
      `./target/release/riffle-cli info "/mnt/c/Users/daisu/Downloads/AmazonPhotos (1)/SDIM0521.DNG"`
      reports full = length 28363616 at offset 60549448, and the same command
      on SDIM0558.DNG reports preview and full as `Some`.
  - Implementation approach:
    - Only `integer()` needs the mask; it is the sole reader that hands a raw
      `SHORT` value back as `u32`. Its callers are `strip_jpeg()`
      (Compression, Photometric, and the LONG strip/size tags), `TAG_ISO`
      in `exif()`, `TAG_ELECTRONIC_FRONT_CURTAIN_SHUTTER` from the Sony
      MakerNote, and `leica_focus_distance()` (LONG). Masking inside
      `integer()` fixes all of them at once, including ISO on files with
      padded SHORTs.
    - Audit result for the other SHORT readers, so they need no change:
      orientation (`find(&ifd0, TAG_ORIENTATION).map(|(v, _)| v as u16)`)
      already truncates to the low 16 bits; `byte()` casts to `u8`;
      `shorts::<N>()` reads from the value offset (N > 2); `embedded()`
      reads LONG tags only; `TAG_SUB_IFDS` is LONG. Do not touch them
      (surgical change).
    - Keep the doc comment on `integer()` accurate: mention that a SHORT
      occupies the low two bytes of the value field and the rest is
      padding.
    - Test naming and style follow the existing tests in the module (e.g.
      `no_embedded_jpeg_at_all_parses_with_both_none`). Build the entries by
      OR-ing garbage into the high half, e.g.
      `0xFFFF_0000 | COMPRESSION_JPEG` and `0x4D47_0000 | PHOTOMETRIC_YCBCR`,
      rather than adding a new helper.
    - Commit as `fix(core): mask SHORT TIFF entries to 16 bits` (Conventional
      Commits, English); the README line can ride in the same PR since it is
      the user-visible outcome of the fix.

## Trade-offs and risks

- Masking scope: masking only in `integer()` is the minimal fix; an
  alternative is to normalize SHORT values once in `read_ifd()` so every
  downstream reader sees a clean `u16`. That would touch the `Entry` layout
  used by `shorts()` (which needs the raw offset for count > 2) and the test
  helpers, so it is more invasive for no current benefit. Chosen: mask in
  `integer()` only.
- Orientation reads through `find(...).map(|v| v as u16)` and is already
  correct by accident of the cast. Left as is.
- Count-2 SHORT entries (both halves meaningful, inline) are not read anywhere
  today; no handling is added.

## Progress

- (none yet)
