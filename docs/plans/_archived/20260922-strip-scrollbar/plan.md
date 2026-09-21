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

# Thumbnail strip scrollbar on classic-scrollbar platforms

## Purpose

On Windows (WebView2) the thumbnail strip `#strip` shows the classic white
~17px scrollbar. `#side` is a fixed 160px, cells sit at `left: 8px; width:
144px` (right edge 152px) and the burst bracket `.cell.burst::after` at
`right: -6px; width: 3px` (155-158px), and `#strip` has `overflow-x: hidden`,
so the scrollbar eats the strip's content width: the right edge of every
thumbnail is clipped and the burst bracket is hidden entirely. The white bar
also clashes with the dark theme. A user screenshot confirmed this. After this
work, the whole cell and the bracket are visible on classic-scrollbar
platforms, the scrollbar is thin and dark, and macOS (overlay scrollbar) looks
exactly as today.

## Steps

- [x] Step 1: Style the strip scrollbar for the dark theme and let the side
      column grow by the scrollbar's footprint
  - Done when:
    - `#strip` has a thin, dark scrollbar (`scrollbar-width: thin;
      scrollbar-color: #444 transparent` or similar) in
      `crates/app/ui/style.css`.
    - On a classic-scrollbar platform the strip's content box is at least
      158px wide, so the full cell and the burst bracket are visible; on macOS
      with overlay scrollbars `#side` still measures 160px.
    - `strip.ts` is untouched (it reads no widths).
    - `todo.md`'s "App: burst grouping's manual checks are still open" section
      gets one extra unchecked item asking the user to confirm on Windows that
      the strip scrollbar is thin and dark and that the thumbnail right edge
      and the burst bracket are no longer clipped (and that the macOS strip
      is unchanged).
    - `mise run ci` passes.
  - Implementation approach:
    - CSS only, in `crates/app/ui/style.css`; keep the change to `#side`,
      `#strip`, `#strip-inner` and, if needed, the `.cell*` rules. Do not
      touch `crates/app/ui/src/strip.ts` or `index.html`.
    - Preferred approach (A, "let the scrollbar add to the width"): give
      `#strip-inner` a fixed `width: 160px` (the current design width, with
      the 8px gutters on both sides), set `#strip { scrollbar-gutter: stable; }`,
      and change `#side` from `flex: 0 0 160px` to `flex: none` (width from
      content) so the column's intrinsic width becomes 160px plus whatever the
      scrollbar gutter occupies: 0 on macOS overlay scrollbars, ~11px for a
      Chromium `thin` scrollbar on Windows. Cells and the bracket keep their
      current absolute offsets. Keep the existing comment on `#side` intact.
    - Verify during implementation that Chromium includes the
      `scrollbar-gutter: stable` gutter in the element's intrinsic
      (max-content) width, so that `#side` actually widens. If it does not,
      fall back to approach B and record the finding in `learnings.md`.
    - Fallback approach (B, "fixed extra width"): keep `#side` at a fixed
      width but raise it to at least 171px with
      `#strip { scrollbar-gutter: stable; }`. This leaves ~11px of dead space
      on the right on macOS; note it in the PR if taken.
    - Verify `#tools`, `#filter`, `#sort` and `#position` still lay out
      correctly when `#side`'s width comes from its content.
    - GUI cannot be driven from this machine: report the Windows and macOS
      visual results as "not verified" in the PR and add the `todo.md`
      manual-check item above rather than claiming a pass.

## Trade-offs and risks

- Approach A vs B: A keeps macOS pixel-identical and adapts to whatever
  scrollbar width the platform uses, but relies on Chromium counting the
  stable scrollbar gutter in intrinsic sizing. B is guaranteed to work but
  wastes ~11px on macOS. The step tries A first and falls back to B.
- Not taken: shrinking the cell (absolute-pixel sizing throughout), hiding the
  scrollbar (`scrollbar-width: none`, loses the scroll cue), and styling every
  scrollbar via `:root` (wider than the report). The user approved the
  `#strip`-only scope.
- `scrollbar-width` / `scrollbar-color` need Chromium 121+; on an older
  WebView2 the bar stays white but the width fix still applies.

## Progress

- Step 1: Took approach A. `#strip-inner` is fixed at `width: 160px`, `#strip`
  has `scrollbar-gutter: stable` and a thin, dark scrollbar
  (`scrollbar-width: thin; scrollbar-color: #444 transparent`), and `#side`
  changed from `flex: 0 0 160px` to `flex: none; width: min-content` so the
  column's intrinsic width (the strip's fixed 160px plus its scrollbar
  gutter) sets the column width. The Windows/macOS visual result is left to
  the manual check added to `todo.md`.

- (2026-09-22) Step 1 complete
