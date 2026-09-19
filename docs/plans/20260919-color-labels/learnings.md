# Learnings

## Step 1: XMP `xmp:Label`

- `locate` now takes the property's local name. The element form no longer
  returns on the `Text` event; it waits for the end tag so it can report the
  whole element range for removal (the start position is remembered at the
  start tag). `quick_xml`'s `LocalName::as_ref()` compares against `&str`
  directly, not `&[u8]`.
- Clearing an element-form label removes its whole line only when nothing but
  whitespace shares the line; otherwise just the element.
- Values are spliced raw with no escaping, per the plan; a label containing
  `"` or `<` would produce broken XMP. Not a concern for the known
  vocabularies.

## Step 2: `.dop` `ColorLabel`

- The PhotoLab 10.0.2.28 fixtures (`_DSC0009`..`_DSC0015`) are tab-indented
  (one tab per depth), LF-only, with `},` on one line; the 10.0.1 fixtures are
  unindented with `}\n,` and a trailing CRLF. The scanner already handled
  tabs; the new tests are the first to exercise it.
- PhotoLab indents an anonymous item's `{` / `}` at the same depth as its
  fields, but a keyed table's closing `}` at the key's depth (e.g. `Sidecar`'s
  final `}` is unindented while its fields have one tab). So "copy the closing
  line's indentation" is right for inserts into `Items[0]` but would give
  an under-indented `Date` inserted into `Sidecar`. `Date` is always present in
  real files, so this was left as the plan specified.
- `CafId` became `CafID` in 10.0.2: the key set is not stable across
  versions (irrelevant to the scanner).
- The eight HSL `Label = "..."` lines (plus `ColorLabel`) are kept apart by
  the depth-matched scanner; clearing removes only the `ColorLabel` line.
- `write_label` with `None` on a sidecar without a label still updates the
  two timestamps (not a byte-identical no-op); the caller decides whether to
  write at all.

## Step 3

- `main` was already at `SCHEMA_VERSION = 4` (exif-filters), so labels took
  v5. `CREATE TABLE IF NOT EXISTS ratings` never adds a column to an existing
  table, so every non-fresh accepted version (2, 3, 4) runs
  `ALTER TABLE ratings ADD COLUMN label TEXT`; v2 runs the `pick` ALTER first.
- `dirty_rows` now returns a `DirtyRow` 4-tuple; `reconcile_sidecars_of`
  strips the label again so the writer's input is unchanged until Step 4.

## Step 4

- The tuples were extended rather than introducing a `Judgement` struct:
  `Pending` / `Message::Set` / `Writer::set` / `set_now` take
  `label: Option<String>` after `pick`, and `reconcile_sidecars_of` now returns
  `index::DirtyRow` unchanged.
- `sidecar::write` skips `write_label` when the label is `None` and the
  (rating-patched) sidecar has none, because `dop::write_label(.., None, ..)`
  still bumps the timestamps. A label-only judgement on a file with no sidecar
  mints via `write_label(None, ..)` directly rather than rating first.
- The frontend keeps a `labels` map filled from `folder_entries` (cleared on
  folder open) and passes the file's current label to every `set_rating`, so
  a star or flag keypress does not clear a label before Step 6 wires the
  label keys.
