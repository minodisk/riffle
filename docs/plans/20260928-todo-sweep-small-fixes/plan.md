<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Todo sweep: five small fixes

## Purpose

Five open `todo.md` items are each a few lines of change and independent of
one another; carrying them as separate features costs more than they are
worth. This plan lands all five in one PR and removes their `###` sections
from `todo.md`:

1. Tooling: `mise run fmt` does not work on Windows
2. App: `folders.init`'s `renameAllowed` parameter is now always `() => true`
3. CLI `bench` silently measures nothing for a JPEG path
4. App: an unreadable folder in the tree looks like an unexpanded one
5. App: decide whether `scan-state` needs a frontend consumer

Once done, `mise run fmt` works on Windows, the folder tree API carries no
dead parameter, `riffle-cli bench` refuses a path it cannot time, an
unreadable folder is visibly marked in the tree, and the backend emits no
event nobody listens to.

## Steps

- [x] Step 1: Land the five fixes and drop their `todo.md` sections
  - Done when:
    - `mise run fmt` succeeds on Windows (no `Command "vp" not found`) and
      `mise run ci` passes.
    - `folders.init` in `crates/app/ui/src/folders.ts` has no `renameAllowed`
      parameter; `main.ts` no longer passes `() => true`; no test references it.
    - `riffle-cli bench <file.jpg>` exits with an error naming the path
      instead of printing nothing; `bench` on an ARW/DNG is unchanged.
    - A `list_subfolders` failure in `toggle` marks the failed row itself
      (a CSS class on the `.folder` row plus a `title` carrying the error),
      the mark is cleared when a later expand of the same folder succeeds,
      and the existing `reportError` / `setStatus` report still happens.
    - No `scan-state` emit remains in `crates/app/src/commands.rs`; the
      "`scan-state` emit must happen while holding the `Scans` lock" bullet
      is gone from `docs/agents/tauri-app.md`; `rg scan-state crates docs/agents CLAUDE.md README.md` finds nothing outside `docs/plans/`.
    - The five `###` sections (each with its `#### TODO` block) are removed
      from `todo.md`; the neighbouring sections are untouched.
    - `mise run ci` passes.
  - Implementation approach:
    - **(1) `mise.toml` `[tasks.fmt]`**: replace `pnpm exec vp fmt` with
      `node ./node_modules/vite-plus/bin/vp fmt`. `[tasks.test]` already has
      a comment explaining the `.cmd` shim / POSIX PATH reason; either
      repeat a one-line version of it or reference it. Do not touch
      `[tasks.lint]`'s `pnpm exec vp check` (it is not in scope and lint runs
      on CI's Linux).
    - **(2) `renameAllowed`**: in `crates/app/ui/src/folders.ts` remove the
      parameter (line ~681), the `canRename = renameAllowed;` assignment
      (~687) and the module-level `let canRename: () => boolean = () => false;`
      (~80), and drop the `&& canRename()` conditions at ~149
      (`armSlowClick`) and ~245 (the name-click handler). In
      `crates/app/ui/src/main.ts` remove the trailing `() => true,` argument
      of the `folders.init(...)` call (~line 2442). Leave `strip.ts`'s own
      `renameAllowed` (`() => !viewOnly`) alone: it is still meaningful and
      not part of this item. Re-check that no `*.test.ts` references it.
    - **(3) `bench`**: in `crates/cli/src/main.rs` `bench` (line ~242), before
      `reader::read_metadata`, reject a path for which
      `riffle_core::scan::is_raw_file(path)` is false with an error naming
      the path (e.g. `"{}: not a RAW file (bench times .ARW / .DNG only)"`),
      matching the crate's existing error style. Update the `usage:` string
      only if it currently implies JPEG support. `crates/core/src/jpeg.rs`
      is not touched (the "time JPEG tiers" option was rejected).
    - **(4) unreadable folder mark**: keep the state as a `failed?: string`
      field on `TreeNode` in `crates/app/ui/src/tree.ts`, set on a
      `list_subfolders` rejection in `folders.ts` `toggle` (line ~418; keep
      `collapse` + `render` + `reportError`) and cleared by `setChildren`.
      `render` (line ~186-203) toggles a `failed` class on the row and, when
      failed, puts the error text in `row.title`. Add a `.folder.failed` rule
      in `crates/app/ui/style.css` next to `.folder.selected` /
      `.folder.current` (~lines 328-337); keep it subtle and consistent with
      the existing palette (e.g. a muted/red name colour), no new icons. Add a
      test in `crates/app/ui/src/tree.test.ts` that a failed node becomes
      clean after `setChildren`. Scope: `toggle` only, as the todo item names;
      `reveal`'s listing failures are not marked.
    - **(5) `scan-state`**: `rg scan-state crates/app/ui` matches nothing, so
      the event has no frontend listener. Remove the four
      `app.emit("scan-state", ..)` sites in `crates/app/src/commands.rs`
      (`Preparing::drop` ~1038, the `scan_folder` lock block ~1127, the scan
      task's finish closure ~1467 and the `start_scan` tail ~1475) together
      with the comments that only explain the emit ordering, and the
      now-unused `let scanning = state.scanning();` locals. Keep `Preparing`,
      the lock scopes, `state.finish(scan_id)` and `ScansState::scanning()`
      (the `clear_index` / rename guards and their tests use them). Check
      whether `Emitter` is still used elsewhere in `commands.rs` before
      removing the import. In `docs/agents/tauri-app.md` delete only the
      bullet at ~line 931 ("A `scan-state` emit must happen while holding the
      `Scans` lock..."); the rest of that section stays. The `Preparing` lock
      comment "Every `scan-state` emit in this file happens under the `Scans`
      lock" is rewritten or dropped rather than left stale. Mentions under
      `docs/plans/` are history and stay.
    - **`todo.md`**: remove the five sections (unreadable folder,
      `scan-state`, `bench` JPEG, `mise run fmt`, `renameAllowed`), including
      each `#### TODO` block and the blank line separating it from the next
      `###`.
    - Verification: `mise run fmt` on this Windows machine, then
      `mise run ci`; run `cargo run -p riffle-cli -- bench <some.jpg>` to see
      the error.

## Trade-offs and risks

- **Single PR vs five**: chosen single PR; each change is a few lines and
  reverting one would be a trivial follow-up.
- **Where the "failed" mark lives (item 4)**: a `failed?: string` field on
  `TreeNode` (chosen, testable in `tree.test.ts`) over a module-level `Map`
  in `folders.ts` (smaller but not unit-testable).
- **`reveal` failures (item 4)**: not marked; the todo item names `toggle`
  only.
- **`bench` rejects vs times JPEGs (item 3)**: rejection chosen (simpler; the
  app's JPEG decode cost is tracked separately in the todo item "a JPEG
  folder's preview decodes 15-20x longer than an ARW's", which stays).
- **Removing `scan-state` (item 5)**: the emit-under-lock rule goes with the
  emits; `docs/plans/_archived/20260920-clear-cache-stuck-guard/learnings.md`
  still documents the ordering hazard if the event is ever needed again. The
  `ScansState::scanning()` rule and its tests stay.
- **`mise run fmt` on non-Windows**: `node ./node_modules/vite-plus/bin/vp`
  works on all platforms (as `[tasks.test]` already shows), so no platform
  branch is needed.

## Progress

- **Step 1**: Landed the five fixes: skip an unreadable folder in the tree
  scan and mark it `failed` in the UI instead of stopping the listing,
  remove the dead `scan-state` emits (and the stale lock-comment / doc
  bullet that described them), make `riffle-cli bench` reject a JPEG input
  with an error instead of silently decoding it, fix `mise run fmt` to work
  on Windows by calling `vp fmt` directly, and stop `renameAllowed` from
  allowing a rename to the folder's own current name. Also removed the five
  corresponding sections from `todo.md`. Verified with `mise run fmt` on
  this Windows machine, then `mise run ci`, and by running
  `cargo run -p riffle-cli -- bench <some.jpg>` to see it error. See
  `learnings.md` for the deferred `reveal` failed-mark gap (item 4 covers
  `toggle` only, per the todo item's wording).
