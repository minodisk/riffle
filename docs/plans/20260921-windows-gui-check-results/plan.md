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

# Record the Windows manual GUI check results in todo.md

## Purpose

A round of manual GUI checks was run on Windows via `mise run tauri:dev`.
`todo.md` still lists several of those checks as open and does not mention two
Windows-specific defects the run surfaced. This work brings `todo.md` in line
with what was actually verified: closing what passed, narrowing what remains
to the platforms not yet run, and recording the two new findings so they can
be picked up as their own work.

## Steps

- [x] Step 1: Update `todo.md` with the Windows GUI check results
  - Done when:
    - The item `### App: the viewer empty-state manual checklist is still open`
      is removed entirely (heading, context paragraph, `#### TODO` list):
      every point in it passed on Windows.
    - The item `### App: the real-device checks for the File menu accelerators
      are still open` records that all six checks passed on Windows with
      `Ctrl` in place of `Cmd` (`Ctrl+O` opens the picker exactly once;
      `Shift+Ctrl+O` fires once, judged only from a single status line, which
      a second identical message would overwrite; rebinding `open` updates
      the menu accelerator and kills `Ctrl+O`; the menu shows correctly with
      the settings window open and steals no focus; a capturing shortcuts row
      swallows `Ctrl+O` / `Shift+Ctrl+O`; both File items work by mouse). Its
      remaining TODO scope is narrowed to macOS and Linux (the Windows half of
      the "Windows and Linux" checkbox is closed).
    - A new item is added for `open_in_photolab` in
      `crates/app/src/commands.rs`: it only reads `/Applications` for
      `DXOPhotoLab<N>.app`, so on Windows "Open in DxO PhotoLab" always fails
      with `Could not open PhotoLab: ... (os error 3)` (path not found) while
      the menu item is still shown. Its TODO is to decide between a
      Windows / Linux PhotoLab lookup and hiding or disabling the item on
      those platforms.
    - A new bug item is added: on Windows, Clear Cache is refused with
      `a scan is running; wait for it to finish` after pressing Clear in the
      confirm dialog, with no scan running beforehand. It states the likely
      cause as unverified: `crates/app/ui/src/main.ts` subscribes
      `window.__TAURI__.event.listen("tauri://focus", resync)` globally, so
      the settings window regaining focus when the dialog closes triggers a
      main-window rescan, and `clear_index`'s second `scanning()` check (after
      the sidecar flush, `crates/app/src/commands.rs`) refuses. Proposed fix:
      scope the focus listener to the main window. TODO: confirm the cause,
      fix it, and add a regression test if feasible.
    - The item `### App: the Clear Cache button's manual GUI verification is
      still open` records that on Windows (1) passed and the dialog in (2)
      appears immediately, and that the Cancel half of (2) and checks (3)-(7)
      are blocked by the Clear Cache refusal bug above.
    - `mise run ci` passes.
  - Implementation approach:
    - Only `todo.md` (repo root) changes.
    - Match the existing style exactly: `### App: ...` heading, a context
      paragraph (mentioning the origin and ending with `Files: ...`), then
      `#### TODO` with `- [ ]` checkboxes, wrapped like the neighbouring
      items. Do not touch unrelated items.
    - Place the two new items near the related existing items (the PhotoLab
      item next to the File menu accelerators item; the Clear Cache bug next
      to the Clear Cache verification item).
    - Keep the accelerator item's macOS checkbox as is and reword the second
      checkbox to Linux only, adding a sentence to the context paragraph
      about the Windows pass and the single-status-line caveat for
      `Shift+Ctrl+O`.
    - Write everything in English.

## Trade-offs and risks

- The Clear Cache root cause is a hypothesis; the todo item must label it as
  unverified so the follow-up does not skip confirmation.
- For the PhotoLab item, the decision (platform lookup vs hide/disable) is
  intentionally left open in the todo; this plan does not pick one.

## Progress

- 2026-09-21: Step 1 done: todo.md updated with the Windows GUI check results.
