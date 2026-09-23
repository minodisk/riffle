# Learnings

## Step 1

- The plan's "`read_rating == None`" does not hold for either format: clearing
  the stars writes `Rating = 0` into an existing sidecar in both XMP
  (`xmp:Rating="0"`, as `clearing_writes_zero_into_an_existing_sidecar` in
  `crates/core/src/xmp.rs` documents) and `.dop` (`dop::write_rating` doc:
  "`rating` `None` means unrated and writes `Rating = 0`"), so both readers
  return `Some(0)`, which the app treats as unrated (`commands.rs` filters `0`
  out). The tests assert `Some(0)`; no code fix was needed.
- The "no sidecar writes nothing" criterion is already covered by
  `clearing_a_label_writes_no_sidecar_when_there_is_none` (the exact
  `None, false, None, true` judgment in both formats), so no new test was
  added for it.
- `judge(...)` in the `sidecar.rs` tests now takes `pick: bool` after
  `rating`; the existing callers pass `false`.
- Verification only: all new tests passed without changing production code.

## Step 2

- `labelKnown` is forced by an optional `forceLabel` field on `Change`, set by
  `judge(next, forceLabel)` for `clearall` and read in `send`; the label
  comparison itself is untouched.
- `judge`'s idempotency check would have swallowed `c` on a file whose label the
  frontend does not know yet (locally everything already looks cleared), so the
  check is skipped when `forceLabel` is set and `entries` lacks the path.
- `reset_restores_the_defaults` in `shortcuts.rs` bound `c` to `clear`; now that
  `c` is `clearall`'s default it uses `q` instead. `c` is in no forbidden list.

## Step 3

- The docs never name the settings labels, so the `0` key label rename to "0 star" needed no docs change.
