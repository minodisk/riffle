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

## Step 2: app backend for a JPEG-only folder

- **Preview page-time measurement: backend-only, the GUI was not run.** The
  implementation agent cannot drive the desktop app, so the `Timing logs`
  `page invoke= decode= total=` figures were not taken. Instead the backend
  `preview` path (`reader::read_preview`) and a full `decode::decode_rgb` of
  its payload, as a stand-in for the webview's decode, were timed in a
  release build on Windows (median of 5, files in the OS cache). No 24 MP
  JPEG was on hand; the closest were two DxO exports:
  - 5349x4012 (21.5 MP), 12.0 MB: read 3.1 ms, decode **197 ms**.
  - 4444x3333 (14.8 MP), 9.2 MB: read 2.4 ms, decode **141 ms**.
  - ARW 1616x1080 preview, 248 KB: read 0.6 ms, decode **9.2 ms**; a DNG's
    1620x1080 preview, 1.07 MB: 0.9 ms / 14.7 ms.
  So a JPEG page decodes 15-20x longer than an ARW's and ships 50x the bytes
  over IPC: far above, as the plan anticipated.
- **The re-encode was measured and not added.** The plan's remedy, a
  DCT-scaled unrotated re-encode capped at 2048 px, was timed with
  `decode::thumbnail_jpeg_near` (the same decode at the smallest `n/8`
  covering the cap, box-average, mozjpeg encode): 2048 px at q85 took
  **183-535 ms** in the backend (306 ms on the 21.5 MP file; 1616 px: 184-267
  ms), then 17-34 ms to decode its 450-500 KB output. That is slower than
  shipping the whole file and decoding it (~200 ms), because mozjpeg's
  default trellis encode costs more than the decode it saves (the same
  finding as "Measure before choosing a JPEG payload over raw pixels" in
  `docs/agents/tauri-app.md`). So `preview` returns the whole JPEG, which
  also keeps the preview at full detail; Linux's 6 MP worker limit
  (`PREVIEW_PIXEL_LIMIT`) still resizes it in the worker. A cheaper path
  needs a different payload, recorded below.
- The JPEG-or-RAW rule lives in `folders::Media` (collect RAWs and JPEGs in
  one pass, `listed()` returns the JPEGs only when there is no RAW), which
  both `commands::read_listing` and `folders::list` use. A JPEG listing
  clears the sidecars it collected, so `reconcile_sidecars_of` sees no stat
  for `foo.jpg` even when `foo.xmp` sits next to it (the XMP kind maps
  `foo.jpg` to `foo.xmp`, so without the clear it would have been parsed).
- `set_rating` and `read_faces` refuse a non-RAW path through one
  `raw_only(path, what)` helper; `set_rating` checks it first, before the
  rating range. The command itself is not unit-testable, so the helper is.
- The JPEG test fixture with an APP1 Exif (`jpeg_with_exif`) lives in
  `index.rs`'s test module, now `pub(crate) mod tests`, and `commands.rs`'s
  tests reuse it. Writing it through a Python heredoc turned `\x00` / `\0`
  in the Rust byte-string literals into raw NUL bytes once; check test
  fixtures written that way with `grep -c` for NULs.

## Step 3: frontend view-only mode

- **Not verified by hand: the GUI was not run.** The implementation agent
  cannot drive the desktop app, so "a `-sequenced` folder opens at its last
  viewed file in capture order" was not checked in the window. It rests on
  code reading: `openDirectory` sets `viewOnly` before its first `ordered()`,
  so the provisional list and every later `refilter` use `capture`, and the
  resume path (`resumeTarget` / `firstEntriesAnchor`) is unchanged. Check it
  by hand before merging.
- **The mode line is persistent, not a one-shot `setStatus`.** The plan
  asked to name the mode once per folder open through `note` / `setStatus`,
  but `show()` and every decoded preview call `setStatus()` and clear the
  note, so a note set in `openDirectory` would vanish as soon as the first
  preview lands. `renderMeta` instead appends `JPEG folder: view only` while
  `viewOnly` holds, the same way the `1:1` indicator is driven by `zoomed`.
- The gate sits in `record` (which the keys' `judge` and the MCP
  `view.judge` both go through) and at the top of `rejectRest`; `runAction`
  still returns `true` for a gated key. `setViewOnly` also clears the undo
  and redo history, since a rescan can turn a folder view-only after its
  RAWs were deleted, and an undo would otherwise `set_rating` a gone path.
- The effective sort is `sortFor(viewOnly, sortKey)`, used by `ordered()`
  and the MCP `get_view`'s `sort`; `sortKey` itself (and the menu's checked
  item) keeps the user's persisted key, which is read once at startup, and
  the disabled `#sort-toggle` keeps `set_sort_order` from being called.

## Deferred issues (todo candidates)

- **CLI `bench` silently measures nothing for a JPEG path.** Basis: Step 1
  implementation; `jpeg::parse` leaves `Arw::preview` / `Arw::full` `None`,
  and `bench` in `crates/cli/src/main.rs` only times the tiers whose field is
  `Some` after `reader::read_metadata`. Either time `read_preview` /
  `read_full` for a JPEG unconditionally or reject non-RAW paths with an
  error. Files: `crates/cli/src/main.rs`, `crates/core/src/jpeg.rs`.
- **A JPEG folder's preview decodes 15-20x longer than an ARW's.** Basis:
  Step 2 measurement above; the plan's re-encode was slower than the full
  decode, so `preview` sends the whole JPEG. A faster first view needs a
  payload that skips the encode, e.g. a DCT-scaled decode (4/8 or 3/8)
  sent as raw pixels, or reading a JPEG's embedded Exif thumbnail first. Check
  it with `Timing logs` in the GUI before choosing. Files:
  `crates/app/src/commands.rs` (`preview`), `crates/core/src/decode.rs`,
  `crates/app/ui/src/` decode worker.
