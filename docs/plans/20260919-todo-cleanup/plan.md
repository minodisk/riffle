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

# Close out six small todo items

## Purpose

`todo.md` carries six small, self-contained items that have been open long
enough: an orphaned Tauri command, a stale note about `riffle-cli bench`'s
crop scaling, a leaked crop bitmap in the UI, unguarded mozjpeg panics in the
CLI, a git helper script that fails on clones without `origin/HEAD`, and a
`mise run git:main` task that leaves the local `main` branch stale (plus the
`merger` agent's report wording that assumes it does not). Fixing them in two
small PRs clears the backlog and stops the tooling failures from surfacing as
spurious `merger` failures. Removing the closed `todo.md` headings happens in
wrap-up, not in a step.

## Steps

- [x] Step 1: Fix the four app/CLI items (`ping`, bench crop scaling, crop bitmap release, mozjpeg panic guard)
  - Done when:
    - `fn ping` and its `ping,` entry in `tauri::generate_handler!` are gone from `crates/app/src/main.rs` and the app crate still builds.
    - `bench()` in `crates/cli/src/main.rs` is confirmed to apply the same sensor→JPEG scaling as `crop()` (both already go through `partial::decode_focus_crop` → `focus_point`); a unit test in `crates/core/src/partial.rs` covers `focus_point` scaling when the JPEG size differs from the sensor size, unless one already exists.
    - Paging to another file in `crates/app/ui/src/main.ts` with the 1:1 view off closes and drops the held crop bitmap, so no stale crop bitmap survives past `show()`.
    - A malformed JPEG passed to `decode_rgb` returns `Err` instead of aborting the process, guarded in one place, and a unit test proves it.
    - `mise run ci` passes.
  - Implementation approach:
    - `ping`: delete `#[tauri::command] fn ping` and the `ping,` line in `generate_handler!` in `crates/app/src/main.rs`. Nothing else references it.
    - Bench scaling: no code change to `bench()` is expected. Verify that both `bench()` and `crop()` call `partial::decode_focus_crop(.., a.shot.focus, ..)`. In `crates/core/src/partial.rs`'s `#[cfg(test)]` block, add a `focus_point` test with `sensor_w/h` different from `w/h` (e.g. sensor 7008x4672, JPEG 3504x2336, point (3504,2336) → (1752,1168)) only if not already covered. Record in `learnings.md` that the todo item was stale.
    - Crop bitmap: in `show()` (`crates/app/ui/src/main.ts`), when `zoomed` is false, `crop?.bitmap.close(); crop = null;`. Keep the existing behaviour when zoomed. Check `draw()` / `cropViewportStale()` tolerate `crop === null`. Leave the "no files" reset paths alone unless trivially the same bug.
    - Panic guard: put the guard inside `decode_rgb` in `crates/core/src/decode.rs`, using `std::panic::catch_unwind(AssertUnwindSafe(..))` mapped to an `anyhow::Error`, mirroring `crates/core/src/scan.rs`'s pattern. CLI callers need no change. Add a test in `decode.rs`'s `mod tests` feeding garbage / truncated JPEG bytes and asserting `is_err()`; build the test against observed mozjpeg behaviour.

- [ ] Step 2: Fix the tooling items (`delete_merged_branches.sh` without `origin/HEAD`, `git:main` fast-forwarding local `main`, merger report wording)
  - Done when:
    - `tools/git/delete_merged_branches.sh` no longer dies with `fatal: ref refs/remotes/origin/HEAD is not a symbolic ref`; on a clone without the symbolic ref it resolves it via `git remote set-head origin -a`, or exits with a message naming that command.
    - `mise run git:main` fast-forwards the local `main` branch to `origin/main` in addition to detaching HEAD there, and does not fail when `main` is checked out in another worktree (it still detaches HEAD and says why `main` was not moved).
    - `.claude/agents/merger.md` and the task description in `mise.toml` describe the sync accurately, and merger's output does not claim the local `main` branch moved unless it did.
    - `mise run ci` is green.
  - Implementation approach:
    - Script: guard the `git symbolic-ref` lookup inside an `if` (the script runs under `set -euo pipefail`), try `git remote set-head origin -a`, then fall back to a clear error. Verify in a throwaway clone under the scratchpad, never in this worktree.
    - `git:main`: `git fetch origin main:main || echo "<notice>" >&2`, then `git checkout --detach origin/main`. Measure that the fetch fails cleanly when `main` is held by another worktree and succeeds otherwise.
    - `merger.md`: reword only the sentences claiming local main moved (intro, section 6, `MERGED` output). Check `.claude/skills/pr/SKILL.md` and `.claude/skills/develop/README.md` references and touch them only if they make the same false claim.

## Trade-offs and risks

- Todo item 2 is stale (PR #29 moved scaling into core); it is closed with a confirming test.
- The panic guard lives in `decode_rgb` (one place, protects future callers).
- `git:main` only fast-forwards `main` when no other worktree holds it; otherwise it prints a notice.
- The script attempts `git remote set-head origin -a` itself and falls back to a message.

## Progress

- (not started)
