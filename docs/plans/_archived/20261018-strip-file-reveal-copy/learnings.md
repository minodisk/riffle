# Learnings: strip-file-reveal-copy

## Step 1

- `contextMenuGroups` now builds its sections inside the function, since the
  reveal label is dynamic; a view-only folder simply leaves out the
  judgment sections and `Rename…` instead of filtering by index.
  `revealLabel` became the third parameter (before `viewOnly` / `count`), so
  every call in `context.test.ts` passes it and the judgment group indexes
  shifted by two.
- `openContextMenu` waits on the `revealLabel` promise like the folder menu;
  the promise is resolved long before any right-click, so the menu still
  opens at once.
- `docs/agents/tauri-app.md`'s "The strip context menu is HTML" note only
  mentions Pick / Reject / Unflag as an example, not the full item list, so
  it was left unchanged.
- The fresh worktree had no `node_modules`, so `mise run fmt` failed on the
  missing `vite-plus` binary until `mise exec -- pnpm install
  --frozen-lockfile` (bare `pnpm` is not on PATH in Git Bash).
- Editing files with Python's text mode on Windows writes CRLF; open them with
  `newline=""` (or strip `\r` afterwards) to keep the repository's LF.

## Deferred issues (todo candidates)

- **Pending manual check (Windows)**: the strip menu's reveal and copy items
  (`crates/app/ui/src/main.ts` `openContextMenu`,
  `crates/app/ui/src/context.ts` `contextMenuGroups`). Steps: open a RAW
  folder, right-click a file, choose `Reveal in File Explorer`; File Explorer
  should open the folder with that file selected. Repeat in a view-only JPEG
  folder. Then select three files, right-click one of them, choose
  `Copy Path` and paste into an editor (three full paths, one per line, no
  trailing newline), then `Copy File Name` (three names with extensions, one
  per line). Step 1's checkbox was ticked on the automated criteria only.
