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

# Rename folders from the tree and files from the strip

## Purpose

Renaming a shoot folder or a RAW file today means leaving Riffle for the file
manager, and coming back to a folder whose index rows, remembered position and
pending sidecar writes all still point at the old path. This adds a `Rename…`
item to the folder tree's right-click menu and to the strip's right-click menu,
each editing the name inline where it is shown, as the OS file managers do,
and the file managers' other way in: a slow second click on the name of the
item that is already current. A folder rename carries the SQLite index along
(every `files` / `ratings` / `folders` row under it, dirty rows and
`last_viewed` included, so nothing is re-extracted and no judgment is lost),
releases the folder watcher's handle first so Windows lets the rename through,
and reopens the current folder under its new path with the tree's expansion
kept. A file rename carries its `.xmp` and `.dop` sidecars and its index rows
with it. Both refuse a collision, an invalid name or a running scan without
touching the disk, and show the OS error when the rename itself fails.

This closes the `todo.md` item
`### App: rename a folder from the folder tree's context menu`. Renaming
files and the slow-click entry are the user's additions on top of that item;
they reuse the same backend shape and the same inline-edit decisions. The
wrap-up phase's todo-curator removes the todo section; no step edits
`todo.md`.

## Steps

- [x] Step 1: Backend folder rename: name validation, index prefix rewrite, watcher release and the `rename_folder` command
  - Done when:
    - A new module `crates/app/src/rename.rs` holds a pure
      `check_name(name: &str) -> Result<(), String>` (rejects empty, `.`,
      `..`, any `/` or `\`, and on every platform the Windows-illegal
      characters `<>:"|?*`, control characters, and a trailing dot or space,
      so a name typed on macOS still copies to an NTFS card) and a pure
      `folder_target(dir: &Path, name: &str) -> Result<PathBuf, String>`
      that refuses a `dir` with no parent (a drive root or `/`), a name that
      fails `check_name`, and a collision: the target exists and is not the
      same directory entry as `dir` (compare `std::fs::canonicalize` of both,
      so a case-only rename on a case-insensitive file system is allowed
      rather than reported as a collision).
    - `Index::rename_dir(&mut self, old: &str, new: &str) -> Result<usize, String>`
      in `crates/app/src/index.rs` rewrites, in one transaction, the `dir` and
      `path` of every `files` and `ratings` row whose `dir` is `old` or under
      it, and the `dir` and `last_viewed` of every `folders` row likewise,
      returning the number of `files` rows moved. Match "under it" with
      `substr(dir, 1, length(?prefix)) = ?prefix` where `prefix` is `old` plus
      the platform separator (not `LIKE`, whose `_` / `%` would match path
      characters). Rows keep their `thumb`, `extractor`, `faces_extractor`,
      `dirty` and `xmp_*` columns untouched.
    - `watch::release_under(app, dir: &str) -> Option<(String, String)>` in
      `crates/app/src/watch.rs` drops the watcher (and clears `State::dir`)
      when the watched canonical folder is `dir` or under it, and leaves it
      alone otherwise; it returns the watched dir and its owner string when
      it dropped one, so a caller that could not use the release for what it
      released it for can put the watch back with `set`. `set` is unchanged.
    - A `rename_folder(app, dir: String, name: String) -> Result<Renamed, String>`
      Tauri command in `rename.rs` (commands live in their feature module, as
      in `sequence.rs` and `folders.rs`), registered in
      `crates/app/src/main.rs`'s `generate_handler!`, does, in this order:
      refuse while `Scans::scanning()` with the existing `SCAN_RUNNING`
      message (make the constant `pub(crate)`); `folder_target`; in
      `spawn_blocking`: `writer.flush(DRAIN_TIMEOUT)`, take the `Scans` lock
      and re-check `scanning()`, canonicalize `dir` to `old_canonical`,
      `watch::release_under`, `std::fs::rename`. If the rename fails, the
      released watch (if any) is set back with `watch::set` before the error
      (`"{name}: {io error}"`) is returned, so a failed rename really changes
      nothing. On success, canonicalize the new path to `new_canonical`, then
      `Index::rename_dir(old_canonical, new_canonical)`. An index failure
      after a successful rename is logged with `log::warn!` but still
      returned as `Ok`: the disk has already moved, so the caller must rebase
      and reopen under the new path regardless; the `Ok` carries a warning
      whose text says the folder was renamed but its cache was not (the rows
      are simply re-extracted on the next open). `Renamed { path: String,
      warning: Option<String> }` carries the new path spelled the tree's way
      (`dir`'s parent joined with `name`, not the canonical one), so the
      frontend can rebase its own strings, and the warning (if any) for the
      status line.
    - Tests: `check_name` and `folder_target` (temp-dir collision, case-only
      rename allowed, root refused) in `rename.rs`; `rename_dir` in
      `index.rs` (a folder with a subfolder, a dirty `ratings` row, a
      `last_viewed`, and a sibling folder sharing the prefix as a plain
      string, e.g. `d:\photos` vs `d:\photos2`, which must not move);
      `release` in `watch.rs` (a watched temp dir is released for itself and
      for its parent with its dir and owner returned, kept for a sibling).
    - `CLAUDE.md`'s Layout paragraph names `src/rename.rs`.
    - `mise run ci` passes.
  - Implementation approach:
    - Mirror `trash.rs` (pure planning, temp-dir tests with the
      `riffle-<module>-<name>-<pid>` naming) and `trash_rejected`'s
      guard order. Hold the `Scans` lock across the disk rename and the index
      write, as `trash_rejected` does.
    - The frontend refuses too while `scanRunning` (Step 2); the backend
      check is the authoritative one, as for `trash_rejected`.
    - `rename_dir` runs on `AppIndex` (the writer connection), inside
      `index::lock`. Follow `set_last_viewed` for the `spawn_blocking` +
      `lock` shape.
    - Do not touch `reconcile`, `EXTRACTOR_VERSION` or `SCHEMA_VERSION`: no
      schema change is needed.
    - Per `docs/agents/tauri-app.md` "A case-insensitive file system makes
      `exists()` match the wrong spelling", tests on a case-only rename
      assert on canonical equality, not on the exact resulting spelling.

- [ ] Step 2: Folder `Rename…` in the tree's context menu, the slow second click, the inline row editor, and reopening under the new path
  - Done when:
    - Right-clicking a non-root folder row shows `Rename…` as its own group
      after `Copy Path` / `Copy Folder Name` and before
      `Sequence JPEG Timestamps…`; a root row (home, a volume) does not offer
      it. `folderMenuGroups` in `crates/app/ui/src/context.ts` takes a
      `canRename: boolean` and `crates/app/ui/src/context.test.ts` covers
      both shapes and the order.
    - A slow second click starts the same edit: a click on the name text of
      the row that is already the open folder (`current`) arms a rename and
      does not reopen the folder; about 500 ms later the edit starts unless,
      meanwhile, the click became a double-click, another click landed
      anywhere, or a key was pressed. A click on the same row outside its
      name (the indent, the expander's neighborhood, the count) keeps
      today's behavior; a click on any other row opens it as today. Root
      rows never arm, nor does any row while a scan runs.
    - Starting the edit (either way) turns the row's name into a text
      input, prefilled with the folder's name and fully selected, focused.
      Enter confirms, Escape cancels, and losing focus (a click anywhere
      else, including a strip cell or another tree row) confirms, as Explorer
      and Finder do. A confirmed name that is empty or equal to the current
      one just ends the edit. While the edit is live no key reaches the
      tree's own handling (arrows, `Home` / `End`, `Enter` / `Left` /
      `Right`, type-ahead, `Escape`'s container blur) or the culling keymap;
      the input gets every key natively except Enter and Escape.
    - A `list()` answer landing mid-edit (an expand or `reveal` in flight)
      re-renders the tree without losing the edit: the input comes back on
      the same row with the typed value, its focus and its selection. A
      right-click or a plain click on the editing row does not open the
      folder or open the menu.
    - The DOM-free decisions live in a new `crates/app/ui/src/rename.ts`,
      tested in `rename.test.ts`:
      - an `InlineRename` state (`kind: "folder" | "file"`, `path`,
        `original`, `value`, `committed`), `editKey(key): "confirm" |
        "cancel" | "native"`, `confirmName(original, value): string | null`
        (trimmed; `null` when empty or unchanged), and a `commit()` that
        returns the decision once and `null` after (so the blur that follows
        an Escape or an Enter cannot fire a second time);
      - `SLOW_CLICK_DELAY = 500` and a `SlowClick` tracker:
        `click(path, alreadyCurrent, now): boolean` arms `path` at `now` and
        returns true only when `alreadyCurrent` (a first click on a
        not-yet-current item never arms; a new arm replaces an older one),
        `cancel()` disarms, `due(path, now): boolean` is true once for an arm
        still standing for `path` with `now - at >= SLOW_CLICK_DELAY` and
        disarms as it answers. Tests: no arm on a first click; arm then
        `due` after the delay; `due` false before the delay; `dblclick`
        (`cancel`) then `due` false; a click on another item replaces the arm
        so `due` for the first path is false; `cancel` from a key; `due`
        answers true only once.
      - `stemLength(name)` for Step 4 can land here now or then.
    - While a scan runs the menu item reports `a scan is running; wait for
      it to finish` in the status line and starts no edit (same check as
      `trashRejected`); the slow click silently does not arm.
    - On success the tree keeps its expansion: a pure `renameFolder(tree,
      oldPath, newPath, newName)` in `crates/app/ui/src/tree.ts` re-keys the
      node and every node under it, replaces the entry in the parent's
      `children` (re-sorted case-insensitively as `list` sorts) and is
      covered in `tree.test.ts`; a pure `rebase(path, oldDir, newDir):
      string | null` (the path under `newDir` that `path` had under `oldDir`,
      using `tree.ts`'s existing `normalize` rules, `null` when `path` is not
      `oldDir` or under it) is exported and tested too. `folders.ts` exports
      `renamed(oldPath, newPath, newName)` that applies both to `tree`,
      `current` and `cursor` and re-renders.
    - When the open folder is the renamed folder or under it, `main.ts`
      reopens it: `openDirectory(rebase(openDir, oldPath, newPath),
      newFolderToken())`, so the strip resumes at the remembered file (the
      rewritten `last_viewed`) and the watcher is set on the new path by
      `scan_folder`. When it is unrelated, nothing but the tree changes.
    - A backend error (collision, invalid name, OS error, scan running) shows
      in the status line via `setStatus(String(err))`; the row shows its old
      name again. A successful `Renamed` with a `warning` still rebases and
      reopens as above, and additionally shows the warning via `setStatus`.
    - `README.md` and `README.ja.md` (in sync) and `docs/usage.md`'s
      folder-tree paragraph mention `Rename…`, the slow second click on the
      open folder's name, the keys (Enter / Escape / click away) and the note
      that the open folder is reopened under its new name.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (`rename_folder`, `Renamed.path`,
      `Renamed.warning`).
    - `folders.ts` keeps `let editing: InlineRename | null`. `render` draws
      the input for the row whose `node.path === editing.path` (an
      `<input type="text" class="name">` in place of the `.name` span,
      `value = editing.value`), and after `replaceChildren` focuses it and
      restores the selection when `editing` is set; the input's `input`
      event stores the value back into `editing`, so a re-render mid-edit
      rebuilds the same state. `render` is called by `toggle`, `reveal` and
      `moveCursor`; none needs to be deferred.
    - Slow click: `folders.ts` holds one `SlowClick` and one timer id. The
      `.name` span gets its own `click` listener: when `node.path ===
      current`, the row is not a root and `canRename()` (a callback from
      `main.ts` returning `!scanRunning`) holds, it calls
      `event.stopPropagation()` (so the row's `open` does not run), then
      `slow.click(path, true, Date.now())`, clears any earlier timer and sets
      `setTimeout(() => { if (slow.due(path, Date.now())) startRename(path);
      }, SLOW_CLICK_DELAY)`. Otherwise it lets the row's `click` open the
      folder as today (the tracker is not consulted; the row becomes
      `current` through `reveal`). The `.name` span's `dblclick`, the
      container's existing `mousedown` listener (any button, any row) and
      `keydown` call `slow.cancel()`; a `mousedown` on the armed name itself
      is the second click's own `mousedown` and comes before its `click`, so
      the order arms after the cancel, as intended. A `startRename` reached
      through the menu item also cancels any pending arm.
    - Start: `folders.startRename(path)` sets `editing`, re-renders.
      Finish: one `finish(decision)` in `folders.ts` runs the `commit()`
      guard, clears `editing`, re-renders, and on `"confirm"` with a new name
      calls the `onRename(path, name)` callback added to `folders.init`;
      `main.ts` invokes `rename_folder` and calls `folders.renamed(...)` on
      success. The input's `blur` handler calls `finish("confirm")`; the
      guard makes the blur after Enter or Escape a no-op, and `finish`'s
      re-render removing the input does not re-enter it.
    - Keys: in `folders.keydown`, before the existing `escape` branch, if
      `editing` is set: `editKey(keyName(event))` — `"confirm"` /
      `"cancel"` call `preventDefault` and `finish`, `"native"` does
      nothing — and return `true` in every case. `main.ts` already returns
      when `folders.keydown` returns `true`, which is what suspends the
      keymap; `folders.hasFocus()` is true because the input is inside
      `#folders`. Add `folders.isEditing()` to `modalOpen()` so the Undo /
      Redo menu accelerators (`main.ts` `undo` / `redo` listeners) stay out
      too; `select-all` already selects a focused input's text.
    - Clicks: the input stops propagation of `mousedown`, `click` and
      `contextmenu` so the row's `open` and the container's right-click
      handling do not fire; the container's existing `blur` listener
      (resetting type-ahead) is harmless.
    - `folders.ts`'s `render` knows each row's `depth`; pass `depth === 0`
      through the existing `contextMenu(path, name, x, y)` callback as a
      fifth argument rather than a new callback, and update the `folders.init`
      call in `main.ts`; `init` also gains `onRename` and `canRename`.
    - Style the input in `crates/app/ui/style.css` to sit where the `.name`
      span sits (same font, no margin shift), so the row does not jump.
    - Do not re-list the parent from the backend to refresh the tree: the
      pure re-key keeps the expansion the todo asks for and avoids a listing
      round-trip; a later expand re-lists anyway.
    - `openDirectory` is the right path (reset, new token) rather than
      `resync`: the folder string changes, so everything keyed by the old
      strings is stale, and the judgments come back from the rewritten index
      rows. See the guide's "Two folder paths" section.

- [ ] Step 3: Backend file rename: sidecars along, index rows rewritten, `rename_file` command
  - Done when:
    - `rename.rs` gains a pure `file_plan(dir: &Path, path: &str, name: &str)
      -> Result<FilePlan, String>`: the RAW must be a RAW directly in `dir`
      (as `trash::plan` checks), `name` must pass `check_name` and
      `riffle_core::scan::is_raw_file`, and the plan lists `(from, to)` pairs
      for the RAW and for each existing sidecar of either format
      (`sidecar::existing_sidecar`, so a `FOO.XMP` spelling moves too), the
      sidecar targets minted by `SidecarFormat::sidecar_path` from the new
      RAW path. Any target that exists and is not the same entry as its
      source is a collision, reported before anything moves.
    - `file_run(plan, rename) -> Result<(), String>` renames the RAW first,
      then the sidecars; if a sidecar rename fails it renames what already
      moved back (best effort, logged) and returns the error, so a failure
      leaves the folder as it was.
    - `Index::rename_file(&mut self, old: &str, new: &str) -> Result<(), String>`
      rewrites `path` in `files` and `ratings` (their `dir` is unchanged) and
      `folders.last_viewed` where it equals `old`, in one transaction.
    - A `rename_file(app, dir: String, path: String, name: String) ->
      Result<Renamed, String>` command, registered in `main.rs`, with the
      same guard order as `rename_folder` (scan check, `file_plan`, flush the
      writer, `Scans` lock re-check, canonicalize `dir`, `file_run`,
      `rename_file` on the index). No watcher release: the watcher holds the
      directory, not its files. `Renamed.path` is the new file path in the
      canonical spelling `list_arw` uses (`canonical dir` joined with `name`),
      so it is directly comparable with `files`.
    - Tests in `rename.rs`: a RAW with both sidecars moves all three; a
      case-only rename is allowed; a name that is not a RAW, a path outside
      `dir`, and a colliding target are refused with nothing moved; a
      sidecar failure rolls the RAW back (inject the failure the way
      `trash.rs`'s `mover` test helper does, through the `rename` function
      `file_run` takes). Tests in `index.rs`: `rename_file` moves the
      `files` and `ratings` rows (dirty flag kept) and `last_viewed`.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (`check_name`, module, command shape).
    - The `.dop` sidecar name embeds the RAW's name (`a.ARW.dop`), so its
      target must be minted from the new RAW name, never by string-replacing
      the stem.
    - A sidecar file the writer is about to write for the old path cannot
      exist after the flush + lock, so no writer-side path remap is needed;
      any judgment made after the rename goes through `set_rating` with the
      new path.

- [ ] Step 4: File `Rename…` in the strip's context menu, the slow second click, and the inline cell editor
  - Done when:
    - The strip's right-click menu ends with a new group holding `Rename…`
      (`contextMenuGroups` in `context.ts`; `context.test.ts` covers it). It
      acts on the focused file (`files[index]`; a right-click always focuses
      the clicked cell), whatever else is selected.
    - A slow second click starts the same edit: a click on the name span of
      the cell that is already the focused file arms a rename; about 500 ms
      later the edit starts unless the click became a double-click, another
      click landed anywhere, or a key was pressed. A click on the thumbnail
      keeps today's behavior (it only collapses the selection to the file,
      which is already the focused one); a click on any other cell's name
      focuses it as today and does not arm. Nothing arms while a scan runs.
    - Starting the edit (either way) turns that cell's name into a text
      input prefilled with the file name, with only the stem selected
      (`stemLength(name)` in `rename.ts`, tested: `DSC01234.ARW` selects 8
      characters; a name with no dot selects all), focused. Enter confirms,
      Escape cancels, losing focus confirms, an empty or unchanged name just
      ends the edit; the backend still validates the extension through
      `is_raw_file`. While the edit is live no key reaches the culling
      keymap, the strip's menu handling or the tree; the input gets every
      key natively except Enter and Escape. A running scan is refused in the
      status line by the menu item, as for folders.
    - The edit survives the strip's own re-rendering: scrolling the strip
      keeps the editing cell in the DOM even outside the virtual range
      (`render` never releases it), and a repaint of that cell's badges
      (`setRating`, `setSharpness`, ...) leaves the input alone. `setFiles`
      arriving mid-edit (a `folder-changed` resync, a filter change) cancels
      the edit, since the cell's index may no longer name the same file, and
      disarms a pending slow click.
    - On success `main.ts` drops the old path from `ratings`, `flags`,
      `labels`, `sharpness`, `touched` and removes undo / redo batches that
      only reference it (as `trashRejected` does, `main.ts` around
      `history.removeWhere`), then calls `resync()` anchored on the new path,
      so the strip keeps its scroll, the renamed cell shows the new name and
      its rating / flag / label come back from the rewritten index rows.
      `resync` takes an optional `anchor` parameter for this; its default
      stays `files[index]`.
    - Errors show in the status line; the disk and the UI are unchanged (the
      cell shows its old name again).
    - `README.md`, `README.ja.md` and `docs/usage.md`'s filmstrip paragraph
      mention `Rename…`, the slow second click on the shown file's name,
      its keys, and that the sidecars move with the file.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 2 and 3 are merged (`rename.ts` with `InlineRename` and
      `SlowClick`, `rename_file`).
    - `strip.ts` keeps `let editing: InlineRename | null` and exports
      `startRename(index)`, `isEditing()` and `finishRename(decision)`;
      `init` gains `onRename(index, name)` and `canRename()` callbacks.
      `createCell` draws the input instead of the `.name` span when `index`
      is the editing index (store the index alongside, since `files[index]`
      is the path); `render` skips releasing the editing cell; `setFiles`
      calls `finishRename("cancel")` and `slow.cancel()` first. The input's
      `input` event stores the value into `editing`; its `blur` calls
      `finishRename("confirm")`; it stops propagation of `mousedown` /
      `click` / `contextmenu` so the cell's select and menu handlers do not
      run.
    - Slow click: `strip.ts` holds one `SlowClick` and one timer id, the
      same shape as `folders.ts`. The cell's `.name` span gets a `click`
      listener that, when `index === current` (the strip's own current
      index, kept by `setCurrent`) and `canRename()` holds, runs
      `slow.click(files[index], true, Date.now())` and sets the
      `SLOW_CLICK_DELAY` timer that asks `slow.due(...)` and then
      `startRename(index)`; the cell's ordinary click (select + focus)
      still runs, which is a no-op for the focused cell. The span's
      `dblclick`, the strip's `mousedown` (any cell, any button) and
      `main.ts`'s `keydown` (through an exported `strip.cancelSlowClick()`,
      called next to `folders`' cancel) disarm it. `startRename` from the
      menu item disarms too.
    - `main.ts` keydown: before the `contextMenu` branch (which closes the
      menu on any key), if `strip.isEditing()`: `editKey(key)` handles
      `"confirm"` / `"cancel"` with `preventDefault` and
      `strip.finishRename`, and returns in every case. Add
      `strip.isEditing()` to `modalOpen()` for the menu accelerators, as
      Step 2 did for the tree.
    - `finishRename` runs `commit()`, clears `editing`, re-creates the cell
      (release + `createCell`) and, on a confirmed new name, calls
      `onRename`; `main.ts` invokes `rename_file` with `dir: openDir`, the
      path and the name, guarding the post-invoke work with `dir !== openDir
      || token !== folderToken` as `trashRejected` does.
    - The cell's name is `baseName(files[index])`, so the new name appears
      once `resync`'s listing lands; the input's own styling in `style.css`
      matches `.cell .name` so the cell does not jump.

## Trade-offs and risks

- **Slow click on the tree's current row swallows the reopen.** Today a
  click on the open folder's row reopens it (a reset open); with the slow
  click, a click on its *name* arms a rename instead and does not reopen,
  while a click elsewhere on the row still reopens. The alternative, arming
  and reopening at once, would reset the strip under the user 500 ms before
  the edit starts, and `openDirectory`'s `reveal` re-render would have to
  preserve the arm. `File > Reload Folder` and the focus rescan cover the
  reopen need.
- **Slow-click timing.** 500 ms is a fixed constant, not the OS
  double-click interval (unreadable from the webview); a very slow
  double-clicker sees an edit start instead of a double-click, which the
  strip and tree have no other use for anyway. `SLOW_CLICK_DELAY` is one
  constant to tune.
- **Blur confirms (planned) vs cancels.** Explorer, Finder and VS Code
  confirm on focus loss; cancelling would throw away a typed name when the
  user clicks the strip to check the photo. The cost is that an accidental
  click elsewhere commits a half-typed name; the backend's `check_name` and
  collision checks still refuse a bad one, and a folder rename is reversible
  by renaming again. The `commit()` single-shot guard is what keeps Escape
  from being followed by a blur-confirm; the DOM-free tests must cover that
  ordering.
- **Re-render mid-edit.** `folders.ts` rebuilds every row on each `render`
  and `strip.ts` recreates cells on scroll, so both editors draw the input
  from module state rather than holding an element. The tree restores focus
  and selection after `replaceChildren`; if a webview drops the selection on
  re-focus it only costs the highlight, not the text. The strip instead pins
  the editing cell; `setFiles` cancels because indices can shift under it
  (the alternative, re-finding the path in the new `files`, is more code for
  a case the scan refusal already makes rare).
- **Running scan: refuse (planned) vs cancel-and-proceed.** The todo says
  "stops the scan". `trash_rejected` and `clear_index` refuse with
  `SCAN_RUNNING`, and `scan_folder`'s cancel-and-join is only reachable from
  a new scan. Refusing is one line and consistent; cancelling means taking
  `running`, setting the flag, awaiting the join handle and emitting
  `scan-state` from a non-scan command, which the `Scans` doc comment warns
  against interleaving. Refusing means a user has to wait for the faces pass
  to end on a big folder before renaming it.
- **File rename: fixed extension vs free name.** The plan lets the user edit
  the whole name (stem preselected) and has the backend require
  `is_raw_file`, so a typo in the extension is refused rather than silently
  making the file vanish from the strip. The stricter alternative, requiring
  the extension to be unchanged case-insensitively, forbids `.arw` →
  `.ARW`; the looser one, allowing any name, drops the file out of the RAW
  listing.
- **Index write after the disk rename (planned) vs before.** Disk first
  makes the file system the source of truth: a failed index write only costs
  a re-extraction on the next open, and the error text says so. Index first
  in a transaction, rolled back on a disk failure, keeps the two in step but
  turns an OS error (an open handle on Windows) into a rollback path that
  needs testing of its own.
- **Reopening the renamed open folder resets the session** (`openDirectory`:
  the undo history, `touched`, the selection and the compare view are
  cleared; the strip position comes back through `last_viewed`). Keeping
  them would mean rebasing every path-keyed map in `main.ts`; the index
  rows already carry every judgment, so nothing is lost, only undo. Noted in
  `docs/usage.md`.
- **Windows open handles.** After `release_under`, the only handles Riffle
  itself holds under the folder are transient (preview reads). Another
  program (PhotoLab, Explorer's preview pane) can still block the rename;
  that is the OS error the status line shows, and nothing is retried.
- **Case-only renames on Windows / macOS** go through `std::fs::rename`
  directly (NTFS and APFS accept them); the collision check compares
  canonical paths so it does not refuse them. On Linux (case-sensitive) a
  case-only rename is an ordinary rename.
- **Home as a rename target.** Roots (home, volumes) hide the item; a home
  reached through a volume root is a non-root row and could be renamed. The
  backend only refuses a path with no parent. Refusing the home directory
  explicitly is one `app.path().home_dir()` comparison if the caller wants
  it.

## Progress

- (none yet)
