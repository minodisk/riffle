# Riffle

A culling app for Sony ARW files: look through them fast, apply ratings, and
mark picks and rejects. Nothing else — no developing, no editing. It never runs
a RAW decoder (LibRaw / rawler); everything comes from the JPEGs already
embedded in the ARW.

## Status

Phase 0.5, Phase 1 (the CLI benchmark) and Phase 2 (the app skeleton) are done.

The app opens a folder, lists the ARW files in it, shows one embedded preview on
a `<canvas>` (decoded in a worker, rotated by the ARW's Orientation), and pages
through them with the left/right arrow keys. Nothing else yet: no prefetch, no
cache, no thumbnail grid, no rating. See [Running the app](#running-the-app).

The CLI from Phase 1:

```sh
cargo build --release
./target/release/riffle-cli info     <file.ARW>            # where the embedded JPEGs are
./target/release/riffle-cli focusbox <file.ARW> <out.png>  # draw the focus box on the preview
./target/release/riffle-cli crop     <file.ARW> <out.png>  # partially decode the focus point at 1:1
./target/release/riffle-cli bench    <file.ARW>...         # measure decode speed
```

**Not confirmed at runtime.** Phase 2 passed `mise run ci`, `tsc --noEmit` and
code review, but the window could never be driven or captured on the machine it
was written on (assistive access denied), so "a 5000-file folder pages end to
end, each page shows the right file, a portrait file comes out upright" is
unverified by eye.

### Phase 4 baseline

The per-page cost of the current implementation, measured on the Rust side only
(`std::fs::read` of the whole ARW plus `arw::parse`) on an Apple Silicon Mac.
**It excludes the IPC hop and `createImageBitmap`**, which could not be measured:

| Folder | n | mean | p95 |
|--------|---|------|-----|
| 5000 symlinks to one ARW, warm page cache | 300 | 7.3ms | 8.8ms |
| 20 distinct 48MB copies, first read | 20 | 19.3ms | 26.6ms |

The whole-file read dominates, which is what Phase 4's seek-based reader is
meant to address.

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
