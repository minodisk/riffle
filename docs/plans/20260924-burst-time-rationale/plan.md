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

# Document why bursts are grouped by time, not by the camera's sequence numbers

## Purpose

Sony ARWs carry per-press drive-mode and sequence records (MakerNote
`ReleaseMode` 0xb049, `SequenceNumber` 0xb04a, and `SequenceFileNumber` in the
enciphered 0x9400 block). A reader of `docs/usage.md` may wonder why Riffle
groups bursts by the 1 s capture-time rule instead of those records. An
investigation of 2134 ILCE-7M5 ARWs from one shoot showed those numbers mark
shutter presses, not moments: the camera restarts them at the pre-capture
boundary and on a quick re-press, while the user counts such frames as one
burst (e.g. `_DSC2663`–`2686` was one burst to the user, but the camera
restarted numbering at 2673 and 2685; the 1 s rule reproduced the user's
grouping 2659–2662 / 2663–2686 / 2687–2689 exactly, both camera-number rules
split it). Recording this rationale in the user-facing docs keeps the
decision from being re-litigated. Docs only; no code changes. The detailed
investigation is in [investigation.md](./investigation.md).

## Steps

- [x] Step 1: Add the rationale to the "Bursts" bullet of `docs/usage.md`
  - Done when: the **Bursts** bullet in `docs/usage.md` contains one or two sentences stating that the grouping deliberately follows capture time rather than the camera's per-press sequence numbering, because a burst is one moment that may span several presses (pre-capture followed by the full press, or a quick re-press); `mise run ci` passes.
  - Implementation approach:
    - Insert the sentences right after the existing "frames shot within 1 s of the previous frame form a burst. The grouping follows capture order whatever the chosen sort, and Leica files ... are grouped by whole seconds." text, before the sentence about the tinted band, so the "what" and the "why" of the rule sit together.
    - Keep the user-facing tone of the surrounding bullets; do not mention tag IDs, ExifTool, or sample file numbers in usage.md. Wording approved by the user: "The grouping deliberately follows time rather than the camera's own per-press sequence numbering: a burst is one moment, and one moment often spans several presses, such as pre-capture frames followed by the full press, or a quick re-press."
    - Reflow the bullet to match the existing hard-wrapped style of the file so the formatter check in `mise run ci` stays green.

## Trade-offs and risks

- Placement: putting the "why" immediately after the rule reads naturally, but a reader skimming for keys may find the bullet long. Alternative is to append it at the end of the bullet. Either is fine; the plan prefers after the rule.
- Wording "the camera's own per-press sequence numbering" is Sony-specific in practice (Leica DNGs carry no such record); phrasing it generically avoids implying Leica has one.

## Progress

- (none yet)
