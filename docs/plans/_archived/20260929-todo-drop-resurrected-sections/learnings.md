# Learnings

- Before deleting each section, confirmed its fix is on `main`:
  `new ResizeObserver(fitViewer)` on `#viewer` in `crates/app/ui/src/main.ts`
  (#558), `fold`/`ignoreCase`/`respell` in `crates/app/ui/src/tree.ts` (#560),
  `fetchRoots` on focus in `crates/app/ui/src/folders.ts` (#558), and
  `drawnChildren` in `tree.ts` (#562).
- The four sections sat in one contiguous block of `todo.md`, so a single
  range deletion removed them.
- None of the other sections closed on 2026-09-28/29 had come back, and no
  `###` heading was duplicated (`grep '^### ' todo.md | sort | uniq -d` is
  empty). A parallel merge that removes a `todo.md` section is easy to undo
  in a "keep both sides" conflict resolution; checking that command after a
  conflicted `todo.md` merge would catch duplicates, though not a resurrected
  single copy.

## Deferred issues (todo candidates)

- Agents: `pr-conflict-resolver` resurrects deleted `todo.md` sections.
  Four parallel `develop` runs on 2026-09-29 each hit a `todo.md` rebase
  conflict and resolved it by keeping both sides, which brought back `###`
  sections that already-merged PRs (#558, #560, #562) had deleted.
  - Change: add a rule to `.claude/agents/pr-conflict-resolver.md` ("2.
    Manual resolution"): in `todo.md` (and similar tracking docs), when one
    side deleted a `###` section, keep it deleted rather than keeping both
    sides; after resolving, check `grep '^### ' todo.md | sort | uniq -d` for
    duplicated headings and compare the branch's deleted headings against
    `origin/main`.
  - Done when: the rule is merged into
    `.claude/agents/pr-conflict-resolver.md`.
