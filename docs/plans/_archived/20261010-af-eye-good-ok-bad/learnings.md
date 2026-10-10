# Learnings

## Step 1

- `README.ja.md`'s `AF eye` item list still read `Good` / `Sharp` / `Soft` /
  `Unknown` (it had missed the `Sharp only` rename of #756), so a plain
  `Sharp only` search did not find it; it now reads `Good` / `OK` / `Bad` /
  `Unknown` like `README.md`.
- The `not_candidate` test in `filter.test.ts` is now
  `"not_candidate (Bad) passes a bad frame only"`; its `soft` local variable
  was left as is (no assertion or body change).

### Icons and colors (Step 1 extension)

- The strip's state comes from the new pure `stripState` in `focus.ts`
  (`markState` wrapped so a file without a focus is `unknown`), and the icon
  per state from `FOCUS_MARK_ICONS` next to `FOCUS_MARK_COLORS`; both are
  pinned in `focus.test.ts`. `markState` itself and `photoTier` are unchanged.
  `strip.ts` keeps its `tiers` / `setTier` / `.tier` names but now stores any
  state other than `unknown` and swaps the cell's SVG per state.
- `stripState` does not look at `manual_focus` (unlike `focusMark`, which
  returns `null` there), so the strip icon follows the same `candidate` the
  filter's `AF eye` items read.
- The gray chosen for `not_candidate` is `#999`: distinct from the `#fff`
  of `unknown`, and readable on the dark UI and, with the mark's dark outline
  and the cell icon's `drop-shadow`, on photos.
- `scan-eye` and `scan` were copied verbatim from `lucide-static@1.48.0`
  (unpkg); neither is in `LICENSE-lucide`'s Feather list, so only the
  `icons.ts` header's name list changed.

## Deferred issues (todo candidates)

- `CLAUDE.md` (the `src/focus.ts` sentence) still says `photoTier` "puts the
  strip's face icon on a good frame"; the strip now shows a per-state icon
  (`FOCUS_MARK_ICONS`, `stripState` in `crates/app/ui/src/focus.ts`). Left
  for the wrap-up since this step does not edit `CLAUDE.md`. Basis: Step 1
  icon extension.
- `todo.md` (around line 597) mentions "another candidate draws dim green";
  the candidate-only mark is now bright green. Basis: Step 1 icon extension
  (`FOCUS_MARK_COLORS` in `crates/app/ui/src/focus.ts`).
- Pending manual check (desktop app, any platform): open a folder with faced
  AF frames, wait for `analyzing N / M` to finish, and check that the strip
  shows a bright green `scan-face` on good frames, a bright green `scan-eye`
  on other focus candidates, a gray `scan` on not-sharp frames and nothing on
  unknown ones; that the filter menu's `AF eye` items show the same icons
  (`Unknown` an empty aligned slot); and that the `f` mark is bright green
  for good and OK frames, gray for Bad and white for unknown, with the gray
  readable on photos. The step's checkbox was ticked on the automated
  criteria (types, tests, `mise run ci`).
