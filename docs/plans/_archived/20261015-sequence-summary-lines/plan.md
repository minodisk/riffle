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

# Sequence preview summary: written count and capture-time changes

## Purpose

The Sequence JPEG Timestamps dialog's summary line `N of M files get a new
time` counts the Rust `changed` flag, which is true when any of
`DateTimeOriginal` / `DateTimeDigitized` / `DateTime` differs from the assigned
time. Exported JPEGs carry `DateTime` (ModifyDate) = export time, so every file
counts as changed (`247 of 247 files get a new time`) while only the rows whose
`DateTimeOriginal` changes are highlighted green (`rowParts` diffs
`row.old` against `row.new`). The line contradicts the list above it.

Once done, the dialog states two things the user actually wants to know: how
many files are written to the output folder (every readable file, changed or
not, since the output is a complete copy), and how many of them get a new
capture time (`DateTimeOriginal` changes), which matches the green rows exactly.
The row dimming follows the same rule, so a row is dimmed exactly when it is
not green. The Rust `changed` semantics and the write (all three tags synced)
stay as they are.

## Decisions (user-approved)

- Two `<p>` elements in `crates/app/ui/index.html`, one fact per line, like
  the `Source:` / `Output:` lines.
- Future tense for the written line, consistent with the rebuild notice's
  `will be rebuilt`: `247 files will be written to Output-sequenced`, then
  `68 of them get a new capture time`.
- The row dimming in `showSequencePreview` switches from `!row.changed` to
  `row.old === row.new`, so dimmed = not green.

## Steps

- [x] Step 1: Replace the preview's summary line with the written count and the capture-time change count, and dim by the capture time
  - Done when:
    - `crates/app/ui/src/sequence.ts` exports pure text helpers that produce,
      from `preview.rows` and `preview.output_dir`:
      1. the written line, e.g. `247 files will be written to export-sequenced`
         (`rows.length`; failed files are not rows, so they are excluded; the
         folder is `baseName(preview.output_dir)`), with `1 file will be
         written to …` in the singular;
      2. the capture-time line, e.g. `68 of them get a new capture time`,
         counting rows with `row.old !== row.new` (the green rows), not
         `row.changed`; `1 of them gets a new capture time` in the singular and
         a natural sentence for zero (e.g. `0 of them get a new capture time`).
    - `changedLine` is removed and its imports in `main.ts` /
      `sequence.test.ts` replaced.
    - `index.html` has two paragraphs (e.g. `sequence-written` and
      `sequence-count`) filled in `showSequencePreview` in `main.ts`.
    - `main.ts` dims a row (`unchanged` class) when `row.old === row.new`.
    - `crates/app/ui/src/sequence.test.ts` covers the plural and singular of
      both lines, the zero case, that a row with `changed: true` but
      `old === new` (the ModifyDate-only case) is not counted, and the output
      folder base name for both `/` and `\` paths. The old `counts the files
      that get a new time` test is replaced.
    - `docs/humans/usage.md` (the **Preview** bullet) and
      `docs/humans/usage.ja.md` describe the two lines and the dimming rule.
    - `mise run ci` passes.
  - Implementation approach:
    - Frontend-only; `crates/core/src/sequence.rs` and
      `crates/app/src/sequence.rs` are not touched. `SequenceRow.changed`
      stays in the type even if the frontend no longer reads it.

## Progress

- (2026-10-15) Step 1 complete
