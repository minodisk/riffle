# Learnings

## Step 1: patch muda to the template-image fork

- `cargo tree -i muda` confirms the fork is in use:

  ```
  muda v0.19.3 (https://github.com/minodisk/muda?rev=ef4fbfda53416382a2eeca7da683a64a8fe0e8ed#ef4fbfda)
  └── tauri v2.11.6
  ```

  `Cargo.lock` now records
  `source = "git+https://github.com/minodisk/muda?rev=ef4fbfda…"` and the
  crates.io `checksum` line is gone.
- `cargo update -p muda` also flipped an unrelated line, swapping `tempfile`'s
  `getrandom 0.4.3` for `getrandom 0.3.4`. Restoring that line by hand and
  re-resolving (`cargo metadata`) left it restored, so the committed lockfile
  diff is the muda source change only.
- PNG verification (throwaway Swift snippet reading
  `NSBitmapImageRep.colorAt(x:y:)`, not committed): the glyph bounding boxes
  are byte-identical before and after — `gearshape` (5,5)-(30,30),
  `arrow.uturn.backward` (7,7)-(27,29), `folder` (5,7)-(29,28) — while an
  opaque pixel went from `r=0.557 g=0.557 b=0.576` (the baked `#8E8E93`) to
  `r=0 g=0 b=0`. Only the colour changed; geometry, canvas (18pt) and
  `pointSize` (12) are untouched.
- `sw_vers` on the regeneration machine: macOS 26.6.2 (25G83) — the same value
  the script's header already carried, so that line did not need editing.
- The git dependency behaved unremarkably locally: cargo fetched
  `github.com/minodisk/muda` once into `~/.cargo/git/db` and every later
  command hit the cache.

## Deferred issues (todo candidates)

- `todo.md`'s `### App: custom menu-item icons don't tint for dark mode` is now
  only partly true: the tinting works, but the temporary
  `[patch.crates-io]` entry in the workspace `Cargo.toml` still needs removing
  once https://github.com/tauri-apps/muda/pull/413 ships in a muda release
  Tauri resolves **and** Tauri exposes the template flag for menu items. Basis:
  this step's implementation plus the plan's "Trade-offs" note that the item
  should be rewritten to the remainder rather than deleted. Files:
  `Cargo.toml`, `docs/agents/tauri-app.md`,
  `tools/macos/export-menu-icons.swift`.

### Visual pass: confirmed

The user confirmed in the running app (`mise run tauri:dev`) that the four
PNG-backed menu items (`Settings...`, `Open Folder…`, `Undo`,
`Open Log Folder`) now tint with the menu appearance — white in dark mode,
black in light mode — and invert together with the label when an item is
highlighted. That last part is what the baked `#8E8E93` grey could never do:
a fixed-colour icon stayed grey while the row's text went white under the
highlight, which is where the mismatch was most visible.

### Scope addition: `Move Rejected to Trash` was the last untinted icon

After the visual pass the user spotted that `Move Rejected to Trash…` alone
stayed colour. Measured on this machine with a throwaway AppKit script:
`NativeIcon::TrashFull` resolves to `NSTrashFull`, a 32x32 colour Finder icon
with `isTemplate == false`, while every other icon in the menu is a template
(`NSFollowLinkFreestandingTemplate`, `NSRefreshTemplate`, and the bundled PNGs
via the muda fork). So it did not follow dark/light mode, did not invert under
the row highlight, and carried a different visual density.

- Fixed by adding `"trash"` to the export script's `symbols` and switching the
  item to `IconMenuItem::with_id` with `icons/menu/trash.png`. The three
  existing PNGs came out byte-identical (same MD5s), so only `trash.png` is
  new. `NativeIcon` stays imported for `FollowLinkFreestanding` / `Refresh`.
- Bounding boxes (36x36 canvas, alpha-only, opaque pixel `r=g=b=0`, no edge
  contact): `gearshape` (5,5)-(30,30), `arrow.uturn.backward` (7,7)-(27,29),
  `folder` (5,7)-(29,28), `trash` (6,5)-(29,31). The new icon sits in the same
  size band as its neighbours.
- **General lesson**: a `NativeIcon` is only a good neighbour if its underlying
  `NSImage` is a template image. The colour Finder-style ones (`NSTrashFull`,
  `NSTrashEmpty`, `NSFolder`) are not, so they clash in a menu whose other
  icons tint. Check `isTemplate` before reaching for a `NativeIcon`.

After a second visual pass the user compared the `Settings...` icon against
Ghostty's and found the shape differed. Ghostty uses the SF Symbol `gear` for
its configuration item; this repo exported `gearshape`. They are two distinct
symbols — `gear` is the classic spoked cog with fine teeth and a hub,
`gearshape` a rounded 8-tooth outline — so the export script's `symbols` now
lists `"gear"` and `crates/app/icons/menu/gearshape.png` is gone.

- Bounding boxes after the swap (36x36 canvas, alpha-only, no edge contact):
  `gear` (4,6)-(30,31) = 27x26, `arrow.uturn.backward` (7,7)-(27,29) = 21x23,
  `folder` (5,7)-(29,28) = 25x22, `trash` (6,5)-(29,31) = 24x27. `gearshape`
  measured 26x26 at the same `pointSize: 12`, so the finer teeth cost one pixel
  of width and nothing else; no `pointSize` change was needed. The other three
  PNGs came out byte-identical (same MD5s).
- **General lesson**: when matching a reference app's menu icon, check which SF
  Symbol it actually passes to `NSImage(systemSymbolName:)` rather than
  assuming from the name. Near-synonyms like `gear` / `gearshape` (and their
  `.fill` / `.circle` variants) render very differently at menu size.

### Scope addition: `Reload Folder` was the last item without an icon

With `Move Rejected to Trash…` and `Settings...` settled, `Reload Folder` was
the only menu item Riffle owns that still had no icon at all. Added
`"arrow.clockwise"` to the export script's `symbols` and switched the item to
the `IconMenuItem::with_id` / `MenuItem::with_id` `cfg` pair its neighbours
use; its id, label, enabled state and `CmdOrCtrl+R` accelerator are unchanged.

- Symbol choice: `arrow.clockwise`, the plain circular-refresh symbol. Ghostty
  uses `arrow.trianglehead.2.clockwise.rotate.90` for its Reload Configuration
  item, but that is a macOS 26-era symbol; `arrow.clockwise` has been available
  far longer and reads the same at menu size. Rendered at 12pt it also measures
  21x26 against the trianglehead variant's 29x25, which would have been the
  widest glyph in the menu.
- Confusability check against `arrow.uturn.backward` (Undo): not an issue.
  Rendered side by side as alpha art at `pointSize: 12`, `arrow.clockwise` is a
  closed ring with a gap and an arrowhead at the top, while
  `arrow.uturn.backward` is an open U with a long horizontal bar and an
  arrowhead pointing left. The silhouettes differ at a glance.
- Bounding boxes (36x36 canvas, alpha-only, no canvas-edge contact): `gear`
  27x26, `arrow.uturn.backward` 21x23, `folder` 25x22, `trash` 24x27,
  `arrow.clockwise` 21x26. In line with its neighbours; no `pointSize` change.
  The other four PNGs came out byte-identical (unchanged in `git status`).
- **CI failure**: moving the last macOS-side `MenuItem::with_id` to
  `IconMenuItem` left `use tauri::menu::MenuItem;` unused on macOS
  (`-D unused-imports`). The import now carries
  `#[cfg(not(target_os = "macos"))]`, mirroring the `Image` / `IconMenuItem`
  imports above it.

### Scope addition: `Check for Updates…` duplicated `Reload Folder`'s circular arrow

Giving `Reload Folder` `arrow.clockwise` made two unrelated items show the same
glyph: `Check for Updates…` was still on `NativeIcon::Refresh`
(`NSRefreshTemplate`), also a circular arrow. The circular arrow genuinely
belongs to "reload", so `Check for Updates…` moved instead, to a PNG-backed
`square.and.arrow.down` — checking for updates is really a download, and
Ghostty uses the same symbol for its Check for Updates item.

- **Lesson**: when adding a menu icon, look at every icon in the app's menus
  together, not just the item being changed. A symbol that is right in
  isolation can collide with a neighbour's meaning, and the collision is only
  visible with the whole menu in view. The previous addition picked
  `arrow.clockwise` on its own merits and never compared it with the
  `NativeIcon`-backed items, which is exactly how the duplicate slipped in.
- Bounding box (36x36 canvas, alpha-only, no canvas-edge contact):
  `square.and.arrow.down` 20x26, against `gear` 27x26,
  `arrow.uturn.backward` 21x23, `folder` 25x22, `trash` 24x27,
  `arrow.clockwise` 21x26. In line; no `pointSize` change. The other five PNGs
  came out byte-identical.
- `NativeIcon` stays imported: `FollowLinkFreestanding` on the PhotoLab item is
  now its only use, so `-D unused-imports` is satisfied without a `cfg` change.
