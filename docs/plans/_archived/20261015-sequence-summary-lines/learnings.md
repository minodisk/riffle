# Learnings

## Step 1

- The helpers are `writtenLine(rows, outputDir)` and `captureTimeLine(rows)`
  in `crates/app/ui/src/sequence.ts`; the capture-time count and the row
  dimming both compare `row.old` with `row.new`, the same comparison
  `rowParts` uses for the green part, so the line, the green rows and the
  undimmed rows always agree. `SequenceRow.changed` stays in the type but the
  frontend no longer reads it.
- The first `mise run fmt` in a fresh worktree exited 1 before
  `node_modules` existed, so `mise run ci`'s lint then flagged the unformatted
  test file; re-running `mise run fmt` after `ci`'s `pnpm install` fixed it.
