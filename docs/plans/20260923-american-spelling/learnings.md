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
- `docs/agents/tauri-app.md` cites
  `docs/plans/_archived/20260919-undo-judgements/learnings.md`. The pass
  rewrote it to `judgments`, which points at a directory that does not exist;
  restore `undo-judgements` there for now. It is the one remaining
  acceptance-grep hit outside `docs/plans/` beyond the allowed exceptions,
  until Step 2 renames that directory.
- The settings-store key accesses (`store.get` / `store.set` in
  `crates/app/src`) are identical before and after the pass.

## Step 2

- The pass needs `(?<![a-z])` / `(?![a-z])` instead of `\b` at the word
  edges: `_` is a word character, so `\b` missed test names quoted in plans
  such as `cancelling_after_the_first_batch_keeps_what_was_written` and
  `scan_started_serialises_the_fields_the_frontend_reads`.
- `aria-labelledby` also appears in
  `docs/plans/_archived/20260920-settings-tabs/plan.md`; it was protected the
  same way as in Step 1.
- Besides the directories the plan names, two review-history directories
  carried a British form in their path (`feature/undo-judgements`,
  `feature/judgement-dots-in-strip`); they were renamed with `git mv` too so
  the directory names match the converted text.
- This plan's own `plan.md` / `learnings.md` and
  `review-history/american-spelling-step-1/` were left as they are (they
  quote the British forms and the old paths); the plan's exception list now
  names them.
- `docs/agents/tauri-app.md` had picked up a new `colour` from the
  context-menu-rating-label wrap-up after Step 1; converted here.
- `maths` (British) became `math` in four plan / review files.
- No intra-doc link or heading anchor contained a converted word.

## Deferred issues (todo candidates)

- Step 2 must also rename `docs/plans/_archived/20260919-undo-judgements/`
  with `git mv` and update the path cited in `docs/agents/tauri-app.md`
  (the Source line of the undo-anchoring learnings entry); otherwise the
  acceptance grep over the whole repository keeps matching `judgement`. This
  is now listed in the plan's Step 2 "Done when". Basis: Step 1
  implementation, `docs/agents/tauri-app.md`,
  `docs/plans/_archived/20260919-undo-judgements/`.
