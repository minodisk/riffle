# Learnings: auto-culling-finished

## Step 1

- The v8-v21 fixtures in `index.rs` all build the current schema with
  `open` and then drop what came later, so the v22 bump meant adding
  `ALTER TABLE folders DROP COLUMN culling_manual;` and `DROP TABLE views;`
  to each of the 13 fixtures that already dropped `culling_finished` (a
  regex over the `DROP COLUMN culling_finished;` line did it in one pass).
  The v2-v7 fixtures need nothing: they create their own tables or drop
  `folders` entirely, and `views` comes from `CREATE TABLE IF NOT EXISTS`.
- A provisional file shown by `openDirectory` before the resume target
  resolves must not count as viewed; `markViewed` reuses the same
  `pendingResume !== undefined` guard as the `set_last_viewed` write in
  `show()`. When the resume target is `files[0]` itself, `refilter` re-shows
  it (`mustReshow`), so it still gets recorded.
- `viewedWriter` keys pending paths by folder, so a batch queued for the
  previous folder is not merged into the next folder's call after a quick
  folder switch.
- Both `failPreview` callers already check that the failure is for the
  current `seq`, so `markViewed` is called from inside `failPreview`
  directly.
- `cargo` is not on the Git Bash `PATH` here; run it as `mise exec -- cargo`.

## Deferred issues (todo candidates)

- **Pending manual check (GUI, Windows/macOS desktop app).** Step 1's
  checkbox was ticked on the automated criteria (`mise run ci`,
  `cargo test -p riffle-app`, vitest); the GUI behaviour needs a hands-on
  check by the user. Steps, with AI analysis on so the folder menu shows
  the check:
  1. Open a RAW folder never viewed before. Hide one frame with a strip
     filter, page through every other frame, then clear the filter and show
     the hidden one. Right-click the folder in the folder tree: expected
     `Culling Finished` checked, and the ranks / status refreshed by the
     refit (`ranking-updated`).
  2. Clear the check by hand, page through the folder again (or reopen
     it): expected the check stays clear.
  3. Open a JPEG-only (view-only) folder and page through all of it:
     expected never marked.
  4. In a fresh RAW folder, jump straight to the last frame only: expected
     not marked.
  5. Page halfway through a fresh RAW folder, quit Riffle, relaunch, reopen
     the folder and view the rest: expected marked.
  Related files: `crates/app/ui/src/main.ts` (`markViewed`),
  `crates/app/ui/src/viewed.ts`, `crates/app/src/rank.rs`
  (`finish_culling`), `crates/app/src/index.rs`.
