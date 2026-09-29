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
