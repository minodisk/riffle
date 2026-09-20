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

# Move rejected files to the Trash

## Purpose

Culling ends with the rejects still in the folder; removing them means going
to another tool (`todo.md`, "App: rejected files cannot be cleared out from
the app"). This adds `File > Move Rejected to Trash…`: after a confirmation
that shows the count, every file of the open folder judged as a reject
(`rating == -1`) is moved to the OS trash together with the sidecars that
exist for it on disk (`.xmp` and `.dop`, whichever are present). Nothing is
unlinked; the Trash's "Put Back" restores the RAW, its sidecar and, since the
index row is kept, its judgement.

Facts the design rests on (from reading the code):

- A reject is `rating == -1` in the frontend `ratings` map
  (`crates/app/ui/src/main.ts`) and in the index `ratings` table; `pick` is a
  separate flag. `Index::entries` joins `files`, so once `reconcile` drops the
  `files` row of a gone file it disappears from the strip without any change
  to `ratings`.
- `sidecar::write` (`crates/app/src/sidecar.rs`) does not check that the RAW
  exists: a reject still inside the writer's 300 ms debounce when its RAW is
  trashed would mint an orphan sidecar. The command therefore drains the
  writer (`writer.flush(DRAIN_TIMEOUT)`) before touching the disk, as
  `clear_index` does.
- `sidecar::existing_sidecar(arw, format)` already does the case-insensitive
  on-disk lookup (`H.ARW.DOP` vs `h.arw.dop`, see `docs/agents/tauri-app.md`).
- `commands::clear_index` is the pattern: refuse while
  `ScansState::scanning()`, native dialog awaited on
  `tauri::async_runtime::channel(1)`, the work in `spawn_blocking`, the
  `scanning()` re-check under the `Scans` lock after the dialog, and the
  `SCAN_RUNNING` message in `#status` when refused.
- The folder watcher fires on RAW deletions (`watch::triggers`), so a
  `folder-changed` -> `resync()` follows a trash anyway; the frontend still
  calls `resync()` itself right after the command returns, and the watcher's
  trigger collapses into `resyncPending`.
- No trash capability is in the dependency tree (`tauri-plugin-fs` is present
  transitively but has no trash API). The `trash` crate (5.2.x; macOS,
  Windows, Linux freedesktop) is added.

## Steps

- [x] Step 1: `trash_rejected` command with the `trash` crate, unit-tested
  - Done when:
    - `crates/app/Cargo.toml` depends on `trash` (latest 5.x) and
      `Cargo.lock` is updated.
    - A new `crates/app/src/trash.rs` holds the logic split into a pure,
      testable part and the one call into the `trash` crate:
      - `plan(dir: &Path, paths: &[String]) -> Result<Vec<Group>, String>`:
        rejects a path whose parent is not `dir` or that is not
        `riffle_core::scan::is_raw_file`; for each accepted RAW collects the
        sidecars that exist on disk for **both** formats via
        `sidecar::existing_sidecar` (made `pub(crate)`), regardless of the
        current `SidecarFormat` setting. `Group { raw: PathBuf, sidecars:
        Vec<PathBuf> }`.
      - `run(groups, mover: impl FnMut(&Path) -> Result<(), String>) ->
        Summary`: per group, the RAW first; if the RAW fails its sidecars are
        left alone (the judgement stays with the file) and the failure is
        recorded; a sidecar that fails after its RAW went is recorded too. It
        never stops at the first failure. `Summary { trashed: usize, failed:
        Vec<Failure { path, message }> }` (`serde::Serialize`), where `path`
        is the RAW's path for a RAW failure and the sidecar's path for a
        sidecar failure, and `trashed` counts RAWs.
    - `commands::trash_rejected(app, dir: String, paths: Vec<String>) ->
      Result<Option<Summary>, String>` (async): `Err(SCAN_RUNNING)` while
      `scanning()`; `Err` when `paths` is empty (the frontend handles the
      zero case before invoking); shows the confirmation `Move N rejected
      files to the Trash?` (singular for 1) with buttons `Move to Trash` /
      `Cancel`, `MessageDialogKind::Warning`, parented to the `main` window;
      `Ok(None)` on cancel; on confirm, in `spawn_blocking`: flush the writer,
      take the `Scans` lock, re-check `scanning()` (`Err(SCAN_RUNNING)`),
      `plan`, `run` with `trash::delete` as the mover, log the summary,
      `Ok(Some(summary))`. Registered in `main.rs`'s `generate_handler!`.
    - Unit tests in `trash.rs` (`cargo test -p riffle-app`, so they run in
      `mise run ci`) using a temp dir and a mover closure that renames into a
      temp "trash" folder (never the real Trash): a RAW with an `.xmp` and a
      `.dop` (one in different case) is grouped with both; a path outside
      `dir` or a non-RAW path is refused by `plan`; `run` continues past a
      failing RAW, leaves that RAW's sidecars in place, and reports it;
      `trashed` and `failed` add up.
    - `mise run ci` passes.
  - Implementation approach:
    - Follow `clear_index` line by line for the guard, dialog and lock order
      (`Scans` then writer; here the writer is flushed before the `Scans`
      lock is taken, which is what `clear_index` does too).
    - Verify which macOS delete method the `trash` version used defaults to
      and pin `NsFileManager` explicitly if the crate exposes
      `TrashContext::set_delete_method`; the Finder/AppleScript route needs
      Automation permission and must not be used. Record the finding in
      `learnings.md`.
    - Do not delete `ratings` rows: `entries()` hides them and a "Put Back"
      then restores the judgement with the file.
    - Non-UTF-8 paths: compare with the same lossy conversion the index uses.

- [x] Step 2: Menu item and frontend flow
  - Done when:
    - `app_menu::build` in `crates/app/src/main.rs` adds `Move Rejected to
      Trash…` (id `trash-rejected`) to `File` after `Reload Folder`, with the
      existing `cfg` split: `IconMenuItem::with_id_and_native_icon` with
      `NativeIcon::TrashFull` (or `TrashEmpty`, whichever renders as a
      template image) on macOS, plain `MenuItem` elsewhere, no accelerator.
      `on_event` emits `trash-rejected` to the frontend, like
      `open-in-photolab`.
    - `crates/app/ui/src/main.ts` listens for `trash-rejected` and:
      - no folder open -> `setStatus("No folder is open")`;
      - `scanRunning` -> `setStatus("a scan is running; wait for it to
        finish")` (the same text as `SCAN_RUNNING`), no invoke;
      - `rejected = allFiles.filter(p => ratings.get(p) === -1)`; empty ->
        `setStatus("No rejected files in this folder")`;
      - otherwise invokes `trash_rejected` with `{ dir: openDir, paths }`;
        `null` (cancel) does nothing; an `Err` goes to `#status`.
      - On a summary: drop the trashed RAWs from `ratings`, `picks`, `labels`,
        `sharpness`, `touched` and from the undo `history` (a later undo on a
        trashed path would call `set_rating` on a missing file and mint an
        orphan sidecar; check `crates/app/ui/src/undo.ts` for the removal
        API and add one if needed, with a test in `undo.test.ts`), add each
        failure to `errors` (`ErrorList`, keyed by the failed path, message
        `${baseName(path)}: could not move to the Trash: ${message}`), set
        `#status` to `Moved N files to the Trash` (plus `, M failed` when
        `failed` is non-empty), then call `resync()`.
      - The handler guards its result with the folder token / `openDir`
        check the other async paths use (see "Give the current folder one
        token" in `docs/agents/tauri-app.md`).
    - `pnpm exec vp test`, `vp check` and `mise run ci` pass.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - `resync()` keeps the current file and scroll position, and its
      `refilter(anchor, true)` moves off a deleted current file to a
      neighbour; `openDirectory` (the reset path) is not the right one here.
    - Any pure helper worth testing (e.g. selecting the rejected paths from
      the map, or composing the status text) lives in a small module with a
      vitest test rather than inline in `main.ts`, as `filter.ts` /
      `undo.ts` do.

- [ ] Step 3: Documentation and the manual verification item
  - Done when:
    - `README.md` describes `File > Move Rejected to Trash…` next to `Reload
      Folder` (user-facing only: what is moved, both sidecar formats, the
      confirmation, that the OS Trash's restore brings the judgement back).
    - `todo.md`: the "App: rejected files cannot be cleared out from the
      app" item is closed, and a new item lists the GUI checks that could
      not be run here (dialog with the right count and buttons; Cancel
      leaving the folder untouched; Confirm moving RAW + `.xmp` + `.dop` to
      the Trash and the strip updating; zero-reject / no-folder / mid-scan
      messages in `#status`; a "Put Back" from the Trash restoring the file
      with its judgement; on Windows and Linux too).
    - `docs/agents/tauri-app.md` gains an entry only if Step 1 or 2 hit a
      pitfall (e.g. the `trash` macOS delete method), sourced to this plan's
      `learnings.md`.
    - `mise run ci` (lychee link check included) passes.

## Trade-offs and risks

- **New dependency.** `trash` 5.2.x is added; its macOS/Windows bindings
  (`objc2`, `windows`) largely overlap what Tauri already pulls in, and it
  uses a mutex around non-thread-safe libc calls on Linux. The dependency-free
  alternative is `fs::rename` into a chosen folder (the todo allows it), but
  that is not the OS Trash (no "Put Back"), so the requirement decides for
  `trash`. If the caller prefers zero dependencies, Step 1's mover becomes a
  rename into `<dir>/Rejected/` and the dialog text changes; the rest of the
  plan holds.
- **Source of the rejected list.** The frontend `ratings` map (chosen) is what
  the strip shows and already owns judgements during a session; the backend
  validates every path. An index query (`WHERE dir = ? AND rating = -1`) is
  the alternative; it can lag an un-awaited `set_rating`. Either way a reject
  pressed a few milliseconds before the menu click may not be in the writer
  when `flush` runs, so its sidecar could be written after the RAW is gone;
  the window is tiny and is accepted, not closed.
- **Both sidecar formats are trashed** whatever the current setting. A
  format switch can leave both on disk, and a sidecar for a gone RAW is
  garbage. The alternative (only the active format) leaves orphans.
- **Partial failure is per RAW and reported, never aborted.** The RAW goes
  first; a failed RAW keeps its sidecars; a sidecar that fails after its RAW
  went is reported as its own failure. `delete_all` would not make this
  atomic either.
- **`ratings` rows are kept** for trashed paths (a restore brings the
  judgement back; the rows are invisible without a `files` row). Deleting
  them is the alternative; it costs a restored file its judgement until the
  sidecar is re-read, which `reconcile_sidecars` does anyway.
- **No accelerator** (chosen): a destructive, confirmed action; Finder's
  `Cmd+Delete` is the obvious candidate if the caller wants one, as a fixed
  accelerator like `Reload Folder`'s, and only checkable by hand.
- **The `Scans` lock is held across the trash loop**, as `clear_index` holds
  it across the clear, so a concurrent `scan_folder` waits; trashing a few
  hundred files is sub-second on every platform.
- **GUI automation is unavailable on this Mac**, so the dialog, the status
  texts and the Trash restore are a manual todo item (Step 3), not a
  verified pass.

## Progress

- (2026-09-20) Step 1 complete
