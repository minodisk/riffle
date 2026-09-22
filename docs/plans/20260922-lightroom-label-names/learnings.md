# Learnings

## Step 1

- `LabelNames::name` needs one lifetime for `&self` and `color`: it returns
  the English `color` itself when the configured name is empty.
- Reads map a label to a colour by the configured name first, then by the
  English name, so a sidecar from an English Lightroom still reads as a
  colour under Japanese names.
- The app call sites in `crates/app/src/sidecar.rs` pass
  `LabelNames::default()` until Step 2 wires the setting.
