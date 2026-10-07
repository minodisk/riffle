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

# Eyes row on one axis

## Purpose

The meta pane's Analysis group shows the closed-eyes judgment as
`Eyes: Closed (81%)` or `Eyes: Open (93%)`: the percentage backs whichever
word was judged, so a reader comparing two files has to flip axes in their
head. Internally there is one value, the closed probability `p` from
`crates/core/src/eyes.rs` (`closed` is exactly `p >= 0.5`). Showing only the
open probability `1 - p` under a label that names it (`Eyes open: 93%` /
`Eyes open: 19%`) puts every file on the same scale. The judgment logic, its
constants and the `eyes_of` return shape do not change.

## Steps

- [x] Step 1: Show the Eyes row as the open probability under an `Eyes open` label, and update the docs
  - Done when:
    - `metaGroups` emits `{ label: "Eyes open", value: "NN%" }` where `NN` is
      the rounded open probability (`1 - probability`), with no `Closed` /
      `Open` word; the row is still left out when `eyes` is `null` /
      `undefined`, and still follows `AF eye in focus`
    - `meta.test.ts` pins: a closed judgment (`probability: 0.814` → `19%`),
      an open one (`0.07` → `93%`), and the boundary `probability: 0.5`
      (state `closed`) → `50%`, plus one case just either side of 0.5 showing
      the percentage follows `1 - probability` (not the state)
    - `docs/humans/usage.md` (the `Eyes` sentence around line 252 and the
      `Eyes` row reference at line 202), `docs/humans/usage.ja.md` (line 19),
      `docs/humans/performance.md` (around line 532), and
      `docs/humans/performance.ja.md` (line 295) describe the new display,
      each EN/JA pair updated in the same PR; `README.md` (around lines
      120-127) and `README.ja.md` (line 57) name the row `Eyes open` and say
      it is the probability the eyes are open; `CLAUDE.md`'s `meta.ts`
      description (line 117, "the `Eyes` state") says the `Eyes open`
      probability
    - `mise run ci` passes
  - Implementation approach:
    - `crates/app/ui/src/meta.ts`: replace `eyesValue` with a formatter of
      `1 - eyes.probability` as a percentage, consistent with `focusPercent`
      (`Math.round(x * 100)` + `%`); rename the label at line 118 to
      `Eyes open`; update the comment above `eyesValue` and the `metaGroups`
      doc comment ("the eye state judged when the file was shown" →
      the open probability). `eyes.state` is then unused by `meta.ts`; leave
      the `Eyes` type in `eyes.ts` as is (the command's shape is out of
      scope)
    - Use `Math.round((1 - p) * 100)` rather than `100 - Math.round(p * 100)`
      and pin it by the test, since the two differ at some inputs
      (e.g. p = 0.505 gives 50 vs 49)
    - Docs: keep the surrounding explanation (which face is judged, the Face
      Landmarker mesh, when the row is left out, looking-down eyes count as
      closed); only the display sentence changes, e.g. "`Eyes open` is how
      likely it is that the eyes ... are open, as a percentage; at 50% or
      below Riffle counts them closed". State the 50% threshold once in
      `usage.md` / `usage.ja.md` only. Mirror the wording in the Japanese
      files
    - No change to `crates/core/src/eyes.rs`, `crates/app/src/commands.rs`
      or `eyes.ts`

## Trade-offs and risks

- The 50% closed threshold is stated once in `usage.md` / `usage.ja.md`, so
  the reader knows what the face mesh overlay and future filters mean by
  closed.
- Label `Eyes open` (not `Open eyes`) keeps subject-first naming next to
  `AF eye in focus`.
- Rounding near the boundary: with no word in the value there is nothing for
  the percentage to contradict, so `Math.round((1 - p) * 100)` is enough; a
  reader seeing `50%` on a file Riffle counts as closed is the only
  ambiguity, which the threshold sentence in the docs resolves.

## Progress

- (none yet)
