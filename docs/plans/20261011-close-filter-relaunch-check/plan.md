<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Close the persist-filter relaunch check todo

## Purpose

The todo item `### App: real-device check that the strip's filter survives a relaunch` asked for a hand check of the persist-filter feature (archived plan `docs/plans/_archived/20261010-persist-filter/plan.md`) on Windows and macOS. The user ran all three scenarios (filter restored with the menu checks and the lit button; `Reset` then relaunch is unfiltered; a bogus `"filter"` value in `settings.json` launches unfiltered without error) on a desktop build and they passed. The feature has no OS-specific code path (tauri-plugin-store settings plus the frontend), so a repeat on the other OS adds nothing. Removing the item keeps `todo.md` to open work only.

## Steps

- [x] Step 1: Remove the item from `todo.md`
  - Done when: the heading `### App: real-device check that the strip's filter survives a relaunch`, its `#### Background` paragraph, its `Files:` line and its `#### TODO` list (the Windows and macOS checkboxes) are gone, together with the blank line separating it from the previous item; every other item is untouched; `todo.md` still ends in a single newline with no trailing blank line; `git diff --stat` against `origin/main` shows only `todo.md` and this plan folder; `mise run ci` passes.
  - Implementation approach:
    - Locate the item by its heading (do not rely on line numbers; `main` may have moved). If it is the last item, delete from the blank line before the heading through the end of the file; otherwise delete from the heading up to (not including) the next `### ` heading, keeping a single blank line between the remaining neighbors.
    - Do not edit `docs/plans/_archived/20261010-persist-filter/learnings.md`; it records what was filed at the time. Write the verification result in this plan's `learnings.md` instead.
    - Commit as `docs(todo): drop the persist-filter relaunch check, verified by hand` (Conventional Commits, English).

## Trade-offs and risks

- Other-OS repeat: dropped, since the code path is identical across platforms. Keeping one checkbox would leave an item nobody intends to run.
- Archived learnings: left as-is (see Step 1).

## Progress

- (none yet)
