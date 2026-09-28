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

# Follow file system changes in the folder tree

## Purpose

The folder tree re-reads a folder's children only when it is expanded
(`toggle` in `crates/app/ui/src/folders.ts`), so a subfolder created,
deleted or renamed while its parent is already expanded — by Explorer or
Finder, by another app, or by Riffle itself (`Sequence JPEG Timestamps…`
writes `<folder>-sequenced` next to the picked folder) — does not show up
until the user collapses and re-expands the parent. The user hit this on
Windows: "a new folder created while a folder is open does not appear in the
tree".

This makes the tree follow the disk: every expanded, visible node holds a
non-recursive watcher on its own folder, a burst of events under it is
debounced in Rust into one `tree-changed` event, and the frontend re-lists
that node (children and RAW count) in place, keeping the expansion of
everything under it. Collapsing a node releases its watcher. `rename_folder`
releases the tree watchers under the renamed folder before the disk rename
(Windows refuses to rename a directory with an open `ReadDirectoryChangesW`
handle), and puts them back when the rename fails.

Roots: the root *nodes* (home, `C:\`, `/Volumes/x`) are ordinary tree nodes
and are watched like any other node while expanded, so a folder created
directly under home appears. The *list of roots* (which volumes are mounted)
stays read once at launch: mount / unmount following is a separate concern
(`docs/usage.md` already documents the restart), and `notify` cannot watch
"the set of drive letters" anyway.

The `todo.md` item `### App: Refresh vs Rescan from the folder tree's context
menu` is affected: live following removes the main reason for a manual
Refresh of the tree, and the open folder's re-index already has the focus
rescan and `File > Reload Folder`. Decided with the user: Step 2 narrows that
item to a manual Refresh as the fallback for a folder whose watch failed.

## Steps

- [x] Step 1: Backend tree watchers: `set_tree_watches` command, per-folder debounced `tree-changed`, release / restore around `rename_folder`
  - Done when:
    - A new module `crates/app/src/treewatch.rs` holds a managed state
      `TreeWatch` (registered in `main.rs`'s `setup` next to
      `watch::Watch::spawn`) that maps each watched tree path (the
      frontend's own spelling, the tree's node key) to its canonical path
      and its watch. Watches are `RecursiveMode::NonRecursive`.
    - A Tauri command `set_tree_watches(app, dirs: Vec<String>) -> Result<(), String>`
      (registered in `generate_handler!`) makes the watched set equal to
      `dirs`: paths no longer in `dirs` are unwatched (their handles
      released), new ones are watched. A path that cannot be watched (a
      network share, a folder that vanished between the listing and the
      call) is `log::warn!`ed and skipped, never an error: the tree must
      still work, and the expand-time re-list is the fallback. The command
      is `async` and does its work in `spawn_blocking`
      (`docs/agents/tauri-app.md`, "Synchronous commands run on the main
      thread").
    - An event under a watched folder that `watch::triggers` accepts (make
      `triggers` `pub(crate)`; it drops events whose paths are all sidecars
      or sidecar write temporaries, so culling in a folder that is also
      expanded does not re-list it) is debounced **per tree path**: one
      `tree-changed { dir }` event is emitted 500 ms after the last event
      for that path, and two folders changing at once each get their own
      event (unlike `watch::run`, whose single pending slot is right for
      the one open folder but would drop a sibling's burst here). `dir` is
      the tree path spelling, not the canonical one, so the frontend can
      key the node directly.
    - `treewatch::release_under(app, canonical_dir: &str) -> Vec<String>`
      drops every watch whose canonical path is `canonical_dir` or under it
      and returns their tree paths; `treewatch::restore(app, dirs: Vec<String>)`
      watches them again. `rename_folder` in `crates/app/src/rename.rs`
      calls `release_under` right after `watch::release_under(&app, &old)`
      and, in the same `if let Err(e) = std::fs::rename(...)` branch that
      restores the open-folder watch, calls `restore` with what it
      released. On success nothing is restored: the frontend's `renamed()`
      re-render sends the new set through `set_tree_watches` (Step 2).
      Watches on the renamed folder's *ancestors* are not released: a
      handle on the parent does not block renaming a child (verify on
      Windows during implementation; if it does, widen `release_under`).
    - Tests in `treewatch.rs` on temp dirs (naming
      `riffle-treewatch-<name>-<pid>`, as `watch.rs` and `rename.rs` do):
      applying a set adds and removes watches (the map's keys equal the
      set; a removed path's watch is gone); `release_under` releases the
      folder itself and a watched descendant, keeps a sibling sharing the
      name prefix (`photos` vs `photos2`), and returns the released tree
      paths; `restore` puts them back; the per-path debouncer collapses a
      burst on one path into one event and keeps two paths' bursts apart
      (same shape as `a_burst_is_collapsed_into_one_event`).
    - `CLAUDE.md`'s Layout paragraph names `src/treewatch.rs` next to the
      `src/watch.rs` description (currently only `watch.rs` is implied by
      `rename.rs`'s description; add both in one short clause).
    - `mise run ci` passes (`cargo test treewatch` for the new tests;
      `riffle-app` is bin-only, so no `--lib`).
  - Implementation approach:
    - Keep `watch.rs` untouched except for `pub(crate) fn triggers`. The
      open-folder watcher has different semantics (one folder, owner string,
      set from inside `scan_folder`) and merging the two would widen a
      module that is already correct.
    - Two shapes to decide between during implementation, after checking
      `notify` 8.2's Windows behavior:
      (a) one `RecommendedWatcher` for the whole tree, `watch(path)` /
      `unwatch(path)` per folder, and the event handler mapping an event
      path to its tree key through a shared `Arc<Mutex<HashMap<PathBuf,
      String>>>` (look up `event.path.parent()`, then the path itself for
      an event on the folder proper); or (b) one `RecommendedWatcher` per
      folder, each handler capturing its own tree key (the shape `watch::set`
      uses). (a) costs one OS watcher thread total and gives a simpler
      release (drop the map entry, `unwatch`); (b) is simpler to reason
      about per entry and a failure to watch one folder is isolated by
      construction. Prefer (a) unless `unwatch` on Windows turns out not to
      release the directory handle promptly (measure: after `unwatch`,
      `std::fs::rename` of that folder must succeed); then use (b), whose
      `Drop` provably releases it (this is what `watch::set` relies on).
      Record the choice and the measurement in `learnings.md`.
    - Canonicalize each path when it is added, and compare canonically in
      `release_under` (`Path::starts_with`), as `watch::release` does; the
      frontend's spelling (`C:\Users\...`) and the canonical one
      (`\\?\C:\Users\...` on Windows, a resolved symlink on macOS) differ.
    - Do not filter to directory events: `notify` reports `CreateKind::Any`
      on Windows and a removed path cannot be stat'ed, so a file event under
      an expanded folder costs one `read_dir` per debounce window, which
      also keeps the RAW count current. `triggers` is the only filter.
    - Mirror the sender / debounce-thread layout of `watch.rs` (`spawn`,
      `run(rx, emit)` with a `HashMap<String, Instant>` of deadlines, waiting
      on the nearest one).

- [x] Step 2: The tree syncs its watched set and re-lists on `tree-changed`; docs and `todo.md`
  - Done when:
    - `crates/app/ui/src/tree.ts` exports a pure
      `watchedFolders(tree: Tree): string[]`: the paths of the nodes that
      are expanded **and drawn** (walk `rows(tree)`, take `node.expanded`),
      sorted, so a node left `expanded: true` under a collapsed parent is
      not watched. `tree.test.ts` covers: a collapsed tree watches nothing;
      an expanded root is watched; an expanded child under an expanded
      parent is watched; an expanded child under a *collapsed* parent is
      not; an expanded node whose listing has not landed yet
      (`children === undefined`) is watched; the result is stable
      (sorted).
    - `folders.ts` calls `set_tree_watches` with `watchedFolders(tree)` at
      the end of `render()` whenever the list differs from the last one
      sent (compare the joined string, updated synchronously at send time),
      swallowing and reporting a rejection through `reportError`. Because
      `render` is the one place every tree change passes through (`toggle`,
      `reveal`, `renamed`, `loadRoots`, `moveCursor`), this covers expand,
      collapse, the reveal chain at launch, and the re-key after a rename
      (which is what restores the watches under the new path after
      `rename_folder` released them).
    - `folders.ts` listens to `tree-changed` (a module-level
      `window.__TAURI__.event.listen`, registered once, like
      `folder-changed` in `main.ts`): if `payload.dir` is a node that is
      still expanded and drawn, it re-lists it with the existing `list()`
      and applies `setChildren` (which keeps the state of children already
      known), then `requestRender()` (not `render()`, so a re-list landing
      mid-press waits for the `mouseup`, as `renamed()` does). A listing
      error is reported through `reportError` and does not collapse the
      node (unlike `toggle`, where the user asked for the expand). A
      payload for a node that is no longer expanded or drawn is dropped
      (the collapse's `set_tree_watches` and an in-flight event can cross).
    - A live inline rename is preserved across the re-list (`render`
      already rebuilds the input from `editing`, and drops the edit when
      the row vanished); nothing extra is needed but it is checked
      manually.
    - The comment above `toggle` ("Expanding always re-lists ...") and
      `docs/usage.md`'s Folders paragraph ("A folder's arrow lists its
      subfolders, again on every expand, so a subfolder created since shows
      up") are updated to say that an expanded folder follows the disk: a
      subfolder created, deleted or renamed under it appears within about a
      second. `README.md` / `README.ja.md` need no change unless a sentence
      is added to the Folder tree bullet; if one is, both change in the
      same PR.
    - `todo.md`'s `### App: Refresh vs Rescan from the folder tree's context
      menu` is narrowed to a manual Refresh as the fallback for a folder
      whose watch failed (`log::warn!`ed network shares, permissions), with
      the "Refresh re-lists the subfolders" motivation rewritten to point at
      this plan (the tree otherwise follows the disk, and the open folder's
      re-index has the focus rescan and `File > Reload Folder`).
    - Manual check on Windows (record in `learnings.md`): with a folder
      expanded, `mkdir` a subfolder in Explorer, rename it, delete it; run
      `Sequence JPEG Timestamps…` on a folder whose parent is expanded and
      see `<folder>-sequenced` appear; rename an expanded folder from the
      tree (and a folder whose ancestor is expanded) and confirm the rename
      succeeds and the tree keeps following under the new name; make a
      rename fail (target exists) and confirm the tree still follows the
      old folder.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (`set_tree_watches`, `tree-changed`).
    - No new module: the sync and the listener are a few lines in
      `folders.ts`; the testable decision (which nodes to watch) goes into
      `tree.ts`, which is the module `folders.ts` already keeps pure and
      tested, since no frontend test mocks `window.__TAURI__`.
    - `setChildren` leaves a vanished child's `TreeNode` in `tree.nodes`;
      it is not drawn (`rows` walks the `children` lists) and `step` /
      `typeAhead` work on `rows`, so no cleanup is needed. A cursor on a
      vanished row lands on the first row at the next arrow key, as today
      after a collapse.
    - Nothing in `main.ts` changes: the strip's own following stays on
      `folder-changed` / `resync`, and the open folder being deleted or
      renamed externally is out of scope (as today).

## Trade-offs and risks

- **Watching an expanded folder pins it on Windows (accepted by the user).**
  `ReadDirectoryChangesW` holds a handle, so while a folder is expanded in
  the tree Explorer (or another app) cannot rename its *ancestors*; the
  folder itself and the folders *under* it stay free (measured in Step 1, see
  `learnings.md`). Today only the open folder's ancestors are pinned. The tree's own `Rename…` is unaffected (Step 1 releases first).
  The alternatives (watching only the open folder's parent, or polling) were
  rejected: the first misses the reported case, the second costs a
  `read_dir` per expanded node per tick on a network share.
- **One watcher with `watch` / `unwatch` (a) vs one watcher per folder (b).**
  See Step 1. The decision is deferred to the step because it hinges on
  whether `notify`'s Windows `unwatch` releases the handle at once, which
  is only measurable there.
- **The todo item (decided: narrow).** Kept as a Refresh fallback for
  folders whose watch failed (SMB, a folder under a path `notify` refuses):
  those users get no live following and no visible sign of it, since the
  failure is only logged. Step 2 edits it directly, since the user asked for
  the update to be part of this work.
- **Two PRs (decided).** Step 1 is not user-visible alone (the command
  exists, nothing calls it) but is fully testable; the split keeps the Rust
  review (watch lifetimes, rename ordering) apart from the UI one.
- **"Expanded and drawn" vs "expanded".** `collapse` leaves descendants
  `expanded: true`, so watching every expanded node would keep handles on
  folders the user cannot see. Watching only drawn nodes means re-expanding
  a parent shows the child's cached children until the child's own watch
  (set by the same render) catches up; `toggle` re-lists on expand anyway,
  so the parent is fresh and the child is at most one debounce window
  stale.
- **File events re-list the folder.** Copying 500 RAW files into an
  expanded folder triggers one `read_dir` per 500 ms of copying. Cheap
  locally; on a network share the same cost the expand already pays.
  Sidecar writes are filtered by `triggers`.
- **Event / collapse race.** A `tree-changed` for a node the user just
  collapsed is dropped by the frontend check, and a collapse's
  `set_tree_watches` in flight while an expand re-adds the same node ends
  in the last set sent winning, which is the current tree. The "last one
  sent" comparison in `render` must be updated *before* the invoke resolves
  (synchronously at send time), or two renders in a row send twice; harmless
  but noisy.
- **A watch that cannot be set** (network volume, permissions) is logged
  and the node behaves as today (re-list on expand). No UI signal is
  planned; the narrowed todo item covers it.
- **`release_under` scope.** Only the renamed folder and its descendants
  are released; if a Windows test shows a parent handle blocks the child's
  rename, `release_under` widens to ancestors too and the frontend re-sync
  after `renamed()` still restores them, since the set is recomputed from
  the tree.

## Progress

- Step 1: one `RecommendedWatcher` shared by the whole tree, `watch` /
  `unwatch` per folder, shape (a) chosen after measuring on Windows that
  `unwatch` releases the handle at once and that a watched *descendant*, not
  the watched folder itself, is what blocks a rename. See `learnings.md` for
  the measurements and the deadlock-safety argument.
- (2026-09-28) Step 1 complete
