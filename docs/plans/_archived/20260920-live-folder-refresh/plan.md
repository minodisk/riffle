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

# Live folder refresh: pick up files added to or removed from the open folder

## Purpose

A folder stays open in Riffle while a card is still copying, or while the
user deletes rejects in Finder / Explorer or PhotoLab. Today the strip and
the SQLite folder index only reflect what was on disk at the moment the
folder was opened; the only way to see a new file is to reopen the folder,
which also throws away the current selection and scroll position.

After this work, (1) refocusing the window or choosing `File > Reload
Folder` brings the index and the strip in line with the disk by scanning only
what changed, keeping the current file and, as far as possible, the scroll
position; and (2) a file-system watcher does the same on its own within
about a second of a change, without ever looping on the app's own sidecar
writes, with (1) as the fallback where no file-system events arrive (network
volumes).

## Decisions (made by the user; not open)

- Two steps, in this order: focus / manual rescan first, watcher second.
- The rescan is a diff: readdir + size/mtime against the index, never a
  full re-extraction.
- The watcher's events are only a trigger. What changed is always
  re-derived from a fresh listing; the paths and kinds in the events are
  never applied to the index or the UI.
- The watcher debounces (~500 ms) and then calls the Step 1 rescan.
- The five design questions the planner raised are settled on its
  recommended options, and are no longer open:
  1. `Reload Folder` carries a **fixed `CmdOrCtrl+R` accelerator** on the
     menu item; it is not a rebindable keymap action.
  2. The focus trigger lives in the **webview** (`tauri://focus`), not in
     Rust. If it turns out not to reach a global `event.listen`, fall back
     to `WindowEvent::Focused(true)` emitting `folder-changed` and note it
     in `learnings.md`.
  3. The watch is set **inside `scan_folder`**, right after `canonicalize`;
     no separate command.
  4. The 500 ms debounce is **hand-rolled**, following `sidecar::run`'s
     `recv_timeout` loop; no `notify-debouncer-*` dependency.
  5. Events whose paths are **all sidecars or `.riffle-tmp` are ignored**
     as triggers. External sidecar edits are picked up on the next focus /
     manual rescan, as they are today on reopen.

## Background (from investigation)

- **The backend already does the diff.** `scan_folder`
  (`crates/app/src/commands.rs:757`) cancels and joins any running scan,
  lists the folder once (`list_folder_in`: RAWs plus sidecars of the
  selected format), stats every RAW, calls `Index::reconcile`
  (`crates/app/src/index.rs:315`), which deletes the `files` rows whose
  file is gone or whose `size`/`mtime_ns` changed and returns only the
  files without a valid row, then `reconcile_sidecars_of`, and hands the
  todo list to `start_scan`, which runs `riffle_core::scan::extract_all`
  over just that list and emits `scan-progress` / `scan-done` (a
  `scan-done` is emitted even when the todo list is empty). `ratings`
  rows are independent of `files`, so a rescan never drops a judgment.
  The README's "second open" numbers are the cost of one such rescan on an
  unchanged folder (a stat-and-query pass, no extraction).
- **The frontend's only open path resets everything.** `openDirectory`
  (`crates/app/ui/src/main.ts:1009`) replaces `allFiles`, clears
  `entries`, `ratings`, `picks`, `labels`, `sharpness`, `touched`,
  `history`, `errors`, sets `index = 0`, and calls `strip.setFiles`, which
  clears the strip's cells and sets `strip.scrollTop = 0`
  (`crates/app/ui/src/strip.ts:351`). This is why a reopen loses the
  selection. The `sidecar-format` and `index-cleared` listeners reopen the
  folder this way on purpose (the index was reset); a rescan must not.
- **`refilter(anchor)`** (`main.ts:443`) rebuilds `files` from `allFiles`
  through the sort and filter, short-circuits when nothing changed, and
  otherwise re-anchors on `anchor` via `anchorAfterFilter`
  (`filter.ts:48`), which falls back to the next passing file after the
  anchor's old position, then the previous one. It calls `strip.setFiles`
  (scroll reset) and then `strip.setCurrent(index)` (scroll the current
  cell into view, "nearest"). `refreshEntries` (`main.ts:596`) re-reads
  `folder_entries`, keeps one request in flight with a pending flag, and
  applies ratings from the index only to paths not in `touched`.
- **Folder token rule** (`docs/agents/tauri-app.md`, "Give the current
  folder one token"): every async result must check `dir !== openDir ||
  token !== folderToken` before touching the UI; a backend event listener
  must guard on its payload the same way. `scanId` is the per-scan guard
  for `scan-progress` / `scan-done`.
- **Menu:** `app_menu::build` in `crates/app/src/main.rs` inserts
  `Open Folder…` (plain `MenuItem`, accelerator from the keymap) and
  `Open in DxO PhotoLab` at the top of `File`; `Settings...` and `Undo`
  carry fixed accelerators (`CmdOrCtrl+,`, `CmdOrCtrl+Z`). Menu events are
  turned into frontend events in `app_menu::on_event` (`open-folder`,
  `undo`, ...) because the frontend owns which folder is open. Only items
  with an icon need the macOS `cfg` split; `MenuItem` is imported
  unconditionally. Auto memory policy: non-culling actions go in the menu,
  not on culling keys.
- **Focus event:** Tauri 2 emits `tauri://focus` / `tauri://blur` to the
  window; the webview already listens to `tauri://drag-drop` through
  `window.__TAURI__.event.listen` under `core:default`, so no capability
  change is expected. The Rust-side equivalent is `WindowEvent::Focused(bool)`
  in the `App::run` closure in `main.rs`, which already matches
  `RunEvent::WindowEvent` for `Destroyed`.
- **Sidecar writer** (`crates/app/src/sidecar.rs`): `Writer::spawn` runs a
  thread with an `mpsc` receiver and a deadline map (`run`, line ~294): it
  is the repository's existing debounce pattern. `write` (line ~434) reads
  the existing sidecar, writes the new bytes to `<sidecar>.riffle-tmp`
  (`TEMP_SUFFIX`, private), `sync_all`s, renames it over the sidecar, stats
  the result and returns the stat; `flush` then calls
  `Index::mark_written`, which clears `dirty` and stores the stat in
  `ratings.xmp_size` / `xmp_mtime_ns`. `existing_sidecar` may also
  `read_dir` the folder (a read, no event). `reconcile_sidecars`
  (`index.rs:779`) parses a sidecar only when its on-disk stat differs from
  the stored one, so a rescan that follows the app's own write parses
  nothing and queues nothing: **one rescan per own write is a no-op, not a
  loop.** The remaining hazards are (a) the cost of that no-op rescan on
  every judgment while culling (list + 5000 stats + reconcile +
  `folder_entries` refresh), and (b) the window between the rename and
  `mark_written` where `reconcile_sidecars` would read the app's own
  sidecar as an external edit (the "snapshot the state you decided on"
  trap): bounded to one extra write, since the second write's stat then
  matches, but it is churn that Step 2 avoids by not triggering on sidecar
  paths at all.
- Every file-system event kind the app cares about (create, remove, rename,
  modify/close-write of a RAW) is subsumed by "list again and compare
  size/mtime", which is why event contents are not needed.
- `notify` is not in `Cargo.lock` yet; the current major is 8 (confirm with
  `cargo add notify`). Its `RecommendedWatcher` is FSEvents on macOS,
  `ReadDirectoryChangesW` on Windows and inotify on Linux; it has no
  built-in debounce (that is the separate `notify-debouncer-*` crates).

## Steps

- [x] Step 1: Rescan the open folder on window focus and on `File > Reload Folder`, keeping the selection
  - Done when:
    - `crates/app/ui/src/main.ts` has a `resync()` (name free) that, when a
      folder is open, invokes `list_arw` for `openDir`, replaces `allFiles`
      with the result, then invokes `scan_folder` + `start_scan` exactly as
      `openDirectory` does (storing `scan_id` into `scanId`, adding
      `sidecar_errors` to `errors`), and re-anchors the view with
      `refilter(currentPath)` where `currentPath` is `files[index]` captured
      before the listing — **without** clearing `ratings`, `picks`,
      `labels`, `sharpness`, `touched`, `history`, `errors` or `entries`,
      and without resetting `index` to 0 or the preview. `entries` for paths
      no longer listed are pruned (or simply left to the next
      `refreshEntries`, which replaces the map wholesale; state which).
    - The current file stays current when it still exists and passes the
      filter; when it was deleted, the view moves to the neighbor
      `anchorAfterFilter` chooses (already the behavior for a filtered-out
      file). The strip's scroll offset is preserved across the rescan
      (clamped to the new list height) rather than reset to 0; if the list
      did not change, nothing visible happens (`refilter`'s short-circuit).
    - Every async result in the path checks the folder token / `openDir`
      guard as `refreshEntries` does; a rescan whose listing lands after the
      user opened another folder is dropped. `resync()` mints no new folder
      token (it is the *same* open); it uses the current one.
    - A rescan requested while a scan is in flight (`scanning !== null` or
      `scanId` set and no `scan-done` yet) is deferred: a pending flag is
      set and the rescan runs from the `scan-done` handler, so a focus
      change during a 5000-file first scan does not cancel and restart it.
      Two triggers within a short window (focus fires more than once when
      switching between the main and settings windows) coalesce into one
      rescan (either the in-flight/pending pair as `refreshEntries` does, or
      a short timer).
    - Trigger 1: the main window's focus, via
      `event.listen("tauri://focus")` in `main.ts` (decision 2). A rescan on
      *launch* focus is harmless but pointless: the first focus after
      `reopenLastFolder` must not double the open (guard on `openDir` being
      set and no scan running is enough).
    - Trigger 2: a `Reload Folder` item in the `File` submenu, right after
      `Open Folder…`, with id `reload-folder`, emitting a `reload-folder`
      event that `main.ts` listens to and routes to `resync()`. Plain
      `MenuItem` on every platform (no icon, so no `cfg` split), with a
      fixed `CmdOrCtrl+R` accelerator (decision 1).
    - `README.md`'s feature list mentions that the open folder is rescanned
      when the window regains focus and on `File > Reload Folder`, and
      that judgments and the current file are kept (user-facing wording
      only, per the README content policy).
    - `docs/agents/tauri-app.md` (frontend section) gains a short note that
      `openDirectory` is the *reset* path and `resync()` the *keep-state*
      path, so the next "the backend changed the folder" event picks the
      right one.
    - `mise run ci` passes. Manual verification (`mise run
      tauri:release:devtools`, GUI automation is unavailable on this Mac):
      with a folder open and a file selected mid-list, copy an ARW/DNG in,
      switch to Finder and back: the file appears in the strip (sorted into
      place), the selection and scroll position are unchanged; delete a file
      the same way: it disappears; delete the *current* file: the view moves
      to a neighbor; `File > Reload Folder` does the same without a focus
      change; press it during a first scan: the scan continues and the
      rescan runs after `scan-done`. Report what was not checked.
  - Implementation approach:
    - No backend index or scan change is expected: `scan_folder` +
      `start_scan` are the diff. If the implementation finds it needs one
      (e.g. `scan_folder` returning the listing so `list_arw` is not a
      second `read_dir`), keep it minimal and say why in `learnings.md`;
      do not restructure `Scans`.
    - Factor the `scan_folder` -> `start_scan` invoke chain out of
      `openDirectory` into a helper both call, rather than duplicating the
      `.then` chain; keep `openDirectory`'s reset semantics unchanged.
    - Scroll preservation lives in `strip.ts`: `setFiles` is the only
      place that touches `scrollTop`, so give it a way to keep the offset
      (an optional argument, or a save/restore pair exported next to it),
      clamped to `files.length * CELL_HEIGHT - clientHeight`. Do not add a
      second list-replacement function that skips the cell reset: indices
      shift when a file is inserted, so the per-index cell/thumbnail store
      must be rebuilt as `setFiles` does.
    - The `sidecar-error` listener guards on `allFiles.includes(path)`;
      after a resync `allFiles` is current, nothing to change there.
    - Frontend tests (`vitest`, `src/**/*.test.ts`): the modules under test
      are the pure ones; `main.ts` is not unit-tested. If a pure helper
      falls out (e.g. "next anchor after a deletion"), `anchorAfterFilter`
      already covers it; add a test only for genuinely new logic.
    - The menu item follows `Open Folder…` (`MenuItem::with_id`, inserted
      via `file.prepend_items`) and `app_menu::on_event`'s emit pattern.

- [x] Step 2: Watch the open folder with `notify` and trigger the Step 1 rescan, debounced, without looping on sidecar writes
  - Done when:
    - `crates/app/Cargo.toml` adds `notify` (current major). A new
      `crates/app/src/watch.rs` owns: a managed state holding the watched
      directory and its `RecommendedWatcher`; a debounce thread (an `mpsc`
      receiver with `recv_timeout`, modeled on `sidecar::run`) that
      collapses a burst of events into one `folder-changed` event carrying
      `{ dir }`, emitted ~500 ms after the *last* event (trailing edge; a
      constant, documented); and a pure `fn triggers(paths: &[PathBuf]) ->
      bool` that returns `false` when every path in the event is a sidecar
      of either format (`SidecarFormat::matches` for both `Xmp` and `Dop`,
      case-insensitive as it already is) or ends with `sidecar::TEMP_SUFFIX`
      (made `pub(crate)`), and `true` otherwise, including for an event
      with no paths. Nothing else about the event (kind, which path) is
      used.
    - The watch is set to the folder `scan_folder` canonicalizes, non-
      recursively, replacing the previous watch when the folder differs
      and leaving it alone when it is the same (a rescan of the same
      folder must not unwatch/rewatch). A failure to watch (e.g. an SMB
      volume) is logged with `log::warn!` and ignored: the focus/manual
      rescan of Step 1 is the fallback and the open must not fail.
    - `main.ts` listens to `folder-changed` and calls Step 1's `resync()`
      only when `payload.dir === openDir` (the event listener outlives every
      folder); the deferral-while-scanning and coalescing rules of Step 1
      apply unchanged, so a burst of copied files that keeps a scan running
      resolves into one trailing rescan after `scan-done`.
    - Loop safety: a Rust unit test in `watch.rs` pins `triggers` for a
      `.xmp`, `.XMP`, `.dop`, `.ARW.dop`, a `.riffle-tmp` path, a mixed
      event (sidecar + RAW -> `true`), an empty path list (`true`) and a
      RAW path (`true`). A test drives the debounce thread through its
      input channel with a fake clock-free burst (send N events, assert one
      output, then one more after a pause) without touching the real file
      system, so CI on all three OSes is not exposed to FSEvents latency.
      The real end-to-end check is manual (below).
    - The watcher is dropped (or the thread ends) with the app; no
      `ExitRequested` work is needed since nothing is pending on disk
      (state this in a comment next to the existing drain).
    - `README.md` says the open folder now updates on its own when files are
      added or removed, and that on volumes without change notifications
      (network shares) refocusing the window or `Reload Folder` does it.
      `docs/agents/tauri-app.md` gains a note recording the no-loop
      reasoning (own writes are filtered by path; even unfiltered they are a
      no-op rescan because the stored sidecar stat matches) and the
      Windows directory-handle caveat below.
    - `mise run ci` passes on all three OSes. Manual verification
      (`mise run tauri:release:devtools`): copy an ARW into the open folder
      without switching windows: it appears within ~1 s; delete one: it
      disappears; rate a series of files quickly while watching
      `Riffle.log` (`Help > Open Log Folder`): no `scan list` / `scan
      reconcile` lines follow the sidecar writes; copy ten files at once:
      one rescan, not ten. Report what was not checked.
  - Implementation approach:
    - Requires Step 1 merged (`resync()` and its deferral are what the
      event drives).
    - Hook the watch update into `scan_folder` right after `canonicalize`,
      through one function in `watch.rs` (e.g. `watch::set(&app, &dir)`),
      so the watched folder can never diverge from the folder being
      scanned and no new command or frontend invoke is needed (decision 3).
    - The notify callback runs on notify's own thread: do nothing there but
      `triggers()` and a `send` to the debounce thread. Emit from the
      debounce thread via `AppHandle` (`Emitter` is safe from any thread,
      as `run_scan` and the writer already rely on).
    - Watch non-recursively (`RecursiveMode::NonRecursive`); the app lists
      only the folder's direct children.
    - A file mid-copy may be picked up by a rescan before the copy
      finishes: the extraction then fails or reads a partial preview and
      the row is written with that size/mtime; the copy's completion fires
      another event, and `reconcile` sees the changed stat and re-extracts.
      No special handling; note it in `learnings.md` if observed.
    - Do not add `PollWatcher` for network volumes; the user chose the
      focus rescan as the fallback.

## Trade-offs and risks

The five design questions below were settled at approval time (see
Decisions); the alternatives are kept because they are the fallbacks if the
chosen option turns out not to work.

### Reload accelerator: fixed `CmdOrCtrl+R` (chosen) vs. a rebindable keymap action

- **Chosen: fixed accelerator on the menu item**, like `Settings...`
  (`CmdOrCtrl+,`) and `Undo` (`CmdOrCtrl+Z`). Zero changes to
  `shortcuts.rs`, the settings shortcut panel or `accelerator_for`, and
  the memory policy says non-culling actions belong in the menu.
- Alternative: a `reload` keymap action with a default of `meta+r` /
  `ctrl+r`, rebindable and shown in the shortcuts tab like `open` and
  `photolab`. More consistent with those two, but it touches `shortcuts.rs`
  defaults and tests, `app_menu::refresh`, and the settings UI.
- In a debug webview `Cmd+R` may also be bound by devtools; the menu
  accelerator takes precedence when the menu is set, but check by hand.

### Where the focus trigger lives: webview `tauri://focus` (chosen) vs. Rust `WindowEvent::Focused`

- **Chosen: webview `event.listen("tauri://focus")`.** No Rust change,
  same mechanism as the drag-drop events, and the handler is already next
  to the state it needs (`openDir`, `scanning`).
- Fallback: match `WindowEvent::Focused(true)` for label `main` in the
  `App::run` closure and `emit("folder-changed")` (the same event Step 2
  emits), so both triggers arrive through one event. Use it if
  `tauri://focus` turns out not to reach a global `event.listen` (Tauri
  targets window events at the window; the main webview should receive
  them, but this is inferred, not measured), and record the switch in
  `learnings.md`.

### Where the watch is set: inside `scan_folder` (chosen) vs. a separate command

- **Chosen: inside `scan_folder`.** It already canonicalizes the
  folder and is the single point every open and rescan passes through, so
  the watch cannot point at a folder other than the one indexed, and there
  is no frontend token race for a "set watch" invoke.
- Alternative: a `watch_folder(dir)` command called from `openDirectory`.
  Keeps `scan_folder` untouched but adds an invoke, a capability-free
  command registration, and another async path needing the token guard.

### Debounce: hand-rolled thread (chosen) vs. `notify-debouncer-mini`

- **Chosen: hand-rolled**, ~30 lines following `sidecar::run`'s
  `recv_timeout` loop, already the repository's pattern, and it lets the
  test drive the channel without the file system.
- Alternative: `notify-debouncer-mini` (or `-full`). One more dependency,
  and its output still needs the path filter and the emit; it saves little.

### Trigger on sidecar changes too?

- **Chosen: no.** Filtering sidecar and temp paths out of the trigger
  removes both the per-judgment no-op rescan while culling and the
  rename-to-`mark_written` window entirely. External sidecar edits
  (PhotoLab writing a `.dop` while Riffle is open) are then picked up on
  the next focus / manual rescan, exactly as they are today on reopen, so
  there is no regression.
- Alternative: trigger on everything and rely on the stat match to make
  own writes a no-op. Live pickup of external sidecar edits, at the price
  of a list + stat + reconcile + `folder_entries` refresh 500 ms after
  every rating during culling, and of the (bounded) double-write window.
  If wanted later, the change is deleting the filter, not redesigning.

### Scroll preservation

- Preserving `scrollTop` across `setFiles` is a small `strip.ts` change; the
  fallback, if it proves awkward, is accepting `refilter`'s existing
  behavior (reset, then `setCurrent` scrolls the current cell to the
  nearest edge), which keeps the *selection* but moves the strip. The
  acceptance criterion is "selection not lost"; the scroll is "as far as
  possible".

### Other risks

- A focus rescan restarts nothing while a scan runs (deferred to
  `scan-done`), but a rescan on an unchanged 5000-file folder still costs
  the README's "second open" pass (stat of every file plus the sidecar
  reconcile) plus a `folder_entries` read of every row on `scan-done`.
  Measure one on a real folder and note it in `learnings.md`; if it is
  visible, the coalescing window can grow.
- `scan_folder` cancels and joins the previous scan before listing; that
  remains true for the deferred rescan too and is fine, since it runs
  after `scan-done`.
- On Windows, `ReadDirectoryChangesW` keeps a handle on the watched
  directory, so the open folder cannot be deleted or renamed while Riffle
  has it open; document it. Unwatch when the folder changes so a
  previously open folder is released.
- FSEvents (macOS) can deliver events late or coalesced, and inotify does
  not follow renames of the directory itself. All of these degrade to
  "the next focus rescan catches it", which is the stated fallback.
- The watcher tests must not depend on real file-system event timing, or
  they will flake on CI; the end-to-end behavior is a manual check (GUI
  automation does not work on this Mac).

## Progress

- (2026-09-20) Step 1 complete
- (2026-09-20) Step 2 complete
