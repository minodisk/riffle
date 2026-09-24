# Learnings

## Step 1: the `Both` sidecar format

- `sidecar::newest` is the single newest-wins comparator (larger `mtime_ns`,
  ties to the kind listed first in `kinds()`, i.e. XMP). `sidecar::write`
  uses it for the label kept when `label_known` is false and for the stat
  handed to `mark_written`; `reconcile_sidecars_of` uses it to pick the file
  compared against the stored stat. A commands test asserts the stored stat
  equals the newest listed stat after a `Both` write, which is what makes a
  reopen right after culling parse nothing.
- The per-kind body of the old `write` became `write_kind`; the final stat now
  comes from `index::stat`, which computes `mtime_ns` exactly as the folder
  listing does, so the two sides cannot drift in how they measure it.
- The folder open derives the kind of the file it parses from its name
  (`SidecarFormat::Dop.matches` else XMP) rather than carrying it next to
  `SidecarStat`, so `index.rs` did not change beyond the schema comment.
- Tripped up once: a test that judged the same path twice with
  `label_known = false` stayed dirty on the second pass. That is the existing
  `mark_written` guard, not a bug: once a row's label is known, an unknown-label
  judgment whose resolved label differs from the stored one is left dirty. The
  test now uses a fresh file per case.
- `watch::triggers` already ignores `*.xmp`, `*.arw.dop` and their
  `.riffle-tmp` temporaries in any mix (its existing test covers
  `["a.xmp", "b.arw.dop"]`), so it needed no change.
- The exhaustive `match format` in the folder-listing test broke the build on
  the new variant; it now also lists under `Both`.
