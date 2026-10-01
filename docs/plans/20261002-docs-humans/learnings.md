# Learnings

## Step 1

- The acceptance grep `git grep ... -- 'docs/*.md' docs/agents` for `](./<name>.md`
  is not empty after the move: git pathspecs without `:(glob)` let `*` match
  `/`, so `docs/*.md` also matches `docs/humans/*.md`, and it lists the moved
  files' own `./cameras.md` cross-links (which are correct from
  `docs/humans/`) plus two backticked mentions in `docs/plans/_archived/**`.
  Nothing outside `docs/humans/` and the archive matches, which is the intent.
  `git grep ... -- ':(glob)docs/*.md' docs/agents` returns nothing.
- Besides the files listed in the plan, the mechanical replace also hit the
  in-progress plan `docs/plans/20261002-error-row-exif-tree-refresh/plan.md`
  (one backticked `docs/usage.md` in a Done-when). It is an open plan, not an
  archive, and the acceptance grep does not exclude it, so it was repointed
  so the agent executing that plan edits the right file.
- The moved files hold no relative links other than the four `../README.md` /
  `../CONTRIBUTING.md` ones and the `./<name>.md` cross-links, so only those
  four needed a `../../` prefix.
- `mise run fmt` first failed with `Cannot find module ...node_modules\vite-plus\bin\vp`:
  the fresh worktree had no `node_modules`. `pnpm` is not on the Git Bash
  PATH; `mise exec -- pnpm install --frozen-lockfile` fixed it.
