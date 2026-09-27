<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Reuse PhotoLab's registered Uuids in a fresh `.dop`

## Purpose

When Riffle writes a judgment for a RAW that has no `.dop` yet,
`crates/core/src/dop.rs` mints a fresh sidecar with two random Uuids
(`Sidecar.Source.Uuid` and `Sidecar.Source.Items[0].Uuid`). DxO PhotoLab
matches sidecar items to its database by `Uuid`. If PhotoLab has already
registered the image (opening the folder in PhotoLab is enough; it writes no
`.dop`), the unknown item Uuid is imported as a new item: a virtual copy that
carries Riffle's pick while the master keeps the database state, and exports
of it get a `_1` suffix. One real folder ended up with 228 such virtual
copies.

Hand-run experiments (2026-09-27, PhotoLab 10.0.1 on Windows) established:

- unregistered image + Riffle fresh `.dop`: imported as master (PhotoLab
  re-mints the Uuids); fine
- registered image + Riffle fresh `.dop` (with or without `.xmp`): virtual copy
- registered image + Riffle `.xmp` only: nothing (XMP pick ignored)
- registered image + Riffle patching a PhotoLab-written `.dop`: applied to the
  master; fine, the existing patch path needs no change
- registered image + Riffle-style fresh `.dop` whose Source Uuid and Item
  Uuid are the database's values, with `Date` / `CreationDate` /
  `ModificationDate` newer than the database item's `ModificationDate`: no
  virtual copy, pick applied to the master. (Older timestamps make PhotoLab
  ignore the sidecar; Riffle stamps "now", so that is fine.)

So a fresh `.dop` should carry the database's Source Uuid and master Item
Uuid when the image is registered, and random ones otherwise.

PhotoLab's database (SQLite) on Windows is
`%APPDATA%\DxO\DxO PhotoLab 10\Database\PhotoLab.db` (a `DxO PhotoLab 9`
sibling may exist; the user runs 10). The macOS location is unknown: no
lookup there. Relevant tables:

- `Folders (Id, Name COLLATE NOCASE, ParentFolderId, ...)`: a path is the
  chain of `Name`s from a root such as `D:`, e.g. `D:` -> `Photos` ->
  `tests` -> `riffle-dop-test-E-uuid`.
- `Sources (Id, Name COLLATE NOCASE, Uuid, FolderId, ...)`: one per file;
  `Uuid` is the `.dop`'s `Sidecar.Source.Uuid`.
- `Items (Id, SourceId, Uuid, CreationDate, ModificationDate,
  StorePickOrReject, ...)`: the master and its virtual copies; the master is
  the item with the earliest `CreationDate` (lowest `Id`); its `Uuid` is the
  `.dop`'s `Items[0].Uuid`.

Out of scope: cleaning up virtual copies that already exist.

## Steps

- [x] Step 1: Let `dop::write_rating` / `dop::write_label` take the Uuids of a fresh sidecar
  - Done when:
    - `crates/core/src/dop.rs` exports a small `Uuids { item: String, source: String }`
      type and both `write_rating` and `write_label` take `uuids: Option<Uuids>`
      (or `Option<&Uuids>`), used only on the `existing == None` path:
      `Some` goes into the template verbatim, `None` mints two random ones as
      today. The patch path (`existing == Some`) ignores the argument and its
      output is byte-for-byte unchanged (the existing tests prove this).
    - `template` takes the `Uuids` type instead of `[String; 2]` (the current
      `[item, source]` ordering is easy to swap by mistake); the existing
      `template` test at `dop.rs` ~line 770 is adjusted.
    - New unit tests on the public functions: a fresh `write_rating(None, ...,
      Some(uuids))` and a fresh `write_label(None, Some("Red"), ..., Some(uuids))`
      contain `Uuid = "<item>"` inside the item and `Uuid = "<source>"` after
      `Items`; with `None` the two Uuids are version-4 and differ from each
      other (reuse the check of `uuids_are_random_version_4`).
    - `crates/app/src/sidecar.rs` compiles by passing `None` at its three call
      sites (`SidecarFormat::write_label` ~line 121, `SidecarFormat::write_rating`
      ~line 144, and the test ~line 1436). No behaviour change in the app yet.
    - The `dop.rs` module doc gains a paragraph on why the Uuids can be
      supplied (PhotoLab matches items by Uuid; a fresh sidecar with unknown
      Uuids on a registered image becomes a virtual copy) and that the
      timestamps must be newer than the database item's, which "now" is.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep `dop.rs` pure: it receives the Uuids, it never looks them up.
    - Mirror how `orientation` was threaded through (a plain extra parameter,
      documented in the `write_rating` doc comment and referenced from
      `write_label`'s). `Writer::set` already carries
      `#[allow(clippy::too_many_arguments)]` precedent (see
      `docs/agents/tauri-app.md`, "Carry every judgment field on every
      write"); add the allow on the core functions only if clippy demands it.
    - `uuid()` stays private and random; PhotoLab's upper-case form is only
      needed for minted values, so pass database values through unchanged.

- [ ] Step 2: Look the RAW up in PhotoLab's database and feed the Uuids to a fresh `.dop`
  - Done when:
    - A new module `crates/app/src/photolab.rs` (declared in `main.rs` next to
      `mod sidecar;`) offers:
      - `pub fn lookup(db: &Path, arw: &Path) -> Option<dop::Uuids>`: opens
        `db` read-only, resolves `arw`'s parent folder through the `Folders`
        chain (root `Name` = the drive prefix such as `D:`, then one row per
        path component, each matched by `Name` under the previous row's `Id`
        as `ParentFolderId`), finds the `Sources` row by `FolderId` and file
        name, then the master `Items` row (`ORDER BY CreationDate, Id LIMIT 1`),
        and returns `Uuids { item: Items.Uuid, source: Sources.Uuid }`. Any
        failure (open error, lock / busy, missing table or column, no row)
        returns `None`. Name comparisons rely on the columns' `COLLATE NOCASE`
        (a `Name = ?` comparison against a column uses the column's collation).
      - `pub fn registered_uuids(arw: &Path) -> Option<dop::Uuids>`: the
        database path resolved from the environment, then `lookup`; `None`
        when the path cannot be resolved or the file does not exist. Windows
        only (`#[cfg(windows)]` as `update.rs` does, or a `database_path()`
        that returns `None` elsewhere); the writer thread has no `AppHandle`,
        so read `std::env::var_os("APPDATA")`, do not use Tauri's path API.
      - The database path is the highest-numbered `%APPDATA%\DxO\DxO PhotoLab N`
        directory that holds `Database\PhotoLab.db` (decided by the user, so an
        upgrade to PhotoLab 11 keeps working). The directory-name parsing
        (`DxO PhotoLab N` -> `N`, ignoring non-matching names) is unit-tested.
      - `lookup` only returns Uuids that look like a UUID (hex digits and
        hyphens in the 8-4-4-4-12 shape); anything else is `None`. The `.dop`
        template writes the Uuids between quotes unescaped, so a value read
        from another program's database must not be able to break the Lua
        literal (raised by the Step 1 local review). Covered by a fixture test.
    - `crates/app/src/sidecar.rs`: `write_kind`'s `current == None` branch
      computes `uuids` once for a `.dop` kind (`(kind == SidecarFormat::Dop)
      .then(|| photolab::registered_uuids(arw)).flatten()`) and passes it into
      `SidecarFormat::write_rating` / `write_label`, which forward it to
      `dop::*`; the `current == Some` branch passes `None`. XMP ignores it.
      When `write_rating` then `write_label` both run on a fresh sidecar, only
      the first mints, so the same `uuids` can be handed to both or moved
      into the first; either is fine as long as the label write patches the
      rated bytes as it does now.
    - Unit tests in `photolab.rs` build a fixture database in a temp dir with
      `rusqlite` (`CREATE TABLE Folders (Id INTEGER PRIMARY KEY, Name TEXT
      COLLATE NOCASE, ParentFolderId INTEGER)`, `Sources (Id, Name TEXT
      COLLATE NOCASE, Uuid TEXT, FolderId INTEGER)`, `Items (Id, SourceId,
      Uuid TEXT, CreationDate TEXT, ModificationDate TEXT,
      StorePickOrReject INTEGER)`; extra real-world columns are not needed)
      and cover:
      - a registered file: returns the Source Uuid and the master Item Uuid;
      - master selection: a source with two items (master with the earlier
        `CreationDate` / lower `Id`, plus a virtual copy) returns the master's;
      - case-insensitive matching: the query path differs in case from the
        stored folder and file names (`d:\photos\...\_dsc0001.arw` vs `D:` /
        `Photos` / `_DSC0001.ARW`) and still matches;
      - fallback: an unregistered file, a wrong folder chain (same file name
        in a sibling folder must not match), a missing database file, and a
        database without the tables all return `None`.
      - The fixture paths are Windows-shaped; if the test must also compile
        and pass on macOS / Linux CI, either gate the tests with
        `#[cfg(windows)]` alongside the code, or make `lookup` split the path
        with `Path::components()` so a POSIX path (`/Photos/...`) resolves
        under a root named `/`, whichever keeps the code smaller. Note the
        choice in `learnings.md`.
    - A `log::debug!` line when the lookup fails with an error (not on a
      plain "not registered"), so a fallback is diagnosable from the log.
    - `CLAUDE.md`'s layout paragraph mentions `src/photolab.rs` in one clause
      (the PhotoLab database lookup that gives a fresh `.dop` the registered
      image's Source and master Item Uuids so PhotoLab does not import it as
      a virtual copy).
    - Manual check on the user's machine, recorded in `learnings.md`: a
      registered image with no `.dop`, judged in Riffle, shows the pick on the
      master in PhotoLab with no new virtual copy; an unregistered image still
      gets random Uuids.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Reuse `rusqlite` (already a dependency, `bundled`), following
      `Index::open_reader` in `crates/app/src/index.rs` (~line 407):
      `Connection::open_with_flags(path, SQLITE_OPEN_READ_ONLY |
      SQLITE_OPEN_NO_MUTEX)` plus `busy_timeout(300 ms)`. PhotoLab may hold
      the database open while Riffle writes; the lookup runs on the sidecar
      writer thread, never on a keypress, so a short wait is acceptable.
    - Verify against the real database during implementation, before writing
      the queries (read a copy, never the live file for writing): the root
      folder row's `ParentFolderId` (NULL or 0?), the root `Name` form (`D:`
      with or without a backslash), whether `Sources.Name` holds the file name
      with extension, the `CreationDate` text format (lexical `ORDER BY` must
      give the earliest), and the database's `journal_mode` (a WAL database
      opened read-only needs a readable `-shm`; if that fails, try the
      `immutable=1` URI open as a second attempt or fall back). Record what was
      found in `learnings.md`.
    - Path splitting: take `arw.parent()`'s `Path::components()`; on Windows
      that is `Prefix("D:")`, `RootDir`, then `Normal` parts. Skip `RootDir`;
      a UNC or verbatim prefix that does not map to a plain drive is a
      fallback (`None`), no special handling.
    - Query shape: a handful of small prepared statements walked in Rust is
      simpler to read and test than one recursive CTE; keep it that way
      unless the folder chain turns out to need one.
    - `write_kind` is the single choke point for minting; do not thread the
      Uuids through `Writer::set` or the index.

- [ ] Step 3: Document the PhotoLab virtual-copy behaviour for users and future agents
  - Done when:
    - `README.md` "Working with other software" gets a short
      `### DxO PhotoLab` subsection (placed before `### Lightroom Classic` or
      after it, matching the format order of the list above it) stating: a
      sidecar Riffle creates for an image PhotoLab has already seen would
      otherwise be imported as a virtual copy, so on Windows Riffle reads the
      image's identifiers from PhotoLab's database (the newest
      `%APPDATA%\DxO\DxO PhotoLab N\Database\PhotoLab.db`) and writes them into
      the new `.dop`; when the database is not found (macOS, or PhotoLab not
      installed) the sidecar still works and PhotoLab imports it as the master,
      unless the image was already registered, in which case it appears as a
      virtual copy; an existing `.dop` is edited in place as before. Keep it to
      a few sentences in the README's voice.
    - `README.ja.md` gets the same subsection in Japanese, in the same PR.
    - `docs/agents/tauri-app.md` gets a new Hit entry next to the `.dop`
      entries (~line 521 onward), e.g. "PhotoLab matches `.dop` items by
      Uuid; a fresh sidecar on a registered image becomes a virtual copy",
      summarising the experiment table from Purpose, the database tables,
      the timestamp condition, the read-only / fallback rule, and the
      correction to the existing note that the database "shadows the
      sidecar" (it does not shadow it; it imports unknown Uuids as new
      items). Source line points at this plan's `learnings.md` once archived
      (`docs/plans/_archived/<this folder>/learnings.md`, written as a code
      span, not a link).
    - `mise run ci` passes (lychee checks the Markdown links).
  - Implementation approach:
    - Assumes Step 2 is merged so the documented behaviour is what ships.
    - English everywhere except `README.ja.md`; keep both READMEs in sync in
      the one PR, as `CLAUDE.md` requires.

## Trade-offs and risks

- **Which PhotoLab version's database.** Decided: scan `%APPDATA%\DxO\` for
  the highest-numbered `DxO PhotoLab N` directory holding
  `Database\PhotoLab.db`, so an upgrade keeps working. It could pick a newer,
  unused install (unlikely: PhotoLab migrates the database on upgrade). No
  setting is added.
- **API shape in `dop.rs`.** Chosen: an extra `uuids: Option<Uuids>`
  parameter on `write_rating` / `write_label`, used only on the fresh path;
  consistent with how `orientation` was added.
- **Testing the wiring end to end.** Not automated (it would need a
  database-path override nobody asked for). `photolab::lookup` is tested
  directly and the wiring is covered by the manual check in Step 2.
- **Master selection.** Earliest `CreationDate` and lowest `Id` coincide; the
  query orders by both. If the real database shows a case where they disagree
  (a copied item keeping an older `CreationDate`), prefer lowest `Id` and note
  it.
- **Timestamps.** PhotoLab ignores a sidecar older than its item's
  `ModificationDate`. Riffle stamps the current time, so this only fails if
  the system clock is behind the last PhotoLab edit; no mitigation planned.
- **Database in use.** A busy database yields `None` after the 300 ms wait,
  so a judgment written while PhotoLab is mid-transaction gets random Uuids
  and may still produce a virtual copy; the retry / debounce in the writer
  does not re-run the lookup for an already-written sidecar. Accepted for
  now; the log line makes it visible.
- **macOS.** No lookup, so the virtual-copy problem persists there for
  registered images until the database location is confirmed. The README
  says so.
- **Cross-platform tests.** The fixture paths are Windows drive paths; CI
  presumably runs the app tests on macOS / Linux too, so the lookup's path
  splitting or the tests need a platform-neutral form. Decide in Step 2 and
  record it.

## Progress

- (2026-09-28) Step 1 complete
