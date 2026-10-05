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

# Record the verified macOS PhotoLab Uuid lookup checks

## Purpose

PR #687 added the PhotoLab Uuid lookup on macOS
(`../_archived/20261005-photolab-uuids-macos/plan.md`) and left three
real-device checks and the external-volume work in `todo.md`. Two of the
checks (and the "patch a PhotoLab-written `.dop`" case) were run by hand on
2026-10-05 with PhotoLab 10 on macOS and passed. This plan drops the finished
items from `todo.md` and records the results, plus three side observations,
in `../../agents/photolab.md`, so the next agent does not re-run or re-discover
them. Docs only; no code changes.

## Steps

- [x] Step 1: Drop the two finished todo items and record the macOS hand-run results in `docs/agents/photolab.md`
  - Done when:
    - In `todo.md`, section "### App: PhotoLab Uuid lookup on macOS: real-device checks and external volumes", the first two `- [ ]` items (the `browse` folder pick check and the never-opened-folder check) are removed; the virtual-copy master-order item and the external-volume item remain unchanged.
    - That section's background paragraph no longer says the lookup was "checked only against fixture databases and a read of the user's PhotoLab 10 database"; it says the lookup was verified by hand on macOS (PhotoLab 10, 2026-10-05) and points to `docs/agents/photolab.md`, while keeping the `/Volumes` sentence.
    - `docs/agents/photolab.md`, section "## PhotoLab matches `.dop` items by Uuid; ...", gains a macOS hand-run list next to the Windows one, in the same bullet style ("case: result"), covering:
      1. registered image (PhotoLab had opened the folder, no `.dop`) + fresh `.dop` from Riffle: Source / Item Uuids equal `ZDOPSOURCE.ZUUID` / `ZDOPINPUTITEM.ZUUID`; pick shown on the master (`ZSHOULDPROCESS = 2`), still one `ZDOPINPUTITEM` row, no virtual copy.
      2. folder PhotoLab never opened + Riffle pick first, then opened in PhotoLab: imported as one picked master, no virtual copy.
      3. patching a PhotoLab-written `.dop` with PhotoLab quit (Riffle wrote `Rating = 4`, `ShouldProcess = 0`, fresh `Date` / `ModificationDate`): on reopening, both applied to the master (`ZRANK 4`, `ZSHOULDPROCESS 2`), no virtual copy.
    - The same section (or the macOS bullets just below it) records three notes:
      - the `ZSHOULDPROCESS` encoding in the macOS database differs from the `.dop`'s `ShouldProcess`: DB `1` = unflagged, `2` = pick; `.dop` `0` = pick, `1` = reject, `2` = unflagged (so do not read one as the other when checking the database).
      - macOS PhotoLab 10 writes a rating change to the `.dop` only when leaving the folder, not immediately, so Riffle shows the old rating until then.
      - an unexplained observation, not reproduced and not pursued: with PhotoLab running on another folder while Riffle set rating 4 + pick on a registered image whose `.dop` PhotoLab had written, PhotoLab later wrote the `.dop` back with `Rating 4` but `ShouldProcess = 2` (pick lost); PhotoLab's `ModificationDate` / `Date` in that `.dop` (12:42:07 / 12:42:17 local) were after Riffle's last write (12:41:52). Phrased as an observation, not a `- [ ]` item.
    - The section's trailing "Source:" bullet gains this plan's path (e.g. `docs/plans/_archived/<this plan folder>/plan.md`, Step 1), consistent with the existing source bullets.
    - `mise run ci` passes (lychee, typos, fmt).
  - Implementation approach:
    - Files: `todo.md` (lines ~957–979 in the current tree), `docs/agents/photolab.md` (the "Hit" section starting at line 72; the Windows list is lines 78–97, the macOS schema bullets lines 116–130).
    - Match the Windows list's style: a short intro line ("Hand-run with PhotoLab 10 on macOS, 2026-10-05, a dev build with PR #687's lookup:") followed by `- case: result.` bullets. Keep the Windows list untouched.
    - The existing bullet at line 122–123 says the master is "taken as the lowest `Z_PK` (unverified: no virtual copy was in the database)". Leave it as is; that check is the todo item that remains.
    - Place the encoding, write-on-leaving-folder and unexplained-observation notes as bullets in the macOS part of the bulleted list (after the `COLLATE BINARY` / mapping bullets or right after the new hand-run list), each one self-contained; no new heading.
    - Do not add any new `- [ ]` items anywhere; the hand checks are done and the unexplained observation is deliberately not a todo.
    - Run `mise run ci`; watch `typos` (e.g. `ZSHOULDPROCESS`, `ZRANK`, `ZDOPINPUTITEM` are already present in the file so they pass) and `lychee` for any new relative link in `docs/plans/**` (write links relative to the plan folder, e.g. `../../agents/photolab.md`).

## Trade-offs and risks

- Where the three notes live: inside the hand-run list vs. as separate bullets in the schema list below. The plan puts them as separate bullets so the hand-run list stays "case: result" like the Windows one; either is acceptable, pick what reads best.
- The unexplained observation could be read as a bug worth a todo. The user chose not to chase it, so it is recorded only as an observation; if it reproduces later, a todo can be added then.

## Progress

- (2026-10-05) Step 1 complete
