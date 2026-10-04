# DxO PhotoLab and its `.dop` sidecar

Read this before touching `crates/core/src/dop.rs`,
`crates/app/src/photolab.rs` or the `.dop` side of
`crates/app/src/sidecar.rs`. It lists what PhotoLab's sidecar format and
database have already broken here, each with the reason.

The tags follow [`tauri-app.md`](./tauri-app.md): **Hit** broke something
here, **Measured** steered a design decision, **Inferred** comes from sources
or docs only.

## `.dop` indentation follows the table, and keys are not stable across versions (Hit)

PhotoLab indents an anonymous item's `{`/`}` at the same depth as its fields,
but a keyed table's closing `}` at the key's own depth. "Copy the closing
line's indentation" is correct for inserts into `Items[0]` but under-indents
an insert into a keyed table such as `Sidecar`.

- Key names change across PhotoLab versions (`CafId` became `CafID` in
  10.0.2); do not treat the sidecar's key set as stable.
- `dop::write_label` with `None` on a sidecar that has no label still updates
  the two timestamps. A caller that wants a true no-op must not call it.
- Source: `docs/plans/_archived/20260919-color-labels/learnings.md`, Step 2.

## `.dop`'s `Items[0]` needs a minimal `Settings` block, or PhotoLab reports no images (Hit)

An `Items[0]` table without a `Settings` table makes PhotoLab 10 show "no
images in this folder" for that folder, even though the sidecar otherwise
looks well-formed (`ProcessingStatus`, `Software`, `IPTC`, `CafId` and the
like do not substitute for it). A brace-balanced
`Settings = {\nVersion = "21.0",\n}\n,\n` between `Rating` and
`ShouldProcess` fixes it and keeps the pick. Do not copy a preset from a
sample into it: the preset overrides the user's default one.

PhotoLab 10 also displays the image by the item's `Orientation`, not by the
RAW's EXIF: an item without the key shows unrotated, and an `Orientation`
copied from another file rotates it wrongly. So Riffle writes the file's own
EXIF Orientation (IFD0 tag 0x0112) as `Orientation = n,` between `Name` and
`Rating`: in the fresh template, and, for an item lacking the key, at the
start of the `Rating` line (before the item's closing brace without one). The
sidecar writer takes the value from the index's `files.orientation` row, or
from `riffle_core::reader::read_metadata` when the file has no row yet, with
the index lock released before the file I/O; when neither yields one it
writes no line at all. An existing `Orientation` is never touched.

- Do not trust the earlier (now corrected) claim that a Settings-less
  template was accepted by PhotoLab; that was most likely because the test
  folder was already in PhotoLab's database. The database does not shadow the
  sidecar: it imports an item with an unknown Uuid as a new item (see the
  next entry).
- The explicit `Orientation = 1,` line for a landscape file (EXIF Orientation 1)
  is verified: PhotoLab 10 shows it upright and reads the `.dop`'s rating. Do
  not drop it. For a hand check, write the `.dop` before PhotoLab sees the RAW,
  or write it through the app. If the RAW was copied into a folder PhotoLab
  already views, PhotoLab registers the image first, and the fresh `.dop`'s
  random Uuids then show up as an extra item. That is a race in the test
  procedure, not a Riffle bug.
- Edits queued at the same splice offset apply in the reverse of their push
  order. `write_rating` pushes the `Settings` edit after `ShouldProcess` (so
  `Settings` ends up before it); `write_label` pushes it before `ColorLabel`
  (so an inserted `ColorLabel` ends up before it, alphabetically). Both push
  the `Orientation` edit after `Settings` (and `write_label` before
  `ColorLabel`), so at the item's closing brace the inserts come out
  `ColorLabel`, `Orientation`, `Settings`. Keep this in mind when adding more
  keys that share an insertion point.
- Source: `docs/plans/_archived/20260926-dop-settings-block/learnings.md`,
  Step 1; `docs/plans/_archived/20260926-dop-orientation/learnings.md`,
  Step 1;
  `docs/plans/_archived/20261004-dop-orientation-landscape-verified/learnings.md`,
  Step 1.

## PhotoLab matches `.dop` items by Uuid; a fresh sidecar on a registered image becomes a virtual copy (Hit)

DxO PhotoLab keeps its own SQLite database and matches a `.dop`'s items to it
by `Uuid`. An image is registered as soon as PhotoLab opens its folder, even
without writing a `.dop`. Hand-run with PhotoLab 10.0.1 on Windows:

- unregistered image + fresh `.dop` with random Uuids: imported as the master
  (PhotoLab re-mints the Uuids); fine.
- registered image + fresh `.dop` with random Uuids (with or without an
  `.xmp`): imported as a virtual copy carrying the pick, the master keeps the
  database state, and exports of the copy get a `_1` suffix.
- registered image + `.xmp` only: nothing; PhotoLab ignores the XMP pick.
- registered image + patching a PhotoLab-written `.dop`: applied to the
  master.
- registered image + fresh `.dop` carrying the database's Source Uuid and
  master Item Uuid: applied to the master, no virtual copy, provided `Date` /
  `CreationDate` / `ModificationDate` are newer than the database item's
  `ModificationDate` (older ones make PhotoLab ignore the sidecar; Riffle
  stamps "now").
- registered image + fresh `.dop` with the `Items[0].Uuid` and `Source.Uuid`
  lines omitted (dates newer than the database item's `ModificationDate`):
  imported as a virtual copy carrying the rating / pick; the new `Items`
  row's Uuid is the nil UUID `00000000-0000-0000-0000-000000000000`; the
  master keeps the database state.
- registered image + fresh `.dop` with both Uuids `""`: ignored; no virtual
  copy, nothing applied to the master, database unchanged.

So `sidecar::write_kind`, when it mints a `.dop`, asks `photolab::registered_uuids`
for the registered image's Uuids and passes them to `dop::write_rating` /
`dop::write_label` (`dop.rs` stays pure; it never looks anything up).
In the two Uuid-less runs PhotoLab rewrote neither sidecar and neither
reached the master, so a Uuid-less `.dop` does not get around the lookup
(PhotoLab 10.0.1, Windows, 2026-10-01).

- The database is the highest-numbered
  `%APPDATA%\DxO\DxO PhotoLab N\Database\PhotoLab.db` (Windows only; the
  macOS location is unknown, so there is no lookup there).
- Tables: `Folders (Id, Name, ParentFolderId)` is a chain of names from a root
  row whose `ParentFolderId` is NULL and whose `Name` is the bare drive
  (`D:`); `Sources (Name, Uuid, FolderId)` has one row per file, `Name`
  including the extension, `Uuid` = `Sidecar.Source.Uuid`; `Items (Id,
  SourceId, Uuid, CreationDate, ...)` holds the master and its virtual copies.
  Names are `COLLATE NOCASE`.
- The master is the lowest `Id`. Do not order by `CreationDate`: its
  fractional seconds have a variable number of digits, so a lexical sort can
  misorder items within one second.
- The same drive letter can have several root rows (two removable volumes
  both registered as `E:`). A path that resolves to more than one source is a
  miss, not a guess.
- Open read-only with a short `busy_timeout` (`BUSY_TIMEOUT`, 300 ms) per
  attempt; the database is WAL and PhotoLab may hold it. A query that fails
  with `SQLITE_BUSY` or `SQLITE_LOCKED` (those two codes only) is retried up
  to `BUSY_ATTEMPTS` (6) times, about 1.8 s of busy wait in total, before the
  sidecar is written. On Windows each attempt measured about 0.8 s of wall
  time against a rollback-journal database held by `BEGIN EXCLUSIVE`, since
  SQLite's Windows lock code sleeps between its own lock retries. A final busy
  miss is logged at `log::warn!` and falls back to random Uuids. Any other
  failure (not found, missing table, a value that is not 8-4-4-4-12 hex, since
  the template writes it unescaped) falls back at once and is logged at
  `log::debug!`; a plain miss is not logged.
- Re-patching an already-written `.dop` with the Uuids of a later successful
  lookup was rejected: PhotoLab may already have imported the random-Uuid
  sidecar as a virtual copy, which the rewrite does not undo, and the rewrite
  can race PhotoLab's own writes to the same file. Retrying before the write
  is the only safe window.
- An existing `.dop` is patched as before; the lookup only runs for a fresh
  one.
- Source: `docs/plans/_archived/20260928-dop-photolab-uuids/learnings.md`.

## `photolab::lookup` keys on folder + file `Name`; a rename does not follow through to PhotoLab or a shared sidecar (Hit)

`photolab::lookup` (`crates/app/src/photolab.rs`) finds a RAW's Uuids by its
folder and file `Name`, and a `.dop` Riffle writes carries that `Name`
inside. `rename.rs`'s `file_plan` moves a `.dop` unchanged when a RAW is
renamed, so the sidecar's inner `Name` and PhotoLab's own database still
say the old name until PhotoLab re-indexes — check this against a real
PhotoLab install before assuming the moved `.dop` still matches. Likewise,
`file_plan`, like `trash::plan`, takes `a.xmp` as `a.ARW`'s sidecar even
when an `a.DNG` in the same folder shares that same `a.xmp`; renaming or
trashing `a.ARW` alone carries the DNG's sidecar away too. See
`docs/plans/_archived/20260928-rename-from-tree-and-strip/learnings.md` for
the open follow-ups.

- Source: `docs/plans/_archived/20260928-rename-from-tree-and-strip/learnings.md`, Step 3 and Deferred issues.
