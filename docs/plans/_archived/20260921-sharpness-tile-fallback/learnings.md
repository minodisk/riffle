# Learnings

## Step 1: Sony FocusMode (0x201b)

- Chose `Shot::focus_mode: Option<u8>` (raw value) over `manual_focus: bool`:
  it keeps AF-S / AF-C / DMF distinguishable for later routing and costs
  nothing extra; manual focus is `Some(0)`.
- A BYTE (type 1, count 1) value rides inline in the entry's value field; it
  is read via a small `byte()` sibling of `integer()`.
- `riffle-cli info ~/Downloads/_DSC6978.ARW` (a real ARW, AF-C shot) printed
  `focus mode: 3` next to `focus: 7008 4672 3613 1732`, confirming the parse.
- 0xb04e / 0xb042 (older bodies) are not read; those files keep `None`.

## Step 2

- Routing choice: `score_preview` keeps its `Option<FocusLocation>` signature
  (`None` = tile maximum). The manual-focus decision lives in
  `sharpness::trusted_focus(&Shot)`, which `scan::extract` calls, so the call
  site reads `score_preview(&preview, trusted_focus(&arw.shot))` and the
  FocusMode case is unit-testable without building an ARW.
- Cost on `~/Downloads/2026-02-01 3` (31 Leica M11-P DNGs, all tile path),
  `riffle-cli scan <dir> 1`, warm cache, 3 runs each, per-file mean on a
  worker: before 11.2-12.5 ms, after 12.7-13.1 ms (roughly +1 ms per file,
  within run-to-run noise territory). Not worth a README table row on its own.

## Step 3

- Dropping `files` for every `version < 9` also covers v7, so the v7
  `folders` seeding (which selects from `files`) now seeds nothing. A v7
  database therefore ends with an empty `folders` table; harmless, since
  there are no `files` rows left to evict. The old
  `a_v7_database_keeps_its_files_rows_and_seeds_folders` test was rewritten
  as `a_v7_database_drops_its_files_rows_and_gains_folders`, and the seeding
  kept with a comment.

## Step 4

- No cost row added to the "Sharpness scoring cost" table: Step 2 measured
  about +1 ms per file, too small to state.
