# Learnings: HDR PQ (HEIF) CR3 HEVC preview

## Step 1

- The `PRVW` / `THMB` "version" is the first **byte** of the header, not a
  big-endian u32: the bytes are `01 00 00 00`, so `u32_at(..) == 1` never
  matched and the first real-file run found no preview. `cr3::image` checks
  `buf[payload] == 1`. The JPEG headers start `00 00 00 00`.
- On both HDR PQ samples the version-1 header's length equals the box payload
  minus the 16-byte header, i.e. it spans every box through `IMGD` and the
  trailing `free`, so the HEVC `Embedded` is simply the whole box payload.
  The `PRVW` box ends at 941,064 bytes (R8) and 561,808 (R5 Mark II), inside
  the 1 MiB `HEAD_LIMIT`.
- The four `Canon_EOS_R8_*.CR3` files in `D:\photos\samples\CR3` are JPEG
  CR3s (version-0 headers), so the R8 writes JPEG unless HDR PQ is on; they
  and the other JPEG samples scan with 0 errors and unchanged `Embedded`
  ranges.
- `Arw::hevc` (from #595) was kept as "an `HEVC` track and no `JPEG` one";
  `HEVC_UNSUPPORTED` now only fires when such a file has no readable
  `PRVW` / `THMB`, so the existing tests built from zero-filled images keep
  passing unchanged.
- Cost, release, first cut: decode 65-125 ms, a per-pixel `powf` tone map
  ~220 ms, mozjpeg's default (trellis + progressive scan search) encode
  120-230 ms, so `read_full` took 400-590 ms. Tabulating the PQ EOTF and the
  roll-off + sRGB OETF (4096 steps, linear interpolation, checked within half
  an 8-bit level by a test), running the rows over rayon and encoding with
  `set_fastest_defaults` brought it to 85-125 ms (tone map ~7 ms, encode
  ~17 ms). One-thread `riffle-cli scan`: ~195-230 ms per HDR PQ file against
  ~56 ms for a JPEG CR3.
- hpvcd did not panic on 60 randomly corrupted / truncated `PRVW` streams (it
  returned a frame each time), so the call is not wrapped in `catch_unwind`.
- hpvcd 0.3.2 declares `rust-version = 1.93` (edition 2024) and has no system
  dependencies; its default features are only the SIMD back ends (`neon`,
  `avx`, `sse`), kept on.

## Deferred issues (todo candidates)

- README.md's "CR3 files shot with HDR PQ on (HEIF) hold no JPEG preview, so
  their preview cannot be shown yet" paragraph (and its README.ja.md twin)
  is stale once Step 1 lands, while the new hpvcd credit bullet says the
  previews are decoded. The plan leaves the rewrite to Step 2 on purpose
  (with the `EXTRACTOR_VERSION` bump that re-extracts old error rows).
  Basis: Step 1 implementation. Files: `README.md`, `README.ja.md`.
