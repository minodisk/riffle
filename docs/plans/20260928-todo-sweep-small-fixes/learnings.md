# Learnings: todo sweep, five small fixes

## Step 1

- `mise run fmt` now calls `node ./node_modules/vite-plus/bin/vp fmt`
  directly, the same workaround `[tasks.test]` already used for the `.cmd`
  shim that `pnpm exec vp` resolves to under the task's bash shell on
  Windows. `[tasks.lint]` still uses `pnpm exec vp check` (out of scope).
- Unlike `[tasks.test]`, `[tasks.fmt]` does not run `pnpm install`, so in a
  fresh worktree it fails with `Cannot find module ...vite-plus/bin/vp` until
  `mise exec -- pnpm install --frozen-lockfile` (or `mise run ci`) has run
  once. After that it formats cleanly on Windows.
- The failed-listing mark is `TreeNode.failed`, set by the new
  `markFailed` in `tree.ts` and cleared by `setChildren`, so any later
  successful listing of the folder (an expand or a `tree-changed` re-list)
  removes it. The row shows it as the `failed` class (a red name) and the
  error as the row's `title` in place of the path.
- Removing the `scan-state` emits left `Preparing`'s `app` handle still
  needed (its `Drop` takes the `Scans` state through it), and `Emitter` is
  still used by the other events in `commands.rs`.

## Deferred issues (todo candidates)

- `reveal`'s listing failures in `crates/app/ui/src/folders.ts` do not set
  the failed mark; only `toggle` does, as the todo item named. Basis: plan
  Step 1 scope for item (4). Related: `crates/app/ui/src/folders.ts`,
  `crates/app/ui/src/tree.ts`.
