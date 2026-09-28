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

# Record the outstanding manual GUI checks for the resume-selection fix

## Purpose

PR #535 (`fix(app): select only the resumed file when a folder reopens`,
plan `docs/plans/_archived/20260928-resume-selection/plan.md`) collapsed the
selection to the landed file when a pending resume resolves. Its Step 1
"Implementation approach" specified a manual check in the desktop app that
no agent session could run, since the GUI cannot be driven from one. The
repository records such unrun checks as `todo.md` entries (e.g. "App: the
manual GUI check of `scan-progress` `ready` is outstanding", "App: burst
grouping's manual checks are still open") so a human can run them later and
tick them off. This work adds that entry; nothing else changes.

## Steps

- [x] Step 1: Add the `todo.md` entry for the resume-selection manual checks
  - Done when:
    - `todo.md` has a new `### App: the manual GUI checks for the resume
      landing's selection are still open` heading (or equivalent wording in
      the style of the neighbouring `### App: ...` manual-check headings)
      under `## Cross-cutting / other`.
    - The paragraph under it gives the provenance (Step 1 of
      `docs/plans/_archived/20260928-resume-selection/plan.md`, merged as
      PR #535; the check could not be run from an agent session because the
      GUI cannot be driven from one; the bug was found by the user on a real
      Windows machine) and the files: `crates/app/ui/src/selection.ts`,
      `crates/app/ui/src/main.ts`.
    - A `#### TODO` checklist with unchecked items covering:
      - On Windows, in folder A focus a file other than the first, open
        folder B, return to A: the status shows no `selected` suffix, only
        the landed cell has `.selected`, and a rating key changes only that
        file.
      - The same on macOS.
      - The same with a judgment filter that hides the remembered file, so
        the landing falls on a neighbor: exactly the neighbor is selected.
      - The same with a folder that has no index cache (the `catch` branch
        of `refreshEntries` in `main.ts`).
    - `mise run ci` passes (this includes the markdown lint / format the
      repository runs over `todo.md`).
    - No file other than `todo.md` and this plan changes.
  - Implementation approach:
    - Match the existing manual-check entries' shape exactly: `### App: ...`
      heading, one short paragraph (provenance, why it was not run, `Files:`
      list in backticks), blank line, `#### TODO`, blank line, `- [ ]`
      items wrapped at the file's column width with the 6-space continuation
      indent the other entries use. Platform-specific checks are separate
      checkboxes, as in the burst grouping and Clear Cache entries, so the
      Windows and macOS runs can be ticked independently.
    - Position: insert the entry directly before `### App: Open in Terminal
      from the folder tree's context menu`, which leaves it after the
      `crates/app/src/sequence.rs` comment entry (the `### App: hand-check
      the sequence-run output-folder reveal on Windows` entry the plan
      originally anchored on had already been resolved and removed from
      `todo.md` before this work; see `learnings.md`). Do not reorder or
      edit any other entry.
    - Refer to the archived plan by path in backticks, as the
      `scan-progress` entry does, rather than as a Markdown link, so lychee
      has nothing to resolve.
    - Windows note for the implementation agent: write the entry with the
      Edit tool, not a Bash heredoc (see the `Agents: Bash-tool heredocs on
      Windows mangle doubled backslashes` entry at the end of `todo.md`).

## Trade-offs and risks

- **Position of the entry.** Placing it directly before `### App: Open in
  Terminal from the folder tree's context menu` (chosen) keeps it next to
  the most similar Windows manual-check item, after the
  `crates/app/src/sequence.rs` comment entry that now occupies the spot the
  `sequence-run reveal` entry originally held; appending at the end of the
  file would follow the "newest last" habit of the most recent entries.
- **Platform split.** Listing macOS as its own checkbox records a check
  nobody may run soon; folding it into the Windows item would lose the
  ability to tick Windows alone. The split follows the existing entries.

## Progress

- (2026-09-29) Step 1 complete
