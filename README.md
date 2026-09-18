# Riffle

A culling app for Sony ARW files: look through them fast, apply ratings, and
mark picks and rejects. Nothing else — no developing, no editing. It never runs
a RAW decoder (LibRaw / rawler); everything comes from the JPEGs already
embedded in the ARW.

## Status

Phase 0.5, Phase 1 (the CLI benchmark), Phase 2 (the app skeleton), Phase 3
(the folder index, the filmstrip and the focus box) and Phase 5 (the 1:1 focus
check) are done.

The app opens a folder — through the picker or by dropping a folder, or a
single file (any existing file resolves to its parent folder), onto the
window — lists the ARW files in it, shows one embedded preview on
a `<canvas>` (decoded in a worker, rotated by the ARW's Orientation), and pages
through them. A thumbnail filmstrip runs down the left edge: it is virtualised,
highlights the current file, scrolls to follow paging, and a click on a cell
shows that file. When the index has a `FocusLocation` for the current file, a
focus box can be drawn over the preview; it is hidden by default, `f` toggles
it, and it is placed in unrotated sensor coordinates and rotated with the
image. `Space` toggles a 1:1 focus check.
Still missing: no prefetch, no rating. See [Running the app](#running-the-app).

Keys:

| Key | Action |
|-----|--------|
| `ArrowLeft`, `ArrowUp`, `w`, `a`, `h`, `k` | previous file |
| `ArrowRight`, `ArrowDown`, `s`, `d`, `j`, `l` | next file |
| `o` | open a folder |
| `f` | toggle the focus box |
| `Space` | toggle the 1:1 focus check |
| `1`-`5` | rate the current file that many stars |
| `x` | reject the current file (sticky, not a toggle) |
| `u` | un-reject the current file (does nothing unless it is rejected) |
| `0` | clear the rating or the reject |

### The 1:1 focus check

`Space` toggles a third tier on top of the 400px thumbnails and the 1616x1080
preview: a crop of the full-resolution `JpgFromRaw`, partially decoded out of
the ARW with a ranged read, drawn at one JPEG pixel per device pixel. The crop
is centred on the camera's `FocusLocation` (mapped from sensor coordinates onto
the full JPEG), and on a file without one — manual focus — on the centre of the
frame. It is cut in unrotated coordinates and carried by the same canvas
rotation as the preview, so a portrait file comes out upright. On a file with a
`FocusLocation`, the preview bitmap is drawn around the crop at the same scale,
so the frame stays in context while the crop is decoded; on a file without one,
the canvas stays blank until the crop arrives. Paging while zoomed stays zoomed
and moves to the next file's focus point. There is no panning and no free zoom
level; the crop
is capped at 1024 device pixels per axis and travels over the IPC boundary as
raw RGBA.

Keys held with Cmd/Ctrl/Alt are left to the system. Letter keys are matched
lower-cased, so Shift+J pages like `j`.

### The index

On the first open of a folder, every ARW in it is extracted in parallel
(capture time, `SubSecTimeOriginal`, `FocusLocation`, Orientation and a 404x270
thumbnail, from a bounded 1MiB prefix plus a ranged read of the preview
itself) into a SQLite database. The status
line shows `scanning N / M` while that runs; the first preview does not wait
for it. The database lives in the app cache directory:

- **macOS**: `~/Library/Caches/com.minodisk.riffle/index.sqlite`
- **Windows**: `%LOCALAPPDATA%\com.minodisk.riffle\index.sqlite`

One table holds one row per file path, keyed by the absolute path, with the
metadata above and the thumbnail as a JPEG BLOB. A row is valid for a file iff
its stored `size` and `mtime_ns` still match the file's current `stat`;
anything else is re-extracted. Since the path is the key, renaming a folder
re-scans it and leaves the old rows behind — and deleting rows does not shrink
the database file without `VACUUM`, which nothing runs yet. On the 5000-symlink
folder used for the measurements below, the database came to 104,177,664 bytes
(~20.8KB per row, mostly thumbnail); since every row there is a byte-identical
thumbnail of the same file, a real folder of distinct frames will not be
exactly this. It is a cache:
deleting the file costs one more scan.

The CLI from Phase 1:

```sh
cargo build --release
./target/release/riffle-cli info     <file.ARW>            # where the embedded JPEGs are
./target/release/riffle-cli focusbox <file.ARW> <out.png>  # draw the focus box on the preview
./target/release/riffle-cli crop     <file.ARW> <out.png> [size]  # partially decode the focus point at 1:1
./target/release/riffle-cli bench    <file.ARW>...         # measure decode speed
./target/release/riffle-cli scan     <dir> [threads]       # extract a whole folder in parallel
```

### What has been confirmed, and by what

**Confirmed by hand on macOS (Phase 2)**: the folder picker opens and returns,
cancelling is a no-op, the arrow keys page, and a portrait file comes out
upright.

The first run found a bug nothing else had: `pick_folder` was a synchronous
`#[tauri::command]`, which tauri runs inline on the main thread, and
`blocking_pick_folder` then parked that thread — so the dialog appeared and
froze. It is an `async` command awaiting a channel now. Passing `mise run ci`,
`tsc --noEmit` and three rounds of review had not caught it, because it only
goes wrong once the app is actually running.

**Verified without a GUI (Phase 3)**: the focus box's coordinate transform was
checked numerically against `riffle-cli focusbox` on a real Orientation 8 file
— scaling `FocusLocation` onto the unrotated preview and letting the canvas
rotation carry the box puts the box where the CLI's PNG does: (-140, -26)
from the canvas centre against the CLI's (-140, -25). Everything else below is `mise run ci`
(`cargo test`, clippy, `tsc --noEmit`) plus the measurements in the next
sections.

**Verified without a GUI (Phase 6)**: the six sidecar reconciliation rules are
covered by tests against a temp folder, and the cost the sidecar pass adds to
a folder open was measured on the Rust side (see "What the sidecar pass adds
to a folder open" below, with its conditions). Nothing about how a rating
looks or feels in the running app has been verified here.

**Confirmed by hand on macOS (Phase 3)**: the user ran the app on a real
folder and confirmed the `scanning N / M` progress line, that the app stays
responsive while a real folder scans, thumbnails filling in during that scan
with portrait cells upright, the focus box landing on the subject, a fast
second open, and both drag-and-drop gestures (a folder and a single ARW). The
Phase 3 paging keys (`w`/`a`/`s`/`d`/`h`/`j`/`k`/`l`) and `f` toggling the
focus box are implied by those. "Fast" is the user's impression, not a
measurement; the real-folder numbers are still missing (see the Phase 3
sections below).

**Awaiting the user's confirmation (Phase 3)**: not everything in this phase
has been looked at yet. Still unconfirmed: the filmstrip highlight following
every paging key and key auto-repeat, click-to-page, scrolling a 5000-file
strip, and a drag that leaves the window without dropping.

**Verified without a GUI (Phase 5)**: the focus-point arithmetic of the 1:1
check is verified numerically only, on the Rust side that the CLI and the app
share — `riffle-cli crop` on the real Orientation 8 test file prints
`crop 525x512 at (3344,1476) point (269,256)`, and unit tests cover the
sensor→JPEG scaling, the MCU snap, the edge clamping and the centre fallback.
The `focus_crop` payload header and the Orientation 6/8 width/height swap are
covered by unit tests. The frontend's placement is right by construction (the
same `rotate()` branches as the preview, cropping in unrotated coordinates) but
that is an argument, not a check.

**Awaiting the user's confirmation (Phase 5)**: nothing in the 1:1 check has
been looked at in a running window. Unconfirmed: `Space` showing the eye at 1:1
and upright, `Space` again returning to the preview, paging while zoomed
following the next file's focus point without the old crop flashing, the
manual-focus centre fallback, and — the important one — the end-to-end time
from keypress to pixels. **The 50ms budget is not claimed to be met**: see the
measurements below, where a 1024 crop costs 11ms at the top of the frame and
44ms at the bottom, before the IPC hop and `createImageBitmap`.

**Awaiting the user's confirmation (Phase 6)**: nothing about the rating keys
has been looked at in a running window. Unconfirmed: a rating key changing the
badge and the status line with no perceptible delay, holding `3` down doing
nothing beyond the first press, `x` then `u` then `4` ending at four stars,
mashing keys while paging never badging the wrong file, the strip badge
matching the main view, and one `.xmp` per rated file appearing in the folder
within about half a second. Which tools read `xmp:Rating="-1"` back as a
reject is likewise the user's to confirm.

### Phase 4 baseline

The per-page cost on the Rust side only, on an Apple Silicon Mac.
**It excludes the IPC hop and `createImageBitmap`**, which could not be
measured. Phase 2 read the whole 48MB ARW (`std::fs::read` plus `arw::parse`);
`preview` now reads a bounded 1MiB prefix instead (`reader::read_preview`),
which is where the metadata and the embedded preview live, so both numbers are
given:

| Folder | n | whole file (before) | bounded prefix (after) |
|--------|---|---------------------|------------------------|
| 5000 symlinks to one ARW, warm page cache | 300 | mean 6.6ms / p95 7.7ms | mean 0.06ms / p95 0.13ms |
| 20 distinct 48MB copies, first read | 20 | mean 16.0ms / p95 30.0ms | mean 5.1ms / p95 7.8ms |

Both columns were re-measured together in Phase 3 Step 2, so they compare
like with like; the whole-file numbers are in the same range as the ones
Phase 2 recorded (7.3ms / 19.3ms mean). The symlink folder is 5000 symlinks to the same file and
the copy folder is 20 `cp` copies in a scratch directory, read in file-name
order by a fresh process. `purge` needs root on this machine, so the "first
read" column cannot be guaranteed cold; it is the same procedure the Phase 2
baseline used.

The whole-file read dominated, which is what the bounded read removes. Phase 4
still owns prefetching, but it now starts from this cheaper read.

### Folder scan throughput

`riffle-cli scan` runs the Phase 3 extraction (bounded read, metadata parse,
404x270 thumbnail) over a folder on a dedicated rayon pool, with no database.
On an Apple Silicon Mac with 12 cores:

| Folder | threads | total | files/s |
|--------|---------|-------|---------|
| 5000 symlinks to one ARW, warm page cache | 1 | 34.9s | 143 |
| 5000 symlinks to one ARW, warm page cache | 8 | 5.35s | 935 |
| 5000 symlinks to one ARW, warm page cache | 12 | 4.46s | 1121 |
| 100 distinct 48MB copies, freshly written, page cache not guaranteed cold | 4 | 0.44s | 230 |
| 100 distinct 48MB copies, freshly written, page cache not guaranteed cold | 12 | 0.19s | 529 |

The thumbnails come to 19232 bytes each, i.e. ~96MB for 5000 files.
Extrapolating the freshly-written-copies column to 5000 files gives 9.5s at 12
threads and 21.7s at 4; the CPU cost leaves ~25s of headroom against the
30-second target, but whether that headroom survives on a real, cold-read
folder is not something these numbers establish. A single thread would not
make it (34.9s). More threads than cores does not help: 16 threads was flat
against 12 and doubled the per-file p95.

What this cannot measure: a real 5000-distinct-file folder. Each
freshly-written-copies folder here is 100 `cp` copies scanned once, `purge`
needs root on this machine so nothing is guaranteed cold, and the copies were
still likely warm in the page cache right after being written; the folders
were also scanned in the order they were written, which flatters the higher
thread counts. The real number can only be measured by the user on a real
folder, and on a card reader or slow external disk the scan is disk-bound
regardless.

### Opening an indexed folder again

The second open of a fully indexed folder does no extraction: it stats every
file, reconciles the rows and queries them. On the 5000-file folder:

| Step | Target | Measured |
|------|--------|----------|
| First open, full scan (5000 files, 10 threads, 0 errors) | 30s | 5.55s |
| Second open (stat + reconcile + query, fully indexed) [^1] | 3s | 34.4ms |

Both rows were measured on **5000 symlinks pointing at one real ARW, with a
warm page cache**, so they carry the same caveat as the tables above. The
second open is a stat-and-query number, which the symlinks flatter less than
they flatter a read benchmark, but 5000 lookups of one cached inode's metadata
is still cheaper than 5000 distinct 48MB files' metadata on a card. The first
scan is the same folder and procedure as the scan throughput table above, at
10 threads, a thread count that table does not have a row for; and the real
number on a real folder of 5000 distinct files has never been measured by
anyone; on a card reader or a slow external disk the first scan is disk-bound
regardless.

[^1]: Measured with a temporary `#[ignore]`d test that was removed before
committing, so this number is not reproducible from the committed tree.

### The 1:1 focus check path

The Rust side of one `Space` keypress, on `~/Downloads/_DSC6978.ARW` (α7 V,
Orientation 8, `FocusLocation` 7008 4672 3613 1732, `JpgFromRaw` 7008x4672
baseline 4:2:2, 5,761,112 bytes) on an Apple Silicon Mac. **One real file, warm
page cache, in-process, release build, n=20, medians.** Re-measured against the
Step 1 functions the CLI and the app both call, not copied from planning.
**The IPC hop and `createImageBitmap` are excluded** — they could not be
measured headlessly, and the user has not reported end-to-end timings yet, so
no keypress-to-pixels number exists.

| Step | Median |
|------|--------|
| Ranged read of the 5.76MB `JpgFromRaw` (`reader::read_full`) | 0.7ms |
| Crop at the focus point, 512 / 1024 / 2048 per axis (`partial::decode_focus_crop`, RGBA) | 18.4 / 21.5 / 29.8ms |
| 1024 crop at row 300 / 2336 / 4400 of the 4672-row JPEG (`partial::decode_crop`, RGB) | 10.8 / 27.3 / 44.1ms |

Crop size barely matters; the crop's **row** dominates. `jpeg_skip_scanlines`
on a baseline JPEG still entropy-decodes every skipped row, so a focus point
low in the frame costs four times one at the top, and 44ms leaves nothing of
the 50ms budget for the IPC hop and the bitmap. The payload is raw RGBA (4.2MB
at the 1024 cap) rather than a re-encoded JPEG because re-encoding that crop
with `mozjpeg::Compress`'s defaults measured 49ms at q85 during planning — more
than the decode it follows.

### What the sidecar pass adds to a folder open (Phase 6)

Opening a folder also reconciles the XMP sidecars: one listing of the
directory, a `stat` per sidecar found, and a parse of only those whose
`(size, mtime)` changed since the app last saw them.

| Second open of a 5000-file folder | Measured (median) |
|-----------------------------------|-------------------|
| No sidecars in the folder | 31.3ms |
| 5000 sidecars, all with an unchanged stat | 67.4ms |

**These two numbers are not comparable to the 34.4ms above and do not replace
it.** They were measured on a different folder: 5000 one-KB regular files
named `*.ARW`, not symlinks and not real ARWs, on the local APFS disk with a
warm page cache, on an Apple Silicon Mac. The work timed is the same
sequence a folder open does on the Rust side — list, `stat` every file,
`reconcile`, reconcile the sidecars, `entries` — in a temporary `#[ignore]`d
test (release profile, median of 7 runs after 3 warm-up runs) that was
removed before committing, so neither number is reproducible from the
committed tree. What they do establish is the shape of the cost: on this
machine the sidecar pass roughly doubles a second open, adding about 36ms
for 5000 sidecars, and that cost is a listing plus a `stat` each, not a
parse, because an unchanged stat parses nothing. A first open of a folder
full of foreign sidecars pays the parse as well and was not measured.

## Running the app

Prerequisites:

- **macOS**: Xcode Command Line Tools (`xcode-select --install`). Tauri needs no
  other system dependency there.
- **Windows**: MSVC Build Tools, the WebView2 runtime, and `nasm` (`mozjpeg-sys`
  builds libjpeg-turbo from source and needs it for SIMD on x86).

The Rust toolchain, Node and pnpm all come from `mise install`.

```sh
pnpm install
pnpm tauri dev
```

## Measurements (Apple Silicon Mac, α7 V ARW, n=20)

| Step | Target | Measured (median) |
|------|--------|-------------------|
| 1616x1080 preview extraction | 10ms | 4.4ms |
| JpgFromRaw full decode | 300ms | 84ms |
| 512px partial decode at the focus point | 50ms | 17.5ms |

## The FocusLocation coordinate system

`FocusLocation` is in **unrotated sensor coordinates**. It maps onto both the
preview and the full JPEG unrotated, so draw the box first and apply the
Orientation rotation afterwards. Verified against a real portrait-orientation
file (Orientation 8 / Rotate 270 CW).
