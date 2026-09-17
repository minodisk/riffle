# Riffle

A culling app for Sony ARW files: look through them fast, apply ratings, and
mark picks and rejects. Nothing else — no developing, no editing. It never runs
a RAW decoder (LibRaw / rawler); everything comes from the JPEGs already
embedded in the ARW.

## Status

Phase 0.5, Phase 1 (the CLI benchmark), Phase 2 (the app skeleton) and Phase 3
(the folder index, the filmstrip and the focus box) are done.

The app opens a folder — through the picker or by dropping a folder, or a
single file (any existing file resolves to its parent folder), onto the
window — lists the ARW files in it, shows one embedded preview on
a `<canvas>` (decoded in a worker, rotated by the ARW's Orientation), and pages
through them. A thumbnail filmstrip runs down the left edge: it is virtualised,
highlights the current file, scrolls to follow paging, and a click on a cell
shows that file. When the index has a `FocusLocation` for the current file, a
focus box is drawn over the preview; it is placed in unrotated sensor
coordinates and rotated with the image. Still missing: no prefetch, no 1:1
focus check, no rating. See [Running the app](#running-the-app).

Keys:

| Key | Action |
|-----|--------|
| `ArrowLeft`, `ArrowUp`, `w`, `a`, `h`, `k` | previous file |
| `ArrowRight`, `ArrowDown`, `s`, `d`, `j`, `l` | next file |
| `o` | open a folder |
| `f` | toggle the focus box |

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
./target/release/riffle-cli crop     <file.ARW> <out.png>  # partially decode the focus point at 1:1
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

**Awaiting the user's confirmation (Phase 3)**: nobody has seen this phase's UI
running. `pnpm tauri dev` needs the GUI and `osascript` assistive access is
denied on the development machine, so the following are unconfirmed rather than
confirmed: the Phase 3 keys (`w`/`a`/`s`/`d`/`h`/`j`/`k`/`l` paging and `f`
toggling the focus box) actually working in the running app, the filmstrip
(thumbnails filling in during a scan, portrait cells upright, the highlight
following every paging key and key auto-repeat, click-to-page, scrolling a
5000-file strip), the `scanning N / M` progress line, the focus box landing on
the subject's face, the drag-and-drop gestures (a folder, a single ARW, a drag
that leaves without dropping), that the app stays responsive while a real
folder scans, and that the second open of a real folder is under 3s.

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
