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

# Auto-mark Culling Finished once every file has been previewed

## Purpose

`folders.culling_finished` (image-embeddings Step 7,
`../_archived/20261012-image-embeddings/ranking.md`) decides which folders'
unpicked frames count as "not chosen" negatives in the personal ranking's
refit. Today the user sets it by hand from the folder tree's right-click
menu. Culling in Riffle means stepping through the filmstrip until every
frame has been shown in the preview, so the app can set the mark itself
once that has happened, across sessions, and the user no longer has to
remember to. The manual check stays as an override: it can set the mark
without viewing everything and clear an auto-set mark, and a cleared mark
is never auto-set again.

Decisions adopted (user decisions on 2026-10-13, plus defaults the user
approved with the plan):

- Criterion (user): every file of the folder has been shown in the preview
  at least once, across sessions. Jumping to the last frame alone does not
  count.
- The manual check stays (user).
- A file counts as viewed when its preview has actually been displayed in
  the main view: the decoded bitmap drawn for the current `seq` in
  `main.ts`'s worker `message` handler (where `shown` is set), or the
  preview failure shown for the current file by `failPreview` (an
  unreadable file would otherwise block the folder forever). Thumbnails
  scrolled past in the strip, the compare pane and provisional anchors
  before a resume target resolves do not count. Jumping to the last frame
  therefore views one file, not the folder.
- The history is persistent and filter-independent: files a filter hides
  stay unviewed until shown. It is tracked whatever `aiEnabled` is, so
  turning AI on later sees it; only the refit is gated on `aiEnabled`.
- Storage is a new `views` table, not a column on `files`: `reconcile`
  deletes and pass 1 `INSERT OR REPLACE`s `files` rows on any size / mtime
  / `EXTRACTOR_VERSION` change, so a column there would be wiped by every
  re-extraction. `views` is tiny (path, dir, viewed_at).
- `views` survives the 30-day / size eviction, so a folder abandoned
  mid-cull still finishes when the user returns weeks later; a manual
  Clear Cache drops it with everything else.
- The all-viewed decision is made in the frontend against `allFiles` (the
  folder's complete `list_arw` listing). The index's `files` rows are
  incomplete while a scan re-extracts, so the backend cannot judge
  completeness reliably and does not try; `finish_culling` only applies the
  override rules.
- Once auto-set the mark stays even when new files appear. A manual toggle
  of any kind (set or clear) records `folders.culling_manual = 1`, and the
  auto path never touches a folder with it set.
- JPEG-only (view-only) folders are excluded: no views recorded, no
  auto-mark.
- Nothing here changes a judgment.

## Steps

- [x] Step 1: Record previews in the index and auto-mark a fully viewed folder
  - Done when:
    - `SCHEMA_VERSION` is 22: `views (path TEXT PRIMARY KEY, dir TEXT NOT NULL, viewed_at INTEGER NOT NULL)` with a `views_dir` index, created with `CREATE TABLE IF NOT EXISTS` so every older database gains it empty (as `embeddings` did at v20), and `folders.culling_manual INTEGER NOT NULL DEFAULT 0` appended at the end of `CREATE TABLE folders`, added to v8–v21 databases by an `ALTER TABLE` guarded with `(8..22).contains(&version)`; `21` joins the accepted-version list.
    - `Index` gains `mark_viewed(dir, paths: &[String])` (`INSERT OR IGNORE`, keeping the first `viewed_at`), `viewed_files(dir) -> Vec<String>`, `finish_culling(dir) -> bool` (`UPDATE folders SET culling_finished = 1 WHERE dir = ?1 AND culling_finished = 0 AND culling_manual = 0`, creating the `folders` row first as `set_culling_finished` does; returns whether it changed anything), and `set_culling_finished` also sets `culling_manual = 1`. `rename_dir` and `rename_file` carry `views` with the other per-file tables; `clear` drops `views`; `evict_folder` keeps them (the view history outlives eviction). Since an evicted folder's `folders` row (and with it `culling_manual`) goes away, a returning folder can be auto-marked again; that is accepted.
    - Tauri commands: `mark_viewed(dir, paths)` and `viewed_files(dir)` in `commands.rs` (next to `last_viewed` / `set_last_viewed`, same canonicalize + `spawn_blocking` + log-not-return shape for the write), and `finish_culling(dir)` in `rank.rs`, which calls `Index::finish_culling` and, only when it returned `true` and `AppAi::enabled()`, spawns the same background `refit` thread `set_culling_finished` does (factor that spawn into one helper). All three registered in `main.rs`.
    - Frontend: a new `crates/app/ui/src/viewed.ts` with `allViewed(allFiles, viewed: ReadonlySet<string>): boolean` (false for an empty listing) and `viewedWriter(send)` that batches paths: one `mark_viewed` invoke in flight, paths pushed meanwhile collected into the next call (the `lastViewedWriter` pattern in `resume.ts`, with accumulation instead of last-wins). `main.ts` loads `viewed_files` in `openDirectory`'s `Promise.all` with `last_viewed`, keeps a `Set` per open folder (cleared on open, pruned nowhere else: a path gone from `allFiles` simply stops mattering), adds `files[index]` to it from the worker `message` handler when `shown` is set for the current `seq` and from `failPreview` when the failure is for the current file, skipping `viewOnly` folders and when `openDir` is null, pushes it through the writer, and when the set now covers `allFiles` invokes `finish_culling` once per folder open (a boolean guard reset in `openDirectory`; the backend is idempotent, the guard only avoids an invoke per page turn). The `ranking-updated` event the refit emits already refreshes the ranks.
    - Tests (Rust, `index.rs`): `a_v21_database_gains_views_and_culling_manual_and_keeps_its_folders` (fixture: `DROP TABLE views`, `ALTER TABLE folders DROP COLUMN culling_manual`, `PRAGMA user_version = 21`; asserts `SCHEMA_VERSION`, files kept, `culling_finished` kept, `finish_culling` works); every existing fixture that fakes v8–v21 also drops `culling_manual` (and `views`, for fidelity) so the new `ALTER` does not hit a duplicate column; `mark_viewed` / `viewed_files` round trip and idempotence; `finish_culling` sets an untouched folder and returns `true`, returns `false` on a second call and on a folder the user toggled (`set_culling_finished(true)` then `false`, i.e. the manual clear); `set_culling_finished` records `culling_manual`; `rename_dir` / `rename_file` carry `views`; `evict` keeps a folder's `views` and `clear` drops them. Tests (vitest, `viewed.test.ts`): `allViewed` on an empty listing, a partial set, a full set, a set with extra (gone) paths; `viewedWriter` sends one batch at a time and coalesces paths pushed while in flight, and keeps going after a rejected send. `context.test.ts` unchanged (the menu item is the same).
    - Docs: `docs/humans/usage.md` (folder tree paragraph and **Recommended order**: the mark is set by itself once every frame of the folder has been shown in the preview, across sessions, filtered-out frames included, and the check still sets or clears it by hand, a hand-cleared folder staying clear) with `usage.ja.md` in sync; `README.md` / `README.ja.md` Recommended order bullet; `CLAUDE.md` layout (the `index.rs` sentence gains `views` and `culling_manual`, the `rank.rs` sentence gains `finish_culling`, the frontend list gains `src/viewed.ts`); `docs/agents/tauri-app.md` ranking bullet (how the mark is auto-set, why `views` is its own table and why the frontend decides completeness), plus a line under the `SCHEMA_VERSION` guard entry for the v22 fixtures if anything new was learned.
    - `mise run ci` and `cargo test -p riffle-app` pass.
    - Pending manual GUI check, recorded in `learnings.md`: open a RAW folder, page through every frame (including one hidden by a filter, then unhide and show it), right-click the folder and see `Culling Finished` checked and the status / ranks refreshed; clear it by hand, page the folder again and see it stay clear; a JPEG-only folder never gets the mark; the last frame alone does not finish the folder; quitting mid-folder and resuming later completes it.
  - Implementation approach:
    - Schema changes follow the `docs/agents/tauri-app.md` "Bumping `SCHEMA_VERSION`" entry: the column goes at the end of `CREATE TABLE folders`, the guard is range-keyed, and migration asserts read `SCHEMA_VERSION`.
    - `mark_viewed` failing only loses history, so the command logs (`log::warn!`) rather than returns an error, like `set_last_viewed`.
    - Keep the current behaviour that `Culling Finished` only appears in the menu while `rankingOn`; an auto-set mark made while AI is off becomes visible when AI is turned on. No new UI.
    - MCP companion, `tree.ts` and `context.ts` need no change.

## Trade-offs and risks

- **A failed preview counts as viewed.** Chosen so a corrupt or unsupported file (e.g. an HDR CR3 the decoder refuses) cannot block the folder.
- **The frontend decides completeness.** The backend `finish_culling` trusts the caller; a stale `allFiles` (a file added by the watcher between listing and the check) could finish a folder one frame early. The watcher's `resync` refreshes `allFiles` and the mark stays by design, so this is accepted. Alternative, rejected: a backend check against `files` rows, which are incomplete during re-extraction.
- **Any manual toggle blocks the auto path**, not only a clear: a user who checks the box by hand and never clears it is unaffected, and a set-then-clear is the clear case the user asked for.
- **Eviction forgets `culling_manual`** with the `folders` row while `views` survive, so a hand-cleared folder evicted and later reopened can be auto-marked on its next full view. Rare (the folder must sit unopened for 30 days) and accepted.
- **Timing of `finish_culling` versus the batched `mark_viewed` writes.** The auto-mark does not read `views`, so it may land before the last batch and that is harmless. If the backend later verifies completeness, the writer must flush first.

## Progress

- (none yet)
