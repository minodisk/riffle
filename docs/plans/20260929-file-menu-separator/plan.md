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

# File menu doubled separator on Windows and Linux

## Purpose

On Windows and Linux the File menu shows two separators in a row: Open Folder,
Reload Folder, Settings..., Check for Updates…, ─, ─, Close Window, Quit. In
`app_menu::build` (`crates/app/src/main.rs`) the File submenu is built with
`file.prepend_items(&[&open_folder, &reload_folder, &separator])`, then, on
non-macOS, `file.insert_items(&[&settings, &check_updates, &separator], 2)`.
Index `2` dates from when File also held Move Rejected to Trash and Sequence
JPEG Timestamps; #537 removed those, so `2` now lands before the prepended
separator instead of after it. The comment describes the intended layout
(Settings and Check for Updates "at the end of File's own items, above Close
Window and Quit"): Open Folder, Reload Folder, ─, Settings..., Check for
Updates…, ─, Close Window, Quit. Fixing it removes the duplicate separator and
ties the insertion point to the prepended items so it cannot drift again.

## Steps

- [x] Step 1: Insert Settings and Check for Updates after the prepended File items
  - Done when:
    - `crates/app/src/main.rs` `app_menu::build` inserts `settings`,
      `check_updates` and the trailing separator at the index right after the
      items prepended to File, derived from the prepended slice's length
      rather than a literal `2`.
    - `mise run ci` passes.
    - Manual check by the user on Windows: the File menu reads Open Folder,
      Reload Folder, ─, Settings..., Check for Updates…, ─, Close Window, Quit
      with a single separator between each group.
  - Implementation approach:
    - Only the non-macOS branch (`#[cfg(not(target_os = "macos"))]` block at
      `main.rs` around lines 221-229) changes; the macOS branch, which puts
      both items in the app menu, stays as is.
    - Bind the prepended slice to a local (e.g. `let own = [&open_folder,
      &reload_folder, &separator]; file.prepend_items(&own)?;`) and pass
      `own.len()` as the `insert_items` index, so the index follows the
      prepended items automatically. `PredefinedMenuItem::separator(handle)?`
      must be bound to a local first because a slice of `&dyn IsMenuItem`
      cannot hold a temporary across the two calls; the current code already
      mixes `MenuItem` / `IconMenuItem` and `PredefinedMenuItem` references in
      one slice, so keep the array of `&dyn IsMenuItem<R>` form.
    - Keep the existing comment; it already states the intended placement.
    - `docs/agents/tauri-app.md` does not describe the File menu order
      (verified by grep), so no doc change.
    - Because this machine is Windows, the verification of the rendered menu
      is a manual run by the user, not something the agent can assert; note
      the outcome in `learnings.md`.

## Trade-offs and risks

- Deriving the index from the prepended slice's length versus counting
  `file.items()?` at insertion time: the slice length is simpler and does not
  depend on what the default menu already had in File (Close Window, Quit),
  which is what the "own items" should sit above. Counting `file.items()` would
  place them at the very end, below Quit, which is wrong. Use the slice length.
- The Linux File menu is created empty by this code path and then receives the
  same items, so the fix applies identically there; it is not verifiable on
  this machine, but the logic is platform-independent.

## Progress

- (none yet)
