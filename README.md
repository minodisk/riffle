# Riffle

A culling app for Sony ARW files: look through them fast, apply ratings, and
mark picks and rejects. Nothing else — no developing, no editing. It never runs
a RAW decoder (LibRaw / rawler); everything comes from the JPEGs already
embedded in the ARW.

## Status

Phase 0.5, Phase 1 (the CLI benchmark) and Phase 2 (the app skeleton) are done.

The app opens a folder, lists the ARW files in it, shows one embedded preview on
a `<canvas>` (decoded in a worker, rotated by the ARW's Orientation), and pages
through them. Paging is bound to the arrow keys (left/up for the previous file,
right/down for the next), to WASD (`w`/`a` previous, `s`/`d` next) and to HJKL
(`h`/`k` previous, `j`/`l` next); `o` opens a folder. Keys held with
Cmd/Ctrl/Alt are left to the system. Nothing else yet: no prefetch, no
cache, no thumbnail grid, no rating. See [Running the app](#running-the-app).

The CLI from Phase 1:

```sh
cargo build --release
./target/release/riffle-cli info     <file.ARW>            # where the embedded JPEGs are
./target/release/riffle-cli focusbox <file.ARW> <out.png>  # draw the focus box on the preview
./target/release/riffle-cli crop     <file.ARW> <out.png>  # partially decode the focus point at 1:1
./target/release/riffle-cli bench    <file.ARW>...         # measure decode speed
./target/release/riffle-cli scan     <dir> [threads]       # extract a whole folder in parallel
```

**Confirmed by hand on macOS**: the folder picker opens and returns, cancelling
is a no-op, the arrow keys page, and a portrait file comes out upright.

The first run found a bug nothing else had: `pick_folder` was a synchronous
`#[tauri::command]`, which tauri runs inline on the main thread, and
`blocking_pick_folder` then parked that thread — so the dialog appeared and
froze. It is an `async` command awaiting a channel now. Passing `mise run ci`,
`tsc --noEmit` and three rounds of review had not caught it, because it only
goes wrong once the app is actually running.

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
