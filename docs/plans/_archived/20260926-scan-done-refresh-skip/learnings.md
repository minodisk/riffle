# Learnings

## Step 1

- `Index::reconcile` now returns `(todo, removed)` and `Index::reconcile_sidecars`
  `(to_parse, cleared)`; both counts come from `tx.execute`'s affected-row
  count inside the existing transactions, so no extra query was needed.
  `reconcile_sidecars_of` returns `cleared + parsed.len()` as its third tuple
  element, and `scan_folder` adds the `removed` count to it for
  `ScanStarted.changed`. The `scan reconcile:` line reports `removed=N` and
  the `scan sidecars:` line `changed=N` (the sidecar part only).
- The test call sites were adjusted mechanically (`.0` on the tuple, an extra
  `_` in the destructuring); `sidecar.rs` also calls `reconcile_sidecars` in
  two tests and needed the same adjustment.
- In a Git Bash shell `cargo` is only on the path through `mise exec --`;
  `riffle-app` has no lib target, so `cargo test -p riffle-app` (not `--lib`).
- Closing the todo item "App: conditionally skip `rebuildExifMenu`'s DOM
  rebuild in `refreshEntries`": the user measured `exif=1.1ms` at 2134 rows
  and 0.1-0.5 ms on smaller folders (Windows, debug build, timing logs on),
  so the DOM rebuild is not a cost worth a guard. The item can be deleted
  with that reason.
- The first `mise run ci` failed on `clippy::type_complexity` for
  `Result<(Vec<(String, SidecarStat, bool)>, usize), String>`; the element
  type got a `SidecarToParse` alias next to `SidecarStat` in `index.rs`.
