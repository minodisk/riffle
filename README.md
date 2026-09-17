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
like with like; the whole-file numbers match the ones Phase 2 recorded
(7.3ms / 19.3ms mean). The symlink folder is 5000 symlinks to the same file and
the copy folder is 20 `cp` copies in a scratch directory, read in file-name
order by a fresh process. `purge` needs root on this machine, so the "first
read" column cannot be guaranteed cold; it is the same procedure the Phase 2
baseline used.

The whole-file read dominated, which is what the bounded read removes. Phase 4
still owns prefetching, but it now starts from this cheaper read.

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
