# Learnings

## Step 1

- The full JPEG size lives on `riffle_core::partial::Crop` as
  `image_width`/`image_height`, so `decode_crop` callers get it too. The crop
  header carries it as two `u16`s at offsets 28/30 (a JpgFromRaw never exceeds
  65535 px per axis), and the kind tag moved from 4 (`CROP_KIND_RGBA_V2`) to 5
  (`CROP_KIND_RGBA_V3`); 3 (`CROP_KIND_RGBA_V1`) and 4 were both already used,
  so 5 is the next free value.
- The placeholder still uses the sensor fallback until the current file's crop
  arrives (option a); the rect maths is in `crates/app/ui/src/zoom.ts`.

## Step 2

- `list_arw_in` and the new `list_folder_in` share one private `list_dir(dir,
  Option<SidecarFormat>)`; `list_arw_in` passes `None`, so its behaviour is
  unchanged. `reconcile_sidecars_of` now takes the sidecar map, and the tests
  go through a `reconcile_listed` helper that lists afresh on each call (several
  tests rewrite sidecars between calls, so reusing one listing would be wrong).
- The crate is on Rust edition 2021: `if ... && let` chains do not compile.
- Reading the sidecar format before the listing does not widen any race: the
  same `format` value is used for both the listing and the reconcile, exactly
  as the single later read did.

## Step 3

- Eviction runs from `setup` (`commands::spawn_eviction`, called right after
  `app.manage(Scans)`) on a plain `std::thread`, not from `run_scan`'s
  completion path. The thread holds the `Scans` lock and then the writer lock
  for the whole evict + `VACUUM`, and skips if `Scans.running` is already set,
  so no scan runs during it; a `scan_folder`/`start_scan` issued meanwhile
  waits. Lock order is `Scans` then writer; nothing else holds the writer while
  taking `Scans`.
- Measured `VACUUM` (M3 Pro, release, ignored test
  `vacuum_cost_on_a_100_mb_index`): 5000 rows of 20.8 KB thumbnails (106 MB in
  use), evicting half and vacuuming took ~220 ms, leaving 53 MB.
- The size cap is checked on pages in use (`page_count - freelist_count`),
  since a delete only moves pages to the freelist until the `VACUUM`.
- The reader connection was not observed erroring during `VACUUM`: under WAL
  the vacuum writes through the WAL like any transaction, so readers keep their
  snapshot. This is reasoning plus the passing reader test, not a dedicated
  concurrent test.
- The v7 -> v8 migration seeds `folders` from `files` with `opened_at = now`;
  v2-v6 still drop `files` (condition is now `version < 7`), so they get no
  seeded folders, which is fine since they have no `files` rows.
