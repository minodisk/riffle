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

## Round 1 review (fix/undo-selection)

- Review feedback: the `path !== shownPath` gate alone did not protect compare
  mode, because there `judge()` records `compareActivePath`, which a pane
  click can change without moving `index`. A single-file undo in compare mode
  could therefore collapse a multi-selection `loadCompare()` was using as the
  comparison set. Fixed by skipping the collapse in `step()`
  (`crates/app/ui/src/main.ts`) while `comparing` is true, and updated the
  trade-offs section of `plan.md` to describe the extra guard.

## Deferred issues (todo candidates)

- `mise run fmt` does not work on Windows: its `pnpm exec vp fmt` hits the
  `.cmd` shim / POSIX PATH problem that the `test` part of `mise run ci`
  avoids by invoking `node ./node_modules/vite-plus/bin/vp` directly. Basis:
  Step 1's local checks of this plan. Related file: `mise.toml` (`[tasks.fmt]`).
  Change: call `node ./node_modules/vite-plus/bin/vp fmt` in `[tasks.fmt]` as
  `[tasks.test]` does. Done when `mise run fmt` succeeds on Windows without
  "Command \"vp\" not found" and `mise run ci` still passes.
