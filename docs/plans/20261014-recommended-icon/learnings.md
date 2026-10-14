# Learnings: recommended-icon

## Step 1

- The strip's base `.cell span` rule (absolute, full width, padding) matches
  every descendant span, so the face and sparkles spans inside the new
  `.cell span.marks` flex row needed `position: static; width: auto;
  padding: 0` to sit in the row. The row took over `.cell span.tier`'s
  placement and halo; the `[hidden] { display: none }` rule moved to the
  row's children, so a hidden face icon leaves no gap before the sparkles.
- `refilter` is the one place that rebuilds the strip (`strip.setFiles`
  clears the per-index sets), so `applyRecommended()` sits there next to
  `applyTiers()`; `refreshRanks` and `setRanking`'s off branch call it too.
  A folder open (`strip.setFiles` in the open path) starts with an empty
  `fileRanks`, so nothing needs applying until `refreshRanks` answers.

## Deferred issues (todo candidates)

- Pending manual check (Step 1 of this plan, ticked on the automated
  criteria): on the desktop app (Windows), open a folder with bursts while
  AI analysis is on and the embeddings are in. Expected: the cell of each
  burst's first-ranked frame shows the `sparkles` icon in the bottom-left
  row, right of the face icon when the frame is also good (alone at the
  left edge otherwise), not colliding with the burst count `n/m` at the
  bottom-right; a lone frame shows none. Then click `Refit Ranking` in
  Settings > Culling (icons may move), change the sort and a filter (icons
  stay on the same files), and turn AI analysis off in Settings (every icon
  disappears). Basis: the plan's Step 1 Done-when; files
  `crates/app/ui/src/strip.ts`, `crates/app/ui/src/main.ts`,
  `crates/app/ui/style.css`.
