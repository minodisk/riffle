# Learnings

## Step 1

- A case-preserving perl pass over an explicit stem list (not a blanket
  `s/ise/ize/`) was enough; the protected literals (`aria-labelledby`, the
  quoted `"cancelled"` / `"CANCELLED"` GitHub conclusions, the
  `run Release cancelled` fixture, and the backticked `cancelled` row in
  `merger.md`) were swapped for placeholders before the pass and restored
  after it.
- The first pass's `-ise` rule was case-sensitive and missed `Serialise` /
  `Normalise` at the start of doc comments; re-run with `/i`.
- The plan's acceptance grep does not cover every British form. A broader
  sweep (`-is(e|ed|es|ing|ation)`, `-tre`, `-our`) found `capitalise(d|s)`,
  `localised`, `canonicalise(d|s)`, `virtualised` / `virtualisation`,
  `quantisation`, `rasterisation`, `authorised`, and `millimetres`, which were
  converted too (including the test name
  `reads_a_lightroom_label_color_over_the_localized_label`).
- `todo.md` cites `docs/plans/_archived/20260919-undo-judgements/learnings.md`.
  The pass rewrote it to `judgments`, which points at a directory that does
  not exist, so it was restored. It is the one remaining acceptance-grep hit
  outside `docs/plans/` beyond the allowed exceptions, until Step 2 renames
  that directory.
- The settings-store key accesses (`store.get` / `store.set` in
  `crates/app/src`) are identical before and after the pass.

## Deferred issues (todo candidates)

- Step 2 must also rename `docs/plans/_archived/20260919-undo-judgements/`
  (not listed in the plan, which only names the `filter-colour-label`
  directories) with `git mv` and update the path cited in `todo.md`
  (the Source line of the undo learnings entry); otherwise the acceptance
  grep over the whole repository keeps matching `judgement`. Basis: Step 1
  implementation, `todo.md`, `docs/plans/_archived/20260919-undo-judgements/`.
