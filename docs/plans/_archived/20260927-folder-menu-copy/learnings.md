# Learnings

## Step 1

- The folder name passed to the menu is `node.name` from the tree row. Rows
  added by `reveal` for a folder outside home and the volumes get
  `rootOf(path)` as their name (the root path itself, e.g. `C:\` or `/`), so
  `Copy Folder Name` copies what the tree shows there too.
- `navigator.clipboard.writeText` from the folder menu is not hand-checked
  on any webview yet; the existing `todo.md` item on the settings window's
  Copy buttons covers the same unverified call.
- A fresh worktree has no `node_modules`, so `mise run fmt` failed with
  `Command "vp" not found`; `mise exec -- pnpm install --frozen-lockfile`
  fixed it.

## Deferred issues (todo candidates)

- Hand-check `Copy Path` / `Copy Folder Name` on WebView2 (Windows),
  WKWebView (macOS) and WebKitGTK (Linux); if a webview refuses the write,
  switch to `tauri-plugin-clipboard-manager` with a
  `clipboard-manager:allow-write-text` capability. Basis: plan.md
  "Trade-offs and risks" (no hand check possible in this step). Files:
  `crates/app/ui/src/main.ts`.
