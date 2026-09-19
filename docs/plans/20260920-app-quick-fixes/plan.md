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

# Close out three `crates/app` items in todo.md: exact zoom placeholder, single folder listing, bounded index

## Purpose

Three independent `todo.md` items under "App" are each small enough for one PR
and share no code:

- **`focus_crop`'s header has no full-JPEG size.** `drawZoom()` in
  `crates/app/ui/src/main.ts` scales the preview placeholder by
  `focus.sensor_w / bitmap.width`, which is exact only when the JpgFromRaw is
  sensor-sized. `crop_payload` (`crates/app/src/commands.rs`) has a reserved
  4-byte word at offset 28 that can carry the JPEG's width and height as two
  `u16`s, and `decode_region` (`crates/core/src/partial.rs`) already knows
  them (`cinfo.output_width/height`) but does not return them.
- **Folder open lists the directory twice.** `scan_folder` calls
  `list_arw_in` and then `reconcile_sidecars_of` calls `list_sidecars_in`, two
  `read_dir` passes over the same folder. `list_arw_in` is also the body of the
  `list_arw` command and is used by ~14 tests, so it must keep its behaviour.
- **The SQLite index is never pruned or `VACUUM`ed.** `crates/app/src/index.rs`
  holds every folder ever opened at ~20.8 KB per row; `reconcile` only deletes
  rows for files that changed inside a folder being reopened, and nothing ever
  shrinks the file.

Removing the three resolved `todo.md` headings happens in wrap-up, not in a
step.

## Steps

- [x] Step 1: Carry the full JPEG size in the `focus_crop` header and use it for the zoom placeholder
  - Done when:
    - `riffle_core::partial::FocusCrop` (or `Crop`) exposes the decoded JPEG's
      full width and height; `crates/cli` still compiles unchanged apart from
      any struct-literal it builds (it builds none; only
      `crates/app/src/commands.rs`'s `crop_payload_header_encodes_every_field`
      test constructs the struct by hand).
    - `crop_payload` writes the full width at offset 28 (`u16`) and the full
      height at offset 30 (`u16`), the header table comment is updated, and
      the kind tag is bumped to `CROP_KIND_RGBA_V3` (the existing comment
      records that v1 -> v2 was a header change; keep that convention).
      `CROP_HEADER_LEN` stays 32.
    - `crop_payload_header_encodes_every_field` asserts the two new fields, and
      `a_synthetic_arw_crops_around_the_centre_without_a_focus_point` asserts
      the full size is 400x300 for its gradient JPEG.
    - `drawZoom()` scales the placeholder from the header's full size when the
      crop for the current file (`crop.cropSeq === seq`) is present, and the
      placeholder-rect arithmetic lives in a pure function in a new
      `crates/app/ui/src/zoom.ts` with a `zoom.test.ts` (vitest, same pattern
      as `sort.ts`/`sort.test.ts`) covering: exact scale from full dims, the
      sensor fallback when no full dims are known, and the focus point landing
      at the origin in both cases.
    - `mise run ci` passes.
  - Implementation approach:
    - `decode_region` in `crates/core/src/partial.rs` reads `iw`/`ih` at line
      ~97; add `image_width`/`image_height` to `Crop` (so `decode_crop`
      callers get it for free) or to `FocusCrop` only. Prefer `Crop`: it is
      where the decode's own facts live, and `decode_focus_crop` copies through.
    - Frontend: `main.ts` reads the header with a `DataView` at
      `requestCrop()`; add `fullWidth: header.getUint16(28, true)` and
      `fullHeight: header.getUint16(30, true)` to the `crop` record, update the
      `CROP_KIND_RGBA_V2` constant/check to V3.
    - The placeholder is drawn before the crop arrives, so there is a window
      with no exact size. Keep the sensor fallback for that window only
      (decided; see "Trade-offs"). Memoising per path is optional if the
      flicker turns out visible.
    - `docs/agents/tauri-app.md` needs no change unless a new pitfall is hit.

- [x] Step 2: List the folder once for both RAW files and sidecars in `scan_folder`
  - Done when:
    - `scan_folder` performs exactly one `std::fs::read_dir` on the folder for
      RAW discovery and sidecar discovery combined; `list_sidecars_in` is
      removed (grep returns nothing) and `reconcile_sidecars_of` takes the
      sidecar map as a parameter instead of listing.
    - `list_arw_in` keeps its signature and behaviour (`list_arw` command and
      the existing tests untouched), still returning `Err` for an unreadable
      directory and skipping unreadable entries.
    - A new test in `commands.rs` builds a folder with mixed-case RAW files,
      `.xmp`/`.dop` sidecars of both cases, a non-RAW file and a subdirectory,
      and asserts the combined listing's RAW list equals `list_arw_in`'s and
      its sidecar map matches what the old `list_sidecars_in` returned for each
      `SidecarFormat` (keyed by lower-cased name, values `(path, size,
      mtime_ns)`).
    - `mise run ci` passes.
  - Implementation approach:
    - Add `fn list_folder_in(dir: &Path, format: SidecarFormat) ->
      Result<(Vec<String>, HashMap<String, SidecarStat>), String>` (or a small
      struct) that walks `read_dir` once, sorting RAWs by file name like
      `list_arw_in` and filtering sidecars with `format.matches`. To avoid
      duplicating the filter/sort, factor the shared per-entry logic so
      `list_arw_in` becomes `list_folder_in`-with-no-sidecars or both call one
      internal iterator over `DirEntry`s. Either way `list_arw_in`'s public
      behaviour is the acceptance criterion, not its body.
    - In `scan_folder`, the sidecar format is currently read *after* the
      listing (`let format = *index::lock(&app.state::<AppSidecarFormat>().0)`
      at line ~697); move that read before the listing so the single pass
      knows which sidecar names to match. Note `switch_sidecar_format` holds
      `AppSwitchLock`; check that reading the format earlier does not widen any
      race the existing comments in `commands.rs` describe.
    - Error semantics differ today: `list_arw_in` errors on an unreadable
      directory, `list_sidecars_in` returns an empty map. With one listing the
      error surfaces once from the RAW side, which `scan_folder` already
      propagates with `?`; keep that.
    - The `reconcile_sidecars_of` tests (lines ~1160-1500) call
      `list_arw_in(&root)` then `reconcile_sidecars_of(&dir, &listed, ...)`;
      switch them to the new listing function with minimal edits (a shared
      helper in the test module is fine).

- [ ] Step 3: Evict stale folders from the index and `VACUUM` afterwards
  - Done when:
    - The index records when each folder was last opened (`folders(dir TEXT
      PRIMARY KEY, opened_at INTEGER NOT NULL)`, touched from `reconcile`),
      and an `Index::evict(now, policy) -> Result<EvictSummary, String>`
      deletes the `files` rows (and the `ratings` rows with `dirty = 0`) of
      folders selected by the policy, then runs `VACUUM` when at least one row
      was deleted.
    - Policy (decided by the user: age limit plus size cap): a folder is
      evicted when it was last opened more than `MAX_AGE` (30 days) ago, and,
      after that, least-recently-opened folders are evicted until the database
      (`page_count * page_size`) is under `MAX_BYTES` (1 GiB, ~50k rows at the
      measured 20.8 KB/row). Both constants are documented next to `BATCH` with
      the reasoning.
    - `ratings` rows with `dirty = 1` are never deleted (they are unwritten
      judgements, per the `SCHEMA_VERSION` comment), and a folder with any
      dirty row keeps its `folders` row so it is reconsidered next time.
    - `SCHEMA_VERSION` becomes 8; a v7 database migrates in place (no `files`
      drop: the current `version != 0 && version != SCHEMA_VERSION` drop
      condition must become `version < 7`), and the migration seeds `folders`
      from `SELECT DISTINCT dir FROM files` with `opened_at = now` so the first
      launch after the upgrade does not throw away every thumbnail.
    - Eviction runs once per launch off the main thread (see approach), and
      never while a scan is running.
    - Tests in `index.rs`: an old folder is evicted and a fresh one kept; LRU
      order under the size cap (use a tiny cap in the test); a dirty rating
      survives eviction of its folder; after evict + `VACUUM` the file size
      (or `page_count`) is smaller than before; a v7 database migrates with
      its `files` rows intact and `folders` seeded; `reconcile` updates
      `opened_at`.
    - `mise run ci` passes.
  - Implementation approach:
    - `VACUUM` cannot run inside a transaction and rewrites the whole file, so
      it is not part of `prepare` (which runs on the main thread in `setup`,
      `crates/app/src/main.rs` line ~233) and not part of a folder open.
      Spawn a `std::thread` from `setup` after `app.manage(AppIndex)` that
      takes the writer mutex (`index::lock`) and calls `evict`; the first
      `scan_folder` simply waits behind the lock. Alternatively run it from
      `run_scan`'s completion path; decide in the step and measure `VACUUM`
      time on a ~100 MB database (the todo item's 5000-row figure) so the
      wait is known and recorded in `learnings.md`.
    - Under WAL the read-only `AppIndexReader` connection is fine while the
      writer `VACUUM`s; `open_reader` has a 300 ms `busy_timeout`, so confirm
      `folder_entries`/`thumbnail` do not error during the vacuum (a
      `SQLITE_BUSY` there would show as "no thumbnails" for one call).
    - Time source: `std::time::SystemTime::now()` as seconds since the epoch,
      matching `stat`'s epoch handling; pass `now` into `evict` so tests
      control it.
    - The `opened_at` write belongs in `reconcile` (the folder-open path) so
      no new command is needed.
    - Update the `SCHEMA_VERSION` doc comment with the v8 note, following the
      existing per-version sentences.

## Trade-offs and risks

- **Step 1, the placeholder before the crop arrives.** The header only tells
  the frontend the full size once a crop has come back, so the first frame
  after zooming into a file that has never been cropped has no exact size.
  (a) Keep the sensor fallback for that transient window only (chosen). (b)
  Also memoise full dims per path in the frontend. (c) Store the JPEG size in
  the index (schema bump, rescan of every folder): rejected as far heavier
  than the todo item asks for.
- **Step 1, header versioning.** Redefining the reserved word without bumping
  the kind tag would also work, since the frontend and backend ship together.
  The plan bumps to V3 to keep the "v1 was ..." comment convention honest.
- **Step 2, error semantics.** Today an unreadable folder fails the open via
  `list_arw_in` and sidecar listing failure is silent. Merging them keeps the
  RAW-side error; nothing is lost because a folder whose `read_dir` fails has
  no readable sidecars either.
- **Step 3, eviction policy.** Option A (chosen by the user): age limit plus a
  size cap with LRU by folder. Option B (size cap only) was rejected. The
  constants (30 days, 1 GiB) are guesses, documented so they can be tuned
  without re-planning.
- **Step 3, what eviction deletes.** `files` rows hold the thumbnails (the
  bulk); `ratings` rows are tiny. Deleting clean `ratings` rows too keeps the
  index a pure cache for evicted folders (the sidecar is the source of truth
  and is re-read on the next open). Dirty rows must stay.
- **Step 3, `VACUUM` versus `auto_vacuum = INCREMENTAL`.** Incremental
  auto-vacuum only takes effect on a new database or after a full `VACUUM`;
  the explicit `VACUUM` after an eviction is simpler. Revisit only if the
  measured `VACUUM` time is unacceptable.
- **Step 3, v7 migration.** Changing the `files`-drop condition is the risky
  edit: a mistake would rescan every folder once (a cache loss, not data
  loss). The migration test pins it.
- **Ordering.** The three steps touch disjoint code; any order works.

## Progress

- (2026-09-20) Step 1 complete
- (2026-09-20) Step 2 complete
