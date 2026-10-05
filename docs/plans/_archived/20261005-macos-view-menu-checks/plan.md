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

# Drop the verified macOS View menu checks from todo.md

## Purpose

On 2026-10-05 the user ran `mise run tauri:dev` at main `9460fef5` on a real
Mac and passed all three macOS real-device checks in the
`### App: real-device checks for the View menu` section of `todo.md`: the
three `View` items sit above `Enter Full Screen` behind a separator with
`Alt+Cmd+ArrowLeft` / `ArrowRight` / `ArrowDown` and checks matching the
panes; each key toggles its pane exactly once with the check following,
`View > Filmstrip` twice hides / shows with correct checks, and with Settings
open `View > Folders` changes nothing; rebinding `toggleStrip` to
`ctrl+alt+s` shows it in the item after the menu rebuild, the checks survive
the rebuild, `Ctrl+Alt+S` toggles once, and `Reset` restores
`Alt+Cmd+ArrowDown`. The repo convention is to delete verified items (e.g.
PR #674), and the section's background paragraph still says the macOS branch
was "only reviewed by reading", which is now false. After this work the
section lists only what is still pending (Windows).

## Steps

- [x] Step 1: Delete the three verified macOS items and update the background paragraph of the View menu section in `todo.md`
  - Done when:
    - The three `- [ ] On macOS, ...` items (lines 234–244 at `9460fef5`) in
      `### App: real-device checks for the View menu` are gone; the five
      `- [ ] On Windows, ...` items are byte-for-byte unchanged.
    - The background paragraph no longer says the macOS `cfg` branch of
      `app_menu::build` / `refresh` was only reviewed by reading, nor that
      reverting muda's native toggle on click was confirmed only by reading
      its source. Instead it states that both were confirmed on a real Mac
      (2026-10-05, `mise run tauri:dev`) and remain unrun on Windows. The
      Windows-pending sentences (GUI checks never run; `view-pane-names`
      label check never run, covered by the first Windows TODO) stay.
    - No other section of `todo.md` changes; no code changes.
    - `mise run ci` passes.
  - Implementation approach:
    - Docs only. Edit only `todo.md`. Keep the `Files:` line, the `#### TODO`
      heading and the Windows items as they are.
    - Suggested rewrite of the affected sentences in the background
      paragraph (keep the surrounding text; wrap at the existing ~78 columns):
      "CI covers the build and the tests. The macOS `cfg` branch of
      `app_menu::build` / `refresh` and the revert of muda's native toggle on
      click (muda toggles the check before it sends the event) were confirmed
      on a real Mac on 2026-10-05 (`mise run tauri:dev`): the items, their
      accelerators, the checks, the single fire per key and the rebuild on a
      rebind all behaved. The Windows GUI checks (the menu, the checks, the
      accelerators) were never run: the step was ticked on the automated
      criteria only."
    - Commit as `docs(todo): drop the verified macOS View menu checks`
      (Conventional Commits, English), matching PR #674's style.
    - Mark this step done and add the Progress entry in the same PR.

## Progress

- (2026-10-05) Step 1 complete
