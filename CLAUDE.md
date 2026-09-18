# Riffle

A culling app for Sony ARW and Leica DNG files. See [README.md](./README.md)
for what it is and the current status.

## Layout

A Cargo workspace: `crates/core` (ARW and DNG parsing and JPEG decoding, `riffle-core`,
whose `src/xmp.rs` parses and patches XMP sidecar bytes and `src/dop.rs` DxO
PhotoLab `.dop` sidecar bytes), `crates/cli` (the
benchmark CLI, including the `scan` folder-extraction benchmark), `crates/app`
(the Tauri 2 desktop app, whose `src/index.rs` is the SQLite folder index and
`src/sidecar.rs` the coalescing sidecar writer thread and `SidecarFormat`, the
XMP-or-`.dop` setting chosen from the `Sidecar` menu and persisted in the
`sidecarFormat` key of the settings store).

The frontend lives under `crates/app/ui` (TypeScript compiled by `tsc` only, no
bundler) and is type-checked by `mise run ci`.

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
