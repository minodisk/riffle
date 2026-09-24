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

## Step 2

- The home volume's duplicate is dropped by canonical equality, plus, on
  macOS only, by "a volume whose canonical path is an ancestor of home":
  `/Volumes/Macintosh HD` resolves to `/`, which never equals the home
  directory. That ancestor rule is macOS-specific: on Windows home is
  `C:\Users\<name>`, an ancestor of which is `C:\` itself, but `C:\` is a
  real, independently browsable root (unlike macOS's volume alias for `/`),
  so applying the same rule there would silently drop it from the tree. A
  consequence of the macOS rule: when home itself lives on an external
  volume, that volume is not listed separately.
- Windows skips entries with `FILE_ATTRIBUTE_HIDDEN` (a `MetadataExt`
  one-liner) in addition to dot-names. The `x86_64-pc-windows-gnu` target is
  installed in this WSL environment, so `cargo check --target
  x86_64-pc-windows-gnu` in `crates/app` verifies the `cfg(windows)` code
  locally.
- On Linux `/mnt` also lists WSL's `wsl` and `wslg` directories next to
  `c` / `d`; the plan lists every child directory of `/mnt`, so they appear as
  roots.

## Step 3

- The DOM part lives in `crates/app/ui/src/folders.ts` next to the pure
  `tree.ts`. `reveal` awaits a promise that settles when `folder_roots` has
  answered, since the launch-time reopen of the last folder can reach
  `openDirectory` before the roots arrive; the roots are asked for after
  `Promise.allSettled` of the `sort_order` and `shortcuts` invokes.
- `reveal` takes a `stillCurrent` closure (`token === folderToken` in
  `openDirectory`) instead of minting its own counter, per the guide's "one
  token" rule, and checks it after every `list_subfolders` await.
- The chain `ancestorsWithin` returns includes the open folder itself, and
  `reveal` expands it too, so the open folder's own RAW count badge shows and
  its subfolders are one click away.
- `ancestorsWithin` builds the chain in the root's spelling plus the path's
  own segments, which matches the keys `list_subfolders` produces
  (`dir.join(name)`). On a case-insensitive file system a folder opened with
  a differently cased path (other than the drive letter) finds no node at
  the first mismatching level; the reveal then stops expanding there and
  nothing is highlighted.
- `String.prototype.replaceAll` was avoided in `tree.ts` (a regex `replace`
  instead) because the build targets `safari13`, which lacks it.

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
- `folder_roots` on WSL lists `/mnt/wsl` and `/mnt/wslg` (WSL's internal
  mounts) as roots next to `/mnt/c` / `/mnt/d`, since every child directory of
  `/mnt` is taken. A filter (e.g. only single-letter drive mounts under
  `/mnt` on WSL) could hide them. Basis: Step 2 implementation. Files:
  `crates/app/src/folders.rs` (`volumes`).
- A folder opened with a path whose case differs from the listing's (possible
  on macOS / Windows, e.g. a typed or dropped path) is not revealed past the
  first mismatching level, since `tree.ts` compares paths case-sensitively
  apart from the drive letter. `reveal` could match children
  case-insensitively on those platforms. Basis: Step 3 implementation. Files:
  `crates/app/ui/src/tree.ts` (`ancestorsWithin`),
  `crates/app/ui/src/folders.ts` (`reveal`).
- A `list_subfolders` failure collapses the row again and reports through
  `setStatus`; the row itself carries no error mark, so an unreadable folder
  looks like an unexpanded one once the status line changes. Basis: Step 3
  implementation. Files: `crates/app/ui/src/folders.ts` (`toggle`).
- `folder_roots` is invoked once at launch (`loadRoots`) and never again, so a
  card or drive mounted afterward (a new `/Volumes/*`, `/media/*/*` or drive
  letter) does not appear in the tree until the app restarts. Re-invoking
  `folder_roots` and `addRoots` on window focus or on each `reveal` would
  cover it. Basis: review feedback, Round 1 item 2. Files:
  `crates/app/ui/src/folders.ts` (`loadRoots`), `docs/usage.md`.
- A folder reachable from two roots (e.g. on Windows, the home root
  `C:\Users\me` and `C:\` > `Users` > `me`, since Step 2 keeps both; or any
  root under `/` that `rootOf` adds) shares one `TreeNode`, keyed by path
  alone. Expanding either row expands both, and the subtree and the
  `.current` highlight are drawn twice. Keying `expanded` (and the map itself)
  by the row's root-plus-path, or not listing a root's own path as a child of
  another root, would fix it. Basis: review feedback, Round 1 item 4. Files:
  `crates/app/ui/src/tree.ts` (`TreeNode`, `Tree.nodes`).
