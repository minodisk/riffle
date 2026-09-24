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

# Explain both ways to match the color label names with Lightroom

## Purpose

The Sidecar tab of the settings window (`crates/app/ui/settings.html`, the
`#label-names` block) says the `xmp:Label` names "must match the names in
Lightroom's color label set" but does not say how to get there. A user can
either type their own Lightroom color label set into these fields, or change
Lightroom's color label set to the names in these fields. Spelling out both
options in the note, including the Lightroom Classic menu that edits the color
label set, removes the guesswork. Wording only; no behavior, storage or
default-name change.

## Steps

- [x] Step 1: Extend the note above the color label name fields
  - Done when:
    - The `<p>` inside `<div id="label-names">` in
      `crates/app/ui/settings.html` still states that the names must match
      Lightroom's color label set and additionally says there are two ways to
      make them match: edit the names here to match the user's own Lightroom
      color label set (the "Lightroom default (Japanese)" and "Reset to
      English" buttons fill in Lightroom's built-in sets), or change
      Lightroom's color label set to the names in these fields.
    - For the second option, the note names the menu: in Lightroom Classic,
      Metadata > Color Label Set > Edit. It names Lightroom Classic explicitly,
      since whether Lightroom (the cloud-based desktop app) can edit color
      label names is unverified.
    - The note does not imply a fixed Riffle-owned name list; Riffle writes
      whatever the fields contain.
    - Only that paragraph changes. No change to `crates/app/ui/src/settings.ts`,
      the `labelNames` store key, the default names, the buttons, or the
      READMEs.
    - Text is English, plain and short; keep the existing
      `<code>xmp:Label</code>` markup and the file's formatting so
      `pnpm exec vp fmt` produces no diff.
    - `mise run ci` passes.
  - Implementation approach:
    - Nothing in `crates/app/ui/src` or the tests reads the paragraph's text
      (only the `label-names` id and the two button ids are referenced), so
      this is a pure HTML edit.
    - The `#label-names` block is shown only when the XMP format is selected,
      so the note can name Lightroom directly.

## Trade-offs and risks

- The README only says the `xmp:Label` name is configurable per color and does
  not repeat the "must match" note, so it stays untouched (user decision).
- The menu path is given for Lightroom Classic only.

## Progress

- (2026-09-25) Step 1 complete
