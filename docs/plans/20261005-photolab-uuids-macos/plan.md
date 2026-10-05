<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# PhotoLab Uuid lookup on macOS

## Purpose

`crates/app/src/photolab.rs::registered_uuids` gives a fresh `.dop` the Source
Uuid and master Item Uuid DxO PhotoLab already registered for the image, so
PhotoLab applies the sidecar to the master instead of importing a virtual copy
(see `../../agents/photolab.md`). It is `#[cfg(windows)]`; on macOS it returns
`None`, so every fresh `.dop` for a registered image still becomes a virtual
copy there (`todo.md`, "App: PhotoLab's virtual-copy fix only works on
Windows"). PhotoLab on macOS writes a `.dop` only for an edited image even
with "Automatically export sidecars" on, so registered images without a
`.dop` are the common case and the lookup is needed.

Investigation on the user's Mac (PhotoLab 10, 2026-10-04 / 05):

- Database: `~/Library/DxO PhotoLab v10/DOPDatabaseV10.dopdata` (SQLite, WAL;
  `defaults read com.dxo.PhotoLab10 DOPDatabasePath` points at it; the
  preference plist is binary). The version is in both the directory and the
  file name.
- Schema is Core Data, not the Windows one:
  - `ZDOPFOLDER (Z_PK, Z_ENT, ZPARENT, Z5_PARENT, ZTYPE, ZNAME,
    ZLOCALIZEDNAME, ZUNIQUEID)`. `Z_ENT` 5 = `DOPFolder`, 6 = `DOPVolume`
    (from `Z_PRIMARYKEY`). Volumes have `ZPARENT` NULL. The boot volume row
    is `ZTYPE = 2`, `ZNAME = 'Macintosh HD'`, `ZUNIQUEID` = the APFS UUID of
    the **Data** volume (`/System/Volumes/Data`), not of `/`. A second
    volume row `ZTYPE = 1`, `ZNAME = 'DxO PhotoLab'` has no children and an
    unknown purpose. `ZLOCALIZEDNAME` is NULL everywhere. Folder rows chain
    through `ZPARENT` (`Z5_PARENT` 6 for a volume parent, 5 for a folder):
    `/Users/mino/Downloads` is volume → `Users` → `mino` → `Downloads`.
  - `ZDOPSOURCE (Z_PK, Z_ENT 18, ZPARENT = folder Z_PK, ZNAME, ZUUID)`,
    `ZUUID` = `Sidecar.Source.Uuid`.
  - `ZDOPINPUTITEM (Z_PK, Z_ENT 8, ZSOURCE = source Z_PK, ZUUID)` =
    `Items[0].Uuid`. No virtual copies existed to check ordering; the master
    is taken as the lowest `Z_PK`, as Windows takes the lowest `Id`.
  - Name indexes are `COLLATE BINARY` (case-sensitive), unlike Windows'
    `NOCASE`. `(ZPARENT, ZNAME)` had no duplicates in folders or sources.
- Verified against two `.dop` files PhotoLab wrote: their Source / Item
  Uuids equal `ZDOPSOURCE.ZUUID` / `ZDOPINPUTITEM.ZUUID`.
- Opening the database with `immutable=1` misses rows that only exist in the
  `-wal` file; a plain read-only open sees them (same as the Windows
  learnings).
- External volumes (`/Volumes/X`) were not observed; only the boot volume is
  in this database.

## Steps

- [x] Step 1: Extend `photolab.rs` to macOS, with fixture tests and docs
  - Done when:
    - On macOS, `registered_uuids` returns the database's Source Uuid and
      master Item Uuid for a registered file on the boot volume, and `None`
      for an unregistered file, a path that resolves to more than one source,
      a path that cannot be mapped (not absolute, `.`/`..` components,
      under `/Volumes/<name>`), a missing / busy / foreign database, or a
      value that is not 8-4-4-4-12 hex.
    - Unit tests gated `#[cfg(all(test, target_os = "macos"))]` build a
      Core Data-shaped fixture database in the test (as the Windows tests
      do) and cover: a registered file; a master before a virtual copy
      (lower `Z_PK` wins); names compare case-sensitively (`_dsc0001.arw`
      misses); a folder chain under the `ZTYPE = 1` volume is not matched;
      two `ZTYPE = 2` volume rows resolving to two sources yield `None`;
      misses and failures (missing database, missing tables) yield `None`;
      the Uuid literal guard; the version parse of the directory / file
      names. Existing Windows tests and the Windows code path are untouched.
    - `mise run ci` passes (macOS and Windows CI jobs both run the tests).
    - `docs/agents/photolab.md` describes the macOS database location,
      schema, the boot-volume mapping and the case-sensitive names, and
      drops "the macOS location is unknown" in both places.
    - `todo.md`: the "App: PhotoLab's virtual-copy fix only works on
      Windows" section is replaced by a short section listing the pending
      manual checks (below) and the external-volume follow-up.
    - The doc comment at the top of `photolab.rs` no longer says "Windows
      only".
  - Implementation approach (as far as it is known):
    - Keep `registered_uuids` as the public entry; make the per-OS pieces
      `#[cfg(windows)]` / `#[cfg(target_os = "macos")]`, with the existing
      `#[cfg(not(windows))]` stub narrowed to
      `#[cfg(not(any(windows, target_os = "macos")))]`. Share what is
      OS-independent (`lookup`'s error-to-debug-log wrapper, `is_uuid`, the
      read-only `Connection::open_with_flags` + 300 ms `busy_timeout`
      opening) by gating it `#[cfg(any(windows, target_os = "macos"))]`
      rather than duplicating; `query` and `folder_names` / `database_path`
      / `version` get a macOS counterpart. Do not refactor the Windows
      `query` beyond moving shared lines; its tests must stay as they are.
    - Database path (macOS): scan `$HOME/Library` (via
      `std::env::var_os("HOME")`) for directories named `DxO PhotoLab vN`
      and pick the newest `N` whose `DOPDatabaseVN.dopdata` is a file,
      mirroring the Windows newest-version scan. Do not read
      `com.dxo.PhotoLab10.plist`: it is a binary plist (a new crate or a
      `defaults` subprocess for the lookup on every fresh `.dop`), and the
      directory scan already needs no version knowledge. Record the
      `DOPDatabasePath` preference in `photolab.md` as the fallback to check
      if a user reports a miss.
    - Path mapping (macOS): the folder chain of `arw.parent()` is its
      `Component::Normal` parts after `RootDir`; `CurDir` / `ParentDir` or a
      relative path → `None`. A path whose first component is `Volumes` →
      `None` (external volumes unverified; see the todo follow-up). Riffle
      canonicalizes folder paths on open, so the `/Volumes/Macintosh HD`
      symlink to `/` does not normally reach the lookup; confirm by reading
      the open path in `commands.rs` rather than adding a `canonicalize`
      call in the lookup.
    - Query (macOS): roots are `SELECT Z_PK FROM ZDOPFOLDER WHERE ZPARENT IS NULL
      AND ZTYPE = 2` (do not match `ZNAME = 'Macintosh HD'`: a renamed boot
      volume would break it; do not match `ZUNIQUEID`: it is the Data
      volume's UUID, which needs IOKit / `diskutil` to obtain); children
      `SELECT Z_PK FROM ZDOPFOLDER WHERE ZPARENT = ? AND ZNAME = ?`
      (no `Z_ENT` predicates: Core Data numbers entities per model and a later
      PhotoLab may shift them); sources `SELECT Z_PK, ZUUID FROM ZDOPSOURCE WHERE ZPARENT
      = ? AND ZNAME = ?`; master `SELECT ZUUID FROM ZDOPINPUTITEM WHERE
      ZSOURCE = ? ORDER BY Z_PK LIMIT 1`. Walk every matching chain and
      treat more than one source as a miss, as Windows does. Names compare
      as the database stores them (`COLLATE BINARY`); do not add `NOCASE`.
    - Fixture (tests): `CREATE TABLE ZDOPFOLDER (Z_PK INTEGER PRIMARY KEY,
      Z_ENT INTEGER, ZPARENT INTEGER, Z5_PARENT INTEGER, ZTYPE INTEGER,
      ZNAME VARCHAR, ZLOCALIZEDNAME VARCHAR, ZUNIQUEID VARCHAR)` and the
      matching `ZDOPSOURCE` / `ZDOPINPUTITEM` subsets; rows for the two
      volumes (`ZTYPE` 1 and 2), `Users` → `mino` → `Pictures` → `shoot`,
      a source with a master and a copy, and an empty sibling folder, in the
      shape of the Windows fixture.
    - Docs: in `photolab.md`'s virtual-copy entry, add the macOS bullets
      next to the Windows ones (location, tables, `ZTYPE = 2`, `COLLATE
      BINARY`, `immutable=1` missing WAL rows); keep the entry's existing
      wording otherwise.
    - Pending manual checks to list in `todo.md` (not part of CI):
      - `~/Pictures/photolab-export-test/browse` (3 ARWs registered, no
        `.dop`): picking one in Riffle (`mise run tauri:dev`, `.dop` format)
        must yield a `.dop` whose Uuids equal the database's and PhotoLab
        must show the pick on the master with no virtual copy.
      - An unregistered folder: still imported as the picked master.
      - A folder on an external volume (`/Volumes/X`): find how PhotoLab
        records it (`ZTYPE`, `ZNAME`, `ZUNIQUEID`), then extend the mapping;
        until then the lookup yields `None` there.
      - A source with a virtual copy: confirm the master is the lowest
        `ZDOPINPUTITEM.Z_PK`.

## Trade-offs and risks

- **Database path: directory scan vs. the preference plist.** Recommended:
  scan `~/Library/DxO PhotoLab vN/DOPDatabaseVN.dopdata` for the newest `N`
  (mirrors Windows, no new dependency, one `read_dir`). The alternative,
  `com.dxo.PhotoLabN DOPDatabasePath`, follows a user-relocated database but
  is a binary plist (needs the `plist` crate or a `defaults read` subprocess,
  and still needs the version to know the domain). If a user reports a miss
  with a relocated database, add the preference as a second source then.
- **Boot volume: `ZTYPE = 2` vs. `ZUNIQUEID`.** `ZTYPE = 2` is one observed
  row's value, not documented. Matching `ZUNIQUEID` would be exact but it is
  the Data volume's APFS UUID, which the standard library cannot read
  (`statfs` gives no UUID; IOKit or `diskutil info /System/Volumes/Data`
  would). Recommended: `ZTYPE = 2`, with the two-roots → `None` rule as the
  guard if a future PhotoLab registers more `ZTYPE = 2` volumes.
- **External volumes yield `None`** until a real device is checked. This is
  the pre-existing behaviour (random Uuids, possible virtual copy), not a
  regression; it is listed as a todo follow-up.
- **Case sensitivity.** The database compares names `COLLATE BINARY`. The
  lookup follows it, so a file reached through a differently-cased path
  (APFS is case-insensitive by default) misses. Riffle's paths come from
  `read_dir`, so they carry the on-disk case, which is what PhotoLab stored;
  if a miss is ever traced to this, switch the comparison, not the schema.
- **Master ordering** by lowest `Z_PK` is assumed from Windows; unverified on
  macOS (no virtual copies in the database). Listed as a manual check.
- **Single step.** The code, tests and docs land in one PR because the docs
  only describe what the code does and the todo item closes with it.

## Progress

- (none yet)
