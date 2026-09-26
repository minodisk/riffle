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

# Resume a folder at the last viewed photo

## Purpose

Culling a folder rarely happens in one sitting. Today every folder open (a
click in the tree, a drop, the last-folder reopen at launch, a switch back
from another folder) selects the first file, so the user has to page or
scroll back to where they stopped. Remembering, per folder, the file that was
current in the strip, and restoring it on the next open of that folder, lets
the user resume where they left off, including after a crash or a kill
mid-session.

The position is "the last viewed file", not "the last judged file": sidecars
carry no timestamp, users revisit and re-rate earlier files, and many files
are passed without a mark.

## Steps

- [x] Step 1: Remember the current file per folder in the index and restore it on open
  - Done when:
    - Opening folder A, moving to photo N, opening folder B, and opening A
      again selects N (via the tree, a drop, or the launch reopen).
    - Killing the app while on photo N of A and relaunching (A is the last
      folder) selects N.
    - If the remembered file is no longer in the folder, the first file is
      selected and no error is shown. If it is still in the folder but hidden
      by the strip filter, the nearest passing neighbor is selected (the same
      rule `refilter` uses when a judgment hides the current file).
    - Rust tests cover the `Index` accessors, the migration and the
      eviction interaction; UI tests cover the restore target choice and the
      write coalescer.
    - `mise run ci` passes.
  - Implementation approach:
    - **Index** (`crates/app/src/index.rs`):
      - Add `last_viewed TEXT` (nullable) to `folders` in the
        `CREATE TABLE IF NOT EXISTS folders` statement, bump
        `SCHEMA_VERSION` to 17 and extend the version-history doc comment at
        the top of the file with a v17 line.
      - Migration guard: `ALTER TABLE folders ADD COLUMN last_viewed TEXT`
        keyed to the versions that already have a `folders` table without the
        column (the table was introduced at v8; for older versions the
        `CREATE TABLE` above creates it with the column). Verify the exact
        lower bound against the existing guards before writing it, per the
        "Bumping `SCHEMA_VERSION` can strand an old per-version column guard"
        entry in `docs/agents/tauri-app.md`, and re-check every existing
        guard on the bump.
      - `Index::last_viewed(&self, dir: &str) -> Result<Option<String>, String>`
        (a `SELECT last_viewed FROM folders WHERE dir = ?1`).
      - `Index::set_last_viewed(&mut self, dir: &str, path: &str) -> Result<(), String>`
        as an upsert: `INSERT INTO folders (dir, opened_at, last_viewed)
        VALUES (?1, ?2, ?3) ON CONFLICT (dir) DO UPDATE SET last_viewed =
        excluded.last_viewed`, with `now_secs()` for a fresh row. The upsert
        makes the write independent of whether `reconcile` (which creates the
        row on scan) has run yet for this open. `reconcile`'s own upsert only
        sets `opened_at`, so it keeps the column; leave it as is.
      - `evict_folder` and `clear` need no change: dropping the `folders` row
        drops the position, which is the intended bound.
      - Tests, in the existing `tests` module next to the `opened_at` /
        `set_opened_at` helpers: set/get round trip; unknown dir is `None`;
        `set_last_viewed` before any `reconcile` creates the row and a later
        `reconcile` keeps the value; `evict_folder` of a folder without
        ratings forgets it; a v16 database gains the column and keeps its
        `folders` rows (fixture built with `open`, then
        `ALTER TABLE folders DROP COLUMN last_viewed; PRAGMA user_version = 16;`,
        following the v15 test's shape).
    - **Commands** (`crates/app/src/commands.rs`, registered in
      `crates/app/src/main.rs`'s `generate_handler!`):
      - `last_viewed(app, dir: String) -> Option<String>`: `async`, canonicalize
        `dir` with the existing `canonicalize`, read through `AppIndexReader`
        inside `spawn_blocking` (as `folder_entries` does); `None` when the
        index is unavailable or on error. It returns the stored path as is;
        the frontend decides whether that file is still listed.
      - `set_last_viewed(app, dir: String, path: String)`: `async`,
        canonicalize, `spawn_blocking` with the `AppIndex` writer lock; a
        failure is logged, not returned (as `remember_folder`). Both must be
        `async` + `spawn_blocking`: the writer lock can be held by a scan's
        `write_batch`, so a synchronous command would block the main thread.
    - **Frontend**:
      - New module `crates/app/ui/src/resume.ts` (+ `resume.test.ts`) with
        two pure pieces:
        - `resumeTarget(remembered: string | null, allFiles: readonly string[]): string | undefined`:
          `undefined` when `remembered` is `null` or not in `allFiles`
          (falls back to the first file), otherwise `remembered`. It only
          checks membership: at open, entries (capture time, ratings, flags,
          labels) are not loaded yet, so whether the remembered file passes
          the strip filter and where its nearest passing neighbor sits cannot
          be decided from `allFiles` alone.
        - `firstEntriesAnchor(pending: string | undefined, provisional: string | undefined): string | undefined`:
          `pending` when set, else `provisional` — the anchor `refilter`
          should resolve against on the first `folder_entries` refresh after
          an open.
        - A write coalescer (one `{ dir, path }` in flight at a time, only
          the latest pending value kept, the next send issued when the
          in-flight one settles; the `send` function is injected so tests
          run without Tauri). This serializes the writes so two
          `spawn_blocking` tasks cannot land out of order, and collapses
          a held-down arrow key into a few writes. Each value carries its
          own `dir`, so a write for the previous folder that is still pending
          at a folder switch is not misattributed.
      - `main.ts`:
        - A module-level `pendingResume: string | undefined` holds the queued
          resume target for the open until entries make it possible to
          resolve where it lands.
        - In `openDirectory`, invoke `last_viewed` alongside `list_arw`
          (`Promise.all`; the existing `token !== folderToken` guard covers
          both). `entries`, `ratings`, `flags` and `labels` are all cleared
          before `files` is computed from `allFiles` (previously `flags` /
          `labels` were cleared later, so the provisional `files` briefly
          judged against the previous folder's values). `pendingResume` is
          set to `resumeTarget(remembered, allFiles)`; `index` starts at `0`
          (the provisional anchor, since capture time and judgments are not
          loaded yet).
        - In `refreshEntries`, the first refresh after an open (this is the
          only reader of `pendingResume`) calls
          `refilter(firstEntriesAnchor(pendingResume, files[index]))` instead
          of the plain `refilter()` every other refresh uses, then clears
          `pendingResume`. This is where entries (and so capture-time order,
          ratings, flags and labels) are loaded, so `refilter`'s own
          `anchorAfterFilter` now resolves the remembered file's nearest
          passing neighbor correctly, in capture-time order.
        - In `show()`, when `openDir !== null`, `files[index] !== undefined`
          and `pendingResume === undefined`, push
          `{ dir: openDir, path: files[index] }` to the coalescer (whose
          `send` invokes `set_last_viewed`). The `pendingResume` guard keeps
          the provisional `show()` call at the end of `openDirectory` (before
          entries load) from overwriting the remembered file with the
          provisional anchor.
        - `resync()` needs no change: it anchors on `files[index]`, which by
          the time a resync can run has already settled on the restored file.
      - Add `resume.ts` to the frontend list in `CLAUDE.md`'s Layout
        paragraph, and mention `folders.last_viewed` in the `index.rs`
        description there (one clause each; keep the paragraph's style).
    - README.md / README.ja.md: no change; they do not describe folder
      reopening at this level.
    - Record in `learnings.md` anything the migration bump or the writer-lock
      contention turns up.

## Trade-offs and risks

- **Index vs settings store.** The index was chosen: the `folders` table is
  already per-folder state keyed by the canonical dir, the position is
  bounded by the existing eviction, and the write path (async command,
  `spawn_blocking`, writer lock) exists. The settings store would need a
  growing dir→path map with no eviction and is documented as "preferences,
  not a cache". Consequence of the choice: `Clear Cache` and age-based
  eviction (a folder not opened within `MAX_AGE_SECS` and holding no dirty
  rating) forget the position, and the change costs a `SCHEMA_VERSION` bump.
- **Fallback when the remembered file is filtered out.** Chosen: the nearest
  passing neighbor via `anchorAfterFilter`, because that is what the app
  already does when the current file drops out of the filter, so the
  behavior stays consistent.
- **Write frequency.** Every `show()` triggers a (coalesced) upsert. The
  upsert is one row in a small table, but it takes the writer lock the scan's
  `write_batch` also takes; with a scan running it waits behind one batch
  transaction, off the main thread. If this turns out measurable during a
  scan, a short trailing delay (~200 ms) in the coalescer is the fix, at the
  cost of losing at most that window on a kill.
- **Quick A → B → A switch.** The `last_viewed` read for the second open of
  A can run before A's last coalesced write has landed, restoring the
  previous file instead of the very last one. Accepted as is (a sub-second
  race that lands one file off); the fix, if wanted, is to have
  `openDirectory` await the coalescer's in-flight write before invoking
  `last_viewed`.
- Out of scope, possibly a future todo: a "jump to the next unmarked photo"
  shortcut.

## Progress

- (none yet)
