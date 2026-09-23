<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Re-extract cached index rows when the extractor changes

## Purpose

`Index::reconcile` (`crates/app/src/index.rs`) treats a `files` row as valid
while its `size` / `mtime_ns` match the file on disk. Nothing invalidates a row
when the extraction code itself changes, so a cache built before #376 (mask
SHORT TIFF entries) keeps serving stale results for files that now extract
fine: on Windows the index for "AmazonPhotos (1)" still holds SIGMA fp L
SDIM0558.DNG as an error row (`no embedded preview`) and SDIM0552.DNG with a
5 KB thumbnail and a sharpness score from the old JPEG tier selection. Error
rows in particular are never retried.

Recording an extractor version on every `files` row and treating a mismatch
like a size/mtime change makes any change to what
`riffle_core::scan::extract` produces (ARW/DNG parsing, embedded JPEG tier
selection, thumbnail generation, face detection, sharpness) reach existing
caches by bumping one constant, without dropping the whole `files` table (the
v9/v10 approach) and without touching `ratings`.

## Steps

- [x] Step 1: Store an extractor version per `files` row and re-extract rows whose version is stale
  - Done when:
    - `crates/app/src/index.rs` has a `const EXTRACTOR_VERSION: i64 = 1;`
      next to `SCHEMA_VERSION`, with a doc comment saying when to bump it:
      any change to what `riffle_core::scan::extract` produces, i.e. ARW/DNG
      parsing or embedded JPEG tier selection (`crates/core/src/arw.rs`),
      thumbnail generation, face detection or the sharpness score
      (`crates/core/src/scan.rs`, `sharpness.rs`, `faces.rs`); and that a
      bump re-extracts every row (including error rows) on the next scan of
      each folder, keeping `ratings`.
    - The `files` table has an `extractor INTEGER NOT NULL DEFAULT 0` column;
      `SCHEMA_VERSION` is 12, `12` is described in its doc comment (v12 added
      `files.extractor`; a v11 database gains it in place with `ALTER TABLE`,
      defaulting to `0` so every existing row is re-extracted once, and keeps
      `files`, `ratings` and `folders`), `11` is added to the whitelist in
      `prepare`, and the `ALTER TABLE` guard is keyed as
      `(10..12).contains(&version)` (never `!= SCHEMA_VERSION`, per the
      guide pitfall). Since `files` is dropped for `version < 10` and
      recreated with the column, the guard only has to fire for 10 and 11;
      a plain `version != 0 && version < 12` would also fire for v2-v9
      databases, which don't have `files` yet, and would fail with a
      duplicate column, discarding the cache.
    - `write_batch` writes `EXTRACTOR_VERSION` into `extractor` in both the
      `Ok` and the `Err` insert.
    - `reconcile` selects `extractor` too and a row is valid only when
      `(size, mtime_ns)` match **and** `extractor == EXTRACTOR_VERSION`;
      stale rows are deleted and their files returned to scan, exactly as for
      a size/mtime change. The module doc and the `reconcile` doc comment
      mention the version.
    - Tests in the `tests` module of `index.rs`:
      - a row written at the current version and then set to an older one
        (`UPDATE files SET extractor = 0`, or a v11 fixture without the
        column) is returned by `reconcile` with unchanged size/mtime, is gone
        from `entries`, and is served again after `write_batch`; an error row
        (`write_batch` with `Err`) at the old version is returned too;
      - rows at the current version with unchanged size/mtime are not
        returned by `reconcile` (both a success row and an error row);
      - a v11 database (build with `open` + `write_batch`, insert a
        `ratings` row, then `ALTER TABLE files DROP COLUMN extractor; PRAGMA
        user_version = 11;`) opens at 12, keeps its `files` row (visible in
        `entries`) and its rating, and `reconcile` reports the file stale.
      - the existing migration tests' `assert_eq!(version, 11)` move to 12.
    - `mise run ci` passes.
  - Implementation approach:
    - `reconcile` currently keys `known` as `HashMap<&str, (i64, i64)>`;
      extend the tuple with `extractor` (or compare a `(size, mtime, ver)`
      triple against `(f.size, f.mtime_ns, EXTRACTOR_VERSION)`) in both the
      delete filter and the returned list so the two stay in lock-step.
    - `NOT NULL DEFAULT 0` rather than a nullable column keeps the comparison
      an `i64` and makes "no version recorded" (old DB) equal to "version
      0", which is never current because the constant starts at 1.
    - Setting `EXTRACTOR_VERSION = 1` on a fresh column whose default is `0`
      is the bump the requirements ask for: every row from before this change
      is re-extracted once.
    - `entries`, `thumbnail`, `evict`, `clear` and `commands.rs` need no
      change; the `write_batch` callers in `commands.rs` tests keep working.

- [x] Step 2: Document the extractor version in the agent guide
  - Done when:
    - `docs/agents/tauri-app.md` gains a short section in the "Rust side"
      part, next to "Bumping `SCHEMA_VERSION` can strand an old per-version
      column guard", stating: bump `EXTRACTOR_VERSION` (not
      `SCHEMA_VERSION`) when extraction output changes (the same list of
      files as the constant's comment); bump `SCHEMA_VERSION` only when the
      table layout changes; and that a bump re-extracts every row, including
      error rows, on the next scan of each folder, keeping `ratings` (source:
      this plan's `learnings.md`).
    - `CLAUDE.md`'s layout paragraph for `src/index.rs` mentions the
      extractor version in a few words, matching the existing one-line style.
    - `mise run ci` passes (lychee link check covers the docs).
  - Implementation approach:
    - Assumes Step 1 is merged. Text only; keep to the guide's existing
      "(Hit)/(Inferred)" heading style.

## Trade-offs and risks

- **Per-row column vs. a whole-database extractor version.** A single stored
  version checked in `prepare` could drop `files` when it changes, like the
  v9/v10 migrations. Per-row (chosen) re-extracts lazily only the folders
  actually opened, keeps thumbnails of unopened folders in place for `evict`
  to age out, and lets `reconcile` handle it with the existing
  delete-and-rescan path. Cost: one extra integer per row and a schema bump.
- **Deleting stale rows before the rescan.** `reconcile` deletes a stale row
  immediately (see the comment in `a_changed_mtime_invalidates_the_row`).
  After a version bump a whole folder shows placeholders until its rescan
  fills in, instead of the old thumbnails. The plan keeps the existing
  behavior so `entries` never serves known-stale data (e.g. the wrong
  sharpness).
- **Migration guard.** Keying the `ALTER TABLE` to `(10..12).contains(&version)`
  (not `!= SCHEMA_VERSION`, and not a plain `version != 0 && version < 12`,
  which would also fire for v2-v9 databases that don't have `files` yet and
  fail with a duplicate column) is essential; the guide records the previous
  failure. Adding the column to a table that already has it would fail
  `prepare` and discard the cache (including dirty ratings), which is why the
  migration test matters.
- **Every folder is re-extracted once after upgrading**: a full rescan per
  folder on first open. This is the intended effect and the same cost as the
  v9/v10 upgrades.

## Progress

- Step 1: done (`7d59c28`, feat(app): re-extract index rows written by an
  older extractor). Added `files.extractor` / `EXTRACTOR_VERSION`, keyed the
  `ALTER TABLE` guard to `(10..12).contains(&version)`, and covered it with
  migration and `reconcile` tests. See `learnings.md`.
- (2026-09-24) Step 1 complete
