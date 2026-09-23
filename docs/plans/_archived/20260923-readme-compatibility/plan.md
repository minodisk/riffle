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

# README compatibility list update

## Purpose

The `## Compatibility` section of `README.md` is behind what has actually been
verified and pins Lightroom lines to specific versions that are not needed for
the unverified entries. This work marks Ubuntu as verified (the user confirmed
Riffle runs on Ubuntu), drops the never-tried Fedora line, and removes the
version numbers from the two Lightroom lines so the list reads as a product
checklist rather than a snapshot of one machine.

## Steps

- [x] Step 1: Update the `## Compatibility` section of `README.md`
  - Done when:
    - Under `### OS` / `- Linux`, the list is exactly `- [x] Ubuntu` (the
      `- [ ] Fedora` line is removed; no other OS line changes)
    - Under `### Sidecar formats and software` / `- XMP`, the two Lightroom
      lines read exactly
      `- [ ] Adobe Lightroom (Windows; not Lightroom Classic)` and
      `- [ ] Adobe Lightroom Classic (Windows, Japanese UI)`; the Capture One
      line and `- [x] DxO PhotoLab 10` are unchanged
    - No other part of `README.md` changes
    - `mise run ci` passes (lychee link check, formatting, etc.)
  - Implementation approach:
    - File: `README.md`, the Linux list and the Lightroom lines of the
      `## Compatibility` section
    - Keep the existing checkbox-list style (`- [x]` / `- [ ]`, two-space
      nested indent) and touch nothing else in the section, including the
      Discussions / issue-template links
    - Commit as `docs(readme): update the compatibility list`
    - No other file needs a matching edit (see trade-offs)

## Trade-offs and risks

- `todo.md` (`### App: the Lightroom 9.5.1 round-trip check is still open`)
  and archived plans under `docs/plans/_archived/` still name "Lightroom
  9.5.1" and "Lightroom Classic 2026". These are historical records of which
  version was actually tested, not a compatibility list, so they stay.
- `docs/usage.md` mentions "Lightroom (XMP)" without a version, and
  `.github/ISSUE_TEMPLATE/os.yml` uses "Ubuntu 24.04" only as a placeholder
  example; neither lists Fedora or Lightroom versions, so nothing to sync.

## Progress

- (2026-09-23) Step 1 complete
