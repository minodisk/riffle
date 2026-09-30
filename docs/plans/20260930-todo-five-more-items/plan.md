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

# Close five more small todo.md items

## Purpose

`todo.md` carries five more fully specified items: a Windows-flaky sidecar
switch test, an unthrottled focus rescan, `Expand All` / `Collapse All` in
the folder tree's context menu, a first-launch dialog that never asks for
Lightroom's UI language, and trash undo / redo runs left pointing at a
folder's old path after a rename. Each step closes one item and removes its
`###` section from `todo.md` in the same PR.

Every step touches `todo.md`. The sections of Steps 4 and 5 are adjacent
(`### App: the first-launch dialog does not ask for Lightroom's UI
language` at ~line 882 sits right above `### App:
\`a_dirty_row_is_written_to_both_sidecars_after_a_switch_to_both\` is flaky on
Windows` at ~901), and Step 5's section (`### App: renaming a folder after a
Move Rejected to Trash run…`, ~line 1190) sits right below the `### Agents:
Bash-tool heredocs on Windows mangle doubled backslashes` section that
`docs/plans/20260929-todo-five-small-items/plan.md` Step 5 removes. So run
the steps one after another, each branched from `main` after the previous
PR merged, never in parallel, and rebase onto `main` before opening each PR.
The order below puts the smallest, CI-affecting fix first and the
trash-rebase step (the one adjacent to the other plan's section) last, so
it is most likely to land after that plan's Step 5.

Steps 3 and 4 of this plan touch `crates/app/ui/src/tree.ts` /
`folders.ts` and `main.ts`, which the other plan's Step 4 also edits
(`relation` / `rebase` / `renameFolder` case-insensitivity in `tree.ts`,
`renamed` in `folders.ts`). This plan adds new functions and menu items and
does not change those three functions, so a rebase conflict is at most
adjacent-line noise; resolve it by keeping both sides.

## Steps

- [x] Step 1: Make `a_dirty_row_is_written_to_both_sidecars_after_a_switch_to_both` wait for the sidecar writer deterministically
  - Done when: `crates/app/src/sidecar.rs`'s `Writer::flush` returns
    `bool` (`true` when the writer's drain reply arrived within `timeout`,
    `false` on the timeout or a gone thread); the test in
    `crates/app/src/commands.rs` builds its `Writer` with an `on_error` that
    collects every message into a shared `Vec<String>`, flushes with a
    generous timeout (30 s, not `DRAIN_TIMEOUT`'s 2 s), asserts the flush
    returned `true` and the collected errors are empty *before* reading
    the two sidecars, so a slow or failed write fails with the writer's own
    message instead of `NotFound`; the root-cause analysis is recorded in
    `learnings.md`; the `### App:
    \`a_dirty_row_is_written_to_both_sidecars_after_a_switch_to_both\` is
    flaky on Windows` section is removed from `todo.md`; `mise run ci`
    passes (`cargo test -p riffle-app` covers the change).
  - Implementation approach:
    - Planning-time analysis: the test queues with `set_now` (deadline
      `Instant::now()`) then calls `writer.flush(DRAIN_TIMEOUT)`. The
      writer thread handles the `Set`, then either times out at once and
      writes (`flush(Some(now))`) or receives the `Flush` and writes
      (`flush(None)`), and only then replies, so message order is not the
      race. The two ways the read at the old `commands.rs:3869` can see no
      file are: (a) `Writer::flush`'s `rx.recv_timeout(DRAIN_TIMEOUT)`
      giving up after 2 s while the writer is still in `write_kind`'s
      `File::create` + `sync_all` + `rename` for two files on a loaded
      Windows runner (the `rename` alone retries up to 5 times with
      20-100 ms sleeps on a sharing violation), or (b) the write failing
      (its error only goes to `on_error`, which the test's
      `switch_writer` prints to stderr and forgets). Both are silent
      today; the fix makes the wait long and both outcomes assertable.
    - `Writer::flush`: change `let _ = rx.recv_timeout(timeout);` to
      return `rx.recv_timeout(timeout).is_ok()`; every production caller
      (`switch_format`, `rename_folder`, `trash_rejected_*`,
      `set_label_names`, the quit path) keeps ignoring the result
      (`let _ =` or a bare statement; check clippy's `unused_must_use`
      does not fire, `bool` is not `#[must_use]`).
    - In the test: replace `switch_writer(index.clone())` with a writer
      whose `on_error` pushes `format!("{}: {message}", path.display())`
      into an `Arc<Mutex<Vec<String>>>`; keep `switch_writer` for the
      other tests. The `sidecar.rs` tests' `eventually` helper is the
      existing precedent for a long budget ("it only costs time when the
      assertion is failing anyway").
    - The two sibling tests in the same shape
      (`a_judgment_made_during_a_switch…` and
      `a_pick_set_during_a_switch…`, right above it) may get the same
      treatment in the same PR since it is the same fix; note in
      `learnings.md` whether they were changed.
    - Files: `crates/app/src/sidecar.rs`, `crates/app/src/commands.rs`,
      `todo.md`.

- [x] Step 2: Throttle the focus rescan and let a scan with nothing to do end at once
  - Done when: the `tauri://focus` listener in `crates/app/ui/src/main.ts`
    calls `resync()` only when no scan was started in the last
    `FOCUS_RESCAN_INTERVAL` (5 000 ms) — `File > Reload Folder`,
    `folder-changed` and the trash / rename paths keep rescanning
    unconditionally; `crates/app/ui/src/refresh.ts` exports the pure
    `focusRescanDue(lastScanAt: number | null, now: number): boolean` and
    the constant, with Vitest cases in `refresh.test.ts` (never scanned,
    inside the interval, exactly at it, past it); in
    `crates/app/src/commands.rs` `scan_folder` inserts no `pending` entry
    when the first pass has nothing to do *and* `Index::faces_todo(&dir)`
    is empty, so `start_scan` takes its existing no-pending path and emits
    the empty `scan-done` / `faces-done` at once instead of spawning the
    two-pass task; the `### App: every main-window focus runs resync() and
    holds scanRunning until faces-done` section is removed from `todo.md`;
    `mise run ci` passes.
  - Implementation approach:
    - Frontend: record `lastScanAt = Date.now()` in `startScan` (it is
      shared by `openDirectory` and `resync`, so the focus that follows a
      picker dialog closing right after an open is skipped rather than
      deferred into a second full scan). The listener becomes
      `if (!settings.isOpen && focusRescanDue(lastScanAt, Date.now())) resync();`.
      Keep the listener's comment about the settings modal and add why
      the throttle exists (the on-disk changes the focus rescan would
      catch are also caught by the folder watcher; the rescan is a
      belt-and-braces re-listing, so skipping it for a few seconds loses
      nothing). 5 s is chosen as the smallest interval that collapses an
      alt-tab back and forth into one scan; do not make it configurable.
    - `folders.ts` has its own `tauri://focus` listener (`fetchRoots`);
      it is cheap and out of scope, leave it.
    - Backend: in `scan_folder`, after `todo` is computed and while the
      index is still available, `if todo.is_empty()` call
      `index::lock(&index).faces_todo(&dir)` (it is what `run_faces_pass`
      would call first; its side effect of marking untestable rows done is
      the same either way) and, when that is empty too, return the
      `ScanStarted` without the `state.pending.insert(...)`. Keep the
      `state.latest_id != scan_id` check and `state.pending.clear()` as
      they are so a superseded id is still refused. `start_scan`'s `let
      Some(pending) = ... else { emit_empty_scan_events(&app, "", scan_id) }`
      already covers the rest; `ScansState::scanning()` is then false as
      soon as `scan_folder` returns, which is right because nothing runs.
      Update the doc comments of `scan_folder` and `start_scan` that say
      the events are emitted "with no work done" only for an unavailable
      cache or a superseded scan.
    - Tests: the scan-state tests in `commands.rs`'s `tests` (from
      `a_finished_scan_leaves_no_scan_in_progress` on) drive `ScansState`
      directly, not `scan_folder`, so cover the backend change with an
      `index.rs` test only if a cheap one exists for "a fully scanned
      folder has an empty `faces_todo`"; otherwise verify by hand with
      `Timing logs` on: after a focus on an up-to-date folder, the log
      shows `scan prepare … todo=0` and `faces-done` with no `scan
      faces` pass line in between, and the status line's `scanning` never
      appears. Record the result in `learnings.md`.
    - Files: `crates/app/ui/src/main.ts`, `crates/app/ui/src/refresh.ts`,
      `crates/app/ui/src/refresh.test.ts`, `crates/app/src/commands.rs`,
      `todo.md`.

- [x] Step 3: Add `Expand All` / `Collapse All` to the folder tree's context menu
  - Done when: right-clicking a single non-root folder offers `Expand All`
    and `Collapse All` in their own group right after `Rename…`
    (`crates/app/ui/src/context.ts` `folderMenuGroups`, actions
    `expandAll` / `collapseAll`; not offered on a root, nor on a multi
    selection, like `Rename…`), with `context.test.ts` cases for the
    single-folder, root and multi-selection shapes; `Expand All` expands
    the folder and every subfolder under it, listing each as it goes, and
    `Collapse All` collapses every subfolder under it and leaves the folder
    itself as it is; `crates/app/ui/src/tree.ts` exports the pure
    `collapseAll(tree, path)` (and the descendant walk it uses) with
    `tree.test.ts` cases (nested expanded folders all collapse, the clicked
    folder stays expanded, an unknown path leaves the tree as is, a
    folder not yet listed is skipped); `docs/usage.md`'s folder-menu
    paragraph names the two items; the `### App: expand / collapse all
    subfolders from the folder tree's context menu` section is removed
    from `todo.md`; `mise run ci` passes.
  - Implementation approach:
    - `tree.ts` is pure state; add `collapseAll` next to `expand` /
      `collapse` using `update` over a recursive walk of `node.children`
      through `tree.nodes` (a child not in `nodes` or with `children ===
      undefined` ends that branch). Place the new code near
      `watchedFolders`, not near the bottom helpers (`relation` / `rebase`
      / `renameFolder`), which the other plan's Step 4 edits.
    - `folders.ts`: export `collapseAll(path)` (`tree = collapseAll(tree,
      path); render();`) and `async expandAll(path)`: expand and `list`
      the folder as `toggle` does (`setChildren`, `render`), then walk the
      listed children depth-first, sequentially awaiting each `list` (no
      parallel fan-out, so a deep tree does not stampede
      `list_subfolders`), stopping a branch when its node was collapsed or
      dropped meanwhile, and on a failed listing calling
      `markFailed(collapse(...))` + `reportError` for that folder and
      going on with the rest. Reuse `drawnChildren` so a root drawn
      elsewhere (home under its volume) is not expanded twice. Each
      render's `syncWatches` sets a `tree-changed` watch per expanded
      drawn folder; that is the existing behaviour of expanding by hand,
      so no watcher change, but note in `learnings.md` how many watches a
      real `Expand All` on a photo archive made and whether Windows
      renames of its ancestors still work (see the `notify` ancestor
      pinning item in `docs/agents/tauri-app.md`).
    - `main.ts`: two new `case`s in the `folders.init` context-menu
      switch calling `folders.expandAll(path)` / `folders.collapseAll(path)`;
      update the comment above `folders.init` that lists the menu's
      items.
    - Menu items are `MenuItem`s with `shortcut: ""` and `checked:
      undefined` like the other folder items; no CSS change
      (`docs/agents/ui-styling.md`).
    - Files: `crates/app/ui/src/context.ts`, `crates/app/ui/src/context.test.ts`,
      `crates/app/ui/src/tree.ts`, `crates/app/ui/src/tree.test.ts`,
      `crates/app/ui/src/folders.ts`, `crates/app/ui/src/main.ts`,
      `docs/usage.md`, `todo.md`.
    - Manual check (Windows GUI available): `Expand All` on a folder with
      two or more levels shows every level; `Collapse All` folds them
      back with the clicked folder still open; a folder that fails to
      list shows its error and the rest still expand.

- [x] Step 4: Ask for Lightroom's UI language in the first-launch dialog when XMP or Both is chosen
  - Done when: choosing `Lightroom (XMP)` or `Both` in `#format-dialog`
    reveals, inside the same dialog box, a second block (a one-line
    question, a `select.select` listing the `crates/core/i18n/` presets by
    `meta.name`, and a `Continue` button) instead of closing; `DxO
    PhotoLab (.dop)` closes at once as today; `Continue` invokes
    `choose_sidecar_format` with the chosen format and then
    `set_label_names` with the selected preset's `names`, closes the
    dialog and opens the `FormatGate` only when both succeeded (an error
    shows in `#format-error` with the controls re-enabled, as the format
    buttons do today); the select is preselected to the preset whose
    `code` matches `navigator.language`'s primary subtag, else the first
    (English); the Tab trap in `main.ts`'s keydown handler cycles over the
    dialog's *visible* controls in both phases; the settings modal's
    `open()` re-reads `label_names` so the Lightroom-language fields show
    what the dialog saved; `crates/app/ui/src/firstrun.ts` holds the pure
    parts (`asksLanguage(format)`, `defaultPreset(presets,
    navigatorLanguage)`) with `firstrun.test.ts` cases; `README.md` "First
    steps" 1 and `README.ja.md`'s matching line, and `docs/usage.md`'s
    "Ratings and sidecars" / `Tab` row, mention the language question; the
    `### App: the first-launch dialog does not ask for Lightroom's UI
    language` section is removed from `todo.md`; `mise run ci` passes.
  - Implementation approach:
    - Presets come from the existing `label_names` command
      (`{ names, presets: [{ code, name, names }] }`, English first);
      fetch it in `showFormatDialog` and fill the select with
      `new Option(preset.name, preset.code)` exactly as `settings.ts`
      does. `LabelPreset` / `LabelNames` types are in
      `crates/app/ui/src/labels.ts`.
    - Keep the phase state in `main.ts` (a `chosenFormat: string | null`
      beside `formatButtons`): a format click on XMP / Both stores it,
      un-hides the language block and focuses the select; a click on DxO
      keeps today's path. Order of the two invokes on `Continue`:
      `set_label_names` first, then `choose_sidecar_format` (reversed in
      review round 1: the format is the one key that keeps the dialog from
      coming back, so it goes last and a failed names save leaves the dialog
      to reappear); `set_label_names` emits `sidecar-format`, which is
      harmless with no folder open. `Continue` does nothing while no preset
      matches the select. Consider
      letting a second click on a format button while in phase two just
      change `chosenFormat` (no back button needed).
    - Tab trap: replace the fixed `formatButtons` cycling with a list
      built at keydown time from the dialog's `button, select` elements
      that are not inside a `[hidden]` ancestor (the settings modal has a
      similar visibility filter around `settings.ts:440`; `cycleFocus` in
      `modal.ts` gives the index arithmetic).
    - Styling: the select is `class="select"`, `Continue` is
      `class="button outline"` like the three format buttons (do not
      introduce a second `primary`; if `primary` is used, add it to the
      table in `docs/agents/ui-styling.md`); the block sits in
      `#format-box` under `#format-choices`, hidden with the `hidden`
      attribute so `.dialog-box [hidden] { display: none }` applies; any
      new id rule sets layout only.
    - `settings.ts`: in `open()`, alongside the `sidecar_format` re-read,
      invoke `label_names` again and `showLabelNames(names)` (the presets
      list is static, re-filling it is fine too).
    - Files: `crates/app/ui/index.html`, `crates/app/ui/src/main.ts`,
      `crates/app/ui/src/firstrun.ts`, `crates/app/ui/src/firstrun.test.ts`,
      `crates/app/ui/src/settings.ts`, `crates/app/ui/style.css` (layout
      only, if needed), `README.md`, `README.ja.md`, `docs/usage.md`,
      `todo.md`.
    - Manual check: clear the settings store (or run with a fresh
      `--config`/app-data dir), launch, pick `Lightroom (XMP)`, see the
      language block, pick 日本語, `Continue`; then Settings > Sidecars
      shows レッド … パープル; relaunch shows no dialog; a fresh store with
      DxO picked never shows the language block; Tab / Shift+Tab stay
      inside the dialog in both phases. Record in `learnings.md`.

- [x] Step 5: Follow a folder rename in the recorded trash runs and the frontend's trash undo entries
  - Done when: after `Rename…` on a folder, an undo of a `Move Rejected to
    Trash` run whose files were under it restores them into the renamed
    folder, a redo moves the restored files (now under the new path) to
    the Trash again, and the frontend's `TrashEntry.dirs` used for the
    "reopen / refresh the open folder" decision point at the new path;
    `crates/app/src/trash.rs` has `Runs::rename_dir(old, new)` with tests
    (a run's files under `old` gain the rebased restore destination while
    their Trash lookup key stays the original path; an undone run's
    `back` paths are rebased in place; files elsewhere are untouched;
    `old == new` is a no-op), `restore` is tested to check `exists` on and
    push `back` with the destination, and `rename_folder` in
    `crates/app/src/rename.rs` calls `Runs::rename_dir` with the same
    canonical `old` / `new` it hands `Index::rename_dir`; `crates/app/ui/src/undo.ts`
    exports `mapTrashDirs(fn)` (the trash-entry counterpart of
    `mapJudgments`) with `undo.test.ts` cases, and `main.ts`'s
    `renameFolder` maps `history` and `redoable` through it with `rebase`
    from `tree.ts`; the `### App: renaming a folder after a Move Rejected
    to Trash run leaves the trash run pointing at the old path` section is
    removed from `todo.md`; `mise run ci` passes.
  - Implementation approach:
    - Planning-time finding that shapes the design: on Windows / Linux an
      undo looks the file up in the Trash by its *original* path
      (`restore_run` → `trash_key(&trashed.path)`), and
      `trash::os_limited::restore_all` puts it back at that original
      path (the Windows Recycle Bin fails when that folder is gone, so the
      missing folders are created first); only macOS (`restore_recorded`)
      renames `trashed_at` to `trashed.path` and so can be pointed
      anywhere. So `Trashed.path` must keep the original (lookup) path
      and a rename adds a separate destination: add `restore_to:
      Option<PathBuf>` to `Trashed` (`None` normally; `Default`-able,
      `Clone`, `PartialEq`) and a `Trashed::destination(&self) -> &Path`
      (`restore_to` or `path`). `Runs::rename_dir` sets `restore_to` on
      every `moved` entry whose `path` (or existing `restore_to`) is
      under `old` (prefix `old + MAIN_SEPARATOR`, the rule
      `Index::rename_dir` uses; the recorded paths and `rename_folder`'s
      `old` / `new` are both canonical, `\\?\`-prefixed on Windows), and
      rewrites the `undone` lists' paths in place (they are the files as
      they now sit on disk, so a plain rebase is right; a redo moves them
      from the new location and records a fresh run).
    - `trash::restore`: check `exists(trashed.destination())`, push
      `back` and `restored` with the destination; the `mover` gets the
      `Trashed` already. macOS `restore_recorded`: rename `at` to
      `trashed.destination()`. Non-macOS `restore_run`: after
      `restore_all([item])` succeeds and `restore_to` is set, `fs::rename(
      &trashed.path, dest)` (the destination's parent exists, the folder
      was renamed to it) and then `fs::remove_dir(old parent)` ignoring
      its error (it only removes an empty folder, which is what the Trash
      recreated); a failed second rename is that file's failure message
      and leaves it where the Trash put it. Cover the pure part in
      `trash.rs`'s tests through the `in_trash` / `mover` closures, and
      keep the OS-level part small.
    - `rename.rs`: after `index::lock(&index).rename_dir(&old, &new)`
      (inside the same `Scans` lock hold), `app.state::<trash::Runs>()
      .rename_dir(&old, &new)`; the runs are in-memory only, so nothing
      to persist.
    - Frontend: `undo.ts` `mapTrashDirs(fn: (dir: string) => string)`
      returns `(entry) => isJudgments(entry) ? entry : { ...entry, dirs:
      entry.dirs.map(fn) }`; `main.ts` `renameFolder`, in the `.then`
      right after `folders.renamed(...)`, does `const moved = (dir) =>
      rebase(dir, path, newPath) ?? dir; history.map(mapTrashDirs(moved));
      redoable.map(mapTrashDirs(moved));` (`rebase` is the tree-key
      spelling the `dirs` came from; the other plan's Step 4 may give it
      an optional `ignoreCase` parameter with a `false` default, which
      this call does not need to pass). `openDirectory` afterwards drops
      only judgment entries (`removeWhere(isJudgments)`), so the trash
      entries survive the reopen as today.
    - Files: `crates/app/src/trash.rs`, `crates/app/src/rename.rs`,
      `crates/app/src/commands.rs` (`restore_run`, `trash_one` untouched),
      `crates/app/ui/src/undo.ts`, `crates/app/ui/src/undo.test.ts`,
      `crates/app/ui/src/main.ts`, `todo.md`.
    - Manual check (Windows): reject two files in `D:\photos\samples\X`,
      `Move Rejected to Trash…`, rename the folder in the tree, `Undo`:
      the files come back into the renamed folder and no empty
      old-name folder is left behind; `Redo` moves them to the Recycle
      Bin again. Record in `learnings.md`, including the macOS path as
      "not run" if no Mac is available.

## Trade-offs and risks

- **Step order and `todo.md` conflicts.** Steps 4 and 5 remove adjacent
  sections, and Step 5's section borders the other in-flight plan's Step 5
  section; running sequentially from a fresh `main` and putting Step 5
  last keeps every rebase trivial. If the other plan's Step 5 has not
  merged by then, the `pr-conflict-resolver` rule it added in its Step 1
  ("a deleted `todo.md` section stays deleted") covers the conflict.
- **Step 1: fix scope.** Making `flush` return `bool` is a one-line API
  change every caller can ignore; the alternative (leave `flush` as is and
  poll with `eventually` for the files) would hide a failed write again.
  The 30 s budget is the same kind of "only costs time when failing" budget
  `sidecar.rs`'s tests use.
- **Step 2: the throttle interval.** 5 s was chosen (not configurable);
  a longer interval risks missing an external change the watcher dropped,
  a shorter one does not collapse a slow alt-tab. Measured from the last
  scan *start* (any trigger), so a focus right after an open is skipped.
- **Step 2: backend change vs. frontend-only.** Clearing `scanRunning` on
  the frontend when `scan_folder` returns `total: 0` was rejected: the
  faces pass may still have work, and `scanRunning` is what defers a
  rescan that would cancel it. Not inserting a `pending` entry when both
  passes are empty keeps every event and guard as they are and ends the
  scan in one event round trip.
- **Step 3: which folders get the items.** Both items are offered on
  non-root single folders only, one rule like `Rename…`: `Expand All` on
  home or a volume would walk the whole disk. If `Collapse All` on a root
  turns out wanted, it is one condition in `folderMenuGroups`.
- **Step 3: `Collapse All` semantics.** Chosen: collapse the subfolders,
  leave the clicked folder as it is (the todo says "every subfolder under
  the clicked folder"). Collapsing the folder itself too is the other
  reading; both are a one-line difference in `collapseAll`.
- **Step 3: watcher count.** `Expand All` on a large archive sets one
  `tree-changed` watch per expanded folder, the same as expanding them by
  hand; the Windows ancestor-pinning item in `tauri-app.md` means a rename
  of an ancestor releases and restores them all. Measure and note it.
- **Step 4: where the language is asked.** Chosen: a second phase inside
  the same dialog (no new dialog, one Tab trap, one gate). A separate
  dialog would double the modal plumbing for one select.
- **Step 4: preselecting by `navigator.language`.** A small convenience
  with a pure, tested function; if it is judged speculative, drop it and
  preselect English (the presets' first entry).
- **Step 5: the platform split.** This is the one item where planning found
  a design point the todo does not mention: on Windows / Linux the Trash
  restores to the original path, so "rebase the recorded path" alone would
  break the Trash lookup. The plan keeps the lookup key and adds a
  destination, with a rename after the restore. The alternative, forgetting
  (dropping) every run and frontend entry that points into a renamed folder
  so undo is simply not offered, is far smaller (a `Runs::forget_under` +
  `history.removeWhere`) but does not meet the todo's "undoing or redoing
  the run after a rename still targets the right folder"; the user chose
  the full design on 2026-09-30.
- **Step 5: the recreated old folder.** Before `restore_all` the missing old
  folders are created and recorded; afterwards only those are removed, and
  only while empty, so a folder the user re-created under the old name
  meanwhile is left alone.

## Progress

- (2026-09-30) Step 1 complete
- (2026-09-30) Step 2 complete
- (2026-09-30) Step 3 complete
- (2026-09-30) Step 4 complete
