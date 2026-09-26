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

# Narrow the JPEG-only folder todo item to view-only scope

## Purpose

The `todo.md` item "App: open and preview JPEG-only folders in the strip"
(currently `todo.md:827-840`) still carries an open question, "decide which
features apply to JPEGs". The user has decided: culling does not need to work
for JPEGs. The item exists only so a user can check a
`sequence-jpeg-timestamps` `<folder>-sequenced/` output (order and times)
inside Riffle. Rewriting the item now records that decision so the eventual
implementation plan does not re-open it.

## Steps

- [x] Step 1: Rewrite the JPEG-only folder item in `todo.md` to a view-only scope
  - Done when:
    - The item's prose says the scope is view-only: a JPEG-only folder opens
      in the strip, shows thumbnails, the preview and the meta pane (EXIF
      rows), and is ordered by capture time so the sequenced order and times
      can be checked.
    - The item lists the explicit out-of-scope culling features: rating, the
      pick / reject flag, color label, sidecar (XMP / `.dop`) writes, and the
      focus cue / face detection.
    - The "Needs its own plan: decide which features apply to JPEGs" sentence
      and the "Decide which existing RAW-oriented features ... apply" checkbox
      are gone; the `Basis:` line (the archived sequence-jpeg-timestamps plan
      reference) stays.
    - No other todo item changes; no code changes.
    - `mise run ci` passes (Markdown fmt check).
  - Implementation approach:
    - Edit only `todo.md` lines 827-840. Keep the existing item style: `###`
      heading, a prose paragraph, `#### TODO`, `- [ ]` checkboxes with
      continuation lines indented six spaces and wrapped at ~80 columns.
    - Keep the heading text as is ("App: open and preview JPEG-only folders
      in the strip"); it already describes the view-only scope.
    - Suggested prose shape: first sentence states the current state (the
      strip lists ARW / DNG only and scan, index, sidecars and the focus cue
      assume RAW); second states the purpose (check the
      `sequence-jpeg-timestamps` `<folder>-sequenced/` output's order and
      times); third states the scope is view-only (strip thumbnails, the
      preview and the meta pane's EXIF rows, ordered by capture time) and
      names the out-of-scope culling features; then the unchanged `Basis:`
      sentence.
    - Suggested checkbox: one `- [ ]` item, e.g. let a JPEG-only folder open
      in the strip with thumbnails, the preview and the meta pane, ordered by
      capture time, without rating, flag, color label, sidecar writes or the
      focus cue.
    - Commit as `docs(todo): narrow the JPEG-only folder item to view-only`.

## Trade-offs and risks

- Whether the ordering should be by capture time (`DateTimeOriginal`) only or
  fall back to filename when EXIF is missing is left to the implementation
  plan; the todo item should say "ordered by capture time" and not commit to a
  fallback. If the caller wants the fallback stated, add it to the checkbox.

## Progress

- (none yet)
