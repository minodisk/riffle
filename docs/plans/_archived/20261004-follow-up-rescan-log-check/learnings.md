# Learnings

## Step 1: Close the todo item and re-tag the `tauri-app.md` item

### Log evidence (Windows, 2026-10-04)

Run with `mise run tauri:release:devtools`, Timing logs on, read from
`Riffle.log` (`Help > Open Log Folder`).

- `D:\Photos\2026\2026-06-20` (1050 RAWs), cold for the current extractor
  version (`todo=1050`):
  - 17:28:28 `open entries rows=437` (the open).
  - 17:28:33 `rescan deferred: trigger=focus` (focus return during the scan).
  - 17:28:36 scan extract done, then `open entries rows=1050`: the
    `refreshOnScanDone` read, expected because the scan wrote rows.
  - 17:29:36 scan faces `canceled=false`, then
    `rescan: trigger=focus deferred=true`.
  - One `open entries rows=1050` for the `faces-done` refresh (the faces pass
    wrote 1050 rows), logged between `scan_id=26`'s `scan list` and its
    reconcile.
  - `scan_id=26` (the drained focus rescan) has `todo=0`, and no further
    `open entries` follows it.
- `D:\Photos\2026\2026-05-05`, focus with no scan running:
  `rescan: trigger=focus deferred=false`, `scan_id=24` `todo=0`, no
  `open entries`.

So the deferred focus rescan reads no `open entries` of its own; the two reads
after the open each follow a pass that wrote rows.

### Notes

- The `faces-done` refresh's `open entries` lands between the next scan's
  `scan list` and its reconcile in the log, so it is easy to misread as the
  rescan's own read. Attribute reads by which pass wrote rows, not by line
  order alone.
