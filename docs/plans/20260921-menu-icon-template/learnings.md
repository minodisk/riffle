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
