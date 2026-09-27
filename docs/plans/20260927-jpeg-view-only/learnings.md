# Learnings

## Step 1: core JPEG reading

- **JPEG thumbnail measurement (release build, Windows, file in the OS
  cache).** `scan::extract` on a 24 MP JPEG (a 6016x4012, 10.3 MB
  JpgFromRaw taken out of a Sigma BF DNG): the plain DCT `1/8` decode gave a
  752x502 thumbnail of **75.5 KB** in ~79 ms. A 7008x4672 α7 V JpgFromRaw gave
  876x584, 78.5 KB, ~62 ms. The ARW path on the same machine is 404x270,
  ~20.4 KB, ~15 ms. That is well above both the 404-px long edge and the
  ~19 KB figure, so the plan's allowed post-decode downsample was added:
  `decode::thumbnail_jpeg_near` decodes at the smallest `n/8` whose long edge
  is not below 404, then box-averages down to 404. After it: 6016x4012 ->
  404x270, **29.3 KB**, ~67 ms; 7008x4672 -> 404x270, 20.7 KB, ~46 ms; two
  5352x4016 / 3384x2538 DxO exports -> 404x304, 31-33 KB, 42-90 ms. The decode
  (plus reading the whole 5-12 MB file) dominates; the resample is cheaper
  than the larger encode it replaces.
- **`thumbnail_jpeg` itself was not changed to pick its scale.** The plan
  said to make `thumbnail_jpeg` pick `n/8` from the source size, but a DNG's
  preview is "the smallest strip JPEG at least 1600 wide", which need not be
  1616 px: any size-based rule other than the fixed `2/8` would change some
  DNG thumbnails and force an `EXTRACTOR_VERSION` bump. So the RAW path keeps
  `thumbnail_jpeg` (fixed `2/8`, byte-identical), and only a `.jpg` / `.jpeg`
  goes through the new `thumbnail_jpeg_near`. Both share one private
  `scaled_thumbnail`, which with `2/8` and no cap runs exactly the old
  operations.
- `jpeg::parse` sets `Arw::preview` / `Arw::full` to `None`: the whole file
  is both, and `reader` hands the file out directly. A caller that inspects
  those fields after `read_metadata` skips a JPEG: the CLI's `bench`, which
  takes explicit paths, measures nothing for a `.jpg` (see the deferred item
  below). The CLI's folder commands list RAW files only.
- `reader::read_metadata` on a JPEG falls back to reading the whole file when
  the 1 MiB head holds no complete Exif segment. That also covers a JPEG
  with no Exif at all, which therefore costs a whole-file read per metadata
  request; only such files pay it.
- `reader.rs`'s test `temp_file` helper now puts the pid before the name, so
  a test file keeps a real `.jpg` extension (`is_jpeg_file` dispatches on
  it).
- The Exif walker is `sequence.rs`'s (`find_exif_tiff`, `Tiff`, `Entry` made
  `pub(crate)`), so `arw.rs` stays little-endian only and untouched. The
  field readers live in `jpeg.rs` and return `None` per bad entry instead of
  failing the parse, unlike `arw::exif`, which errors on an out-of-range
  value.

## Deferred issues (todo candidates)

- **CLI `bench` silently measures nothing for a JPEG path.** Basis: Step 1
  implementation; `jpeg::parse` leaves `Arw::preview` / `Arw::full` `None`,
  and `bench` in `crates/cli/src/main.rs` only times the tiers whose field is
  `Some` after `reader::read_metadata`. Either time `read_preview` /
  `read_full` for a JPEG unconditionally or reject non-RAW paths with an
  error. Files: `crates/cli/src/main.rs`, `crates/core/src/jpeg.rs`.
