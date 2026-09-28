# Learnings

## Step 1

- Measured `notify` 8.2 on Windows 11 (a scratch binary, `RecommendedWatcher`
  = `ReadDirectoryChangesWatcher`, `RecursiveMode::NonRecursive`):
  - Renaming a folder that is itself watched **succeeds**; renaming a folder
    with a watched **descendant** fails with `PermissionDenied` (os error 5).
    So the handle pins the watched folder's ancestors, not the folder itself.
    `rename.rs`'s doc comment said "Windows refuses to rename a watched folder
    or its parent"; it now says "a folder with a watched descendant".
  - A watch on the parent does not block renaming a child: `release_under`
    stays limited to the renamed folder and below, as planned.
  - With one watcher holding several folders, `unwatch` of a descendant
    releases the handle at once: the ancestor rename right after succeeded
    in 20/20 runs, about 0.3 ms after `unwatch` returned. The other watches on
    the same watcher kept delivering events.
  - Hence shape (a): one `RecommendedWatcher` for the whole tree, `watch` /
    `unwatch` per folder, the handler mapping an event path (its parent, else
    itself) to the tree key through a shared canonical-path map.
    `applying_a_set_adds_and_removes_watches` and the `release_under` test
    re-check the release with a real rename of the parent / the folder.
- Two tree paths can canonicalize to one folder (a symlink, `/tmp` vs
  `/private/tmp`), so the shared map holds a list of tree keys per canonical
  path and one OS watch serves them all; `unwatch` happens only when the last
  key goes. Otherwise a second `watch` on the same path would replace the
  first and the first key's `unwatch` would drop the watch the second still
  needs.
- The shared map's lock is never held across `watch` / `unwatch`, so the
  event handler (which takes it on the watcher's thread) cannot deadlock
  against a removal.
- Round 1 review: `release_under` and `restore` used to each take and release
  the `TreeWatch` lock separately, leaving a window between the release and
  `std::fs::rename` (and again after a failed rename) where a concurrent
  `set_tree_watches` could re-watch the path being renamed and make the
  rename fail with `PermissionDenied`. Replaced both with
  `treewatch::with_released`, which holds the `TreeWatch` lock for the whole
  release/rename/restore-on-failure sequence. Still deadlock-safe: the notify
  handler only ever takes the separate `keys` lock, never the `TreeWatch`
  state lock.

## Step 2

- The watched-set sync sits in `render()` before the inline-rename block,
  since that block returns early when the edited row vanished; placing it at
  the literal end would skip the sync on that path.
- The `tree-changed` listener checks membership with `watchedFolders(tree)`
  itself, so "expanded and drawn" has one definition for both the watched set
  and the drop rule. The re-list result is applied with `setChildren` even if
  the node was collapsed or re-keyed meanwhile: `setChildren` on an unknown
  key is a no-op and on a collapsed node only refreshes the cache.
- The Step 1 measurement (a watch pins the watched folder's ancestors, not
  the folder itself) was carried into `docs/usage.md`'s Folders paragraph and
  the plan's Trade-offs bullet, which had said "that folder or its
  ancestors". Only rename was measured, so the docs say "rename", not
  "rename or delete".
- **Manual Windows GUI check: pending, to be run by the user** (the agent
  cannot drive the GUI). Checks:
  1. With a folder expanded in the tree, create a subfolder in Explorer: it
     appears within about a second. Rename it in Explorer: the row follows.
     Delete it: the row goes.
  2. Run `Sequence JPEG Timestamps…` on a folder whose parent is expanded:
     `<folder>-sequenced` appears under the parent.
  3. Rename an expanded folder from the tree (`Rename…`), and a folder whose
     ancestor is expanded: the rename succeeds, and creating a subfolder
     under the renamed folder in Explorer afterwards still shows up.
  4. Make a tree rename fail (a sibling with the target name exists): the
     error shows, and creating a subfolder under the old folder still shows
     up.
  5. Start an inline rename on a folder, then create a sibling folder in
     Explorer: the text box survives the re-list with its text and
     selection.
  6. Permanently delete an expanded folder itself (`rm -rf` on Linux/macOS,
     Shift+Delete in Explorer): no error is shown, and the row disappears
     once the parent's re-list lands.
