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

# Within-burst navigation and a clearer burst mark on the strip

## Purpose

`ArrowLeft` / `ArrowRight` jump between bursts, but stepping through the
frames of one burst is only possible with the plain `ArrowUp` / `ArrowDown`,
which run straight past the burst's ends. New `burstFramePrevious` /
`burstFrameNext` actions (`Alt+ArrowUp` / `Alt+ArrowDown` by default) move one
displayed frame at a time inside the current burst and stop at its ends, so a
burst can be compared frame by frame without overshooting.

The 2px `#888` bracket in the strip gutter that marks a burst is hard to see.
It is replaced by a tinted background band behind the burst's cells and a
count badge, which also replaces the `· n / m in burst` suffix of the
position counter.

## Steps

- [x] Step 1: `burstFramePrevious` / `burstFrameNext` actions
  - Done when:
    - `crates/app/ui/src/burst.ts` exports a pure helper (e.g.
      `burstFrameStep(ids, current, direction)`) with the same signature
      shape as `burstStep`: over the displayed files' burst ids it returns
      `current ± 1` when that neighbour has the same id, else `current`
    - `crates/app/ui/src/burst.test.ts` covers: stepping down and up inside a
      burst, clamping at the first and last displayed member, a singleton
      (unique id) is a no-op in both directions, `[]` is a no-op, and ids
      left non-adjacent by a filter or a sort (e.g. `[0, 3, 3, 7]` and
      `[0, 1, 0]`) only step within a run of equal ids
    - `crates/app/src/shortcuts.rs` `DEFAULTS` gains
      `("burstFramePrevious", &["alt+arrowup"])` and
      `("burstFrameNext", &["alt+arrowdown"])` right after `burstNext`;
      `the_defaults_are_the_full_table` lists them; a test asserts
      `forbidden("alt+arrowup", …)` / `"alt+arrowdown"` is `None` on both
      platforms (the existing `the_defaults_bind_no_key_twice` and
      `the_menu_defaults_are_not_forbidden` also cover the new rows)
    - `crates/app/ui/src/settings.ts` `shortcutLabels` names them
      (`Previous frame in burst` / `Next frame in burst`), so they are
      rebindable in the shortcuts panel like every other action
    - `crates/app/ui/src/main.ts` `runAction` dispatches both to a
      `moveBurstFrame(direction)` next to `moveBurst`, built on the same
      `ids` array (`bursts.get(path)?.burst ?? -1 - at`), returning early
      when the index does not change and otherwise setting `index`,
      `pageKeypressAt` and calling `show()`
    - `README.md` Features "Bursts" bullet mentions `Alt+ArrowUp` /
      `Alt+ArrowDown`; `docs/usage.md` Bursts paragraph and Keys table gain
      the two keys
    - `mise run ci` passes
  - Implementation approach:
    - Menu accelerator check done at planning: `MACOS_MENU`, `MACOS_SYSTEM`,
      `OTHER_MENU`, `OTHER_SYSTEM` in `shortcuts.rs` contain no
      `alt+arrowup` / `alt+arrowdown`, and the `app_menu` items in
      `crates/app/src/main.rs` use only `CmdOrCtrl+…` accelerators, so there
      is no collision. `ctrl+arrow*` is reserved (Mission Control) and
      `shift+arrowup/down` is taken by
      `docs/plans/20260922-strip-multi-select/plan.md` Step 4, hence Alt
    - `keys.ts` `keyName` names the combination `alt+arrowup` from
      `event.code` `ArrowUp`; no change to `keys.ts` is needed
    - Follow `docs/agents/tauri-app.md` "An accelerator string is not
      validated until Tauri parses it": these actions have no menu item, so
      `accelerator_for` is never called for them; nothing to add there
    - The multi-select plan's Step 4 inserts `extendPrevious`/`extendNext`
      "after `burstNext`" too; whichever lands second rebases the
      `DEFAULTS` order and the full-table test
    - If a key name in a doc string is shown in the platform style, write
      `Alt+ArrowUp` / `Option+ArrowUp` the way README already writes
      `Shift+x`; keep it short

- [x] Step 2: Burst band and count badge on the strip, replacing the bracket and the counter suffix
  - Done when:
    - `crates/app/ui/src/burst.ts` `BurstMark` carries what a cell needs to
      draw both the band and the badge: `first`, `last` (as now) plus
      `position` and `size` from `BurstMember` (whole-folder values, as the
      removed status text showed); `burstMarks` fills them and
      `burst.test.ts` is updated
    - `crates/app/ui/style.css` drops `.cell.burst::after` and its
      `burst-first` / `burst-last` caps and instead draws a subtle tinted,
      rounded band behind every cell of a burst that also fills the
      `--cell-gap` above a non-first member so the burst reads as one block
      (top corners rounded on `burst-first`, bottom on `burst-last`); the
      `.cell.current` highlight and `.cell.failed` stay visible on top of it
    - `crates/app/ui/src/strip.ts` `createCell` adds a badge span and
      `paintBurst` fills it: the first displayed cell of a burst run shows
      the burst size (`5`); the current cell, when in a burst of 2+, shows
      `position + 1`/`size` (`3/5`) instead — including when it is that
      first cell; other cells show no badge. `setCurrent` / `highlight`
      repaint the badge of the cell that lost and the one that gained
      `current`
    - `crates/app/ui/src/main.ts` `renderMeta` sets `positionEl` to
      `${index + 1} / ${files.length}` only; the `· n / m in burst` suffix
      is gone
    - `docs/usage.md` Bursts paragraph describes the band and the badge
      instead of the bracket and the counter suffix; README's "bracketed
      together" wording is adjusted if it no longer matches
    - `mise run ci` passes and the change is checked by hand in
      `mise run tauri:dev` on a folder with bursts (band visible, badge on
      the first cell, `n/m` follows the selection, gap filled)
  - Implementation approach:
    - Cells are absolutely positioned inside `#strip` `inner` at
      `index * --cell-height` with `--cell-gap: 8px` between them
      (`style.css` ~L273-290), so a `::before` pseudo-element on
      `.cell.burst` with `top: calc(-1 * var(--cell-gap))` on non-first
      members bridges the gap, as the old bracket's `top: calc(-1px -
      var(--cell-gap))` did. Verify the band stays behind the image, name,
      rating and flag spans (e.g. `z-index: -1` on the pseudo-element inside
      a cell that establishes a stacking context, or `isolation: isolate` on
      `.cell`) and that the band reaches slightly outside the cell box so it
      still reads as a frame around a `.cell.current` background
    - The app is dark only (`body { background: #1c1c1c }`, no
      `prefers-color-scheme` rules in `style.css`); a translucent white tint
      (e.g. `rgba(255, 255, 255, 0.08)`-ish) is legible on the dark
      background and would also survive a future light theme better than a
      fixed grey. Measure by eye that a band is distinguishable from a
      `.cell.current` `#333` background and from `.cell.failed`
    - Badge placement: the top-right corner is taken by `span.rating` and
      the left edge by `span.sharpness`; put the count badge top-left
      (`.cell span.count`, styled like `span.rating`), and confirm it does
      not overlap the sharpness bar
    - Only `paintBurst` and `highlight` in `strip.ts` need the current
      index; `current` is already module state there, so no new API is
      needed from `main.ts`
    - `burstMarks` stays adjacency-based (a burst split by a non-capture
      sort becomes two runs, each with its own band and badge); see
      Trade-offs
    - `docs/agents/tauri-app.md` and archived learnings mention the
      `N / M · a / b in burst` text as the reason for `width: min-content`
      on `#side`; leave those notes as history, do not edit them

## Trade-offs and risks

- Run-based vs id-based stepping (Step 1). `burstStep` walks runs of equal
  ids in the displayed list, so a burst split apart by a rating or name sort
  is two runs. The plan keeps the new helper run-based for consistency with
  `burstStep` and with the band drawn in Step 2 (one band per run). The
  alternative — jump to the nearest displayed index anywhere with the same
  id — would hop across unrelated files when the sort splits a burst, and
  would disagree with the visible band.
- Alt+arrow on each platform. Nothing in the app menu or the reserved
  tables claims `alt+arrowup/down`. On Windows `Alt+Up` is an Explorer
  shortcut, not a global one, and on macOS `Option+Arrow` is text-field
  word navigation, which does not apply here as the main window has no
  focused text field. Check once by hand that the webview does not swallow
  the key (record in `learnings.md`).
- Ordering with the multi-select plan. Both plans append rows after
  `burstNext` in `DEFAULTS`; the second one to merge rebases the table and
  the full-table test. No functional conflict.
- Badge meaning across sort and filter (Step 2). The badge shows the
  whole-folder size (`BurstMember.size`), so a burst of 5 with 3 members
  displayed under a filter shows `5` on its first displayed cell; the band
  covers the 3 shown. That matches what the removed status text showed and
  what `rejectRest` acts on.
- Light-theme legibility. There is no light theme today; a translucent tint
  is the choice least likely to break when one appears, but it cannot be
  verified against a light theme now.

## Progress

(2026-09-22) Step 1 complete
(2026-09-22) Step 2 complete
