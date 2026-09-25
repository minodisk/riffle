# Learnings

## Step 1

- `label_names` (`crates/app/src/commands.rs`) was the one non-test caller of
  `LabelNames::japanese()`. Step 1 keeps its `{"names", "japanese"}` shape so
  the settings window keeps working, taking `japanese` from
  `riffle_core::i18n::preset("ja")`; Step 2 replaces the shape with `presets`.
- Besides `presets()`, `i18n` exposes `preset(code)`; the tests in `xmp.rs`,
  `sidecar.rs` and `commands.rs` use it through a small `japanese()` helper.
- `build.rs` emits `cargo:rerun-if-changed=i18n` (a directory: Cargo scans it
  for any change, including added or removed files) plus one line per JSON
  file. It only picks up `*.json`, so the `README.md` Step 3 adds to
  `crates/core/i18n/` is ignored. Paths go through `{:?}` so a Windows path's
  backslashes are escaped in the generated `include_str!`.
- The xmp module doc comment still cites `"パープル"` as an example of a
  localized `xmp:Label`; it is prose, not a label name the code uses, so it
  stays.

## Deferred issues (todo candidates)

- `en.json`'s `lightroom.colorLabels.verified` is `"Lightroom Classic (English
  UI)"`, with no version or OS, because the English names were not checked
  against a specific install in this step (they are the long-standing
  `LabelNames::default()`). Confirm against a real English Lightroom Classic
  and name the version / OS. Basis: Step 1 implementation. Files:
  `crates/core/i18n/en.json`.
