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

# Folder tree: keep opens off the main thread, and open on a row click

## Purpose

Opening a folder from the tree while the previous folder's scan is still
running (or winding down after a cancel) leaves the app sluggish: the
`Riffle.log` of a debug build on Windows shows `open list` taking 4.6 s and
7.4 s for folders of a few hundred RAWs while 22 rayon threads were reading
from the same drive, against 17-38 ms with no scan running. Later clicks
queue behind those opens, so the tree feels dead.

Two things cause it, both in `crates/app/src/commands.rs`:

1. `list_arw` (~line 657), `remember_folder` (~598, a `store.save`) and
   `start_scan` (~1217) are synchronous `#[tauri::command]`s. Per
   `docs/agents/tauri-app.md` ("Synchronous commands run on the main
   thread"), tauri-macros runs those inline on the main thread, so the
   listing blocks the run loop and every later IPC call.
2. `list_dir` (~175) calls `path.is_file()` for every entry: one full stat
   per file, which under disk contention is seconds for a few hundred
   files. `list_folder_in` (used by `scan_folder`) goes through the same
   function. `DirEntry::file_type()` comes for free from the directory
   listing on Windows; `crates/app/src/folders.rs::list` already uses it.

Separately, a tree row only opens its folder when the click lands on the
name text: `crates/app/ui/src/folders.ts` `render()` attaches the open
handler to the `.name` span, whose `flex: 0 1 auto` in
`crates/app/ui/style.css` makes it just the text's width, while
`.folder:hover` highlights the whole row. Clicking the row's blank area does
nothing.

After this work the tree stays responsive while a scan runs (an open
costs one cheap directory listing on a blocking-pool thread), and the whole
row opens the folder.

Out of scope, as follow-ups: making scan cancellation faster (a canceled
scan still lets its in-flight files finish, 8-15 s in the log), reducing
the number of directory listings per open (`list_arw` then
`scan_folder`'s `list_folder_in`), and the frontend cost of
`refreshEntries`.

## Steps

The user chose a single PR, so the backend and frontend changes are one step.

- [x] Step 1: Run the open-path commands off the main thread, list without a per-entry stat, and open a folder from a click anywhere on its tree row
  - Done when:
    - `list_arw`, `remember_folder` and `start_scan` in
      `crates/app/src/commands.rs` are no longer synchronous
      `#[tauri::command]`s (no `body_blocking` path), and `list_arw`'s
      directory listing and `canonicalize` run under
      `tauri::async_runtime::spawn_blocking`.
    - `list_dir` no longer calls `path.is_file()` per entry; it uses
      `entry.file_type()` and documents the symlink behavior.
    - `start_scan`'s `latest_id` check, `pending.remove`, `state.running =
      Some(..)` store and the `scan-state` emit still happen under one
      continuous hold of the `Scans` lock, exactly as the doc comments above
      `Scans` (~820) and `start_scan` (~1210) describe; those comments still
      read true.
    - Existing `commands.rs` tests pass, plus a listing test for the new
      behavior (symlinked RAW still listed, `#[cfg(unix)]`, modeled on
      `folders.rs::symlinked_dir_and_raw_file_are_counted`).
    - `docs/agents/tauri-app.md` gains a short bullet under "Synchronous
      commands run on the main thread" recording this Hit: a per-entry
      stat in a sync command is seconds under disk contention, and
      `#[tauri::command(async)]` on a non-`async` fn is not
      `spawn_blocking` (see below).
    - In `crates/app/ui/src/folders.ts` `render()`, the open handler is on
      the `.folder` row, not the `.name` span; a click on the `.expander`
      still only toggles and does not also open.
    - `crates/app/ui/style.css`: `cursor: pointer` applies to the whole
      `.folder` row (`.name` keeps its `flex: 0 1 auto` / ellipsis rules).
    - `mise run ci` passes.
  - Implementation approach (backend):
    - Do not use `#[tauri::command(async)]` on a sync fn as the fix for
      `list_arw`. tauri-macros 2.6.3 (`src/command/wrapper.rs`,
      `body_async`) calls a non-`async` fn directly inside the `async move`
      block it hands to `respond_async_serialized`, so the body runs on a
      tokio runtime worker, not a blocking-pool thread. It is off the main
      thread but stalls a runtime worker for the whole listing, which the
      guide's rule ("an `async` command doing blocking IO wraps it in
      `spawn_blocking`") forbids.
    - `list_arw`: `pub async fn list_arw(dir: String) -> Result<Vec<String>,
      String>`, with `canonicalize(&dir)` (a `std::fs::canonicalize`
      syscall, `commands.rs` ~2009), `list_arw_in` and the `open list` log
      line all inside one `spawn_blocking` closure, `.await.map_err(|e|
      e.to_string())?` outside, the way `list_subfolders` in
      `crates/app/src/folders.rs` and the listing block of `scan_folder`
      (~1027-1032) do it. Keep the log line's format (`open list: dir=..
      raws=.. in ..ms`).
    - `remember_folder`: make it `pub async fn` and wrap the `settings` +
      `set` + `save` in `spawn_blocking` (the store handle is an `Arc`, so
      it moves into the closure); keep the "logged, not returned" behavior.
      `set_sort_order`, `set_auto_advance` and `update_keymap` are not on
      the open path and stay as they are.
    - `start_scan`: make it `pub async fn start_scan(app: tauri::AppHandle,
      scan_id: u64) -> Result<(), String>` and move the existing body
      verbatim into a `spawn_blocking(move || { .. })` closure that is
      awaited (the body takes the `Scans` std mutex, which can wait behind
      `spawn_eviction`'s VACUUM or a `scan_folder` critical section; a
      blocking-pool thread is the right place for that wait). The closure
      owns `app`; `app.state::<Scans>()` and the inner
      `tauri::async_runtime::spawn_blocking` for the scan task both work
      from a blocking thread (the runtime handle is global; `scan_folder`
      already spawns from a non-main thread). There is no `.await` between
      the `latest_id` check and the `running` store, so the lock is held
      continuously, as before. Verify by reading the two `// Emit while
      still holding the lock` comments (~1303 and ~1321) against the new
      code; update only the wording that mentions the calling thread, if
      any.
    - `list_dir` (~175): replace `path.is_file()` with the pattern of
      `folders.rs::list` (~120-136): `entry.file_type()`, and only for
      `file_type.is_symlink()` fall back to one `std::fs::metadata(&path)`
      (which follows the link) so a symlinked RAW is still listed, as
      `is_file()` did before. Document that in the function comment. Since
      the "is this a file, following a symlink" block would then exist
      twice, lift it into a small `pub(crate)` helper in `folders.rs` used
      by both, without growing it beyond what both call sites need.
    - The sidecar branch of `list_dir` still calls `index::stat(&path)` for
      entries matching the sidecar format (their size and mtime are the
      point); that stays. Check the branch order so a symlinked non-RAW
      entry does not lose the sidecar path.
    - Tests: `commands.rs`'s `tests` module has `temp_dir` /
      `remove_temp_dir` helpers (~2300-2311) and the `list_arw_in` tests
      around ~3000-3070; add the symlink test next to them under
      `#[cfg(unix)]`. Run `cd crates/app && cargo test list_` (bin-only
      crate: no `--lib`).
    - `main.rs` `invoke_handler` (~551-564) needs no change; async and sync
      commands register the same way.
  - Implementation approach (frontend):
    - Move `name.addEventListener("click", ..)` to `row`; in the expander's
      listener call `event.stopPropagation()` before `toggle(node.path)`,
      so the row handler does not fire for it. The `.count` badge, when
      present, becomes clickable as part of the row, which is intended.
    - Keep `role="treeitem"`, `aria-*` and `dataset.path` as they are;
      `reveal()`'s `.folder.current` lookup is unaffected.
    - No unit test: the frontend tests run under `environment: "node"`
      (`vite.config.ts`), `tree.test.ts` covers only the pure tree
      functions, and adding a DOM environment for one handler is more than
      this change warrants.
  - Manual check (Windows, debug build, folders on D:): open a large folder,
    then click through several folders while the scan runs; `open list` in
    `Riffle.log` should stay in the tens of milliseconds and the tree should
    react to each click. Click the blank area right of a folder name
    (opens), the arrow (only expands/collapses), and the count badge
    (opens). The implementer cannot drive the GUI; leave this to the user
    and say so in the PR body.

## Trade-offs and risks

- **`remember_folder` / `start_scan`: `spawn_blocking` versus a plain
  `async fn`.** A plain `async fn` would also get them off the main thread
  with less code, but would block a runtime worker on the settings write or
  on a contended `Scans` lock. `spawn_blocking` is chosen for both, for
  consistency with `list_arw` and the guide's rule.
- **Symlink behavior in `list_dir`.** Following symlinked RAWs (the
  `metadata` fallback) preserves today's `is_file()` behavior and matches
  `folders.rs::list`, at the cost of one extra stat per symlink only.
- **Windows hidden attribute.** `folders.rs::list` also skips
  `FILE_ATTRIBUTE_HIDDEN` directories via `entry.metadata()`; `list_dir`
  never filtered hidden RAW files and this plan does not add that.
- **No DOM test for the row click.** Verified manually; a regression would
  only be caught by a user.
- **Follow-ups not done here:** faster scan cancellation (in-flight files
  finish; 8-15 s in the log), one directory listing per open instead of
  two (`list_arw` then `list_folder_in`), and `refreshEntries` cost.

## Progress

- 2026-09-25: Step 1 done. `list_arw`, `remember_folder` and `start_scan` are
  now `async fn`s that run their bodies under `tauri::async_runtime::spawn_blocking`
  instead of as synchronous commands on the main thread. `list_dir` no longer
  stats every entry; it uses `entry.file_type()` via the new `folders::kind`
  helper (falling back to `std::fs::metadata` only for symlinks), shared with
  `folders.rs::list`. In `crates/app/ui/src/folders.ts`, the open handler moved
  from the `.name` span to the `.folder` row, with the expander stopping
  propagation so it only toggles. The manual GUI check and the `#[cfg(unix)]`
  symlink test could not be run on the implementer's Windows machine and are
  left to the user and to CI.
