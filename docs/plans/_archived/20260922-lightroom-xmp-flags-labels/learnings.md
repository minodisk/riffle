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
  to `xmp::write_rating`) so today's "XMP ignores a pick" behavior and its
  tests hold until Step 3.
- As the plan's trade-off says, a `.dop` reject now writes `Rating = 0`
  through the shim (stars are `None` on a reject); the
  `a_photolab_sidecar_is_patched_in_place` "the stars are kept" assertion
  was dropped and should come back in Step 3.

## Step 2

- `crates/app/src/sidecar.rs` byte-exact label tests had to learn the
  added `photoshop:LabelColor` and the leftover declaration.
- `write_label` patches in two passes (`xmp:Label`, then
  `photoshop:LabelColor`); the color is inserted as an attribute even when
  the sidecar holds `xmp:Label` as an element, matching the existing insert
  path.
- Clearing a label removes both properties but keeps any `xmlns:photoshop`
  Riffle declared, so a sidecar labeled then cleared is not byte-identical
  to the original (same as `xmpDM:good` in Step 1). Tests expect that.
- `read_label` normalizes `LabelColor` to first-letter-uppercase, rest
  lowercase; a Bridge-only `xmp:Label` is still returned raw.

## Step 3

- `Flag` lives in `riffle-core`, which has no serde or rusqlite, so the app
  maps it by hand: `index::flag_code` / `flag_from_code` for the
  `ratings.flag` column and `flag_name` / `parse_flag` (plus a
  `serialize_with`) for the `"none" | "pick" | "reject"` IPC strings.
- The v11 migration runs for every older version (after the v2 `pick`
  `ALTER` and the label ones), so the v7 / v8 / v9 migration tests, which
  build their database with the current schema and then lower
  `user_version`, now rename `flag` back to `pick` first to look like the
  schema they claim to be.
- The frontend's `rating === 0` from a sidecar holding `Rating="0"` is
  normalized to unrated in `applyRating`, so `ratings` holds only `1`-`5`.
- Sorting by rating used to put rejects last because they were `-1`; the
  sort input in `main.ts` still passes `-1` for a reject so that order is
  kept (not listed in the plan, but otherwise rejects would silently sort
  by their stars).
- `sidecarFormat` in `main.ts` had no reader left once the XMP pick gates
  went, so it and its `sidecar_format` invoke were removed; the settings
  window still uses the command.
- `docs/agents/tauri-app.md` lost the "a pick is only meaningful while
  `.dop` is selected" section and its `pick` column names became `flag`,
  since the code it described is gone (README / CLAUDE.md stay for Step 4).
- The stars-kept assertion deferred from Step 1 is back in
  `a_photolab_sidecar_is_patched_in_place` (a reject on `Rating = 3` keeps
  it). `a_lightroom_folder_reads_its_flags_labels_and_stars` in
  `commands.rs` loads trimmed copies of the L1005428-L1005439 shapes
  (namespaces and judgment attributes only) and checks flags, labels and
  stars; the reference sidecars were only read.
- Manual GUI check left for the user: open `/mnt/d/Photos/2026/2026-09-05`
  under XMP and check 439 picked, 438 rejected, 428-432 purple / blue /
  green / yellow / red, 433-437 5..1 stars; on a copy of the folder, diff a
  sidecar before and after `p` / `x` / `u` / `3` / a color key, and check a
  reject on a starred file keeps `xmp:Rating`.

## Deferred issues (todo candidates)

- (Done in Step 3) Restore the "the stars are kept" assertion in
  `a_photolab_sidecar_is_patched_in_place` (`crates/app/src/sidecar.rs`).
- Confirm the Lightroom 9.5.1 round-trip in the GUI and tick the README
  checklist entry. Open `D:\Photos\2026\2026-09-05` under XMP (expect 439
  picked, 438 rejected, 428-432 purple / blue / green / yellow / red, 433-437
  5..1 stars); on a copy, give a pick, a starred reject and a label in Riffle
  and check Japanese Lightroom shows them, notably the color when
  `xmp:Label` is the English name. Done when the README entry is ticked, or
  the label write is revised if Lightroom ignores `LabelColor`.

## Step 4

- The README "Sidecar formats and software" checklist now names Adobe
  Lightroom 9.5.1 (Windows; not Lightroom Classic) but leaves it unticked:
  the user has not yet confirmed the round-trip in Lightroom. Tick it once
  they do (mention this in the PR).
- The Step 4 commit carries a `Release-As: 0.3.0` trailer so release-please
  cuts 0.3.0 rather than 0.2.1.
