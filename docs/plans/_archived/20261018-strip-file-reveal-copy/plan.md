<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Strip right-click: reveal the file, copy its path or name

## Purpose

The folder tree's right-click menu (`folderMenuGroups` in
`crates/app/ui/src/context.ts`) reveals a folder in the OS file manager and
copies its path or name, but the filmstrip's menu (`contextMenuGroups`) offers
nothing of the kind for a file: getting a file's path into another app, or
finding it in Finder / File Explorer, means leaving Riffle. This plan adds the
same three items to the strip's menu, for RAW folders and view-only JPEG
folders alike: the reveal item under the platform's existing label, `Copy
Path` (the full path) and `Copy File Name` (with extension). On a
multi-selection the copy items copy every targeted file, one per line, and the
reveal item shows the focused file.

Investigation result: no new OS code is needed. `reveal_folder`
(`crates/app/src/folders.rs`) calls `tauri_plugin_opener`'s
`reveal_item_in_dir`, which already opens the parent with the item selected on
every platform (`SHOpenFolderAndSelectItems` on Windows,
`activateFileViewerSelecting` on macOS, D-Bus `FileManager1.ShowItems` on
Linux) and accepts a file path. `REVEAL_LABEL` is `Reveal in Finder` /
`Reveal in File Explorer` / `Open Containing Folder`, served by the
`reveal_label` command, which `main.ts` already fetches once into the
`revealLabel` promise.

Decided with the user: the new items go right after `Select All` (reveal, then
`Copy Path` / `Copy File Name`), mirroring the folder menu's top.

## Steps

- [x] Step 1: Add the reveal and copy items to the strip menu, wire them up, test, and document
  - Done when:
    - Right-clicking a strip cell shows the reveal item (labeled by
      `reveal_label`), `Copy Path` and `Copy File Name` in a normal folder and
      in a view-only JPEG folder, each with an empty shortcut column and no
      checked state.
    - `Copy Path` / `Copy File Name` write the targeted files' values to the
      clipboard, newline-separated when several files are targeted (the same
      `targets(selection, files, index)` list `Move to Trash…` uses, so the
      count the trash label names is the count copied); one value, no trailing
      newline, for a single file.
    - The reveal item opens the OS file manager with the focused file
      selected (`files[index]`, or the Compare-active file while comparing,
      which is what `openContextMenu` already calls `focused`).
    - Clipboard or reveal errors land on the status line via `setStatus`, as
      the folder menu's do.
    - `context.test.ts` covers the composition: the new groups in the normal
      menu, in the view-only menu, that the items carry an empty shortcut and
      `checked: undefined`, and that `count > 1` leaves their labels unchanged
      while `Move N Files to Trash…` still forms; the "unbound action" count
      (currently 20) and the view-only expectation are updated.
    - `docs/humans/usage.md` and `usage.ja.md` (Filmstrip bullet, and the
      **JPEG-only folders** bullet's sentence that the strip menu shows only
      `Select All` and `Move to Trash…`), `README.md` and `README.ja.md`
      (Filmstrip bullet), `CLAUDE.md` (the `src/context.ts` and
      `src/folders.rs` entries, since `reveal_folder` now also reveals a file),
      and `docs/agents/tauri-app.md`'s "The strip context menu is HTML" note
      (if its item list is enumerated there) say the same thing.
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - `context.ts`: extend `contextMenuGroups` with a `revealLabel: string`
      parameter and add the items as sections the view-only filter keeps.
      The current filter is `i === 0 || i === SECTIONS.length - 1` over the
      static `SECTIONS`; the reveal label is dynamic, so either build the
      sections inside the function or mark which sections survive view-only
      explicitly instead of by index. Order: `Select All`, then `[reveal]`,
      then `[Copy Path, Copy File Name]`, then the judgment groups, `Rename…`,
      `Move to Trash…`. Action names: reuse `copyPath` for the path (the
      folder handler is a separate `showMenu` callback, so no clash) and add
      `copyFileName` and `revealFile`; keep `MenuItem` unchanged.
    - `main.ts` `openContextMenu` (~line 3166): the label is a promise
      (`revealLabel`, line 3290); wrap the `showMenu` call in
      `void revealLabel.then((label) => …)` as the folder menu does (line
      3466), or resolve it into a module `let` once at startup. Handle
      `revealFile` → `invoke("reveal_folder", { path: focused })`,
      `copyPath` → `navigator.clipboard.writeText(paths.join("\n"))`,
      `copyFileName` → `paths.map(baseName).join("\n")` (`baseName` at
      main.ts line 715 already exists); errors to `setStatus(String(err))`.
      Note `paths` is `[focused]` while comparing, so copy follows that.
    - `folders.rs`: only the doc comments of `REVEAL_LABEL` ("the folder
      tree's right-click item") and `reveal_folder` change to say the strip's
      file item uses them too. No new command, no capability change (the
      opener is called from Rust, not from the webview).
    - `renameFile` / `trashFiles` keep their special handling in the
      `showMenu` callback; add the three new actions beside them, not in
      `runAction` (they are not keymap actions and must not get a binding).
    - Do not add shortcuts to `shortcuts.rs` or `keys.ts`: the requirement is
      an empty shortcut column.
    - Manual check on Windows (left to the user, recorded as pending): reveal
      from a normal folder and from a view-only folder; multi-select three
      files, copy path and name, paste into an editor.

## Trade-offs and risks

- **Reuse `reveal_folder` vs. a generic `reveal_path` command.** The plan
  reuses `reveal_folder` because `reveal_item_in_dir` already does exactly
  what a file reveal needs and the frontend already calls it; the only cost
  is the name. Renaming it to `reveal_path` is more accurate but pure churn.
- **Drive-root files.** `tauri-app.md` notes `reveal_items_in_dir` returns
  `NoParent` on Windows for a path without a shell parent; a file directly
  under `C:\` could in theory hit `shell_parent_path` quirks. It surfaces on
  the status line like the folder case; not worth a fallback unless the
  manual check shows it.
- **Clipboard permission.** `navigator.clipboard.writeText` already works
  for the folder menu inside the Tauri webview, so no new capability is
  expected.

## Progress

- (2026-10-18) Step 1 complete
- Step 1: the strip right-click menu gained reveal-in-file-manager and copy path / copy name items (commit 1287fa70). The Windows manual check is still pending (see `learnings.md`, Deferred issues).
