# Learnings

## Step 1

- The note uses `&gt;` for the menu path separators
  (Metadata &gt; Color Label Set &gt; Edit) so the HTML stays valid; the
  formatter leaves the entity as is.
- The "buttons below" wording refers to both built-in-set buttons without
  repeating their labels, keeping the note short.
- In a fresh worktree `mise run fmt` fails with `Command "vp" not found`
  because `node_modules` is absent (only `lint` and the tauri tasks run
  `pnpm install`); run `pnpm install --frozen-lockfile` first. The formatter
  then rewrapped the paragraph at the print width.
