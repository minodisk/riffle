# Learnings

## Step 1: backend folder rename

- `Scans`' inner mutex, `ScansState`, `ScansState::scanning` and
  `SCAN_RUNNING` became `pub(crate)` so `rename.rs` can hold the `Scans` lock
  across the rename and the index write, as `trash_rejected` does.
- `folder_target` runs inside `spawn_blocking` (first thing, before the
  writer flush) rather than before it as the plan lists: it canonicalizes and
  stats paths, and blocking IO on an async runtime worker is what
  `docs/agents/tauri-app.md` "Synchronous commands run on the main thread"
  warns against. The refusal still happens before anything on disk changes.
- `Index::rename_dir` first deletes any rows already stored under the new
  path: the rename's target did not exist on disk, so such rows are stale
  cache of a vanished folder, and without the delete the `UPDATE` would fail
  on the `path` / `dir` primary keys. `old == new` returns early so that
  delete can never wipe the moved rows.
- `watch::release` (the lock-free core of `release_under`) matches "under"
  with `Path::starts_with`, which is component-wise, so `photos2` does not
  count as under `photos`. The test builds a real `notify` watcher on a temp
  dir; the Windows handle release itself is not asserted, since notify's
  Windows watcher stops its thread asynchronously on drop.

## Step 2: folder rename in the tree

- The input's `blur` is ignored while `render` swaps the rows (a `rendering`
  flag): Chromium (WebView2) can fire `blur` when the focused input is
  removed by `replaceChildren`, which would otherwise confirm the edit on
  every mid-edit re-render. It is also ignored while `document.hasFocus()`
  is false, so switching to another app does not confirm a half-typed name;
  the input gets focus back when the window does.
- `render` drops `editing` when the edited row is no longer drawn (a listing
  removed the folder); otherwise the live edit would keep swallowing every
  key with no input on screen.
- `folders.keydown` treats every key as native while `event.isComposing`,
  so the `Enter` that commits an IME conversion (Japanese input) does not
  confirm the rename.
- The slow-click timer compares `performance.now()` rather than `Date.now()`
  as the plan sketched: `setTimeout` runs on the monotonic clock, and a
  coarser or adjusted wall clock could make `due` answer false for a timer
  that fired on time.
- The slow click is disarmed by a `mousedown` anywhere in the document, not
  only in the tree container, so a click on the strip cancels it too (the
  plan's "another click landed anywhere"). `main.ts`'s window `keydown`
  calls the exported `folders.cancelSlowClick()` first, whichever element
  has focus.
- After Enter / Escape the tree container takes focus back, so the tree
  keeps the keyboard as Explorer does; a click-away finish leaves the focus
  where the click put it.
- A rename warning (`Renamed.warning`) is shown after the reopen resolves,
  since `openDirectory`'s `show()` clears the status note. The next
  `show()` / decode clears it again, as it does any other note.
- `renameFolder` in `tree.ts` drops stale nodes already keyed under the new
  path, so a folder once listed there (since deleted on disk) cannot
  duplicate the renamed row or overwrite its state.
