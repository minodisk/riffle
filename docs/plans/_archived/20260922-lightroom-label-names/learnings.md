# Learnings

## Step 1

- `LabelNames::name` needs one lifetime for `&self` and `color`: it returns
  the English `color` itself when the configured name is empty.
- Reads map a label to a color by the configured name first, then by the
  English name, so a sidecar from an English Lightroom still reads as a
  color under Japanese names.
- The app call sites in `crates/app/src/sidecar.rs` pass
  `LabelNames::default()` until Step 2 wires the setting.

## Step 2

- Took option (a): `set_label_names` is `async`, and when the names
  actually changed it drains the writer and runs `Index::reset_sidecars`
  under `AppSwitchLock` in `spawn_blocking`, then emits `sidecar-format`
  (payload: the unchanged current format) so `main.ts` reopens the folder.
  `label-names` is emitted on every call, even an unchanged one, so the
  settings window always sees the normalized values.
- `label_names` returns `{"names": {...}, "japanese": {...}}`, both in the
  `labelNames` shape keyed by lowercase color, so Step 3 can take the
  Japanese preset from the backend instead of duplicating the strings.
  `set_label_names` returns the stored names in the same shape.
- The names snapshot rides in `Message::Set` / `Entry` beside `format`;
  `Writer::set` / `set_now` gained a trailing `names: LabelNames` argument,
  which touched every test call site in `sidecar.rs` (patched with
  `LabelNames::default()`).
- `label_names_setting` trims each entry, so a stored `" レッド "` reads as
  `"レッド"`; blank, missing or non-string entries fall back per color.
- The extra argument pushed `Writer::set` / `set_now` to 8 parameters,
  over clippy's `too_many_arguments` limit of 7 (`-D warnings` in CI). They
  carry `#[allow(clippy::too_many_arguments)]` rather than a new parameter
  struct, to keep the change small.

## Step 3

- The Japanese preset is fetched from the `japanese` half of the
  `label_names` response, so its five strings live only in
  `LabelNames::japanese()` in the backend. The English reset is derived in
  `crates/app/ui/src/labels.ts` by capitalizing the color keys, which is the
  backend's default too.
- The label name block's visibility is set both in `showSidecarFormat` (load
  and `sidecar-format` event) and directly in the radio change handler, so it
  follows the radio before the backend answers.
- The visibility rule still needs a manual check in the app.

## Step 4

- Lightroom Classic 2026 facts (user-verified, Windows, Japanese UI): no XMP
  written by default (`Ctrl+S` or the catalog auto-write setting); XMP is read
  on first import only, later changes need `Synchronize Folder...` with
  `Scan for metadata updates`; color labels match `xmp:Label` against the
  color label set names, not `photoshop:LabelColor`; LrC writes a pick as
  `xmpDM:good="true"` + `xmpDM:pick="1"`.
- The README Lightroom Classic 2026 checklist line was left unticked: the full
  round-trip after this feature is not yet confirmed by the user.

## Deferred issues (todo candidates)

- Whether a Riffle-written XMP without `xmpDM:pick` shows the pick / reject
  flag in Lightroom Classic is unverified (LrC itself writes
  `xmpDM:good="true"` + `xmpDM:pick="1"`). Basis: the user's LrC 2026 check in
  Step 4 and plan.md "Trade-offs and risks". Files: `crates/core/src/xmp.rs`,
  `README.md`.
