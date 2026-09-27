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

## Step 3: backend file rename

- `rename_file` makes its `file_plan` after the writer flush and the `Scans`
  lock, not before the flush as the plan lists: a pending judgment the flush
  writes can create a sidecar for the old name, and a plan made before it
  would leave that sidecar behind. The refusals still come before anything
  moves; the flush only writes what was going to be written anyway.
- `dir` is canonicalized before `file_plan`, as `trash_rejected` does before
  `trash::plan`, since `path` comes in `list_arw`'s canonical spelling.
- JPEG files: main now lists a JPEG-only folder's `.jpg` / `.jpeg` files in
  the strip, but that folder is view-only (its strip menu holds only
  `Select All`, nothing writes to it). `file_plan` keeps the plan's
  `is_raw_file` check on both the source and the new name, as `trash::plan`
  does, so a JPEG cannot be renamed; Step 4 should not offer `Rename…` in a
  JPEG folder (it only adds to the menu a JPEG folder does not show).
- PhotoLab: `photolab::lookup` finds a RAW's Uuids by its folder and file
  `Name` in PhotoLab's database, and a `.dop` Riffle writes carries
  `Name = "<RAW name>"` inside. A renamed file's moved `.dop` keeps its old
  Uuids and old inner `Name`, and the database still lists the old name, so
  PhotoLab sees the file as new (or unmatched) until it re-indexes. The
  rename does not touch either; see the deferred item below.
- A case-only rename test asserts on `read_dir`'s names, which carry the
  stored spelling on every file system, rather than on `exists()`.

## Step 4: file rename in the strip

- The strip's editor swaps the cell's `.name` span for the input in place
  (and back when the edit ends) instead of re-creating the cell as the plan
  sketched. `Cell.name` keeps pointing at the detached span, so a badge
  repaint (`setRating`'s label tint) updates the span and leaves the input
  alone, and a click away that ended the edit through `blur` still reaches
  the cell it landed on, even the edited cell's own thumbnail, since only
  the input is detached. `createCell` never has to draw the input: `render`
  never releases the editing cell and `setFiles` cancels the edit first.
- `onRename` takes the path the edit started on rather than the index, so
  the backend call cannot name another file if `files` changed in between.
- On success `main.ts` moves (rather than only drops) the old path's
  `ratings`, `flags`, `labels`, `sharpness`, `entries` and `touched` to the
  new path: the index rows were rewritten with the same values, and with
  the entries gone the capture-time sort would park the renamed cell at the
  end until `scan-done`'s `folder_entries` refresh, jumping the strip. Undo /
  redo batches that only reference the old path are removed, as planned.
- Round 2 local review found that anchoring the post-rename rescan with an
  explicit `resync(anchor)` parameter (an earlier version of this change)
  only held when the rescan ran right away: a scan can start while the
  `rename_file` invoke is still in flight (the window loses and regains
  focus, firing `resync()`), and the stored anchor was then applied
  unconditionally whenever that deferred rescan drained, regardless of
  where the user had since moved. The fix instead patches `allFiles` /
  `files` / `fileIndex` in place at resolve time, so `files[index]` is
  already correct whether the rescan runs immediately or later, and
  `resync()` needs no anchor parameter at all.
- The slow click does not arm on a modified click (Cmd / Ctrl / Shift), which
  is a selection gesture, and `canRename` is `!scanRunning && !viewOnly`, so a
  JPEG-only folder never arms (its menu already has no `Rename…`).
- The key handling for a live file rename sits in `main.ts`'s window
  `keydown` right after the modal branches and before `keyName`'s `null`
  return, treating `event.isComposing` as native like the tree does. After
  Enter / Escape the input is removed and focus falls back to the body, which
  is where the culling keys already work.

## Deferred issues (todo candidates)

- **Opening a subfolder of a folder whose rename is in flight.** Step 2's
  round 4 local reviewer noted, outside the reviewed diff, that clicking a
  subfolder of the folder being renamed while `rename_folder` is still in
  flight can open that subfolder under its old path. Done when: the tree
  refuses (or defers) opening a path under a folder whose rename has not
  returned yet, or reopens it under the rebased path once it returns.
- **PhotoLab after a file or folder rename.** From Step 3's check of the
  PhotoLab Uuid lookup (`crates/app/src/photolab.rs` queries `Sources` by
  `FolderId` and `Name`; `crates/core/src/dop.rs` writes the RAW's `Name`
  into the `.dop`). `rename_file` (`crates/app/src/rename.rs`) moves the
  `.dop` unchanged, so its inner `Name` names the old file and PhotoLab's
  database still holds the old name and folder. Done when: it is known
  (checked with PhotoLab) whether PhotoLab re-matches a renamed RAW with its
  moved `.dop` or imports it as a new image / virtual copy, and the rename
  either patches the `.dop`'s `Name` or the docs say what to expect.
- **A `.xmp` shared by `a.ARW` and `a.DNG`.** From Step 3's `file_plan`
  (`crates/app/src/rename.rs`), which, like `trash::plan`
  (`crates/app/src/trash.rs`), takes `a.xmp` as `a.ARW`'s sidecar even when
  an `a.DNG` in the same folder uses the same `a.xmp`. Renaming `a.ARW`
  carries the DNG's sidecar away. Done when: a rename (and the trash) leaves
  a `.xmp` another RAW of the same stem still uses, or refuses with a
  message.
