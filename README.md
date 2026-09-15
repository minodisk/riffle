# Riffle

A culling app for Sony ARW files: look through them fast, apply ratings, and
mark picks and rejects. Nothing else — no developing, no editing. It never runs
a RAW decoder (LibRaw / rawler); everything comes from the JPEGs already
embedded in the ARW.

## Status

Phase 0.5 and Phase 1 (the CLI benchmark) are done.

```sh
cargo build --release
./target/release/riffle-cli info     <file.ARW>            # where the embedded JPEGs are
./target/release/riffle-cli focusbox <file.ARW> <out.png>  # draw the focus box on the preview
./target/release/riffle-cli crop     <file.ARW> <out.png>  # partially decode the focus point at 1:1
./target/release/riffle-cli bench    <file.ARW>...         # measure decode speed
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
