# Riffle

A culling app for Sony ARW and Leica DNG files. See [README.md](./README.md)
for what it is and the current status.

## Layout

A Cargo workspace: `crates/core` (ARW and DNG parsing and JPEG decoding, `riffle-core`,
whose `src/xmp.rs` parses and patches XMP sidecar bytes (`xmp:Rating` and the
`xmp:Label` colour label) and `src/dop.rs` DxO PhotoLab `.dop` sidecar bytes
(rating, pick / reject and `ColorLabel`), `src/faces.rs` the YuNet face/eye detector, whose ONNX model and licence
live in `crates/core/models/`, and `src/sharpness.rs` the
sharpness score of the embedded preview, taken between the eyes of a detected
face (or on the AF point when it falls inside that face), else around the AF
point, else from the sharpest tile), `crates/cli` (the
benchmark CLI, including the `scan` folder-extraction benchmark), `crates/app`
(the Tauri 2 desktop app, whose `src/index.rs` is the SQLite folder index,
`src/exif.rs` the shooting-settings display formatting shared by the meta pane
and the filter menu, and
`src/sidecar.rs` the coalescing sidecar writer thread and `SidecarFormat`, the
XMP-or-`.dop` setting chosen in the settings window and persisted in the
`sidecarFormat` key of the settings store, and `src/shortcuts.rs` the keymap:
the default keys, whose colour label keys differ per sidecar format, and the
user's overrides, persisted in the `shortcuts` key).

The frontend lives under `crates/app/ui` (TypeScript built by Vite+, configured
in the root `vite.config.ts`; `pnpm exec vp {dev,build,check,fmt,test}`) and is
formatted, linted, type-checked and tested by `mise run ci`; its
`src/context.ts` builds the items of the strip's HTML right-click menu.

## Language

**Write everything in the repository in English.** This covers documentation
(`README.md`, `CLAUDE.md`, `docs/**`, skill and agent definitions), code
comments, commit messages, PR titles and bodies, and any other text that lands
in the repository.

Conversation with the user stays in Japanese; only what gets committed is
English.

## Commits

Conventional Commits, in English. Example:
`feat(cli): add a partial decode benchmark`

## Development

Do all development through the `develop` skill (`/develop`), however small the
change. Do not edit code and open a PR by hand or through `/pr` alone.
