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

# Orientation-neutral preview JPEGs

## Purpose

A portrait RAF (e.g. `D:\Photos\samples\RAF\GFX_100_fujifilm_gfx_100_13.raf`,
GFX 100, Orientation 8) shows upright in the filmstrip but sideways in the
main preview. The `preview` command hands the frontend the embedded JPEG bytes
as-is plus the Orientation in the payload header; the RAF's embedded JPEG
carries a full Exif APP1 with its own Orientation, so `createImageBitmap`
(WebView default `imageOrientation: "from-image"`) rotates it once at decode
and `draw()` in `main.ts` rotates it again by the header value. The original
design (archived `20260917-tauri-skeleton` plan) assumed the IFD0 preview is
a bare JPEG stream without Exif; that holds for ARW but not for RAF, nor for a
plain JPEG (`read_jpeg` returns the whole file), and possibly other formats'
embedded JPEGs. The filmstrip is fine because the index thumbnail is
re-encoded without Exif. The zoom crop (`focus_crop`) is unaffected: it ships
RGBA pixels, not a JPEG.

The fix is structural: every JPEG that leaves `riffle_core::reader` passes
one choke point that rewrites its Exif Orientation to 1, so the header
orientation is the single source of truth for every format, current or
future, and a parser added later is covered without anyone remembering to.

## Steps

- [x] Step 1: Neutralize the Exif Orientation of every JPEG the reader hands out, with an all-format invariant test and a guide entry
  - Done when:
    - `riffle_core::jpeg::neutralize_orientation(buf: &mut [u8])` (name to taste) rewrites IFD0 tag `0x0112` to 1 in place, in the TIFF's own byte order; a JPEG with no Exif, a malformed one, or no Orientation entry is left untouched (no-op, never an error).
    - `reader::read_preview` and `reader::read_full` apply it on their one return path, so the JPEG of every container (`read_jpeg`, `read_embedded` incl. the HEVC-decoded CR3) is covered; `Arw.orientation` is unchanged.
    - Rust unit tests: the neutralizing function on a synthetic JPEG with Orientation 8 in both byte orders (`jpeg::tests::{plain_jpeg, with_exif, W}`), on a JPEG without Exif, and with the Orientation entry absent; and one reader test enforcing the invariant across every supported format: ARW, DNG, NEF, CR3, HDR PQ CR3 (HEVC-converted), RAF, ORF and plain JPEG, asserting for each of `read_preview` / `read_full` that `jpeg::read_exif(&out)` yields orientation 1 (or `Err` for no Exif) while `a.orientation` is the fixture's 6 or 8.
    - Existing byte-equality tests updated to the neutralized bytes: `reader.rs` `a_jpeg_is_its_own_preview_and_full_jpeg`, `a_raf_reads_its_jpeg_by_range_past_the_prefix`, `a_raf_whose_exif_is_past_the_prefix_reads_the_whole_file`; `commands.rs` `a_jpeg_is_served_whole_with_its_exif_and_a_centered_crop` (rename if its name no longer fits).
    - `docs/agents/tauri-app.md` gets an entry recording that the preview payload's JPEG is orientation-neutral by construction, why (RAF / JPEG carry Exif Orientation; WebView applies it at decode; `imageOrientation` options are not relied on because behavior differs across WebView2 / WKWebView / WebKitGTK and `"none"` is deprecated in Chromium), and that the header orientation is the single source of truth.
    - Manual check: the GFX 100 sample above displays upright in the preview, the zoom crop and the compare view, same as the filmstrip; an ARW folder and a JPEG-only folder still display as before.
    - `mise run ci` passes.
  - Implementation approach:
    - Rewrite, do not strip: changing the 2-byte value keeps every offset valid and the rest of the Exif intact, and needs no re-encode. Locate the TIFF with `sequence::find_exif_tiff`, build `Tiff::new`, read IFD0 at `tiff.u32(4)`, walk `ifd_entries`, and for the entry with `tag == 0x0112`, `typ == 3` (SHORT), `count == 1`, write `1u16` at `base + entry.value_field` in the TIFF's byte order. `Tiff` keeps `little_endian` private, so add a small `pub(crate)` accessor (or return the absolute offset and order from a helper) rather than re-parsing the byte-order mark by hand. Ignore any other IFD (the thumbnail IFD1's Orientation, if any, applies to the Exif thumbnail, which nothing here decodes).
    - Apply in `reader.rs` at one place: both `read_preview` and `read_full` end in `read_jpeg` or `read_embedded`; wrap them so the `(Arw, Vec<u8>)` goes through the neutralizer before returning (e.g. a private `fn neutral(r: Result<(Arw, Vec<u8>)>) -> Result<(Arw, Vec<u8>)>` both public functions map through). Do not touch `read_metadata`, which returns no bytes.
    - Consumers that decode with the image / mozjpeg crates (`scan`, `sharpness`, `faces`, `decode::preview_jpeg` for MCP, the CLI) ignore Exif and take `Arw.orientation`, so they are unaffected; no change in `crates/app/src/commands.rs` beyond the test, and none in the frontend (`worker.ts`, `main.ts`).
    - Invariant test fixtures: `reader.rs` tests already build every container (`arw_with_preview`-style ARW, NEF, CR3 incl. the HEVC one, RAF via `raf::tests::{exif_jpeg, raf}`, ORF); feed an Exif-bearing JPEG with Orientation 6 or 8 (`with_exif(&plain_jpeg(..), &w.tiff(&[w.short(0x0112, n)], &[]))`) as the embedded body where the container fixture takes an arbitrary body, and for DNG use the ARW fixture under a `.dng` name (both go through `arw::parse`). The HEVC CR3 output is a fresh mozjpeg JPEG with no Exif, so its assertion is the `Err` arm.
    - Guide entry goes in `docs/agents/tauri-app.md` near the preview payload / worker notes (the "Workers: declare the scope locally" area); the stale assumption is only in the archived plan, which is left as is.
    - Commit: `fix(core): hand out orientation-neutral preview JPEGs` (or similar).

## Trade-offs and risks

- Choke point: core reader (chosen) vs. the app's `payload()` builder. `payload()` is the only place JPEG bytes are wrapped for the frontend, so it would also be a single point, but it leaves the CLI and any future core consumer with orientation-bearing bytes and sits one crate away from where new parsers are added; the reader covers every caller of every format automatically. A newtype (`OrientedJpeg` constructible only through the neutralizer) would add type-level enforcement but touches every signature in core, app and CLI for one invariant the reader-level test already pins; not worth it now.
- Rewrite vs. strip the APP1 segment: stripping also works and would remove the Exif thumbnail, but shifts offsets and needs a segment splice; the in-place rewrite is two bytes and keeps the bytes otherwise identical, which also keeps most existing tests meaningful. If a file ever carried its Orientation in a non-SHORT entry, the rewrite would skip it; treat that as out of scope unless a sample shows it.
- Side effects on the CLI: `riffle-cli` commands that write the preview / full JPEG out to disk now write Orientation 1 into the copy; the file's own orientation is still printed from `Arw`. Note it in `learnings.md` if any CLI doc describes the dump as byte-identical.
- The neutralizer runs on every `read_preview` (each scan entry): it walks only the APP1 IFD0 entries of a buffer already in memory, so the cost is negligible, but confirm no measurable change in the `scan` benchmark if in doubt.

## Progress

- Step 1: done. `reader::neutral` rewrites the Orientation of every JPEG `read_preview` / `read_full` hand out, with the all-format invariant test and the `docs/agents/tauri-app.md` entry. Deviation: the HDR PQ CR3 (HEVC) case is an `#[ignore]` test driven by `RIFFLE_HEVC_CR3`, since it needs a real sample. The manual check (GFX 100 sample upright in preview, zoom crop and compare view) is still pending, as `learnings.md` records.
- (2026-09-30) Step 1 complete
