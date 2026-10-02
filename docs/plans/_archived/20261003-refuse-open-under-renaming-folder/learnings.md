# Learnings

## Step 1

- `tree.ts` already exports `relation(path, dir, ignoreCase)` ("same" /
  "under" / null), so `RenamesInFlight.blocks` in `rename.ts` uses it
  directly; no new path helper was needed and the folding stays in `tree.ts`.
- The in-flight state is a list rather than a set, so two invokes on the same
  path (not reachable today, but cheap to get right) each need their own
  settle before the block lifts. `settle` matches the exact string `main.ts`
  passed to `start`, which is always the same `path` variable.
- `folders.ts` routes both open sites (the plain row click and the keyboard
  `open` command) through one `openFolder`, which reports
  "A folder is being renamed; try again in a moment." via `reportError` and
  returns without a `render()`.
- No hands-on reproduction was attempted: the window is the
  `rename_folder` IPC round trip (tens of milliseconds normally), and there
  is no reliable way to widen it by hand from the UI here. The unit tests on
  the predicate and the two wired call sites are the verification, so the
  todo section was deleted with nothing in its place.
- CI attempt 1 failed on `TS1487` (octal escapes) in `rename.test.ts`: the
  Windows path literals written through a shell heredoc lost one of their
  doubled backslashes (`"D:\photos\2026"`). Write `\` in TS string literals
  with an editor tool, not through the shell. Also, a fresh worktree needs
  `mise exec -- pnpm install --frozen-lockfile` before `mise run fmt`.
