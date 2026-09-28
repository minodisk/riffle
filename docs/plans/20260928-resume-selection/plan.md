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

# Resume landing: select only the resumed file

## Purpose

Reopening a folder resumes at its last viewed file, but the first file stays
selected too: the status reads `N / M · 2 selected`, the first cell keeps a
faint `.selected` frame, and the next judgment key (rating, flag, label)
also hits the invisible first file (`targets()` returns every member once
the selection has more than one). Confirmed on a real Windows machine.

The cause, verified in `crates/app/ui/src/main.ts`:

- `openDirectory` selects the first file before the remembered one is
  known: `index = 0; selection = single(files[index]);`.
- The first `folder_entries` refresh in `refreshEntries` (and its `catch`
  branch for a folder with no index cache) calls
  `refilter(anchor, false, true)` with `anchor` set to the pending resume
  target and `force = true`.
- `refilter` moves `index` onto the resolved anchor and then does
  `selection = prune(selection, files, index)`. `prune`
  (`crates/app/ui/src/selection.ts`) only drops paths no longer listed and
  adds the focused file, so the selection becomes `{first, resumed}`.

PR #509 fixed the same class of bug for a single-file undo / redo with the
pure helper `restore` in `selection.ts`; this work follows that pattern.
After it, the resume landing leaves the selection equal to the landed file,
exactly as an arrow key would.

Other refresh paths were checked and do not need a change: the sort menu
(`refilter()`), a filter change (`refilter()` from the filter menu), the
resync after a folder change (`refilter(target, true)` with
`target = files[index]`), the scan-event `refreshEntries` calls (where
`pendingResume` is already `undefined`, so `anchor = files[index]`) and the
`faces-progress` refilter all anchor on the focused file. There the focus
either stays on the same file (selection unchanged) or, when the filter
hides it, `prune` drops it and adds the nearest passing neighbor, which for
a single selection is exactly `single(neighbor)`. Only the resume landing
moves the focus onto a different, still-visible file while the previously
selected one remains listed.

## Steps

- [x] Step 1: Collapse the selection to the landed file when a pending resume resolves
  - Done when:
    - A pure helper in `crates/app/ui/src/selection.ts` decides the
      selection `refilter` ends with, from the current selection, the
      visible `files`, the new focused index and whether this refresh
      resolved a pending resume target: when it did, it returns
      `single(files[focused])` (the file the landing put the focus on, or
      its nearest passing neighbor when the filter hides the remembered
      file); otherwise it returns `prune(selection, files, focused)`. The
      name and doc comment follow the module's style (short verb-ish name
      like `click`, `extend`, `prune`, `restore`; a comment stating the
      rule and why: `openDirectory` selected `files[0]` before the
      remembered file was known).
    - `refilter` in `crates/app/ui/src/main.ts` uses it in place of the
      `selection = prune(selection, files, index);` line that follows
      `anchorAfterFilter` (the non-empty branch), passing `force`, which is
      already the "this refresh resolves a pending resume" flag (see the
      comment above `refilter` and `mustReshow(force, ...)`). The
      `files.length === 0` branch keeps its `prune`. `refreshEntries` and
      `openDirectory` are not edited.
    - `crates/app/ui/src/selection.test.ts` gets a `describe` for the
      helper with at least: the regression (`single("/a")` selected, focus
      landed at the index of `/b`, resumed -> exactly `single("/b")`); a
      resume whose remembered file is hidden and landed on a neighbor ->
      `single(neighbor)`; a non-resume refresh keeps `prune`'s result (a
      multi-selection with a visible anchor stays, hidden members are
      dropped, the focused file is added). Optionally a `prune` test that
      pins the reason for the helper: `prune(single("/a"), files, 1)` is
      `{"/a", "/b"}`.
    - The `docs/agents/tauri-app.md` entry on the selection invariant
      ("Pruning the selection after a filter/sort/judgment change happens
      only in `refilter` ...") gets one sentence noting the resume-landing
      exception and the helper, next to the #509 note on `restore`.
    - `mise run ci` passes.
  - Implementation approach:
    - Gate on `force`, not on "did the focus move": the other `refilter`
      callers anchor on `files[index]`, and collapsing whenever the focus
      moved would also collapse a multi-selection whose focused member the
      filter just hid, which is existing intended behavior. Do not rename
      the `force` parameter; the change to `main.ts` should be one line
      plus an import, to stay clear of the strip-scroll and scan-wait
      sessions that touch `refreshEntries` / `strip.setFiles`.
    - `paintSelection()` already follows the assignment in `refilter`, and
      `show()` (forced by `mustReshow`) re-renders the status, so no extra
      paint or status call is needed.
    - No compare-mode guard is needed (unlike #509): `openDirectory` calls
      `stopComparing()` before any resume can land.
    - Manual check on the desktop app: in folder A focus a file other than
      the first, open folder B, return to A -> the status shows no
      `selected` suffix, only the landed cell has `.selected`, and a
      rating key changes only that file. Repeat with a judgment filter that
      hides the remembered file (lands on a neighbor) and with a folder that
      has no index cache (the `catch` branch).

## Trade-offs and risks

- **Helper in `selection.ts` (chosen) versus `resume.ts`.** `resume.ts`
  holds the other pure resume-landing decisions (`firstEntriesAnchor`,
  `mustReshow`), so a `hadPendingResume`-keyed helper would also fit there.
  `selection.ts` is chosen because #509's `restore` set the precedent for
  "a focus move that should also move the selection", the helper wraps
  `prune` / `single` which live there, and its test file already covers
  them.
- **Apply inside `refilter` (chosen) versus after it in `refreshEntries`.**
  Resetting in `refreshEntries` would need two edits (the `then` and the
  `catch` branch) plus a `paintSelection()` call each, in the function two
  other sessions are editing. `refilter` already receives the flag and
  already paints.
- **A selection made during the open gap is collapsed.** Between
  `list_arw` returning and the first `folder_entries` landing the user can
  arrow or shift-select; today the landing already yanks the focus to the
  remembered file, and after this change it also collapses that selection.
  This is consistent with the focus jump (an arrow key would do the same)
  and the gap is short; accepted.
- **"Fails before" of the regression test.** As with #509, the new helper's
  tests fail before the change because the export does not exist;
  `refilter` itself is not unit-testable. The optional `prune` test pins
  the cause but passes before and after.

## Progress

- (none yet)
