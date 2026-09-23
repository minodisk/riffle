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

# Color label group in the filter menu

## Purpose

The filter menu (`#filter-menu` in `crates/app/ui/index.html`; `passes` /
`refilter` / `shownFlags` / `shownStars` / `shownExif` in
`crates/app/ui/src/main.ts`) narrows the strip by pick flag, stars and the
EXIF groups, but not by color label. "Only the unjudged files" therefore
cannot be expressed: flag `untagged` AND `0` stars still lets through a file
whose only judgment is a label. This work adds a color label group with a
"No label" entry so `untagged` AND `0` AND `No label` selects exactly the
unjudged files, and defines (and tests) what happens to the cursor when a
judgment makes the current file drop out of the active filter. It closes
the todo.md section "App: no color label group in the filter menu".

Current state the plan is based on:

- Filtering is frontend-only and in memory. There is no Rust-side filter and
  no persistence of the checked items; `shownFlags` / `shownStars` survive a
  folder open, the EXIF sets are cleared in `openDirectory`.
- The static items are declared in `index.html` and collected once with
  `filterMenu.querySelectorAll("[data-flag], [data-stars]")`; each has a
  `click` listener toggling its set, then `filterChanged()` mirrors the sets
  onto `aria-checked`, lights `filterToggle.active` and calls `refilter()`.
  `#filter-reset` clears every set. The pick / reject items carry an
  `<i class="dot">` colored per flag in `crates/app/ui/style.css`
  (`#filter-menu [data-flag="picked"] .dot`).
- Semantics: the checked items of one group are OR-ed, groups AND-ed, an
  empty group passes everything.
- The label is already in the frontend: `labels: Map<path, string>` holds
  the exact sidecar string (`Red`, `Orange`, ...; a foreign name such as a
  custom Lightroom label is kept as-is). `strip.ts` lowercases it and maps
  the seven names in `LABEL_COLORS` to `--label-<name>`, anything else to
  `--label-other` (gray). `applyRating` updates `labels`; `judge` then calls
  `refilter(path)`.
- Drop-out today: `refilter(anchor)` keeps the current file if it still
  passes; otherwise it moves to the next passing file after it in `allFiles`
  order, else the last passing one before it, else shows the empty view.
  This is a de-facto auto-advance under a filter, undocumented and untested.
- Frontend tests exist only for pure logic extracted from `main.ts`
  (`crates/app/ui/src/exif.ts` + `exif.test.ts`, Vitest through
  `pnpm exec vp test`, `include: src/**/*.test.ts`, node environment).
  `main.ts` itself touches `window.__TAURI__` and the DOM at module load and
  is not importable from a test.

## Decisions (user, 2026-09-19)

- Drop-out behavior: option A — the judged file vanishes at once and the
  cursor moves to the next passing file after it (else the last before, else
  the empty view). This is the existing `refilter(path)` behavior, made
  deliberate.
- Foreign label strings: no dedicated "Other" item. They pass only while the
  label group is empty and fail "No label".

## Steps

- [x] Step 1: Extract the filter predicate and the drop-out cursor rule into a pure module with tests
  - Done when:
    - A new `crates/app/ui/src/filter.ts` owns, with no DOM or Tauri
      access: the `Flag` type, the filter-state shape (the flag set, the
      stars set and the `Map<ExifGroup, Set<string>>`), a pure `passes`
      taking the state plus a file's judgment (`rating: number | null`,
      `pick: boolean`, `label: string | null`) and its `exif` (or
      `undefined`), and the anchor rule of `refilter` (given `allFiles`, a
      pass predicate and the previous current path, return the path that
      becomes current: the same path if it passes, else the first passing
      path after it in `allFiles`, else the last passing path before it,
      else `undefined`).
    - `main.ts` calls these and behaves exactly as before (same strip, same
      cursor moves); `openDirectory`, `judge`, `refreshEntries` and
      `filterChanged` are untouched apart from the call sites.
    - `crates/app/ui/src/filter.test.ts` covers the existing behavior:
      empty group passes all; OR within the flag group and within the stars
      group; AND across groups; a reject counts as `rejected` and `0`
      stars; a pick with stars counts as `picked` and its stars; an EXIF
      selection fails a file with no `exif`; and the anchor rule's four
      outcomes (stays, next after, last before, none).
    - `mise run ci` passes.
  - Implementation approach:
    - Behavior-free PR, following the repo precedent (`exif-filters`
      Step 1) so Step 2's diff is the feature alone.
    - Keep `main.ts`'s module-level sets where they are; the pure function
      takes them as arguments. Do not introduce a class or a store.
    - `exifKey` from `exif.ts` is already pure; `filter.ts` may import it.

- [x] Step 2: Color label group in the filter menu
  - Done when:
    - `index.html` gains, after the stars items and before
      `#filter-exif`, an `<hr />` and eight `menuitemcheckbox` buttons with
      `data-label`: `red`, `orange`, `yellow`, `green`, `blue`, `pink`,
      `purple` and `none` ("No label"), each with an `<i class="dot">`
      colored from the existing `--label-<name>` variables in `style.css`
      (`none` gets an outlined dot or the default gray, whichever reads as
      "no label").
    - `main.ts` holds `shownLabels: Set<string>` beside `shownFlags` /
      `shownStars`, keyed by the same lowercase names as the menu plus
      `"none"`. A file's key is `"none"` when it has no label, otherwise its
      label lowercased. `passes` (in `filter.ts`) AND-s the group in with
      the same empty-passes-all rule. The group lists all seven colors
      regardless of the selected sidecar format.
    - `filterItems` also picks up `[data-label]`; the per-item click
      listener toggles `shownLabels`; `filterChanged` mirrors `aria-checked`
      for label items; `filterToggle.active` lights when `shownLabels` is
      non-empty; `#filter-reset` clears it. Like flags and stars it is not
      cleared on folder open and is not persisted.
    - `filter.test.ts` adds: a labeled file fails `none`; a `Red` file
      passes `red` and fails `blue`; two checked colors OR; case
      insensitivity (`Red` matches `red`); a foreign label fails every
      color and `none`; and the target combination: `untagged` + `0` +
      `none` passes an unjudged file and fails a file that has only a
      label, only stars, only a pick, or only a reject.
    - `README.md` "Filter menu" lists the color label among the groups and
      names the "No label" entry, and says a label outside the seven colors
      matches no color item.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Mirror the flag items exactly (static HTML, `dataset`, per-item
      listener) rather than the EXIF rebuild path.
    - CSS: one rule per color, `#filter-menu [data-label="red"] .dot {
      background: var(--label-red); }` etc., next to the existing
      `[data-flag]` dot rules.

- [x] Step 3: Document and test the drop-out behavior (option A)
  - Done when:
    - The rule (the judged file vanishes at once; the cursor goes to the
      next passing file after it, else the last before, else the empty
      view) is stated in a comment on `refilter` and in `README.md`
      "Filter menu".
    - `filter.test.ts` covers the judgment cases: with `untagged` + `0` +
      `none` active, rating, rejecting, picking or labeling the current
      file moves the cursor to the next unjudged file after it; when none
      follows, to the last one before; when it was the only file, to the
      empty view.
    - The `todo.md` section "App: no color label group in the filter menu"
      is removed. If an auto-advance section exists in `todo.md`, it gains
      one sentence saying that under an active filter a judgment that drops
      the file already advances.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 2 is merged.
    - No new mechanism: `refilter(path)` in `judge` already does this. The
      PR is small and mostly tests and documentation, which is intended.

## Trade-offs and risks

- Option B (keep the judged file until the cursor leaves it) was rejected:
  it needs sticky state in `refilter` and makes a later auto-advance
  order-dependent. Option A's lack of visual confirmation is mitigated by
  the separate undo work.
- All seven colors are listed under both formats even though XMP's default
  keys cover only five: files labeled by another tool still carry those
  labels, and it avoids rebuilding the menu on a format switch.
- Merge risk: parallel plans (`undo-judgments`, `auto-advance`) also touch
  `main.ts`. This plan leaves the `keydown` handler untouched.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-20) Step 2 complete
- (2026-09-20) Step 3 complete
