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

# Move Rejected to Trash from the folder tree

## Purpose

When the SSD is nearly full the user wants to clean up in bulk: every reject
under a year folder, or across a handful of shoot folders, in one go. Today
the only way is `File > Move Rejected to Trash…`, which acts on the one open
folder (its label does not say so, which reads as if it targeted every file),
and which relies on the frontend's in-memory flags, so a folder has to be
opened first.

This moves the collection of rejects to the backend (the index's `ratings`
rows where they exist, else the `.xmp` / `.dop` sidecars on disk, so folders
never opened count too), adds the action to the folder tree's right-click menu
with an "including subfolders" variant, lets the tree select several folders
with Cmd/Ctrl / Shift + click and trash them together, and confirms with a
dialog that lists the count per folder and the total space freed. The File
menu item stays, relabeled so its target is unambiguous.

## Steps

- [x] Step 1: Backend reject collection over folders, and `trash_rejected(dirs, recursive)`
  - Done when:
    - `crates/app/src/trash.rs` collects the rejects of a list of folders
      itself, optionally recursing into subfolders, and returns them grouped
      per folder (`Vec<FolderPlan { dir, groups: Vec<Group> }>`, in a stable
      order: the given folders in the given order, each followed by its
      subfolders depth-first sorted by name). A folder that cannot be read
      is reported as a failure of that folder (its path, the error) and does
      not abort the others.
    - For every RAW listed in a folder the flag comes from, in this order:
      the index row when it is dirty (a judgment the writer has not flushed;
      the writer is drained first so this is rare) or when its stored stat
      (`xmp_size`, `xmp_mtime_ns`) equals the stat of the newest sidecar on
      disk (the same rule the folder open applies in
      `reconcile_sidecars_of`); otherwise the newest existing sidecar,
      parsed with `SidecarFormat::read_flag` (`xmpDM:good` / `ShouldProcess`);
      a file with neither row nor sidecar is not a reject. A sidecar that
      fails to parse is a failure of that file, never a reject. JPEG-only
      folders yield nothing.
    - `Index` gains a read accessor for the rows of one dir (path, flag,
      stored sidecar stat, dirty) so the collection needs one query per
      folder through the `ratings_dir` index.
    - The command becomes `trash_rejected(dirs: Vec<String>, recursive: bool)`,
      keeps the native confirmation dialog for now (total count in the
      message), keeps the existing guards (a running scan refused before the
      dialog and again under the `Scans` lock, the sidecar writer drained
      first, the `Scans` lock held across the collection and the moves), and
      returns `Summary` extended with the RAW paths actually moved (the
      frontend no longer knows the list up front) and the per-file /
      per-folder failures. No rejects anywhere yields a distinct `Err`
      string the status line shows ("No rejected files in …").
    - The File menu item is relabeled `Move Rejected in This Folder to Trash…`
      and `main.ts`'s `trashRejected` calls the new command with
      `[openDir]`, `recursive: false`, and prunes its maps and undo history
      from the summary's moved paths instead of a precomputed list;
      `rejectedPaths` is removed if nothing else uses it.
    - Rust unit tests: a folder with index rows only, sidecars only (XMP and
      `.dop`), a stale clean row that the sidecar overrides, a dirty row that
      wins, recursion that finds a nested reject and skips a dot-folder, and
      an unreadable folder reported without aborting the rest. Existing
      `plan`/`run` tests are kept or adapted. TS tests for the adapted
      status/summary handling.
    - `docs/usage.md`, `README.md`, `README.ja.md` name the new label;
      `CLAUDE.md`'s layout paragraph describes `trash.rs` (currently not
      listed) with its collection.
    - The Windows check is done on this machine and recorded in
      `learnings.md`: trashing rejects of the open (watched) folder still
      succeeds, and trashing in a folder that is not open needs no watcher
      change. `crates/app/src/watch.rs` is not modified.
    - `mise run ci` passes.
  - Implementation approach:
    - Canonicalize each given dir with `commands::canonicalize` before
      querying: the index keys `dir` by that spelling (a verbatim `\\?\`
      prefix on Windows). Subfolders found by `read_dir` from a canonical
      parent inherit the spelling.
    - Listing: reuse `read_listing(dir, Some(SidecarFormat::Both))` and
      `stat_sidecars` from `commands.rs` (make them `pub(crate)` or move the
      pair next to the collector), then `crate::sidecar::newest` to pick the
      effective sidecar per RAW, as `reconcile_sidecars_of` does. Cap parsed
      sidecars with the existing `MAX_SIDECAR_BYTES`.
    - Subfolder walk: same visibility rules as `folders::list` (`kind`,
      `is_hidden`, dot prefix); do not follow symlinked directories (cycle
      guard); make the helpers `pub(crate)` rather than duplicating them.
    - Keep the collector pure for tests by taking the index rows as an
      argument (`&HashMap<String, RowFlag>` per folder, fetched by the
      command under the lock) or a closure `Fn(&str) -> Result<Vec<..>>`;
      `Index` tests already build an in-memory database, so either shape is
      testable.
    - The moved-path list in `Summary` is what the frontend prunes by; the
      per-folder structure of the summary is only needed for the Step 4
      dialog's result line, so add it now or in Step 4, whichever keeps the
      diff smaller.
    - Do not touch `crates/app/src/watch.rs` (another session is adding
      per-tree-node watchers there). The `trash` crate moves files under a
      watched directory without releasing it; Windows only refuses to
      rename or delete the watched directory itself.

- [x] Step 2: Folder tree right-click items, this folder and including subfolders
  - Done when:
    - `folderMenuGroups` (`crates/app/ui/src/context.ts`) gains a group with
      `trashRejected` ("Move Rejected to Trash…") and, when the folder is
      not a tree root, `trashRejectedTree` ("Move Rejected to Trash,
      Including Subfolders…"). Placed after the rename group, before the
      sequence group, separated by `<hr>`.
    - `main.ts` dispatches both to `trash_rejected` with the right-clicked
      folder and `recursive`, refusing while `scanRunning` with the same
      status text the rename item uses; the File menu path and the tree
      path share one function.
    - When the open folder is one of the targets (equal to a given dir, or
      under one given with `recursive`; compare with `tree.ts`'s
      normalization, not string equality), the post-summary handling of
      Step 1 runs (prune the maps and the undo history by the moved paths,
      `resync()`); otherwise only the status line is set and the errors
      listed.
    - TS tests: `folderMenuGroups` holds the two items in that position for
      a non-root and only the first for a root; the "open folder among the
      targets" predicate for equal, nested-with-recursive, nested-without,
      and unrelated paths.
    - `README.md`, `README.ja.md` (folder tree bullet and the trash bullet)
      and `docs/usage.md` describe the right-click items.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - The "is the open folder a target" predicate belongs in a pure module
      (`trash.ts`) so it is testable; reuse `tree.ts`'s path normalization
      (`rebase` / `ancestorsWithin` compare the way the tree keys nodes) —
      export a small helper rather than re-implementing it.
    - The frontend cannot know whether the folder holds rejects, so the
      items are always offered; an empty result surfaces as the Step 1
      status message.

- [ ] Step 3: Multi-selection in the folder tree, and trashing the selected folders together
  - Done when:
    - `tree.ts` gains a pure selection state alongside the cursor
      (`selected: ReadonlySet<string>`, `anchor: string | null`) and the
      transitions: a plain click selects only that folder (and opens it, as
      today); Cmd+click (macOS) / Ctrl+click (elsewhere) toggles a folder
      without opening it; Shift+click selects the visible rows between the
      anchor and the clicked row (over `rows(tree)`, the drawn order);
      collapsing a parent or a re-listing that drops a folder removes it
      from the selection; opening a folder by any other route (drop, File >
      Open, the tree keyboard's Enter) resets the selection to it.
    - `folders.ts` draws `.selected` rows (a distinct style from `.current`,
      the open folder, and from `.cursor`), sets `aria-multiselectable` on
      `#folders` and `aria-selected` from the selection, and passes the
      selection to the context-menu callback. Right-clicking a selected
      folder targets the whole selection; right-clicking an unselected one
      first resets the selection to it.
    - With more than one folder selected the menu offers only the two trash
      items, labeled with the count ("Move Rejected in 3 Folders to
      Trash…", "… Including Subfolders…"; the recursive item is omitted
      when any selected folder is a root); they call `trash_rejected` with
      every selected path.
    - TS tests (`tree.test.ts`, `context.test.ts`): toggle, range over drawn
      rows, reset on plain click, pruning on collapse, and the menu labels
      for one and several folders.
    - `README.md`, `README.ja.md`, `docs/usage.md` describe the selection
      and its modifiers.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 2 is merged.
    - Mirror `crates/app/ui/src/selection.ts` (`click(selection, files,
      focused, at, {toggle, range})`) rather than inventing a second model;
      the strip decides the toggle modifier from the platform (`metaKey` on
      macOS, `ctrlKey` elsewhere) — do the same, and note the macOS
      Ctrl+click-is-right-click guard already in `folders.ts`.
    - A Shift+click must not start the slow-click rename and must not open
      the folder; a Cmd/Ctrl+click likewise. The plain click path is left
      as it is.
    - Keyboard range selection (Shift+Arrow) is out of scope; say so in
      `docs/usage.md`'s keys table only if the reviewer asks.

- [ ] Step 4: Confirmation dialog with per-folder counts and the total space freed
  - Done when:
    - The backend splits into `trash_rejected_preview(dirs, recursive)`
      (drains the writer, refuses a running scan, collects, and returns per
      folder: `dir`, reject count, bytes = sum of RAW and existing sidecar
      sizes; plus `total_files`, `total_bytes` and a `size_text` formatted
      with `format_bytes(bytes, SIZE_BASE)`) and `trash_rejected_run(dirs,
      recursive)` (re-collects under the `Scans` lock and moves, no native
      dialog). The old single command is removed.
    - An HTML dialog (like `#sequence-dialog`: `role="dialog"`,
      `aria-modal`, Escape and a Tab trap through `SettingsModal`'s key
      decisions, closed context/filter/sort menus) lists one row per folder
      with its count, hides folders with zero rejects behind a summary line
      ("12 more folders with no rejects"), shows the total ("Move 148
      rejected files (23.4 GB) to the Trash?"), and has `Move to Trash` /
      `Cancel` buttons. Folders that failed to collect are listed with
      their error and excluded from the run.
    - After the run the dialog closes, the status line shows the Step 1
      text, the errors join the sticky list, and the open-folder refresh of
      Step 2 runs when it applies.
    - Rust tests: the byte sum counts the RAW and both sidecars and skips a
      missing file; the preview's per-folder counts match the collection.
      TS tests (`trash.ts`): the row text, the zero-folder summary line, the
      total line singular/plural, and the disabled state of the run button
      when the total is zero.
    - `docs/usage.md`, `README.md`, `README.ja.md` describe the dialog's
      contents.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 3 is merged.
    - Follow `sequence.ts` / `SequenceFlow` for the phase gate (a second
      open while one is under way does nothing) and `main.ts`'s
      `showSequencePreview` for wiring; keep the text builders pure in
      `trash.ts`.
    - The preview does not hold the `Scans` lock while the dialog is open;
      the run re-checks `scanning()` under the lock as the old command did.
    - Sizes come from `std::fs::metadata(..).len()` on the RAW and each
      existing sidecar at collection time; `files.size` in the index is
      not used (it can be stale and is missing for unindexed folders).

## Trade-offs and risks

- **Keep or remove the File menu item.** Kept and relabeled ("Move Rejected
  in This Folder to Trash…"): it stays reachable when the tree pane is
  hidden (`F7`) and through the menu's keyboard access, and it now shares
  the tree's code path. Removing it would shrink the UI but leave the tree
  as the only entry point.
- **Clean index row vs sidecar.** The plan trusts a clean row only when its
  stored stat matches the sidecar on disk (the folder-open rule). Trusting
  every row would be cheaper (no stat per sidecar) but a Lightroom edit made
  since the folder was last opened would be missed, in either direction
  (trashing a file un-rejected elsewhere is the bad case).
- **Entry point for "including subfolders".** Two menu items rather than a
  checkbox in the dialog, because Steps 2 and 3 run with the native dialog.
  Once Step 4 lands a checkbox that re-previews would be the nicer UI; it
  can replace the second item later. The recursive item is hidden on a root
  (home / a volume) so a slip cannot walk a whole drive.
- **Run re-collects instead of taking the previewed paths.** Simpler and
  one source of truth; the moved count can differ from the previewed one
  only if the disk changed between the two, and the summary reports what
  actually happened. The alternative (pass paths, validate each is under a
  target folder, as today's `plan` does) pins the run to what was shown.
- **Multi-selection menu.** With several folders selected only the trash
  items are offered; reveal/copy/rename/sequence keep acting on one folder.
  The alternative (all items, acting on the clicked folder) risks the user
  reading "Rename…" as renaming the selection.
- **Long runs.** Collecting under a year folder on a slow share, and
  trashing thousands of files, both run in one `spawn_blocking` under the
  `Scans` lock with no progress or cancel, as the current command does.
  Opening folders waits meanwhile. If it proves slow, a progress event like
  `sequence-progress` is the follow-up.
- **Stale index rows after the move.** The `files` / `ratings` rows of a
  trashed file in a folder that is not open stay until that folder's next
  open reconciles them (`reconcile_sidecars` treats a gone sidecar as a
  cleared judgment; the scan drops the `files` row). This is the existing
  behavior for the open folder too (its `resync` does it at once). Not
  changed.
- **Watcher / tree-watch conflict.** No change to `watch.rs`. If the
  tree-watch session lands per-node watchers first, they too are
  `NonRecursive` directory watchers, which do not block moving child files
  on Windows; a `folder-changed` for the open folder just coalesces with
  the explicit `resync`.

## Progress

- Step 1: Backend reject collection over folders, and
  `trash_rejected(dirs, recursive)` implemented. `collect_folder` decides the
  flag from the configured sidecar format only (still gathering both formats'
  sidecars to move with a reject), so it agrees with the folder open even
  when a stale sidecar of the other format sits on disk. `Summary` splits
  move failures (`failed`) from files/folders `collect` never got to read
  (`unread`), and the frontend words and counts the two differently.
- (2026-09-28) Step 1 complete
