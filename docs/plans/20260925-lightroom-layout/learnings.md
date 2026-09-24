# Learnings

## Step 1

- Rotating the arrow defaults broke `a_store_from_the_old_defaults_still_loads`:
  its stored `previous` override `["arrowup", "w", "a", "h", "k"]` now
  collides with `burstPrevious`'s default `arrowup`, so `from_overrides`
  leaves the whole override inactive. Following the precedent of #278 (which
  dropped `arrowleft` from the same fixture when burst keys took it), the
  fixture now uses `arrowleft`. The other tests only needed `previous` /
  `next`'s key renamed where they remove or collide with it.
- `README.md`'s quick start (`Page through the shots with ↑ ↓`) and the
  matching line in `README.ja.md` also named the paging arrows; updated to
  `←` `→` in addition to the lines the plan listed.
- `#side` lost `width: min-content` already in this step: with only the
  `Open folder` button in it, min-content would wrap the button's label onto
  two lines. Step 3's "its min-content sizing goes" is therefore done.
- `#position` moved to the left end of `#strip-bar` and `#tools` to the right
  (`margin-left: auto`), Lightroom's filmstrip header layout. The filter and
  sort menus are right-aligned to their toggles (`right: 8px`) since they sit
  at the window's right edge, and `max-height: calc(100vh - 240px)` bounds
  them by the space above the ~220px film block.
- `scrollbar-gutter: stable` is kept on `#strip` as the plan says, but it
  only reserves the inline-axis (vertical scrollbar) gutter, so it has no
  effect on a horizontal scrollbar; see the deferred issue below.

## Deferred issues (todo candidates)

- With classic (non-overlay) scrollbars, e.g. on Windows or macOS set to
  "always show scroll bars", `#strip`'s horizontal scrollbar appears once the
  files outgrow the width and makes `#film` taller, shrinking `#viewer`
  without a window `resize` event, so the canvas is not redrawn until the
  next `draw()`. `scrollbar-gutter: stable` does not cover the block axis.
  Candidates: `overflow-x: scroll`, a fixed `#strip` height, or a
  `ResizeObserver` on `#viewer` (which Step 4's panel toggles might want
  anyway). Basis: Step 1 implementation. Files: `crates/app/ui/style.css`
  (`#strip`), `crates/app/ui/src/main.ts` (the `resize` handler).
- An existing user whose stored `shortcuts` override for `previous` / `next`
  (or another action) contains `arrowup` / `arrowdown` now has that whole
  override left inactive (logged warning), since those keys are the burst
  actions' defaults after the rotation; e.g. an old `previous: ["arrowup",
  "w", "a", "h", "k"]` store loses `w` / `a` / `h` / `k` too. The shortcuts
  panel shows the truth, but a migration could drop just the colliding keys.
  Basis: Step 1 test `a_store_from_the_old_defaults_still_loads`. Files:
  `crates/app/src/shortcuts.rs` (`Keymap::from_overrides`).
