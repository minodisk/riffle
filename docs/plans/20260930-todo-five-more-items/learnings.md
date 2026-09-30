# Learnings

## Step 1: the flaky switch-to-Both sidecar test

- Root cause analysis. The test queues the dirty row with `set_now` (deadline
  `Instant::now()`) and then sends `Flush`. The writer thread handles the
  `Set` first, then either its `recv_timeout(0)` fires and it writes the row
  (`flush(Some(now))`) or it receives the `Flush` and writes it
  (`flush(None)`); either way the reply to `Flush` is sent only after the
  write, so message order is not a race. The read can only see no file when
  (a) `Writer::flush`'s `recv_timeout(DRAIN_TIMEOUT)` gave up after 2 s while
  the writer was still inside `write_kind` for two files (`File::create` +
  `sync_all` + `rename`, and the Windows `rename` retries up to 5 times with
  20-100 ms sleeps on a sharing violation) on a loaded `windows-latest`
  runner, or (b) the write failed, whose message only went to the test's
  `on_error`, which printed it to stderr and forgot it. Both were silent, so
  the failure surfaced one step later as `NotFound` on the read. Which of the
  two happened in the failed run cannot be told from its log; the fix makes
  either one fail loudly at the drain.
- The fix: `Writer::flush` now returns `bool` (`true` when the drain reply
  arrived in time). Production callers keep ignoring it; `bool` is not
  `#[must_use]`, so clippy stays quiet. The test's `switch_writer` now
  collects every `on_error` message into an `Arc<Mutex<Vec<String>>>`, and a
  new `drain` helper flushes with a 30 s budget and asserts both the flush
  result and an empty error list before any sidecar is read.
- The two sibling tests in the same shape
  (`a_rating_set_during_a_switch_lands_in_the_new_format_and_leaves_no_dirty_row`
  and `a_pick_set_during_a_switch_lands_in_the_dop_and_leaves_no_dirty_row`)
  were changed too: `switch_writer` was used only by these three tests, so
  they all go through the collecting writer and `drain` now.
- Only final errors fail the test: a write that fails once on the deadline
  path (`set_now` makes `recv_timeout(0)` usually fire before the `Flush`)
  and then succeeds on the rewrite leaves a `(retrying in Ns)` message, and
  the row and both sidecars are correct. `drain` therefore filters out
  messages containing `" (retrying in "` and asserts the rest is empty, so
  the Windows-flaky test is not made flakier. The full list, retry messages
  included, goes in the assertion message so a transient sharing violation
  stays visible in the log; the timeout message carries it too. This departs
  from the plan's "the collected errors are empty" wording on purpose.

## Step 2: throttling the focus rescan

- `lastScanAt` is set in `startScan`, which `openDirectory` and `resync`
  share, and the focus listener stamps it too right before it calls
  `resync()` (which reaches `startScan` only after its `list_arw` resolves,
  so a second focus in that window would otherwise pass the throttle), so
  the throttle is measured from the last scan start or focus trigger. `File > Reload Folder`, `folder-changed` and the trash / rename
  paths call `resync()` directly and are not throttled. A focus inside the
  interval is dropped, not deferred; a focus during a running scan that is
  past the interval still goes through `resync()`'s `resyncPending` path.
- Backend: `scan_folder`'s reconcile `spawn_blocking` now also calls
  `Index::faces_todo(&dir)` under the same index lock when the first pass's
  `todo` is empty, and only inserts the `pending` entry when either pass has
  work. The `index` handle is moved into the sidecar reconcile closure later
  on, so the check had to live in the earlier closure; doing it there also
  avoids a second lock and `spawn_blocking`. A failed `faces_todo` counts as
  "not idle", so the faces pass runs and logs the error as before.
  `faces_todo`'s side effect (marking untestable rows done) is the same as
  when `run_faces_pass` calls it first.
- No new Rust test: the "a fully scanned folder has an empty `faces_todo`"
  case is already asserted in `index.rs`'s eye-focus test (`faces_todo` is
  empty after `write_faces`), and `scan_folder` itself needs a Tauri
  `AppHandle`, which no existing test builds.
- The first try at removing the `todo.md` section cut it at the next `##`
  match, which was the section's own `#### TODO`; check the next `### `
  heading when deleting a section by line range.

## Step 3: Expand All / Collapse All in the folder tree's menu

- `tree.ts` gained `descendants(tree, path)` (the listed folders under
  `path`, depth first, through `drawnChildren` so a root drawn elsewhere,
  such as home under its volume, is left to its own row) and
  `collapseAll(tree, path)` on top of it, which returns the same `Tree`
  object when there is nothing under `path` (an unknown or unlisted folder),
  so the tests can assert `toBe`.
- `list_subfolders` children carry `is_link` (a symlink or junction), and
  `expandAll` lists such a folder without descending, since `list` follows
  links and `a/loop -> a` would otherwise grow without end (review round 1).
- `folders.ts`'s `expandAll` walks recursively, awaiting one
  `list_subfolders` at a time, and keeps the chain of paths from the clicked
  folder so a branch stops as soon as any ancestor on it was collapsed or
  dropped (a rename re-keys the nodes, so a rename mid-walk also stops it).
  It expands a folder before listing it, as `toggle` does, so leaf folders
  end up expanded too: they show their RAW count badge and each gets a
  `tree-changed` watch, exactly as when expanded by hand.
- The tree's own `collapseAll` is imported into `folders.ts` as
  `collapseUnder` because `folders.ts` exports its own `collapseAll(path)`
  for `main.ts`.
- The multi-selection shape needed no new test: the existing
  `folderMenuGroups with several folders selected` case compares the whole
  menu with `toEqual`, so it already asserts the two items are absent.

## Step 4: Lightroom's UI language in the first-launch dialog

- The language block (`#format-language`) sits in `#format-box` between
  `#format-choices` and the note, hidden with `hidden`; its id rule is
  scoped `:not([hidden])` because an id rule outranks
  `.dialog-box [hidden] { display: none }` (`docs/agents/ui-styling.md`).
  `ui-styling.md`'s component table now lists `#format-language-select`
  under `select` and `Continue` under `button outline`.
- `saveFormat` in `main.ts` chains `set_label_names` and then
  `choose_sidecar_format` (the format last, since it is the one key that
  gates the dialog); a rejection of either lands in `#format-error` and
  re-enables every control. If the names save rejects, the format is not
  saved and the dialog comes back on the next launch; `Continue` again in
  the same session retries both (re-choosing the same format is a no-op in
  `choose_format`).
- A click on XMP / Both while the language block is shown just replaces
  `chosenFormat`; a click on DxO saves at once from either phase. The
  chosen button gets no visual state (there is no pressed style in the
  component classes); the question text stays generic.
- If `label_names` fails the select stays empty, the error shows in
  `#format-error`, and `Continue` does nothing (no preset matches), so the
  dialog cannot close without a language; DxO stays available.
- The Tab trap now filters `button, select` by `closest("[hidden]")` and
  reuses `modal.ts`'s `cycleFocus`, so both phases cycle over what is
  visible.

## Step 5: trash runs following a folder rename

- `Trashed` got `restore_to: Option<PathBuf>` and `destination()`;
  `Trashed.path` stays the original path, the key the Windows / Linux Trash
  is looked up by (`trash_key(&trashed.path)`). `Runs::rename_dir(old,
  new)` rebases each recorded file's current destination (`restore_to`
  or `path`) with the `old + MAIN_SEPARATOR` prefix rule of
  `Index::rename_dir`, and sets `restore_to` back to `None` when the rebased
  destination is the original path again (a rename there and back), so no
  needless second move runs. Undone runs' `back` lists are rebased in place.
- `restore` checks `exists`, and reports `restored` / `failed` / `back`,
  with the destination, so the frontend's `restoredInto(openDir, …)` sees
  the files under the reopened (renamed) folder. `Restored::none` reports
  the destination too.
- Non-macOS `restore_run`: a file with `restore_to` first gets its missing
  old ancestors created (`trash::create_missing`, returning exactly the
  ones created): trash 5.2.9's Windows `restore_all` fails with "file not
  found" when the original folder is gone (only the freedesktop backend
  runs `create_dir_all`), verified with a probe (review Round 1). After
  `restore_all` succeeds, `trash::relocate(from, to, created)` does a
  `fs::rename`, then `remove_dir` on only the created folders, deepest
  first, each while empty, so a pre-existing empty old-name folder is never
  removed. A failed move is that file's failure ("put back at …
  but not moved into the renamed folder"), leaving it where the Trash put
  it; its sidecars then stay in the Trash by `restore`'s RAW-first rule.
  macOS `restore_recorded` renames straight to `destination()`.
- A case-only rename on Windows (`x` → `X`) rebases too; `restore_all`
  puts the file into the same folder, nothing is created, and the second
  rename is a same-folder no-op, so nothing is lost.
- `rename_folder` calls `Runs::rename_dir(&old, &new)` right after the
  index rewrite, under the same `Scans` lock, outside the `if let Some(index)`
  so the runs follow even with no index. The first scripted edit also hit
  the identical tail of `rename_file` (same `));` / `drop(state)` shape);
  caught in the diff and reverted, so anchor such edits on unique context.
- Frontend: `undo.ts`'s `mapTrashDirs(fn)`; `renameFolder` maps `history`
  and `redoable` through it with `rebase(dir, path, newPath) ?? dir`
  (case-sensitive, the tree-key spelling the dirs came from).

## Deferred issues (todo candidates)

- **Pending manual check (Step 2, Windows, the user's):** verify the idle
  scan ends at once with `Timing logs` on. Steps: open a folder and let its
  scan and faces pass finish; wait more than 5 s; switch to another app and
  back to Riffle. Expected: `Riffle.log` shows `scan prepare: … todo=0`
  with no `scan extract` or `scan faces` line after it, and the status
  line's `scanning` never appears. Also alt-tab away and back twice within
  5 s: only the first focus logs a `scan list` line. Not run by the
  implementation agent (no GUI session); Step 2's checkbox was ticked on the
  automated criteria (Vitest cases for `focusRescanDue`, `mise run ci`).
  Basis: plan Step 2 "Tests" bullet. Files: `crates/app/ui/src/main.ts`,
  `crates/app/src/commands.rs`.
- **Pending manual check (Step 3, Windows, the user's):** exercise
  `Expand All` / `Collapse All` in the folder tree. Steps: right-click a
  folder with two or more levels of subfolders (e.g. under
  `D:\photos`), choose `Expand All`; then `Collapse All` on the same
  folder; then `Expand All` on a folder containing a subfolder that cannot
  be listed (e.g. one with its read permission denied). Expected: every
  level appears after `Expand All`; `Collapse All` folds them all back
  with the clicked folder still open; the unreadable folder shows its
  error (status line, row marked failed) and the rest still expand. Also
  measure how many `tree-changed` watches a real `Expand All` on a photo
  archive makes (the number of expanded drawn folders; `Timing logs` or
  a debugger on `set_tree_watches`) and check that renaming an ancestor
  of the expanded folders still works on Windows (see the `notify`
  ancestor pinning item in `docs/agents/tauri-app.md`). Not run by the
  implementation agent (no GUI session); Step 3's checkbox was ticked on
  the automated criteria (Vitest cases in `tree.test.ts` /
  `context.test.ts`, `mise run ci`). Basis: plan Step 3 "Manual check" and
  its watcher-count bullet. Files: `crates/app/ui/src/folders.ts`,
  `crates/app/ui/src/tree.ts`, `crates/app/ui/src/main.ts`.
- **Pending manual check (Step 4, Windows, the user's):** run the
  first-launch dialog with a fresh settings store (delete the app-data
  settings store or use a fresh app-data dir). Steps: launch, pick
  `Lightroom (XMP)`: the language block appears with the select
  preselected by the OS language; pick 日本語, `Continue`; open Settings >
  Sidecars: the Lightroom-language fields show レッド … パープル; relaunch:
  no dialog. With another fresh store pick `DxO PhotoLab (.dop)`: the
  dialog closes at once, no language block. In both phases Tab /
  Shift+Tab stay inside the dialog and reach the select and `Continue`
  only once they are shown. Not run by the implementation agent (no GUI
  session); Step 4's checkbox was ticked on the automated criteria
  (Vitest cases in `firstrun.test.ts`, `mise run ci`). Basis: plan Step 4
  "Manual check". Files: `crates/app/ui/src/main.ts`,
  `crates/app/ui/index.html`, `crates/app/ui/src/settings.ts`.
- **Pending manual check (Step 5, Windows, the user's):** undo / redo a
  trash run after renaming its folder. Steps: reject two files in a copy
  under `D:\photos\samples\X` (ideally one with both sidecar formats),
  `Move Rejected to Trash…`, rename the folder in the tree (`Rename…`),
  `Undo`. Expected: the files and their sidecars come back into the
  renamed folder, the strip (reopened under the new path) shows them with
  their reject marks, and no empty old-name folder is left behind (also
  with a recursive run over a parent whose subfolder is renamed); `Redo`
  moves them to the Recycle Bin again. Not run by the implementation agent
  (no GUI session); Step 5's checkbox was ticked on the automated criteria
  (`trash.rs` tests for `Runs::rename_dir`, `restore` with a destination
  and `relocate`, the `undo.test.ts` case, `mise run ci`). Basis: plan
  Step 5 "Manual check". Files: `crates/app/src/trash.rs`,
  `crates/app/src/commands.rs` (`restore_run`), `crates/app/src/rename.rs`,
  `crates/app/ui/src/main.ts`.
- **Pending manual check (Step 5, macOS, not run):** the same undo / redo
  after a rename on a Mac, where `restore_recorded` renames the Trash file
  straight to the new destination (no `relocate`). Expected: the files come
  back into the renamed folder and no old-name folder appears. Not run (no
  Mac available); covered only by the platform-independent `restore` test.
  Basis: plan Step 5 "Manual check". Files: `crates/app/src/trash.rs`
  (`restore_recorded`).
