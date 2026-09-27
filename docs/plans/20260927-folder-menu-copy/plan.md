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

# Copy Path / Copy Folder Name in the folder tree's context menu

## Purpose

Right-clicking a folder in the tree offers only the OS reveal item and
`Sequence JPEG Timestamps…`. Getting a folder's path into another program
(a terminal, a RAW developer's import dialog, a chat) means going through the
file manager. Two cheap items put the absolute path, unquoted, or just the
folder's name on the clipboard. Five larger folder-menu ideas that came up
alongside (Open in Terminal, rename, refresh vs rescan, expand / collapse all,
sidecar rewrite / delete) are recorded in `todo.md` for later rather than
built now. A sixth idea, sequencing from the folder menu with the folder
preselected, already ships (`sequenceTimestamps` → `sequenceTimestampsOf`),
so it is not recorded.

## Steps

- [x] Step 1: Add `Copy Path` and `Copy Folder Name` to the folder context menu and record the five deferred folder-menu items in `todo.md`
  - Done when:
    - Right-clicking a folder in the tree shows `Copy Path` and
      `Copy Folder Name` as one group between the reveal item and the
      sequence group; `Copy Path` puts the folder's absolute path on the
      clipboard with no quotes, and `Copy Folder Name` puts the folder's name
      as the tree shows it (the last path component; `C:\` for a Windows drive
      root, the last component for home, `/` for the Linux root row).
    - A refused clipboard write is reported through the status line, the
      way other menu actions report their errors, and does nothing else.
    - `folderMenuGroups` in `crates/app/ui/src/context.test.ts` covers the
      new items and their order.
    - `todo.md` holds five new `### App: ...` sections (one per deferred
      item) under `## Cross-cutting / other`, each with the context
      paragraph, `Files:`, and a `#### TODO` list in the file's existing
      style.
    - `README.md`, `README.ja.md` (in sync) and `docs/usage.md` mention the
      two items where they describe the folder tree's right-click menu.
    - `mise run ci` passes.
  - Implementation approach:
    - Frontend only; no Rust change and no new dependency. The menu is the
      HTML one: extend `folderMenuGroups` in `crates/app/ui/src/context.ts`
      with `copyPath` / `copyFolderName` items (label `Copy Path`,
      `Copy Folder Name`, `shortcut: ""`, `checked: undefined`) as one group
      after the reveal item and before the sequence group. Update the doc
      comment above `folderMenuGroups`.
    - Pass the row's name along with its path: change `folders.ts`'s
      `contextMenu` / `init` callback from `(path, x, y)` to
      `(path, name, x, y)` using `node.name`, and the `folders.init` call in
      `crates/app/ui/src/main.ts`. This reuses the backend's naming
      (`crates/app/src/folders.rs` `node()`), which already handles drive
      roots, instead of re-splitting the path in TypeScript.
    - In `main.ts`'s folder-menu `switch`, add the two cases calling
      `navigator.clipboard.writeText(...)` (the facility `settings.ts`
      `copyText` already uses) with `.catch((err) => setStatus(String(err)))`.
      Do not copy the select-text fallback from `settings.ts`; there is no
      text element to select in the menu.
    - `todo.md`: append the five sections at the end of
      `## Cross-cutting / other`, matching the surrounding style (`### App:
      title`, a paragraph with the known constraints, `Files:`, `#### TODO`
      with `- [ ]` bullets). Carry the constraints for each item into the
      paragraph:
      - Open in Terminal: per-platform terminal choice, Linux ambiguous;
        `crates/app/src/folders.rs`, `crates/app/ui/src/context.ts`,
        `crates/app/ui/src/main.ts`.
      - Rename: stop the scan and flush the sidecar writer; the OS rename
        fails on Windows with open handles and needs its error shown; rewrite
        the path / dir prefixes in the SQLite `files` / `ratings` / `folders`
        tables, all keyed by absolute path, including subfolders and dirty
        `ratings` rows so unsynced sidecar writes and `last_viewed` survive
        and thumbnails / analysis are not re-extracted; reopen the current
        folder under its new path keeping the tree expansion; an inline edit
        UI on the tree row; `crates/app/src/index.rs`,
        `crates/app/src/sidecar.rs`, `crates/app/src/commands.rs`,
        `crates/app/ui/src/folders.ts`, `crates/app/ui/src/tree.ts`.
      - Refresh vs Rescan: Refresh = re-list subfolders and RAW counts,
        light; Rescan = re-index the open folder, heavy; semantics undecided,
        leaning toward a single Refresh that reloads the tree and runs an
        incremental scan of the open folder.
      - Expand / collapse all subfolders: `crates/app/ui/src/tree.ts`,
        `folders.ts`.
      - Rewrite / delete the folder's sidecars, delete behind a confirmation
        dialog: `crates/app/src/sidecar.rs`, `crates/app/src/commands.rs`,
        `crates/core/src/xmp.rs`, `crates/core/src/dop.rs`.
    - Docs: one clause each in `README.md` (line ~63, "Right-click a folder
      to reveal it ... or copy its path or name"), the matching sentence in
      `README.ja.md` (line ~46), and the folder-tree paragraph in
      `docs/usage.md` (lines ~37-43).
    - Commit as `feat(app): copy the folder path or name from the tree's context menu`
      (the todo additions ride in the same PR).

## Trade-offs and risks

- **`navigator.clipboard.writeText` vs `tauri-plugin-clipboard-manager`.**
  The plan reuses the web API the settings modal already uses, per the
  "no new dependency" constraint. `todo.md` already records that this call
  is unverified on WebKitGTK / Windows / macOS webviews; a right-click menu
  click is a user gesture, so it should be allowed, but if a webview refuses
  it the item only shows an error in the status line. If that happens on a
  shipped platform, the fallback is the Tauri clipboard plugin plus a
  `clipboard-manager:allow-write-text` capability, as a follow-up.
- **Where the folder name comes from.** Passing `node.name` from the tree
  reuses the backend's naming; a separate `folderName(path)` helper in
  TypeScript would duplicate `folders.rs` `node()` and could disagree with the
  tree on edge cases, so it is not taken.

## Progress

- (2026-09-27) Step 1 complete
