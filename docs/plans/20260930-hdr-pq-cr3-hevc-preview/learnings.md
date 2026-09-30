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

## Step 2

- main was already at `EXTRACTOR_VERSION` 8 (the HDR PQ CR3 error message of
  `20260930-heif-cr3-message`), so the bump went to `9`, not `8` as the plan
  first said; the plan text was adjusted.
- The EOS R8 row in `docs/cameras.md` comes from `riffle-cli info` on
  `D:\photos\samples\CR3\R8.CR3` (the HDR PQ sample) and the five JPEG R8
  samples: every one has a `focus` point in a 6000x4000 frame and a
  `SubSecTimeOriginal`; `cr3::parse` sets `focus` and `focus_frame` together
  from `AFInfo2`, so the row is `✓ | ✓ | – | ✓`.
- `docs/raw-formats.md` also said Riffle "has no HEVC decoder" for HEIF CR3s;
  its sentence was updated with the README paragraph (user-facing, not named
  in the plan).
- The deferred README item from Step 1 is resolved by this step.

## Deferred issues (todo candidates)

- README.md's "CR3 files shot with HDR PQ on (HEIF) hold no JPEG preview, so
  their preview cannot be shown yet" paragraph (and its README.ja.md twin)
  is stale once Step 1 lands, while the new hpvcd credit bullet says the
  previews are decoded. The plan leaves the rewrite to Step 2 on purpose
  (with the `EXTRACTOR_VERSION` bump that re-extracts old error rows).
  Basis: Step 1 implementation. Files: `README.md`, `README.ja.md`.
- Resolved in Step 2: the README / README.ja.md HDR PQ paragraph above was
  rewritten.
- The removed `todo.md` section "Core: HDR PQ (HEIF) CR3 files cannot be
  opened" carried one open item that is not HDR-PQ-specific: "Keep the EXIF
  of a file whose extraction failed: an error row stores no metadata
  (`Entry` / `write_batch` in `crates/app/src/index.rs`), so bursts, the
  filter menu and capture-time order do not see the file, although
  `read_metadata` parses it." It still applies to any file whose extraction
  fails (a corrupt JPEG, an HEVC stream hpvcd cannot decode). Basis: Step 2
  removal of the todo section, as the plan required. Files:
  `crates/app/src/index.rs`, `todo.md`.
- `todo.md`'s "App: real-device check of the HDR PQ (HEIF) CR3 message in
  the strip, viewer, meta pane and sidecars" is obsolete: `R8.CR3` and
  `R5m2.CR3` no longer fail, so the message it asks to check no longer
  appears for them. It should be removed (or replaced by the pending manual
  check below) at wrap-up. Basis: Step 1 / Step 2 implementation. Files:
  `todo.md`.
- Pending manual check (the user's; Step 2's checkbox was ticked on the
  automated criteria, `mise run ci`): on Windows, build with
  `mise run tauri:release:devtools` and open `D:\photos\samples\CR3`
  (samples never committed). Expected: `R8.CR3` and `R5m2.CR3` show a
  correctly colored thumbnail and preview (tone-mapped from HDR PQ, not
  washed out or tinted), the meta pane's EXIF rows, and the strip's sharpness
  bar; if the folder was scanned before, the old error rows re-extract once
  (`EXTRACTOR_VERSION` 9); pressing `z` shows a (soft) crop of the 1620x1080
  preview with no error in the status line. If the colors look off, tune the
  tone map constants in `crates/core/src/hevc.rs`.
