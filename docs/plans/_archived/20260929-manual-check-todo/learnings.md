# Learnings: manual-check-todo

## Step 1

- The `todo.md` line numbers the plan cited (376-419, 990-1044) had drifted by
  the time of implementation; the reference items were found by heading with
  `rg -n '^### ' todo.md` instead (`### App: real-device checks for the View
  menu` at line 168, the wait-for-scan item at line 1047).
- The plan asked the curator's duplicate search to cover the headings and the
  `#### TODO` lines, but its own example (`strip-keep-scroll-on-rescan` inside
  `### App: Windows real-device check of the merged folder-tree, scan-wait and
  strip-scroll work`) names the feature only in the Background. The curator
  text therefore searches every line with `rg -n` and maps the hits to items
  through the heading line numbers, which still avoids reading `todo.md` in
  full.
- This plan has no manual check, so its own wrap-up is a live test: the curator
  should report "0 manual checks".
- `mise run fmt` first failed in this fresh worktree with
  `Cannot find module ...\node_modules\vite-plus\bin\vp`: `node_modules` was not
  installed and `pnpm` is not on the Git Bash `PATH`. `mise exec -- pnpm
  install --frozen-lockfile` fixed it.
