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

# EXIF filters in the strip-pane filter menu

## Purpose

The filter menu added in #88 (`#filter-menu` in `crates/app/ui/index.html`,
`passes` / `refilter` / `shownFlags` / `shownStars` in
`crates/app/ui/src/main.ts`) narrows the strip by pick flag and stars. This
work adds groups for the shooting settings — aperture, shutter speed, ISO,
focal length, camera and lens — so a folder can be narrowed to, say, every
frame shot at f/1.4 with the 50 mm.

Current state the plan is based on:

- The scan already parses every value needed. `riffle_core::arw::parse`
  fills `Shot` (`make`, `model`, `lens_model`, `exposure_time`, `f_number`,
  `estimated_f_number`, `iso`, `focal_length`, ...) for both ARW and DNG, and
  `scan::extract` runs it on every file. `scan::Entry` keeps only
  `capture_time`, `subsec` and `focus` and drops the rest, so the index
  (`files` table in `crates/app/src/index.rs`) never sees them. Adding them
  costs no parsing and no IO; the thumbnail encode (7 ms/file) stays the
  scan's cost.
- The metadata pane reads them per file on demand through the `metadata`
  command (`read_metadata` in `crates/app/src/commands.rs`), which also owns
  the display formatting (`decimal`, `shutter`, the `make model` join,
  `f/2 (est.)` for an estimated aperture).
- The frontend holds every indexed row in `entries: Map<path, IndexedFile>`,
  refreshed from `folder_entries` on `scan-progress` / `scan-done`, and
  calls `refilter()` after each refresh. `passes(path)` is the single
  predicate; the menu's items are static HTML picked up by
  `filterMenu.querySelectorAll("[data-flag], [data-stars]")`.
- `files` is a cache (the README says so); only `ratings` holds
  non-derivable state (`dirty` rows). `SCHEMA_VERSION` is 3; `prepare`
  accepts 0, 2 and 3, migrating v2 in place, and discards anything else.

Semantics, identical to the existing groups: the checked values of one group
are OR-ed, the groups AND-ed, and a group with nothing checked lets
everything through. Each group lists only the values present in the open
folder (for focal length, only the ranges that contain a frame).

## Steps

- [x] Step 1: Carry the shooting settings through `scan::Entry` and share the display formatting
  - Done when:
    - `riffle_core::scan::Entry` carries the whole `Shot` (either as a
      `shot: Shot` field replacing the three copied fields, or as the
      individual fields; pick the one that leaves `index.rs` and the tests
      smallest). `Shot` derives `Clone` if needed. `riffle-cli scan` and the
      app build unchanged in behaviour.
    - A new `crates/app/src/exif.rs` owns the display formatting that
      `read_metadata` has today: `decimal`, `shutter`, the camera join, the
      aperture label including `(est.)`, the focal length label. It exposes
      one function that turns a `&Shot` into the labelled values (a struct
      with `camera`, `lens`, `aperture`, `shutter`, `iso`, `focal_length`,
      each optional; the four numeric ones as `{ value: f64, label: String }`
      so a later consumer can sort by value and display by label).
      `read_metadata` builds `Metadata` from it; the pane's output is
      byte-identical to before.
    - The existing formatting behaviour is covered by unit tests in
      `exif.rs` (`1/250`, `1.3"`, `f/2.8`, `f/2 (est.)`, `50 mm`,
      `SONY ILCE-7M5`, make-only, model-only, all-None).
    - `mise run ci` passes.
  - Implementation approach:
    - No schema change in this step; `write_batch` only follows the field
      rename if `Entry` changes shape. The point is a small, behaviour-free
      PR so Step 2's diff is the schema alone.
    - `exif.rs` lives in `crates/app`, not `riffle-core`: formatting is
      presentation, and core stays parsing. Register it in `main.rs`
      alongside `index` and `sidecar`.
    - The shutter's `value` is seconds (`Rational::value()`), the
      aperture's is the f-number (the estimated one when `f_number` is
      absent, matching the pane), ISO is the integer, focal length is mm.

- [x] Step 2: Store the settings in the index and return them from `folder_entries`
  - Done when:
    - `files` gains columns for camera, lens, f-number (plus whether it is
      estimated), exposure time as an unreduced numerator/denominator (so
      `1/250` is shown as the camera wrote it), ISO and focal length.
      `write_batch` fills them from the `Entry`; an error row leaves them
      NULL.
    - `SCHEMA_VERSION` is 4. `prepare` accepts 0, 2, 3 and 4. v3 -> v4
      deletes every `files` row (the thumbnails are re-derived on the next
      open of each folder) and adds the columns; `ratings` is untouched, so
      no `dirty` judgement is lost. A v2 database goes through v2 -> v3 ->
      v4 in one open. The existing `unsupported index schema version`
      discard path still handles anything else.
    - `IndexedFile` gains `exif: Option<Exif>` (serde-serialised), built
      with Step 1's `exif.rs` from the stored columns: `None` for an error
      row, otherwise the labelled values with each field optional.
    - Unit tests in `index.rs`: a written row round-trips every field
      through `entries()`; an error row has `exif: None`; a v3 fixture (a
      `files` row with a thumbnail plus a `dirty = 1` `ratings` row) opens
      with no `files` rows, the dirty row intact, and a reopen is v4; the
      existing v2 test still passes.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Column choice: store raw values, not labels, and format on read via
      `exif.rs`. This keeps one formatting implementation and lets the
      label change without a schema bump.
    - `folder_entries` and `entries()` need no signature change; the
      frontend's `IndexedFile` interface simply gains the field (TypeScript
      ignores unknown fields until Step 3 declares it).
    - Do not touch `reconcile`: the rescan trigger for a stale row remains
      the size/mtime mismatch; the migration's `DELETE` is what forces the
      one-time rescan.

- [x] Step 3: EXIF groups in the filter menu (manual GUI confirmation pending, carried to Step 4)
  - Done when:
    - `IndexedFile` in `main.ts` declares `exif` mirroring the Rust struct.
    - Six sets beside `shownFlags` / `shownStars` hold the checked items
      per group (`shownCamera`, `shownLens`, `shownAperture`,
      `shownShutter`, `shownIso`, `shownFocal`; or one
      `Map<group, Set<string>>`). `passes(path)` AND-s them in with the
      same "empty set passes all" rule, reading `entries.get(path)?.exif`;
      a file without the value (no EXIF, error row, or its row has not
      landed yet during the scan) fails a group that has a selection.
    - Focal length is filtered by range, not exact value (the user's
      decision): `<24 mm`, `24–35 mm`, `35–50 mm`, `50–85 mm`,
      `85–135 mm`, `135–200 mm`, `>200 mm`, each range half-open
      `[lower, upper)` so a frame belongs to exactly one. Only the ranges
      that contain at least one frame of the folder are listed, in range
      order.
    - The menu grows a section per group, rebuilt from `entries` after
      every `refreshEntries()`: a heading, then one `menuitemcheckbox` per
      distinct label present in the folder (per range for focal length),
      sorted by numeric value for aperture, shutter and ISO and
      alphabetically for camera and lens. A group with no values in the
      folder (e.g. no lens model) is not shown. The static pick/stars items
      keep working; the dynamic items use the same classes and
      `aria-checked` styling. `Reset` clears every set.
    - The six sets are cleared on folder open (a label from one folder means
      nothing in another) and, after a rebuild, any checked item no longer
      present in the folder is dropped from its set so a stale selection
      cannot hide everything.
    - The menu gets `max-height` (about 70vh) and `overflow-y: auto` so a
      long list scrolls instead of leaving the window.
    - `filterToggle.active` lights while any group has a selection.
    - **(manual, GUI automation is unavailable)** the user confirms on an
      ARW folder and on the M11-P DNG folder: each group lists only values
      (ranges) present, checking one narrows the strip, two checks in a
      group OR, checks across groups AND, `Reset` restores everything,
      opening another folder clears the EXIF selections, and the pane's
      metadata matches the checked item for the shown file.
    - `mise run ci` passes (`tsc --noEmit` is the only frontend check).
  - Implementation approach:
    - Assumes Step 2 is merged.
    - `filterItems` is a static `NodeList` captured once; either re-query
      after each rebuild or attach one delegated `click` listener on
      `#filter-menu` that reads `dataset`. The existing per-item listener
      pattern (`data-flag` / `data-stars`) can be extended with e.g.
      `data-group` + `data-value`.
    - Rebuild the dynamic sections only when the set of items changed, or
      cheaply enough that a 10/s `scan-progress` stream does not matter;
      the existing `refreshEntries()` throttle already keeps one request in
      flight, so rebuilding on each landed refresh is acceptable.
    - The range table is frontend-only; Steps 1-2 store the raw mm, so
      changing the ranges later needs no schema change.
    - The `customizable-shortcuts` plan edits the `keydown` handler in the
      same file; rebase carefully, the regions do not overlap.

- [ ] Step 4: Documentation and the user's confirmations
  - Done when:
    - `README.md` "The index" lists the shooting settings among what the
      scan stores and states the v4 rule (a pre-v4 database re-scans each
      folder once on its next open, ratings kept); the filter menu's
      description gains the EXIF groups, the focal length ranges and their
      OR/AND semantics; any stale "no filtering by rating" line is
      corrected. Step 3's manual results go in the "confirmed / awaiting the
      user's confirmation" split the README already uses.
    - `CLAUDE.md` "Layout" mentions `crates/app/src/exif.rs`.
    - `docs/agents/tauri-app.md` gains any pitfall Steps 1-3 hit (or
      nothing, if none).
    - `mise run ci` passes.

## Trade-offs and risks

### Migration: delete `files` on v3 -> v4 (chosen) versus a lazy EXIF-only backfill

- **Chosen: delete every `files` row.** One full rescan per folder on its
  next open: 5-12 s per 5000 files on the measured Apple Silicon machine,
  well under a second for a few dozen DNGs, with the existing
  `scanning N / M` status and the first preview not waiting. Zero new code
  paths; the exact mechanism a size/mtime mismatch already uses. `ratings`
  (the only non-derivable table) is untouched.
- **Alternative: keep thumbnails, add the columns, and backfill EXIF with a
  second head-only pass.** Needs a second scan pipeline, a way to tell "not
  yet backfilled" from "no EXIF", and its own progress/cancel handling. Not
  worth it for a one-time cost.
- Not an option: bumping the version without a migration. `prepare` would
  discard the whole database including dirty `ratings` rows.

### Focal length: ranges (chosen by the user) versus exact values

- Ranges cap the list at seven items even for a zoom-heavy folder. The cost
  is that values inside one range (e.g. 40 and 45 mm) are not separable.
  Accepted.
- The stored value is the raw mm, so switching to exact values later is a
  Step 3-only change.

### Menu layout: one scrolling column with headings (chosen) versus flyout submenus or collapsible groups

- Headings plus scroll is the smallest change to the existing menu and
  keeps every group visible at once. If the manual check finds it too tall,
  collapsing each EXIF group under its heading is the next step; the data
  shape does not change.

### Files without a value

A frame with no lens model (or an error row) fails any lens filter. There is
no "Unknown" item to select them explicitly; adding one is a follow-up if
wanted.

### Estimated aperture

A manual lens without contacts yields `f/2 (est.)`, which lists as its own
item distinct from a measured `f/2`. This mirrors the pane and keeps
estimates visible.

### Merge risk

`main.ts` is also edited by the in-flight `customizable-shortcuts` plan (the
`keydown` handler). The filter code is a separate region; expect a trivial
rebase.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-19) Step 2 complete
- (2026-09-19) Step 3 complete (manual GUI check carried to Step 4)
