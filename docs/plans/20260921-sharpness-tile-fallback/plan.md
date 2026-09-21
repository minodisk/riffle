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

# Sharpness score: tile maximum when there is no trustworthy AF point

## Purpose

`riffle_core::sharpness::score_preview` scores a 256 px window around the
focus point and falls back to the image centre when the file has none. A
Leica DNG (no focus location) or a Sony frame shot in manual focus therefore
gets the centre's sharpness, which is misleading whenever the subject is
off-centre. After this work the window around the Sony AF `FocusLocation`
(maker note 0x2027) is still scored as today, but a file without one, or a
Sony frame taken in manual focus (maker note 0x201b `FocusMode == 0`), is
scored as the maximum Laplacian variance over a grid of preview tiles, so a
frame that is sharp anywhere ranks above a frame that is sharp nowhere.

Scope: no user-facing setting, no change to the zoom/crop path
(`partial::focus_point` and `decode_focus_crop` keep their centre fallback).

## Decisions

- DMF (`FocusMode == 6`) stays on the AF window; only `0 = Manual` goes to
  tiles.
- Tile stride is `WINDOW` (non-overlapping).
- Cached scores are invalidated by a schema bump (Step 3).
- No manual-focus Sony ARW is available, so the `FocusMode == 0` case is
  verified only by synthetic tests. A real manual-focus Leica folder
  (`~/Downloads/2026-02-01 3`, M11-P DNGs) is available for checking the tile
  path in Step 2.

## Facts established while planning

- Sony `FocusMode` is maker note tag `0x201b`, type BYTE (int8u), count 1,
  value inline in the entry; `0 = Manual, 2 = AF-S, 3 = AF-C, 4 = AF-A,
  6 = DMF, 7 = AF-D` (exiftool Sony table; verified on an ILCE-7M5 ARW,
  `~/Downloads/_DSC6978.ARW`, value 3). It is not encrypted. `arw.rs` has
  no `TYPE_BYTE` yet and `integer()` only accepts SHORT/LONG.
- All 123 local sample ARWs are AF-C with a `FocusLocation`. Whether an MF
  frame carries a (centred or stale) `FocusLocation` is unknown, which is
  why the routing is keyed on `FocusMode`, not on the presence of
  `FocusLocation`.
- All local Leica M11-P DNGs carry no `FocusLocation` (only Leica
  `FocusDistance`), so they take the centre fallback today.
- Scores live in `files.sharpness` in the SQLite index and are recomputed
  only when the `files` rows are dropped (schema bump, v7 precedent) or the
  user clears the cache.

## Steps

- [x] Step 1: Core: parse the Sony `FocusMode` maker note tag into `Shot`
  - Done when:
    - `crates/core/src/arw.rs` reads tag `0x201b` out of the Sony maker note
      IFD (the same IFD `TAG_FOCUS_LOCATION` is read from) into a new field
      on `Shot` (`focus_mode: Option<u8>` with the raw value, or
      `manual_focus: bool`; pick one and record why). Absent tag or non-Sony
      note keeps the default
    - `integer()` (or a small sibling) accepts a BYTE (type 1, count 1)
      entry, whose value rides inline in the entry like SHORT/LONG; add
      `const TYPE_BYTE: u16 = 1` next to the existing type constants
    - Unit tests in `arw.rs` using the existing `tiff_with_exif` builder
      (extend it, or add a sibling, so the maker note can carry a `0x201b`
      BYTE entry): value `0` yields manual focus; value `3` and an absent tag
      do not; a Leica-style note (`a_non_sony_maker_note_is_skipped`) still
      yields the default
    - `riffle-cli info` (`crates/cli/src/main.rs`, the block printing
      `focus:`) prints the focus mode on its own line, and is run once on a
      real ARW to confirm the parse; the result goes in `learnings.md`
    - `mise run ci` passes
  - Implementation approach:
    - `Shot` is `Default` and every literal construction outside `arw.rs`
      uses `..Shot::default()`, so the new field needs no changes there
    - Do not store the field in the index: the score is computed at scan
      time in `riffle_core::scan::extract`, and nothing else needs it
    - Do not read `0xb04e` / `0xb042` (older bodies); mention this limitation
      in a comment

- [x] Step 2: Core: tile-maximum score without a trustworthy AF point, and route by focus mode
  - Done when:
    - `crates/core/src/sharpness.rs` gains a pure function that, given the
      gray buffer and its size, covers the image with a grid of
      `WINDOW`-sized tiles (stride `WINDOW`, the last column/row clamped
      inside the image the way `window_at` clamps, so the whole image is
      covered) and returns the maximum `laplacian_variance` over the tiles
    - `score_preview`'s input says whether an AF point is trustworthy (keep
      the signature and let `scan::extract` pass `None` for a manual-focus
      frame, or take an enum; pick the one that keeps the call site obvious
      and record the choice). With a trustworthy `FocusLocation`, the window
      around `focus_point(...)` is scored exactly as before; otherwise the
      tile maximum is returned
    - `crates/core/src/scan.rs` `extract` routes: `FocusLocation` present and
      not manual focus -> focus window; otherwise -> tiles
    - Unit tests (synthetic JPEGs via the existing `jpeg` / `checker` /
      `blur` helpers):
      1. with an AF focus location, only the window around it counts
      2. without a focus location, an image sharp only in one corner scores
         higher than the same image blurred everywhere, and its score equals
         (within a small tolerance) `laplacian_variance` of the corner window
      3. a Sony frame with a centred `FocusLocation` but `FocusMode == Manual`
         scores the corner-sharp image through the tile path (score `> 0`),
         whereas the same `FocusLocation` in AF scores `< 1.0`
    - `partial::focus_point`, `decode_focus_crop` and `crates/cli` behaviour
      are unchanged
    - `mise run ci` passes
  - Implementation approach:
    - Reuse `laplacian_variance` and `Window`; no new crates
    - Run `riffle-cli scan` before/after on `~/Downloads/2026-02-01 3`
      (all-tile path) at one thread count, warm cache, and note the per-file
      cost in `learnings.md`. A note, not a gate
    - Update the module doc comment to describe both paths

- [x] Step 3: App: bump the index schema so cached scores are recomputed
  - Done when:
    - `crates/app/src/index.rs`: `SCHEMA_VERSION = 9`; `prepare` accepts 8 and
      drops `files` for `version != 0 && version < 9` (the v7 precedent),
      keeping `ratings` and `folders`; the doc comment above
      `SCHEMA_VERSION` gains a v9 sentence in the existing style
    - Tests: existing migration tests asserting `user_version == 8` are bumped
      to 9; a new test builds a v8 database with `files`, `ratings` and
      `folders` rows and checks that after `prepare` the `files` rows are gone
      and `ratings` / `folders` rows are intact
    - `mise run ci` passes
  - Implementation approach:
    - Check the v7 `folders` seeding, which selects from `files`; note in a
      comment if it now sees an empty table

- [x] Step 4: Documentation
  - Done when:
    - `README.md` "Sharpness cue" bullet says the score is taken around the
      AF focus point when the camera recorded one, and otherwise (manual
      focus, Leica DNG) as the sharpest region of the frame. If Step 2
      measured a cost change worth stating, add it to the existing
      "Sharpness scoring cost" table in the same format
    - `CLAUDE.md` "Layout": the `sharpness.rs` clause no longer says only
      "focus-point"
    - `mise run ci` passes

## Trade-offs and risks

- A maximum rewards any high-contrast sharp region, e.g. a rim-lit
  background edge on a frame whose subject is soft. This is inherent to the
  "sharp anywhere" definition; the cue remains relative.
- A strip mixing AF and MF frames compares a window score with a tile-max
  score. Both are Laplacian variances over 256 px windows, so the scale is
  comparable; noted, not addressed.

## Progress

- (2026-09-21) Step 1 complete
- (2026-09-21) Step 2 complete
- (2026-09-21) Step 3 complete
