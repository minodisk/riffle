# Learnings

## Step 1

- The new test
  `a_pick_set_during_a_switch_lands_in_the_dop_and_leaves_no_dirty_row`
  (`crates/app/src/commands.rs`) was run once against the unfixed
  `index.rs` and failed, as the plan required. The first assertion to go
  was `assert!(entry.pick)` (the row's pick had been zeroed), ahead of the
  `dirty_rows` assertion the plan predicted; the `.dop` itself already
  held `rating = 4` and `pick = true`, so the failure is index-side only,
  exactly the bug described. After dropping the `pick = 0` clause from
  `Index::reset_sidecars` the test passes and the whole `riffle-app` suite
  is green.
- The row's pick is read through `entries` (the same `write_batch` +
  `index::stat` trick the rating race test uses); no test-only accessor
  was needed, since `Entry` already carries `pick`.
- Residual from the plan's Trade-offs, checked: the UI does render a pick
  in XMP mode. `crates/app/ui/src/main.ts` only gates the `pick` *action*
  on the format (`case "pick"` returns early unless `sidecarFormat ===
  "dop"`), while rendering goes through `applyRating` ->
  `strip.setRating`, and `crates/app/ui/src/strip.ts` draws the flag dot
  from the `picks` set with no format check. So after a Dop -> Xmp switch,
  a dirty row that keeps `pick = 1` and is replayed on the next open would
  show its dot in XMP mode until a sidecar re-read or a new judgement.
  Not data loss (XMP never stores a pick), and the user chose the plain
  drop over `reset_sidecars(format)` knowing of this residual.

## Step 2

- The todo heading "App: a pick made during a sidecar format switch is lost
  on the next open" and its TODO are gone from `todo.md`.
- The guide entry went into `docs/agents/tauri-app.md` under "Rust side",
  just before "Folder-index eviction: lock order and where it runs", next to
  the other index/SQL entries. Its Source line points at the post-archive
  path (`docs/plans/_archived/20260920-format-switch-pick-race/learnings.md`,
  Step 1), matching how every other entry in that file links.

## Deferred issues (todo candidates)

- A pick kept by `reset_sidecars` is rendered in XMP mode. Basis: the
  Trade-offs section of this plan asked the implementer to verify the
  residual, and the check above confirms the strip draws the pick dot
  regardless of the current sidecar format. Files:
  `crates/app/ui/src/strip.ts` (the flag dot, `setRating`),
  `crates/app/ui/src/main.ts` (`applyRating`, `case "pick"`),
  `crates/app/src/index.rs` (`reset_sidecars`). Out of scope for this
  step, which is Rust-side only.
