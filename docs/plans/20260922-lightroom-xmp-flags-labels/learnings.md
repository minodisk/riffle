# Learnings

## Step 1

- `xmp.rs` now takes an `Ns { uri, prefixes }` for every lookup; the
  fallback prefix is `<first preferred>N` (`xmp1`, `xmpDM1`). `write_rating`
  patches in two passes (`Rating`, then `xmpDM:good`), re-parsing the
  spliced text in between, which keeps each splice independent.
- Removing `xmpDM:good` leaves any `xmlns:xmpDM` declaration Riffle added
  in place (only the attribute or element goes), as `write_label` already
  does for `xmp:Label`.
- The Step 1 shim in `crates/app/src/sidecar.rs` reports a reject as
  `Some(-1)` for both formats, not only XMP: `dop::read_rating` used to do
  that itself, so dropping it for `.dop` would have lost rejects in the UI.
  `read_pick` stays `Ok(false)` under XMP (and the shim never passes a pick
  to `xmp::write_rating`) so today's "XMP ignores a pick" behaviour and its
  tests hold until Step 3.
- As the plan's trade-off says, a `.dop` reject now writes `Rating = 0`
  through the shim (stars are `None` on a reject); the
  `a_photolab_sidecar_is_patched_in_place` "the stars are kept" assertion
  was dropped and should come back in Step 3.

## Deferred issues (todo candidates)

- Restore the "the stars are kept" assertion in
  `a_photolab_sidecar_is_patched_in_place` (`crates/app/src/sidecar.rs`)
  once Step 3 passes real stars with a reject (basis: Step 1 shim).
