# Learnings

## Step 1: Shrink the glyph inside the 18pt canvas

- Regenerated on macOS 26.6.2 (25G83) (`sw_vers`), which matches the header
  comment's existing "Last run on macOS" line, so no refresh was needed.
- Measured the committed PNGs with a throwaway swift script in the scratchpad
  (`NSBitmapImageRep(data:)`, alpha > 0.05 bounding box, divided by the 2x
  scale):

  | image | canvas | glyph ink | bounding box (px) | edge contact |
  |---|---|---|---|---|
  | `gearshape.png` | 18pt (36x36 px) | 15.0 x 15.0 | x[3,32] y[3,32] | no |
  | `arrow.uturn.backward.png` | 18pt (36x36 px) | 14.5 x 13.0 | x[3,31] y[6,31] | no |
  | `folder.png` | 18pt (36x36 px) | 17.0 x 13.0 | x[1,34] y[5,30] | no |

  No glyph touches pixel 0 or 35 in either axis, so nothing is clipped. The ink
  matches the plan's pre-measured expectations exactly.
- Mild surprise: `folder` leaves only 1px (0.5pt) of margin horizontally at
  `pointSize: 14`, which confirms the plan's decision that 14 is the largest
  shared size that does not clip it.
- The script change is small because the old code already centered the
  configured symbol's reported `size` rect; only the rect's container had to
  become `canvasSize` while `pointSize` stayed the symbol configuration's.

### Follow-up: 14 was still too large; the user chose `pointSize: 12`

- `pointSize: 14` measured clip-free and matched the ink of the `NativeIcon`
  template images, but reading the running app's menu against the
  **OS-provided** items (`Cut` / `Copy` / `Paste` in the Edit menu) showed the
  PNG icons still visibly larger. The user chose 12.
- The useful finding: the `NativeIcon` templates
  (`NSFollowLinkFreestandingTemplate`, `NSRefreshTemplate`) are **not** the
  right size reference. They pad their glyph to ~16pt of ink and themselves
  read larger than the rest of the menu. The OS-injected menu items are the
  reference to match.
- Re-measured the regenerated PNGs the same way (alpha > 0.05 bounding box
  divided by the 2x scale, plus an explicit edge-contact check):

  | image | canvas | glyph ink | bounding box (px) | edge contact |
  |---|---|---|---|---|
  | `gearshape.png` | 18pt (36x36 px) | 13.0 x 13.0 | x[5,30] y[5,30] | no |
  | `arrow.uturn.backward.png` | 18pt (36x36 px) | 10.5 x 11.5 | x[7,27] y[7,29] | no |
  | `folder.png` | 18pt (36x36 px) | 12.5 x 11.0 | x[5,29] y[7,28] | no |

  Nothing touches pixel 0 or 35 in either axis, and the numbers match the
  expectations exactly. `folder`'s horizontal margin grew from 1px at 14 to
  5px at 12, so clipping is no longer anywhere near.

### Mid-step scope addition: `Open Folder…` gets `folder.png`

- The user asked to fold the separate `todo.md` item (`Open Folder…` has no
  macOS menu icon) into this PR, so `crates/app/src/main.rs` — which the plan
  originally froze — now builds `Open Folder…` as an `IconMenuItem` behind
  `#[cfg(target_os = "macos")]`, with the plain `MenuItem` kept as the
  non-macOS twin, exactly like `Settings...`.
- The user chose to reuse the existing `folder.png` for both folder-ish items
  (`File > Open Folder…` and `Help > Open Log Folder`) rather than exporting a
  second, differentiated symbol. The duplication is accepted deliberately: the
  two menus are never visible at the same time, so the repeat is never seen.
  No new PNG was exported and the export script is unchanged by this addition.
- `todo.md`'s item and its `#### TODO` block were deleted in this commit (the
  glyph-size item is still left to the wrap-up's todo curator).

### Visual pass: confirmed at `pointSize: 12`

The user compared the menu in the running app and confirmed the icons now read
at the same size as their neighbors, `Open Folder…`'s new `folder.png`
included. `pointSize: 14` had been rejected in the same way one iteration
earlier, so 12 is the settled value.

### Side experiment: muda's missing `setTemplate` is a one-line gap

While the visual pass was open, the user asked whether the gray `#8E8E93`
fill could be avoided. A throwaway spike answered it: muda 0.19.3 was copied
to a scratchpad directory, one line added to `menuitem_set_icon` in
`src/platform_impl/macos/mod.rs`

```rust
nsimage.setTemplate(true);
```

and patched in with `[patch.crates-io] muda = { path = ... }`. A `path` (or
`git`) source is required — patching crates.io to another crates.io version is
rejected outright (`patch for 'muda' points to the same source`), and the
patched version must still satisfy `tauri`'s `muda = "^0.19"`, which is why
the fork stays on 0.19.3 rather than moving to 0.20.

Result, confirmed visually: every bundled PNG icon tints with the menu
appearance — white in dark mode, black in light mode — exactly like the
OS-provided `Cut` / `Copy` / `Paste` items. The baked `#8E8E93` gray is
ignored entirely, since a template image contributes only its alpha channel.

Consequences worth acting on:

- The gray fill in `tools/macos/export-menu-icons.swift` becomes dead weight
  once the icons are templates, and the script can drop the color step.
- No upstream issue asks for this. All 92 muda issues (open and closed) were
  checked; the nearest are #262 (missing Sequoia predefined items), #240
  (`Submenu::set_icon`) and #97 (Windows menu bar theming), none of them this.
  On the Tauri side the hits are all about the *tray* icon, which already has
  `set_icon_as_template` — so the concept is wired up for trays and simply
  never extended to menu items.
- The spike was reverted; `Cargo.toml` and `Cargo.lock` carry none of it.

The user chose to both upstream the fix to `tauri-apps/muda` and adopt a fork
in the meantime; that work is tracked separately from this plan.

## Deferred issues (todo candidates)

- **Upstream or fork muda for `setTemplate` on custom menu-item images.**
  The spike above proves a one-line change in muda 0.19.3's
  `menuitem_set_icon` (`nsimage.setTemplate(true)`) makes the bundled menu-icon
  PNGs tint with the OS menu appearance instead of sitting at a fixed gray.
  The change needed: open an upstream PR against `tauri-apps/muda` adding that
  call, and — until it lands — adopt a `path`/`git`-patched fork of muda 0.19.3
  through `[patch.crates-io]` in `Cargo.toml`. Once the icons are templates,
  drop the now-dead `#8E8E93` fill step from
  `tools/macos/export-menu-icons.swift`. Note the patch source must be `path`
  or `git` (a crates.io-to-crates.io patch is rejected) and must satisfy
  `tauri`'s `muda = "^0.19"`, so the fork stays on 0.19.3 rather than 0.20.
  Done when `Settings...`, `Undo`, `Open Folder…` and `Open Log Folder` tint
  white in dark mode and black in light mode in the running app, and the gray
  fill step is gone from the export script. Related:
  `tools/macos/export-menu-icons.swift`, `Cargo.toml`,
  `crates/app/icons/menu/*.png`.
