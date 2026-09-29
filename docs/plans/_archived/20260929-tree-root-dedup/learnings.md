# Learnings

## Step 1

- `treeKey` took only the drawn rows, so `canExpand` had no way to see
  `tree.roots` for `drawnChildren`. Its signature changed to take the `Tree`
  and derive the rows itself (`treeKey(tree, cursor, key)`); the one caller
  in `folders.ts` (`keydown`) and the `treeKey` tests were updated to match.
  That is one line of `folders.ts` beyond the expander branch, which may
  meet the parallel work on that file.
- `docs/usage.md` said "A folder's arrow lists its subfolders", which a home
  hidden under `C:\Users` would contradict, so it gained a clause.
- On Windows, a Python heredoc edit in text mode writes CRLF, and `'''...'''`
  strings eat `\U` / `\\` sequences; the repository is LF
  (`core.autocrlf=false`), so write with `newline=""` and check backslashes
  in Windows-path test fixtures afterwards.
- The worktree had no `node_modules`: `mise exec -- pnpm install
  --frozen-lockfile` first, then `mise exec -- node
  ./node_modules/vite-plus/bin/vp test <file>` runs one test file.
