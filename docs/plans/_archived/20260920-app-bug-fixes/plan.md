<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# App bug fixes

## Purpose

Close three small `todo.md` items in `crates/app`:

- A shortcut override that is skipped only because it collides with the
  current sidecar format's defaults (e.g. `reject: ["6"]` under XMP, where
  `6` is the `red` label default) is dropped from the stored `shortcuts`
  value the next time the user rebinds anything, because `update_keymap`
  (`crates/app/src/commands.rs`) persists `Keymap::overrides()` of the
  resolved keymap. The override should survive so it still applies after a
  switch to `.dop`.
- `canceling_after_the_first_batch_keeps_what_was_written`
  (`crates/app/src/index.rs`) cancels from a polling watcher thread and only
  "wins" the race by having 40 batches' worth of files; a faster runner can
  finish the scan first. It should cancel at a deterministic point.
- The second TODO under "a `sidecar-error` event can be missed under
  key-mashing" (revert the optimistic "has sidecar" flag) turned out to be
  stale: PR #54 (`f1ba56d`) already removed the frontend `sidecars` set and
  the optimistic `sidecars.add(path)`; `has_sidecar` remains only as an
  unused field of the `IndexedFile` interface in `crates/app/ui/src/main.ts`.
  No code change is needed; the wrap-up removes that TODO line (the sticky
  error display TODO above it stays). See "Trade-offs and risks".

Scope note: the in-flight `app-quick-fixes` plan (branch
`docs/plan-app-quick-fixes`: `focus_crop` full JPEG size, single folder
listing in `scan_folder`, index pruning/`VACUUM`) is another worktree's work.
Keep edits to `commands.rs` and `index.rs` minimal to avoid conflicts, and do
not touch `scan_folder`, `crop_payload`, or `run_scan`'s completion path.

## Steps

- [x] Step 1: Keep shortcut overrides that are inactive under the current sidecar format
  - Done when:
    - A stored `shortcuts` value such as `{"reject": ["6"]}` loaded under XMP
      (where the override is skipped) is still present in `Keymap::overrides()`
      after an unrelated `add`/`remove`/`reset` (e.g. `add("zoom", "z")` yields
      `{"reject": ["6"], "zoom": [...]}`), so `update_keymap` persists it and
      `switch_sidecar_format` (which rebuilds from `AppShortcutOverrides`)
      applies it under `.dop`.
    - Editing the action that holds an inactive override (`add`/`remove` on
      `reject` under XMP) replaces the inactive entry with the resolved keys;
      `reset(action)` drops it; `reset_all` drops everything (the setting is
      deleted, as `reset_shortcuts` documents).
    - Round trip holds: `Keymap::from_overrides(Some(&k.overrides()), format)`
      resolves to the same bindings as `k` under both formats, and a value
      loaded under XMP then saved then loaded under DOP applies the inactive
      override.
    - Rust unit tests in `crates/app/src/shortcuts.rs` cover the cases above;
      `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Keep the fix inside `Keymap` (`crates/app/src/shortcuts.rs`) so
      `update_keymap` and `switch_sidecar_format` in `commands.rs` stay as they
      are. Avoid editing `commands.rs` in this step.
    - Suggested shape: `from_overrides` records the entries it skipped for a
      key conflict in a new field on `Keymap`, and `overrides()` merges those
      entries in alongside the resolved diff. `add`/`remove` on an action, and
      `reset(action)`, remove that action's inactive entry; `reset_all` clears
      the field.
    - Preserve only conflict-skipped entries. Entries skipped for an unknown
      action, an invalid shape, the reserved `p`, or a platform-forbidden key
      keep being dropped.
    - `Keymap` derives `PartialEq`; tests that compare
      `from_overrides(...) == Keymap::defaults(format)` for a skipped override
      will need to compare `bindings()` instead, or the new field must be
      excluded from equality. Keep the existing tests' intent.
    - Update the doc comments on `overrides()` and `from_overrides` to say a
      conflicting override is kept in the stored value.

- [x] Step 2: Cancel `canceling_after_the_first_batch_keeps_what_was_written` from a deterministic point
  - Done when:
    - The test in `crates/app/src/index.rs` no longer spawns a polling watcher
      thread nor depends on `BATCH * 40` files: it sets `cancel` from the
      `progress` callback once `done >= BATCH`, and its assertions
      (`total >= BATCH`, `total < files.len()`, rows written == `total`) hold
      by construction.
    - The test passes repeatedly locally (a loop of 20+) and `mise run ci`
      passes on all three CI platforms.
  - Implementation approach (as far as it is known):
    - `run_scan` throttles `progress` to `PROGRESS_INTERVAL` (100 ms) plus the
      first and last item, so make the interval injectable: add a
      `progress_interval: Duration` parameter to `run_scan`; the `start_scan`
      call site in `commands.rs` passes `PROGRESS_INTERVAL`, the test passes
      `Duration::ZERO`. The other test calling `run_scan` passes the constant.
    - In `on_item`, `flush` of a full batch runs before `done.fetch_add`, so by
      the time `progress(done, _)` reports `done >= BATCH` the first batch has
      been handed to `write_batch`. Do not assert the exact number of rows at
      cancel time.
    - Keep the file count comfortably above `BATCH + threads` (e.g.
      `BATCH * 4`) with a comment explaining why the count is no longer
      load-bearing.
    - Update `run_scan`'s doc comment. Do not touch the completion path after
      `extract_all`.

## Trade-offs and risks

- **Item 2 ("revert the has-sidecar flag") has no step.** After #54 the
  frontend has no optimistic "has sidecar" state. The TODO is treated as
  resolved by #54 and only that line is removed in wrap-up. Dropping the unused
  `has_sidecar` field from `IndexedFile` is out of scope.
- **Inactive entry visibility.** The settings window lists `bindings()`, so a
  preserved inactive override is invisible until the format switches; it is
  cleared by `reset(action)` or `reset_all`. A UI hint is out of scope.
- **Step 2 API choice.** An injectable interval is the smallest deterministic
  option; moving the throttle into `start_scan` is the fallback.
- **Conflict risk with `app-quick-fixes`.** Step 2 touches one call site in
  `commands.rs` and `run_scan`'s signature/doc in `index.rs`; rebase whichever
  lands second.

## Progress

- (2026-09-20) Step 1 complete
- (2026-09-20) Step 2 complete
