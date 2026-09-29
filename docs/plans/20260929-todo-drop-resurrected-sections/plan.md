<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Drop todo.md sections resurrected by parallel conflict resolutions

## Purpose

Four PRs merged in parallel on 2026-09-29 (#558, #560, #562, #563) each
removed a fixed item from `todo.md`, but the conflict resolutions of the later
ones kept "both sides", so four already-fixed sections came back on `main`:

- `### App: a horizontal strip's scrollbar can grow `#film` and shrink the viewer without a resize event` (fixed by #558)
- `### App: the folder tree does not reveal a differently-cased open path` (fixed by #560)
- `### App: the folder tree's roots don't pick up a volume mounted after launch` (fixed by #558)
- `### App: a folder reachable from two tree roots is expanded and highlighted twice` (fixed by #562)

This plan removes them again.

## Steps

- [x] Step 1: Remove the four resurrected sections from `todo.md`
  - Done when:
    - The four `###` sections above (each with its body and `#### TODO`
      block) are gone from `todo.md`; every other section is untouched,
      including the section #560 added ("App: tree.ts's
      `relation`/`rebase`/`renameFolder` stay case-sensitive on
      case-insensitive filesystems").
    - Also confirm none of the other sections closed on 2026-09-28/29 came
      back ("`mise run fmt` does not work on Windows", `renameAllowed`,
      `bench` JPEG, unreadable folder, `scan-state`, `reveal`'s listing
      failures, always-null Maker note fields, old shortcut override
      conflicts) and that no `###` heading is duplicated; remove any that did.
    - `mise run ci` passes.
  - Implementation approach: delete the sections with the Edit tool; before
    deleting each, confirm from `git log -p` / the merged PRs that the fix is
    on `main` (e.g. `ResizeObserver` in `crates/app/ui/src/main.ts`,
    `respell` / `ignoreCase` in `crates/app/ui/src/tree.ts`, the focus
    `fetchRoots` in `crates/app/ui/src/folders.ts`, `drawnChildren` in
    `tree.ts`).

## Trade-offs and risks

- None beyond deleting the wrong section; each deletion is checked against
  the code on `main` first.

## Progress

- (none yet)
