# Learnings

## Step 1

- The helper is `restore(selection, files, shown, path)` in
  `crates/app/ui/src/selection.ts`. `step()` calls it right after `commit(...)`
  and before the hidden-by-filter early return: a hidden `path` is not in
  `files`, so the helper already leaves the selection alone there and no extra
  branch is needed in `main.ts`.
- Gating on `path !== shownPath` also covers the changed-list case where
  `refilter`'s `prune` has already produced `{B, A}`; a test pins that the
  pruned pair collapses to `single(A)`.
- `mise run fmt` fails on Windows at `pnpm exec vp fmt` ("Command \"vp\" not
  found"): the same `.cmd` shim problem `mise.toml` already works around for
  `vp test` by calling `node ./node_modules/vite-plus/bin/vp`. `cargo fmt`
  ran; the frontend formatting was verified by `mise run ci`'s lint task
  ("All 93 files are correctly formatted").

## Deferred issues (todo candidates)

- `mise run fmt` does not work on Windows: its `pnpm exec vp fmt` hits the
  `.cmd` shim / POSIX PATH problem that the `test` part of `mise run ci`
  avoids by invoking `node ./node_modules/vite-plus/bin/vp` directly. Basis:
  Step 1's local checks of this plan. Related file: `mise.toml` (`[tasks.fmt]`).
