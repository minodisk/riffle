# Learnings

## Step 1

- The "Bursts" bullet in `docs/usage.md` is hard-wrapped at about 80 columns,
  though one earlier line already runs past it. Only the lines around the
  inserted sentences were reflowed, to keep the diff small.
- In a fresh worktree `mise run fmt` fails with `Command "vp" not found`
  because `node_modules` is not installed yet; `mise run ci` runs the pnpm
  install in its lint task, after which `mise run fmt` succeeds.
