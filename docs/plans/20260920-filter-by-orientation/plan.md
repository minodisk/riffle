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

# Filter by orientation (portrait / landscape)

## Purpose

The filter menu narrows the strip by pick flag, stars, colour label and the
shooting settings, but not by whether a frame is portrait or landscape. When
culling for a layout that needs one shape (a cover, a vertical feed, a
double-page spread), the user has to eyeball every thumbnail. After this work
the filter menu has a `Portrait` / `Landscape` group that behaves like the
other static groups: checks within the group are OR-ed, the group is AND-ed
with the rest, and `Reset` clears it.

## Background (from investigation)

- The index stores no image dimensions. `files` in
  `crates/app/src/index.rs` has an `orientation INTEGER` column (the EXIF
  `Orientation` tag, IFD0 0x0112, read by `parse` in
  `crates/core/src/arw.rs` for both ARW and DNG, defaulting to 1 when the tag
  is absent) and nothing for width/height. `IndexedFile.orientation: u16`
  already reaches the frontend through `Index::entries` (`crates/app/src/index.rs`)
  and `interface IndexedFile` in `crates/app/ui/src/main.ts`, and is kept in
  the `entries` map there. **No schema change or migration is needed**; the
  feature is frontend-only.
- EXIF rotation is *not* applied to anything stored: the thumbnail blob is the
  unrotated preview ("Unrotated thumbnail JPEG; the caller carries the
  Orientation", `crates/core/src/scan.rs`), and the strip and viewer rotate at
  draw time (`orientation === 6 ? "cw" : orientation === 8 ? "ccw" : ...` in
  `crates/app/ui/src/strip.ts`; `quarterTurn` in `main.ts`). Every Sony and
  Leica sensor Riffle reads is landscape, so the displayed shape is decided by
  the tag alone: 6 or 8 (a quarter turn) is portrait, 1 or 3 is landscape.
- A row whose extraction failed stores `orientation = NULL`, and `entries`
  reports it as `1`; such a row also has `exif = None` / `has_thumb = false`.
  Per the user's decision, these count as `Landscape` (option A below).
- Square images cannot occur: the classification uses the rotation tag of a
  landscape sensor, not the pixel dimensions, so there is no width == height
  case to handle. (A DxO crop lives in the `.dop`, not in the raw, and Riffle
  does not read crops.)
- The filter's pure logic is `passes` in `crates/app/ui/src/filter.ts`
  (tested in `filter.test.ts`); the static groups (flags, stars, labels) are
  `role="menuitemcheckbox"` buttons in `crates/app/ui/index.html` separated by
  `<hr />`, selected in `main.ts` by
  `filterMenu.querySelectorAll("[data-flag], [data-stars], [data-label]")`,
  and mirrored in `filterChanged`, the per-item click handler, the
  `filter-reset` handler, and the two `filterToggle.classList.toggle("active", ...)`
  expressions. The EXIF groups are built dynamically into `#filter-exif`
  from values present in the folder; orientation has a fixed two-item domain,
  so it follows the static-group pattern, not the EXIF one.

## Steps

- [ ] Step 1: Add the orientation group to the filter logic and the filter menu
  - Done when:
    - `crates/app/ui/src/filter.ts` exports `type Orientation = "portrait" | "landscape"`
      and a pure `orientationOf(tag: number): Orientation` (6 and 8 ->
      `"portrait"`, everything else -> `"landscape"`); `FilterState` gains
      `orientations: Set<Orientation>`; `passes` takes the file's orientation
      tag (as a fourth parameter, `number | undefined`, beside `exif`) and
      ANDs the new group like the others: an empty set passes everything,
      otherwise the file passes iff its `orientationOf(tag)` is checked. An
      `undefined` tag (the entry has not arrived yet) fails a non-empty
      selection, the same way a missing `exif` fails an EXIF selection.
    - `crates/app/ui/src/filter.test.ts` covers `orientationOf` for 1, 3, 6,
      8 and an out-of-range value, and `passes` for: empty group passes
      both shapes; `portrait` passes 6/8 and fails 1/3; `landscape` the
      reverse; both checked passes both; `undefined` fails a non-empty
      selection; the group ANDs with the existing ones (e.g.
      `untagged` + `portrait`). The existing `state()` helper is extended
      with an `orientations` argument (default empty) so the current tests
      stay as they are.
    - `crates/app/ui/index.html` has, after the `No label` item and before
      `<div id="filter-exif">`, an `<hr />` and two buttons in the same shape
      as the label items: `role="menuitemcheckbox"`, `aria-checked="false"`,
      `data-orientation="portrait"` with text `Portrait`, and
      `data-orientation="landscape"` with text `Landscape`. No `.dot` icon
      (that is the pick-flag / colour-label marker), and no heading, matching
      the other static groups.
    - `crates/app/ui/src/main.ts`: a `shownOrientations = new Set<Orientation>()`
      beside `shownFlags` / `shownStars` / `shownLabels`; the `filterItems`
      selector includes `[data-orientation]`; the per-item click handler and
      `filterChanged` map `data-orientation` to `shownOrientations` the same
      way they map the other three; the `filter-reset` handler clears it; both
      `"active"` toggles count it; the local `passes(path)` passes
      `shownOrientations` in the state and `entries.get(path)?.orientation`
      as the tag.
    - Selecting `Portrait` and/or `Landscape` narrows the grid; `Reset` and
      unchecking restore it; the filter button lights up while either is
      checked. Verified by hand in `mise run tauri:dev` on a folder with both
      shapes, and noted in `learnings.md`.
    - `mise run ci` passes.
  - Implementation approach:
    - Frontend only; do not touch `crates/app/src` or `crates/core`.
    - Keep `Judgement` as it is (rating / pick / label); the orientation is
      index data like `exif`, so it travels as a separate argument.
    - Style is already covered by the `#filter-menu button` rules in
      `crates/app/ui/style.css`; no CSS change is expected. If the two
      text-only items look misaligned next to the dotted label items, note
      it in `learnings.md` rather than adding new rules speculatively.
    - Match the existing ternary chains in `filterChanged` and the click
      handler rather than restructuring them.

- [ ] Step 2: Document the orientation group in the README
  - Done when:
    - The **Filter menu** bullet in `README.md` lists orientation
      (`Portrait` / `Landscape`) among the groups, and states in one clause
      that it is decided by the file's EXIF Orientation (a quarter turn is
      portrait), so a frame the camera did not tag as rotated counts as
      landscape.
    - `mise run ci` passes (lint covers the Markdown).
  - Implementation approach:
    - Assumes Step 1 is merged.
    - User-facing wording only (see the README content policy in auto
      memory): no implementation detail beyond the one clause above.

## Trade-offs and risks

- **Source of the orientation** (decided at planning time, easy to revisit):
  - EXIF `Orientation` tag already in the index (chosen): zero backend
    change, no migration, no rescan of existing caches. Correct for every
    camera Riffle supports, since their sensors are landscape.
  - Alternative: add `width` / `height` columns (bump `SCHEMA_VERSION` to 9,
    extend the `prepare` migration and both `INSERT` statements, parse
    `ImageWidth` / `ImageLength` in `crates/core/src/arw.rs`, extend
    `IndexedFile`), then compare `width` and `height` after applying the
    tag. Costs a schema migration and a rescan of every folder, and for the
    supported cameras yields the same answer as the tag. Only worth it if a
    square- or portrait-sensor source is ever added. Not taken.
- **Square images**: not a case under the chosen design (the tag of a
  landscape sensor is never square). If the width/height alternative is
  ever taken, a rule is needed (own `Square` item, or fold into
  `Landscape`).
- **Files whose extraction failed**: `entries` reports `orientation = 1`
  for them, so they count as `Landscape`. Chosen by the user over the
  alternative of treating them as unknown: it is the simplest rule, and a
  broken file shows as landscape in the grid anyway (no thumbnail, no
  rotation).
- **Orientations 2, 4, 5, 7 (mirrored variants)**: no camera writes them;
  5 and 7 are mirrored quarter turns. `orientationOf` maps only 6 and 8 to
  portrait, matching how `strip.ts` / `main.ts` already rotate (they treat
  5 and 7 as no rotation). Keeping the two in step avoids a frame that is
  drawn landscape but filtered as portrait. Not worth special-casing.
- **Step count**: two steps rather than three, because `FilterState` gaining
  a required field forces the `main.ts` call site to change in the same PR
  as `filter.ts`; splitting the markup from the logic would leave a step
  that is not independently reviewable.

## Progress

- (none yet)
