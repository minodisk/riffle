# Learnings

## Step 1

- Verified by the user on 2026-09-28 with Lightroom Classic on Windows
  (Japanese UI), three Sony ARWs imported and then given stars and a red label
  by Riffle through XMP:
  - With no action, Lightroom Classic picked up nothing and showed no
    metadata-conflict badge on the thumbnails.
  - `Metadata > Read Metadata from File` (right-click the selected photos in
    the Library grid) picked up the stars and red label of the selected photo.
  - `Synchronize Folder...` with `Scan for metadata updates` picked up the
    stars and labels of every photo in the folder.
- The README bullet now names the per-photo route first and the folder-wide
  route second; the next plan touching this section does not need to
  re-verify these three facts.
- In a fresh worktree `mise run fmt` fails with `Command "vp" not found`
  because `node_modules` is absent; `mise exec -- pnpm install
  --frozen-lockfile` first fixes it (bare `pnpm` is not on the Git Bash PATH).
