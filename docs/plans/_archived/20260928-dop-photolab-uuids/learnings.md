# Learnings: dop-photolab-uuids

## Step 1

- `uuids: Option<Uuids>` went in as the last parameter of `dop::write_rating` /
  `dop::write_label` (after `now`, matching `template`'s order). That makes
  `write_rating` seven parameters, which is exactly clippy's
  `too_many_arguments` limit, so no `#[allow]` was needed.
- The owned `Option<Uuids>` (not `Option<&Uuids>`) keeps the fresh path free of
  clones: the value moves straight into `template`. Step 2 hands the same
  lookup result to both `write_rating` and `write_label` only when both mint,
  which never happens (the label write patches the rated bytes), so moving it
  into the first call is enough.
- The test helper that pulls the Uuids back out of a template splits on the
  `}\n,\n}\n,\n` that closes the item and `Items`; the item Uuid is the first
  `Uuid = "..."` line before it and the source Uuid the first one after it.
- A Python heredoc edit that wrote Rust `"\n"` escapes produced literal
  newlines inside the string; editing Rust string escapes through a second
  language's escaping is error-prone, prefer the Edit tool for those lines.

## Step 2

- What the real database (PhotoLab 10, read from a copy) shows:
  - A root folder row has `ParentFolderId` NULL (not 0) and `Name` is the bare
    drive (`D:`, no backslash). `Folders` rows are `FolderTypeDiscriminator =
    'EFDOPVolume'` at the root.
  - **The same drive letter can have more than one root row**: two removable
    volumes were both registered as `E:` (told apart only by `UniqueId`, a
    volume GUID). Two SD cards both holding `E:\DCIM\100MSDCF\_DSC0001.ARW`
    would then match two sources. The lookup walks every matching folder chain
    and, when the path resolves to more than one source, returns `None`
    (random Uuids, the pre-existing behaviour) rather than guess. Covered by a
    fixture test.
  - `(ParentFolderId, Name)` and `(FolderId, Name)` are unique otherwise (0
    duplicates, case-insensitively), and `Sources.Name` is the file name with
    its extension (`_DSC0006.ARW`).
  - `CreationDate` is text like `2026-09-27 14:41:53.855521Z`, with a
    **variable number of fractional digits** (trailing zeros dropped), so a
    lexical `ORDER BY CreationDate` can misorder two items within the same
    second (`.8Z` sorts after `.81Z`). Across all 1772 sources with more than
    one item, the lowest `Id` is never later than another item's
    `CreationDate`, so the master query orders by `Id` alone (the plan's
    "prefer lowest `Id`" fallback), not `CreationDate, Id`.
  - `journal_mode` is `wal`. A read-only rusqlite open works; when no `-wal` /
    `-shm` exist (PhotoLab closed), SQLite creates them next to the database
    even on a read-only connection (the directory is writable). That does not
    touch the database contents and matches what PhotoLab itself does, so no
    `immutable=1` fallback was added (it would also ignore a live WAL and read
    stale data).
  - Every `Sources.Uuid` and `Items.Uuid` is upper-case 8-4-4-4-12.
  - A throwaway test run against the copy returned the expected
    `04B2552F-...` / `E584D7F8-...` pair for
    `D:\Photos\tests\riffle-dop-test-E-uuid\_DSC0006.ARW`, the master (not the
    virtual copy) for `D:\Photos\riffle-dop-test\_DSC0006.ARW` (the
    registered, pre-move path), and `None` for the moved path.
- Cross-platform choice: everything except the non-Windows
  `registered_uuids` stub (which returns `None`) and the tests is
  `#[cfg(windows)]`, and the tests are `#[cfg(all(test, windows))]`. That was
  smaller than making the path split POSIX-aware (which nothing would use) and
  avoids dead-code warnings on macOS / Linux. The Windows CI job runs the
  tests.
- `folder_names` also accepts a verbatim drive prefix (`\?\D:\...`, what
  `canonicalize` returns); UNC and other prefixes are a plain miss.
- Only the first write of a fresh sidecar mints, so the lookup result moves
  into that call and the follow-up label write passes `None`.
- A failure is logged with `log::debug!` (open error, busy, missing table);
  a plain miss is not.
- **Manual check (user, 2026-09-28, PhotoLab 10.0.1, `mise run tauri:dev`)**:
  passed. `D:\Photos\tests\riffle-dop-test-F-registered\_DSC0009.ARW` was
  opened in PhotoLab first (registered, no `.dop`), then picked in Riffle:
  PhotoLab showed the pick on the master and no virtual copy.
  `riffle-dop-test-G-unregistered\_DSC0009.ARW`, never opened in PhotoLab
  before the pick, was likewise imported as the picked master.

## Step 3

- The README's `### DxO PhotoLab` subsection went after `### Lightroom
  Classic`, following the format list's order (Lightroom first, PhotoLab
  second). It says "identifiers" rather than "Uuid" to stay in the README's
  user-facing voice; the Uuid detail lives in `docs/agents/tauri-app.md`.
- The guide's new Hit entry sits right after the `Settings` / `Orientation`
  `.dop` entry, and that entry's "the database shadows the sidecar" line was
  corrected in place to point at it, rather than deleted, so the history of
  the wrong claim stays readable.
- The guide entry documents what Step 2 shipped, not the plan's original
  wording: master = lowest `Id` (not `CreationDate`), duplicate drive-letter
  roots are a miss, and the UUID-shape check on database values.
