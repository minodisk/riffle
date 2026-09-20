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
