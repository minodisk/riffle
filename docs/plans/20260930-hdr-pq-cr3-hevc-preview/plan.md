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

# HDR PQ (HEIF) CR3: decode the HEVC previews

## Purpose

A CR3 shot with HDR PQ on stores HEVC, not JPEG, in `PRVW`, `THMB` and the
first track. `crates/core/src/cr3.rs` `parse` takes an image only when it
starts with a JPEG SOI, so such a file parses with no `preview`, the reader
fails with "no embedded preview" and the strip shows it as failed
(`todo.md` "Core: HDR PQ (HEIF) CR3 files cannot be opened";
`docs/agents/raw-metadata-parsing.md` "HDR PQ (HEIF) files carry no JPEG").
Every EOS R8 sample and half of the R5 Mark II samples on raw.pixls.us are
like this, which is why the R8 is missing from the README list.

The user chose the pure-Rust `hpvcd` crate (crates.io `hpvcd` 0.3.x,
BSD-3-Clause OR Apache-2.0) to decode them. Once done, these files get a
thumbnail, a viewable preview, a sharpness score and face detection like any
other RAW, with colors that look right after tone mapping the PQ / BT.2020
frame to sRGB. Only the `PRVW` / `THMB` images are decoded; the full-size
HEVC image in the first track (a `GRID` of tiles) stays out of scope, and
`full` falls back to the HEVC `PRVW` (the user's decision, see "Trade-offs
and risks").

What a throwaway spike found (2026-09-30, on `D:\photos\samples\CR3\R8.CR3`
and `R5m2.CR3`; the samples are the user's and are never committed):

- `PRVW` (inside the preview `uuid`) starts with a 16-byte header:
  `01 00 00 00 | 00 02 | u16 width 1620 | u16 height 1080 | ff ff |
  u32 data length`, then a run of ISOBMFF-style boxes: `CISZ` (20 bytes;
  coded size 1664x1080), `hvcC` (a standard HEVCDecoderConfigurationRecord
  with `lengthSizeMinusOne` = 3, whose arrays hold the VPS / SPS / PPS),
  `colr` (`nclx`: primaries 9 BT.2020, transfer 16 PQ, matrix 9, full
  range), `pixi` (3 channels of 10 bits), `IMGD` (a u32 total length, then
  4-byte length-prefixed NAL units: four IDR slices for `PRVW`, one for
  `THMB`), sometimes a trailing `free`.
- `THMB` version 1 has the same header shape (`01 00 00 00 | 00 02 |
  u16 320 | u16 214 | ff ff | u32 length`, i.e. the length at 12, not at 8
  as in the JPEG `THMB` that `parse` reads today) and the same boxes; coded
  320x320, visible 320x214.
- The stream is HEVC Main 4:2:2 10 (profile_idc 4, RExt), CTB 32, several
  slices with different slice QPs, no SAO, no tiles / WPP and no
  conformance window: `PRVW` is coded 1664x1088 and must be cropped to the
  header's 1620x1080 (the right / bottom excess is CTB padding).
- `hpvcd::decode_hevc` on an Annex B stream (start code + VPS / SPS / PPS
  from `hvcC`, then the `IMGD` NALs) returns one `VideoFrame`;
  `frame.to_yuv()` gives 16-bit planes (4:2:2, chroma width = w / 2) and
  `frame.signaled_color()` reports BT.2020 / PQ / BT.2020 NCL, full range.
  Release timings: `THMB` ~8 ms, `PRVW` 75-120 ms. The decoded image was
  visually correct on both samples.
- The tone map used in the spike, which looked right but is not tuned:
  full-range YCbCr (BT.2020 NCL) -> R'G'B' -> PQ EOTF (ST 2084) -> nits / 203
  (reference white = 1.0) -> BT.2020 -> BT.709 matrix -> `v / (1 + v / 4) *
  1.25` -> sRGB OETF -> 8 bit.
- `rusty_h265` / `rust_h265` do 4:2:0 only; `oxideav-h265` corrupts
  multi-slice pictures. Neither is usable.

## Steps

- [x] Step 1: Decode HEVC `PRVW` / `THMB` to a JPEG in `riffle-core` so
      every downstream consumer stays unchanged
  - Done when:
    - `cr3::parse` returns a `preview` (and the `THMB` fallback) for an
      HEVC `PRVW` / `THMB`, marked as HEVC, with the range the decoder
      needs; a JPEG `PRVW` / `THMB` gives exactly what it gives today (the
      existing tests pass unchanged).
    - When the first track is not a JPEG and the `PRVW` is HEVC, `full` is
      that HEVC `PRVW` (the reverse of the existing `THMB`-then-`full`
      fallback), so `read_full` returns the 1620x1080 tone-mapped JPEG and
      `z` shows a crop of it with no error. A JPEG CR3's `full` is unchanged.
    - `reader::read_preview` (and `read_full`) on an HDR PQ CR3 returns JPEG
      bytes of the header's visible size (1620x1080 for `PRVW`), tone-mapped
      to sRGB, through the bounded prefix and the ranged-read / whole-file
      retry paths as for a JPEG `PRVW`.
    - A new `crates/core/src/hevc.rs` is the only place that names `hpvcd`.
    - Unit tests without real samples cover: the `PRVW` / `THMB` box walk
      (`hvcC` arrays and `IMGD` NAL units into an Annex B stream, on
      synthetic bytes, asserting the byte stream), the version-1 header
      (width / height / length offsets), `cr3::parse` marking an HEVC image
      (and aliasing `full` to it) and still refusing garbage, and the tone
      map (PQ black -> 0, PQ code of 203 nits on the neutral axis -> sRGB
      white or close to it, monotonic in luminance, chroma neutrality of a
      gray). An `#[ignore]` test decodes a real file named by an env var
      (follow `faces.rs`'s `RIFFLE_FACE_JPEG` pattern, e.g.
      `RIFFLE_HEVC_CR3`) and checks the JPEG's size and that the picture is
      not all one color.
    - `cargo test -p riffle-core` and `mise run ci` pass; `README.md` and
      `README.ja.md` credit hpvcd next to the YuNet / Lucide notices
      ("(BSD-3-Clause OR Apache-2.0)"), `CLAUDE.md`'s layout paragraph names
      `src/hevc.rs`, and `docs/agents/raw-metadata-parsing.md` replaces the
      "HDR PQ (HEIF) files carry no JPEG" section with what the format is
      and how it is decoded (the box layout above, the coded-vs-visible
      crop, the version-1 `THMB` header, the `full` fallback, the timings).
  - Implementation approach (as far as it is known; omit if unknown):
    - Integration point: the reader, not the app. `reader::embedded_from`
      already turns an `Embedded` into the bytes every consumer gets
      (`scan::extract` -> `thumbnail_jpeg`, `score_preview`,
      `detect_around`; the app's `read_preview` command -> the UI's
      `image/jpeg` Blob; `mcp.rs` -> `preview_jpeg`; the CLI's
      `decode_rgb`). After it has the embedded bytes (sliced from the
      prefix or read by range), when the `Embedded` is HEVC it calls
      `hevc::to_jpeg(bytes) -> Result<Vec<u8>>` and returns that JPEG. So
      no consumer, the frontend, the index schema or the thumbnail cache
      format changes. Alternative not taken: a new pixel path (RGB out of
      the reader) would touch `scan.rs`, `sharpness.rs`, `faces.rs`,
      `partial.rs`, three app commands and the UI's Blob handling.
    - Marking: `arw::Embedded { offset, length }` needs to say which codec
      the bytes are, e.g. a `codec: Codec` field (`Jpeg` / `Hevc`) with
      the three existing constructors (`arw.rs` x2, `nef.rs`, `cr3.rs` x2)
      and the tests updated, or an equivalent on `Arw`. Keep `Copy`. Check
      `commands.rs` / `mcp.rs` / the CLI for pattern matches on `Embedded`
      before choosing the shape.
    - The HEVC `Embedded` range: cover the whole `PRVW` / `THMB` payload
      including the 16-byte header (so `hevc::to_jpeg` reads the visible
      width / height itself and needs no extra fields), or the data after
      the header plus the size in the marker. Either way `parse` must read
      the version-1 header's length at 12 for `THMB` (the JPEG `THMB`
      keeps 8) and must not demand the data itself in `buf`: the header
      must be in the prefix, the boxes may run past it, exactly as the
      JPEG case (`a_prefix_holding_the_prvw_header_parses`). Verify on the
      samples that the header's data length spans all the boxes up to and
      including `IMGD` (or the `free`).
    - `full` fallback: `partial::decode_focus_crop` and the app's zoom path
      read `full` through the same reader, so aliasing `full` to the HEVC
      `PRVW` `Embedded` needs no change outside `cr3.rs` / the reader. Check
      that nothing treats `full == preview` as an error or skips it.
    - `hevc.rs`: (1) walk the boxes with the same 8-byte big-endian header
      shape `cr3.rs` uses (reuse or mirror `header` / `children`); (2) from
      `hvcC` take the arrays (skip the 22-byte fixed part, then
      `numOfArrays`, each `array_completeness/NAL_unit_type`, `numNalus`,
      per NALU `u16 length` + bytes) and from `IMGD` the 4-byte
      length-prefixed NALs; (3) build Annex B (`00 00 00 01` before each
      NAL) and call `hpvcd::decode_hevc`, taking the first frame (the docs
      also say a length-prefixed stream is auto-detected; Annex B is what
      the spike verified, use it); (4) `to_yuv()` -> tone map -> crop to
      the visible size -> `mozjpeg::Compress` at the preview's size, RGB,
      as `decode::preview_jpeg` encodes (baseline; quality can be the
      `PREVIEW_QUALITY` / `THUMBNAIL_QUALITY` level used elsewhere; pick
      one and say why in a comment). Wrap the hpvcd call in
      `catch_unwind` like `decode_rgb` does for mozjpeg only if hpvcd is
      seen to panic on bad input; otherwise map its error.
    - Tone map: implement the spike's chain in plain Rust (f32 per pixel;
      1620x1080 is 1.7 Mpx so a straightforward loop is fine; no SIMD
      work). Use `signaled_color()` only to assert / fall back: the Canon
      files are always BT.2020 / PQ / full range, so if the frame signals
      something else, still decode with the same chain (log nothing; core
      has no logger) rather than fail. Keep the constants in one place
      with the ST 2084 / BT.2020 references in comments.
    - Prefix reads: `PRVW` ends by ~940 KB on the measured JPEG bodies;
      measure where the HEVC `PRVW` ends on both samples and record it in
      the guide (it decides whether the 1 MiB `HEAD_LIMIT` still avoids
      the ranged read).
    - Cost: ~100 ms per `read_preview` of an HDR PQ file in release, paid
      by `extract`, again by `extract_faces` (it re-reads the preview), per
      page turn and per `z`; JPEG CR3s pay nothing new. Measure the release
      scan cost on the two samples with `riffle-cli scan` and note it in
      `learnings.md` / `docs/performance.md` only if it is surprising; no
      cache is added in this plan.
    - Cargo: add `hpvcd = "0.3"` to `crates/core/Cargo.toml` with default
      features unless a smaller feature set decodes HEVC alone; check the
      crate's MSRV against `mise.toml` (`rust = "1.97"`) and the build on
      the three CI targets (the crate is pure Rust with SIMD, so no system
      library; confirm there is no `unsafe`-heavy feature gate that fails
      on aarch64 macOS).
    - Docs in this step: `README.md` / `README.ja.md` license notice line
      (the two stay in sync), `CLAUDE.md` layout, `raw-metadata-parsing.md`
      CR3 section. Do not touch the device list or the HEIF limitation
      sentence yet (Step 2, once the app shows the files).
- [ ] Step 2: Re-extract the index and update what the user reads
  - Done when:
    - `EXTRACTOR_VERSION` in `crates/app/src/index.rs` is bumped to `8`
      with its doc comment extended ("`8` decodes the HEVC `PRVW` / `THMB`
      of HDR PQ CR3 files"), so the error rows of these files are
      re-extracted on the next scan (the index keeps error rows as
      authoritative; only a bump re-extracts them).
    - `README.md` and `README.ja.md`: the CR3 device list gains
      `Canon EOS R8`, and the sentence "CR3 files shot with HDR PQ on (HEIF)
      hold no JPEG preview and cannot be opened yet" (and its Japanese
      twin) becomes a statement of what is supported: the preview is
      decoded and tone-mapped from HEVC, and the 1:1 view (`z`) shows a crop
      of that 1620x1080 preview rather than the full-size image.
      `docs/cameras.md` gains the EOS R8 row (fill the four columns from the
      sample's `AFInfo2` / `SubSecTimeOriginal` as the other Canon rows were
      filled).
    - `todo.md`: the "Core: HDR PQ (HEIF) CR3 files cannot be opened"
      section is removed (its decision is made and implemented).
    - `mise run ci` passes.
    - Pending manual check (listed in the PR body, done by the user):
      open `D:\photos\samples\CR3` in a release build
      (`mise run tauri:release:devtools`); `R8.CR3` and `R5m2.CR3` show a
      correctly colored thumbnail and preview, the meta pane's EXIF rows,
      and the strip's sharpness bar; pressing `z` shows a (soft) crop of the
      preview with no error in the status line.
  - Implementation approach (as far as it is known; omit if unknown):
    - Assumes Step 1 is merged.
    - The `EXTRACTOR_VERSION` doc comment's rule is explicit: any change to
      which preview bytes `reader.rs` returns needs a bump. The bump costs
      every user one full re-extraction of each folder on its next open;
      that is the documented mechanism and there is no per-format bump, so
      accept it.
    - If the R8 sample's `AFInfo2` shows a point, the `docs/cameras.md`
      row is `✓ | ✓ | – | ✓`-shaped like the other EOS rows; check it
      rather than copy it.

## Trade-offs and risks

- **The full-size image (`full`) for an HDR PQ CR3.** Its track is an HEVC
  `GRID` of tiles at the sensor size (24 MP on the R8); decoding it in
  pure Rust would take seconds per `z` press and needs a tile-assembly
  path nothing else in the codebase has, so it stays out of scope. Decided
  (user, 2026-09-30): `full` is set to the HEVC `PRVW`, so `read_full`
  returns the 1620x1080 tone-mapped JPEG, `partial::decode_focus_crop` cuts
  the crop out of it, and `z` shows a soft, upscaled crop with no error. It
  costs one more ~100 ms decode per zoom and can mislead about sharpness at
  1:1. The option not taken was leaving `full` = `None`, which makes `z`
  show "no embedded JpgFromRaw" in the status line.
- **Where the decode lives.** Chosen: inside `reader::embedded_from`, so
  the `(Arw, Vec<u8>)` contract "the bytes are a JPEG" stays true for
  every consumer. The price is that `read_preview` now does CPU work for
  one file kind, and that `extract` + `extract_faces` decode the same
  preview twice per scan (~0.2 s per HDR PQ file in release, on the scan
  thread pool). If a folder of thousands of HDR PQ files turns out to be
  a real use, a decoded-preview cache would be a follow-up, not this
  plan.
- **`Embedded` gains a codec.** Adding a field to a `Copy` struct that
  three parsers construct is a small, mechanical change, but the
  alternative (a parallel `Option<HevcPreview>` on `Arw`) leaves two
  sources of truth for "what is the preview". Prefer the field; the
  implementer may choose the other if pattern matches in the app make
  the field awkward.
- **Tone map fidelity.** The spike's curve (`v / (1 + v / 4) * 1.25`
  after normalizing 203 nits to 1.0) is a visual guess, not Canon's
  rendering; highlights above ~4x reference white compress, and colors
  outside BT.709 clip after the matrix. The acceptance is "correctly
  colored" by the user's eye on two samples. If they judge it off, tune
  the constants in `hevc.rs` (they are isolated on purpose) rather than
  change the pipeline.
- **hpvcd is a young crate (0.3.x).** Its API may move; that is why it
  is confined to `hevc.rs`. A stream it cannot decode (a future body
  writing 4:2:0 or tiles) surfaces as the file's extraction error, the
  same as a corrupt JPEG today. HEVC is patent-encumbered; the user
  accepted the pure-Rust decoder, and the docs make no claim either way.
- **No synthetic HEVC stream in tests.** hpvcd has no encoder, so the
  decode itself is only exercised by the `#[ignore]` env-var test on a
  real file (never committed). The unit tests therefore stop at the
  Annex B bytes and the tone map math; CI does not decode HEVC.
- **`THMB` version 1 header offsets** differ from the version 0 (JPEG)
  header `parse` reads today (length at 12 vs 8); reading the wrong slot
  on a JPEG `THMB` would break the existing fallback, so branch on the
  header's first u32 / version byte and keep the existing tests green.

## Progress

- (2026-09-30) Step 1 complete
