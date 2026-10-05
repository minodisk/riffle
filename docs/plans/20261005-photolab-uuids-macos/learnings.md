# Learnings

## Step 1: Extend `photolab.rs` to macOS

- `Path::components()` normalizes away interior `.` components, so
  `/Users/./mino` yields the same components as `/Users/mino`; only a leading
  `.` (a relative path) shows up as `Component::CurDir`. The macOS mapping
  therefore rejects `..` and relative paths, and an interior `.` resolves to
  the same folder (correct, since it names the same folder). The first test
  draft expected `/a/./b` to miss and failed.
- The shared pieces (`lookup` with its busy retry, `is_busy`, `is_uuid`,
  `BUSY_TIMEOUT` / `BUSY_ATTEMPTS`, and a new `open` for the read-only +
  `busy_timeout` connection) are gated `#[cfg(any(windows, target_os =
  "macos"))]`. The Windows `query` only lost its open lines to `open`; its
  tests are unchanged. The macOS `query` duplicates the walk / sources /
  master tail on purpose, per the plan's "do not refactor the Windows
  query".
- `database_path` on macOS delegates to `newest_database(library)` so the
  directory / file name scan (`DxO PhotoLab vN/DOPDatabaseVN.dopdata`) is
  testable against a temp directory.
- Confirmed the folder open path canonicalizes (`commands::canonicalize` in
  `list_arw` and the scan commands), so no `canonicalize` is added in the
  lookup; `/Volumes/Macintosh HD/...` would be rejected by the `/Volumes`
  rule anyway.
- A one-off run (not committed) of `registered_uuids` against the user's
  real PhotoLab 10 database for `/Users/mino/Downloads/_DSC6978.ARW`
  returned the Source / Item Uuids the database holds for it.

## Deferred issues (todo candidates)

- Pending manual check (macOS, PhotoLab 10): in
  `~/Pictures/photolab-export-test/browse` (3 ARWs registered, no `.dop`),
  pick one in Riffle (`mise run tauri:dev`, sidecar format `.dop`). Expected:
  the written `.dop`'s Source / Item Uuids equal `ZDOPSOURCE.ZUUID` /
  `ZDOPINPUTITEM.ZUUID`, and PhotoLab shows the pick on the master with no
  virtual copy. Step 1's checkbox was ticked on the automated criteria. Basis:
  plan Step 1 "Pending manual checks"; file `crates/app/src/photolab.rs`.
  Already written into `todo.md` as part of this step's acceptance criteria.
- Pending manual check (macOS): picking in a folder PhotoLab never opened
  still imports the image as the picked master. Same basis; in `todo.md`.
- Pending manual check (macOS): with a virtual copy in PhotoLab, the master
  is the lowest `ZDOPINPUTITEM.Z_PK`. Same basis; in `todo.md`.
- External volumes (`/Volumes/X`) get no lookup: find how PhotoLab records
  them (`ZTYPE`, `ZNAME`, `ZUNIQUEID`) and extend the macOS mapping in
  `crates/app/src/photolab.rs`. Same basis; in `todo.md`.
