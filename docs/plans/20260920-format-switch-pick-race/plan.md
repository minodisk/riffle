<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Keep a pick made during a sidecar format switch

## Purpose

A pick set while a sidecar format switch is in flight is lost on the next
folder open (`todo.md`, "App: a pick made during a sidecar format switch is
lost on the next open"). `switch_format` (`crates/app/src/commands.rs`) swaps
the current format, drains the writer, persists the setting and then calls
`Index::reset_sidecars`. A judgement made after the swap already observes the
new format (`.dop`) and is queued with `pick = 1`, but `reset_sidecars`
zeroes the pick of every dirty row (`UPDATE ratings SET xmp_size = NULL,
xmp_mtime_ns = NULL, pick = 0 WHERE dirty = 1`), including that one. The
writer then lands the `.dop` with the pick and calls `Index::mark_written`
with `pick = 1`, whose `pick = ?5` guard no longer matches the row's
`pick = 0`, so the row stays dirty with `pick = 0` and the next open replays
it over the sidecar, stripping the pick.

After this work a pick set during a switch survives the switch and the next
open, and the `pick = true` variant of the format-switch race test that
`robustness-cleanup` (#218) ran locally and dropped is a real test in
`commands.rs`.

## Background (from investigation)

- The `pick = 0` clause in `reset_sidecars` is redundant in both switch
  directions:
  - `set_rating` (`crates/app/src/commands.rs`, the command) forces
    `pick = pick && rating != Some(-1) && format == SidecarFormat::Dop`,
    and the UI refuses the `pick` action outside `.dop`
    (`crates/app/ui/src/main.ts`, `case "pick"`). A row can therefore
    only hold `pick = 1` if it was judged while `.dop` was current. On an
    Xmp -> Dop switch, no pre-swap dirty row has a pick to drop, and a
    post-swap (race-window) row's pick is a legitimate `.dop` pick.
  - On a Dop -> Xmp switch, `sidecar::write` already ignores the pick for
    XMP (`let pick = *pick && format == SidecarFormat::Dop;`) and calls
    `mark_written` with the judgement's own `pick`, so a kept `pick = 1`
    on a dirty row still matches and the row is cleared normally.
- `mark_written`'s `pick = ?5` guard is correct and must stay: it is what
  keeps a row dirty when a keypress lands during a write (see
  `a_pick_is_stored_beside_the_rating_and_guards_mark_written`).
- `switch_format`'s ordering cannot close the window: the race is with any
  concurrent `set_rating` command, not with `persist`; `reset_sidecars`
  must follow the drain (the drain is what lands the old format's dirty
  rows), and holding the index mutex across the drain would deadlock on the
  writer's own `mark_written`.
- The deferred `pick = true` test does not exist in the tree; it was run
  locally and removed from PR #218
  (`docs/plans/_archived/20260920-robustness-cleanup/learnings.md`, Step 4).
  The rating-only test it was cloned from is
  `a_rating_set_during_a_switch_lands_in_the_new_format_and_leaves_no_dirty_row`
  in `crates/app/src/commands.rs`.
- `rating` and `label` are not exposed to this class of bug:
  `reset_sidecars` only nulls the stat and zeroes `pick`; it leaves
  `rating`, `label` and `label_known` untouched, so `mark_written`'s
  `rating IS ?4 AND label IS ?6` guard still matches after a reset. The
  existing test `a_sidecar_reset_keeps_the_label_of_a_dirty_row` and the
  rating-only race test cover this.

## Steps

- [x] Step 1: Stop `reset_sidecars` from zeroing the pick of dirty rows, and add the pick race test
  - Done when:
    - `crates/app/src/commands.rs` has a second race test beside
      `a_rating_set_during_a_switch_lands_in_the_new_format_and_leaves_no_dirty_row`
      (e.g. `a_pick_set_during_a_switch_lands_in_the_dop_and_leaves_no_dirty_row`)
      that follows the reproduction in the todo: start in `Xmp`, call
      `switch_format(.., Dop, persist)` where `persist` does
      `index.set_rating(&dir, &path, Some(4), true, None, true)` and
      `writer.set(path, Some(4), true, None, true, format)`, then
      `writer.flush(DRAIN_TIMEOUT)`. It asserts the `.dop` exists with
      `read_rating == Some(4)` and `read_pick == true`, no `.xmp` was
      written, the row's rating is `Some(4)` and its pick is `true`
      (after the `write_batch` trick the rating test uses so `entries`
      joins `files`), and `dirty_rows(&dir)` is empty.
    - The test fails on `main` before the fix (the `dirty_rows` assertion)
      and passes after; the implementer confirms this by running it
      before the `index.rs` change and records the result in
      `learnings.md`.
    - `Index::reset_sidecars` in `crates/app/src/index.rs` no longer sets
      `pick = 0`; its `UPDATE` becomes
      `UPDATE ratings SET xmp_size = NULL, xmp_mtime_ns = NULL WHERE dirty = 1`.
      Its doc comment drops the two sentences about the pick and states
      instead that the judgement (rating, pick and label) is kept, and
      why the pick is safe to keep (a pick is only ever set under `.dop`,
      and an XMP write ignores it).
    - `a_sidecar_reset_keeps_the_label_of_a_dirty_row` in `index.rs` is
      updated to assert the pick is kept too (`true` in the expected
      `dirty_rows` tuple) and renamed to say so, e.g.
      `a_sidecar_reset_keeps_the_judgement_of_a_dirty_row`.
    - `mise run ci` passes.
  - Implementation approach:
    - The fix site is `reset_sidecars` only, and the fix is the plain drop
      of the `pick = 0` clause (decided by the user at planning time; see
      "Trade-offs and risks"). Do not touch `mark_written`'s guard or
      `switch_format`'s ordering, and do not introduce a
      `reset_sidecars(format)` parameter.
    - Clone the new test from the rating-only race test; reuse
      `switch_writer`, `sidecar_index`, `list_arw_in`, `rating_of` and
      the `write_batch` + `index::stat` trick. Read the pick back through
      `SidecarFormat::Dop.read_pick`. For the row's pick, either extend
      the check through `entries` the way `rating_of` does or query
      `dirty_rows` before/after; do not add a test-only accessor to
      `Index` if an existing one serves.
    - Commit the test before the fix (or run it once against the
      unfixed `index.rs`) so the failure is observed, then fix.
    - Keep the change surgical: the SQL clause, the doc comment, the two
      tests. No other `reset_sidecars` behaviour (dropping clean rows,
      nulling the stat, keeping the label) changes.

- [ ] Step 2: Close the todo item and record the pitfall in the app guide
  - Done when:
    - The heading "App: a pick made during a sidecar format switch is lost
      on the next open" and its TODO are removed from `todo.md`.
    - `docs/agents/tauri-app.md` gains a short "Hit" entry under "Rust
      side" stating that `reset_sidecars` must not rewrite any field
      `mark_written` guards on (`rating`, `pick`, `label`), because a
      judgement made in the switch window has already been queued with
      the value the row held, and a mismatch leaves the row dirty and
      replays a stale value on the next open. Source line points at this
      plan's `learnings.md`.
    - `mise run ci` passes (lint covers the Markdown).
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Follow the existing entry format in `docs/agents/tauri-app.md`
      (heading with the tag, Why / What broke / Rules, Source).

## Trade-offs and risks

- Fix site, three candidates named in the todo:
  - `reset_sidecars` (chosen): drop `pick = 0`. Smallest change,
    removes the only writer of a guarded field outside `set_rating` and
    the sidecar reads. Safe because a pick can only be set under `.dop`
    (command and UI both enforce it) and an XMP write ignores it.
  - `mark_written`: relaxing the `pick = ?5` guard (e.g. clearing `dirty`
    on a rating/label match alone) would clear the race-window row but
    leave it at `pick = 0` while the `.dop` holds the pick, so the index
    and the sidecar disagree until the next open re-reads it; and the guard
    is what protects a keypress made during a write. Not taken.
  - `switch_format` ordering: no reordering closes the window, since the
    race is with any concurrent `set_rating`, not with `persist`;
    `reset_sidecars` must follow the drain, and serialising `set_rating`
    against the switch would mean holding the index mutex across a drain
    that itself needs the mutex (`mark_written`). Not taken.
- Alternative within the chosen site, considered and declined by the user
  at planning time: make `reset_sidecars` take the new format and zero
  `pick` only when switching to `Xmp` (`reset_sidecars(format)`). That
  preserves the original intent of not showing a `.dop` pick in XMP mode
  on a row whose drain write failed or that belongs to a folder not yet
  reopened. The plain drop was chosen because that residual case is narrow
  (a failed drain write or a never-reopened folder), the kept pick is
  harmless to disk (XMP never stores it) and is overwritten by the next
  sidecar read, and a conditional keeps a second code path that the race
  test cannot distinguish.
- Visible residual after the plain drop: after a Dop -> Xmp switch, a dirty
  row that keeps `pick = 1` and is replayed on the next open ends clean
  with `pick = 1` in the index (the XMP holds no pick). `entries` could
  then report `pick = true` in XMP mode for that file until a sidecar
  re-read or a new judgement. Whether the UI renders a pick in XMP mode
  was not verified at planning time; the implementer should check
  `crates/app/ui/src/main.ts` and note it in `learnings.md`. It is not a
  data loss in either direction.
- `label` and `rating` exposure: none today (see Background). The guide
  entry in Step 2 is what keeps a future `reset_sidecars` edit from
  reintroducing the class of bug for those fields.

## Progress

- (none yet)
