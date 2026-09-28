# Learnings

## Step 1: Backend reject collection over folders

- `trash::collect` takes the index rows through a closure
  (`Fn(&str) -> Result<HashMap<String, RowFlag>, String>`), so its tests
  feed hand-built rows and never open SQLite; `Index::row_flags` has its own
  test in `index.rs`.
- The flag rule mirrors `reconcile_sidecars` one step further than the plan
  spelled out: a clean row whose sidecar is gone is not a reject when its
  stored stat is set (the folder open would clear it), but is trusted when
  its stat is `(None, None)` (the folder open keeps such a row as it is).
- A dirty row wins even over a sidecar that changed, unlike the folder open
  (where the sidecar wins). The writer is drained before each collection, so
  a dirty row left at that point is a write that failed; the plan chose to
  trust the judgment the user made in the app.
- `trash_rejected` collects twice: once before the native dialog (for its
  count, without the `Scans` lock) and again under the lock for the move, as
  Step 4's preview / run split will. The per-folder `FolderPlan::dir` is only
  logged for now; the per-folder summary for the frontend is left to Step 4.
- `Summary.trashed` was replaced by `Summary.moved` (the RAW paths); the
  count is its length. Collection failures (an unreadable folder, a sidecar
  that does not parse, an index query error) go in `Summary.unread`, kept
  apart from `Summary.failed` (a move that was attempted and failed): nothing
  was tried on an unread path, so it must not read as a failed move or count
  toward `trashedStatus`'s "N failed".
- The flag `collect_folder` uses is decided from the configured
  `SidecarFormat`'s kinds only, the same ones the folder open reads; `Both`'s
  kinds are still gathered to know what sidecars move with a reject, but a
  leftover sidecar of the other format (e.g. a stale `.dop` from an earlier
  `Both` setting, or one PhotoLab keeps rewriting) never overrides the
  configured format's flag.
- The subfolder walk uses `DirEntry::file_type().is_dir()`, which is false for
  a symlink (and for a Windows junction, which std reports as a symlink), so a
  link is never followed; `folders::is_hidden` became `pub(crate)` for it.
- Windows check (on this machine, Windows 11): a throwaway test (not
  committed) put a `notify::recommended_watcher` `NonRecursive` watch on a
  canonical (`\?\`-prefixed) temp dir, as `watch.rs` does for the open
  folder, and ran `trash::run` with the real `TrashContext::default()` over a
  RAW and its `.xmp` inside it. Both went to the Recycle Bin with no failure,
  so trashing the rejects of the open (watched) folder needs no watcher
  change, and a folder that is not open has no watch at all. Contrary to the
  plan's note, renaming the watched directory itself also succeeded in that
  run (notify's `ReadDirectoryChangesW` handle is opened with
  `FILE_SHARE_DELETE`); `rename.rs` still releases the watcher first, which
  is harmless. The GUI check (the File menu item on an open folder, the
  status line and the refreshed strip) remains a manual check for the user.

## Step 2: Folder tree right-click items

- `folderMenuGroups`' second parameter changed from `canRename` to `root`,
  since it now gates both `Rename…` and the recursive trash item; the only
  caller already had `root` at hand.
- `tree.ts` exports `relation(path, dir)` (`"same"` / `"under"` / `null`),
  comparing the way `ancestorsWithin` / `rebase` do; `trash.ts`'s
  `opensTarget` builds on it. `rebase` was left as it is rather than rewritten
  over `relation`.
- `trashRejectedIn` decides whether to prune and `resync()` from the open
  folder at the time the command returns, not the one when it started, so
  the old `folderToken` guard is gone: the pruning only touches the moved
  paths, and a folder opened mid-run that turns out to be a target is
  re-listed. The status line and the failures are now always shown, even
  when the user switched folders meanwhile (the File menu path used to drop
  them).

## Step 3: Multi-selection in the folder tree

- The selection model lives in `tree.ts` (`TreeSelection`, `selectOnly`,
  `clickSelect`, `pruneSelection`) rather than reusing `selection.ts`: the
  strip's model always keeps the focused file selected, while the tree's has
  no such member (the open folder can be toggled off, and the selection can
  go empty). The transitions mirror `click` otherwise.
- Pruning runs at the top of `folders.ts`'s `render()` against the drawn
  rows, so a collapse, a `tree-changed` re-list and a reveal all drop what is
  no longer drawn in one place. `renamed()` rebases the selected paths
  first, or the renamed folder would fall out of the selection.
- The selection is reset to the opened folder in `reveal()`, which every
  open goes through (tree click, Enter, drop, File > Open, the reopen at
  launch), once at its start (the raw spelling) and again at its end (the
  tree's spelling of the chain), so `aria-selected` follows the open folder.
- The toggle modifier is `metaKey` on macOS and `ctrlKey` elsewhere (the
  strip accepts either on every platform; the tree cannot, since macOS
  Ctrl+click is the right-click `folders.ts` already guards). A Shift+click
  `mousedown` is `preventDefault`ed so the webview does not select the rows'
  text, and focuses the container by hand to keep "a click gives the tree the
  keyboard".
- `folderMenuGroups` gained a `count` parameter (default 1); with more than
  one folder it returns only the trash group, and `root` then means "any
  selected folder is a root". The context-menu callback gained a `targets`
  argument (the selection in drawn order) that `main.ts` passes to
  `trashRejectedIn`.

## Step 4: Confirmation dialog

- `trash_rejected_preview` still returns the Step 1 `Err` ("No rejected
  files in …") when nothing is rejected and nothing failed to read, so the
  plain empty case stays a status line; the dialog opens with zero rejects
  only when some folder or file could not be read, and then lists those with
  `Move to Trash` disabled.
- The preview's bytes are summed per folder by `trash::preview` from
  `std::fs::metadata` on each RAW and each sidecar in its group; the size
  text is formatted by the command (`format_bytes(bytes, SIZE_BASE)`) and
  passed in as a closure, so `trash.rs` does not depend on `commands.rs`'s
  formatter and its test can pass a trivial one.
- The canonical dirs the backend returns carry the verbatim `\\?\` prefix on
  Windows; `trash.ts`'s `shownPath` strips it (and turns `\\?\UNC\` back
  into `\\`) for the dialog rows.
- `TrashFlow` is a smaller `SequenceFlow`: no run id or early event, since
  `trash_rejected_run` returns the summary itself. A running move cannot be
  canceled, so both buttons are disabled while it runs and Escape does
  nothing; the Tab trap returns early when no button is enabled.
- The dialog, the sequence dialog and the settings modal exclude each other
  (`trashFlow.busy` in `startSequence` and the `open-settings` guard, and
  `sequenceFlow.busy` / `settings.isOpen` in `trashRejectedIn`), since the
  window's keydown handler routes to only one of them.
- The run is not deferred through `whenIdle` (a folder switch would discard
  it and leave the dialog open in `running`); a scan that started while the
  dialog was up is refused by the backend under the `Scans` lock and the
  error shows in the status line.
- Bash heredocs through the agent's Bash tool on this Windows machine halve
  doubled backslashes even with a quoted delimiter (`"\\\\?"` lands as
  `"\\?"`); write text holding backslashes with the Write / Edit tools, or
  a script file written by them, instead.

## Deferred issues (todo candidates)

- **Bash heredocs on this Windows machine mangle doubled backslashes**
  (candidate for promotion into `CLAUDE.md`, or an agent-tooling note).
  - Basis: in Step 4, text holding a doubled backslash written through a
    Bash-tool heredoc, even with a quoted delimiter, landed with the
    backslashes halved (see the Step 4 note above).
  - Change: add a rule telling agents on Windows to write text holding
    backslashes (Windows paths, verbatim `\\?\` prefixes, regex escapes) with
    the Write / Edit tools, or a script file written by them, not a heredoc.
  - Done when: the rule is where future sessions read it before editing, and
    writing a doubled-backslash string both ways and diffing the results
    confirms the behavior it describes.
