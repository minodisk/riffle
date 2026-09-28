# Learnings: windows-gui-check-results

## Step 1

- The plan listed strip-scroll as in-flight work, but
  `strip-keep-scroll-on-rescan` had already merged as #528 by the time the
  step ran. Following the agreed grouping (merged work in one `###` item,
  in-flight work in another), its Windows check went into the merged item,
  whose heading became `... merged folder-tree, scan-wait and strip-scroll
  work`, and the in-flight item's heading dropped strip-scroll.
- The plan's duplicate-heading check `rg -n '^### ' todo.md | sort | uniq -d`
  can never print anything, because `-n` prefixes each line with its unique
  line number. Strip the numbers first (`rg '^### ' todo.md | sort | uniq -d`)
  for a check that actually detects duplicates; that form was also empty.
- No wait-for-scan, undo-trash-rejected, strip-scroll or resume-selection
  manual-check item existed in `todo.md` on this branch, so none was
  referenced instead of added.
- The Move Rejected to Trash macOS line's "mid-scan case writes its message
  to `#status`" was stale after `wait-for-scan` (a press during a scan is now
  held by `crates/app/ui/src/idle.ts`); only that phrase was reworded.
