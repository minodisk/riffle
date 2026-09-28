<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Replace the app icon with the film-frame design

## Purpose

The current app icon (the contact-sheet design from
`../_archived/20260923-contact-sheet-icon/plan.md`) is replaced by a design
the user finalized and approved in conversation: a horizontal 35mm film frame
on a dark rounded-square body. A 3:2 gradient photo with square corners sits
in the middle; ten black sprocket holes per row, flush with the photo's left
and right edges and with a faint white glow, run above and below it; the edge
print reads `RIFFLE 60FPS CULLER` centered on top and `12` centered with a
smaller `▶12A` on the bottom (DIN Condensed Bold, faint glow); a flat green
pick sticker sits at the photo's top-left with a tiny peel at its lower right.
The body keeps the same grid as today's `source.png` (824x824 centered on a
1024 canvas, 100px transparent margin, corner radius 185).

Unlike the three previous icons, this one was not composited by hand from
pixels: a Python script writes an SVG and headless Chrome renders it. That
makes the design parametric (hole count, text, sticker position, radius are
constants), so this plan also commits the generator, the way
`tools/macos/export-menu-icons.swift` is committed for the menu glyphs.

The approved render lives outside the repository at

`/private/tmp/claude-501/-Users-mino-ws-github-com-minodisk-riffle/1d6e1a51-2077-4d55-a157-1d92c7bef202/scratchpad/svg/icon.png`

(sha256 `bafeea13c574aab5c00d7ebd4cec3233ca1c51afaaea1f5cad00e42acb837d33`,
1024x1024 RGBA, alpha bbox `(100, 100, 924, 924)`). Its sources are in the
same folder: `gen.py` (writes `icon.svg`), `icon.svg`, and `photo.png` (the
3:2 gradient photo, 1536x1024 RGB, sha256
`abb28d5dd939ed8558b25575eccd070cb03d36bc4f7fdc8917562d3ac32a4638`). The SVG
was rendered with

```
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --disable-gpu --hide-scrollbars --default-background-color=00000000 --window-size=1024,1024 --screenshot=<abs>/icon.png file://<abs>/icon.svg
```

Current state the plan is based on:

- `crates/app/icons/` holds `source.png` plus the Tauri-generated set
  (`32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.png`,
  `icon.ico`, `icon.icns`, `Square{30,44,71,89,107,142,150,284,310}x*Logo.png`,
  `StoreLogo.png`) and `menu/`, the SF Symbol glyphs for menu items
  (`folder`, `gear`, `trash`, ...) exported by
  `tools/macos/export-menu-icons.swift` and referenced from
  `crates/app/src/main.rs`. They do not derive from the app icon and
  `tauri icon` does not write there.
- Every other place that shows the app icon reads a file `tauri icon`
  regenerates, so none of them changes: `README.md` and `README.ja.md` embed
  `crates/app/icons/128x128@2x.png` at line 1; the About dialog embeds
  `icons/128x128.png` (`crates/app/src/main.rs`, `AboutMetadata.icon`);
  `crates/app/tauri.conf.json` `bundle.icon` lists the generated files. No
  document describes the icon's design.
- The established procedure (`../_archived/20260920-app-icon/plan.md`,
  `../_archived/20260923-contact-sheet-icon/plan.md`, and the
  "`tauri icon` also writes `ios/` and `android/` icons" entry in
  `../../agents/tauri-app.md`): from the repository root,
  `pnpm install --frozen-lockfile && pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`,
  then `rm -r crates/app/icons/ios crates/app/icons/android`. It leaves
  `tauri.conf.json` and `menu/` untouched.
- `mise run ci` (`lint` + `test`) inspects neither PNGs nor Python; the
  visual checks are manual. tauri-cli 2.11.5, Pillow 12.3.0 and Chrome are
  available on this machine.
- `gen.py` as it stands has leftovers: the unused `stickerfill`
  radialGradient and `stickershadow` filter in `<defs>` (used ids are
  `photo`, `keep`, `holeglow`, `stickerclip`, `innerglow`, `flapshadow`),
  the comment "Sprocket holes: 8 per row" while `HOLES = 10`, the comment
  "like the original icon's mark" that refers to the previous icon, and it
  writes `icon.svg` to the current directory with a cwd-relative
  `href="photo.png"`.

## Steps

- [x] Step 1: Replace `source.png` with the approved render and regenerate the bundled icons
  - Done when:
    - `crates/app/icons/source.png` is byte-identical to the scratchpad render
      above (`cmp` exits 0; sha256
      `bafeea13c574aab5c00d7ebd4cec3233ca1c51afaaea1f5cad00e42acb837d33`),
      1024x1024 RGBA.
    - Every generated icon under `crates/app/icons/` (`32x32.png`,
      `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`,
      `icon.icns`, all `Square*Logo.png`, `StoreLogo.png`) is regenerated from
      the new `source.png` by `pnpm exec tauri icon`; `git status` shows each
      of them modified and nothing else under `crates/app/icons/`.
    - `crates/app/icons/ios/` and `crates/app/icons/android/` do not exist.
    - `crates/app/icons/menu/` and `crates/app/tauri.conf.json` have no diff.
    - `Read` `crates/app/icons/128x128@2x.png` and `32x32.png`: corners
      transparent, the film frame, sprocket holes, photo and green sticker
      visible; at 32px the silhouette and the sticker read, the text is
      texture.
    - `README.md` / `README.ja.md` are unchanged (they already point at the
      regenerated `128x128@2x.png`).
    - `mise run ci` passes and the app crate builds (`cargo build` for the
      package in `crates/app/Cargo.toml`), since `main.rs` embeds
      `icons/128x128.png` at compile time.
  - Implementation approach:
    - `cp` the scratchpad render over `crates/app/icons/source.png`; do not
      open and re-save it with Pillow (that changes the bytes).
    - If the scratchpad file is gone, stop and ask rather than re-render:
      the render is the approved artifact, and Step 2's reproducibility is
      not yet proven.
    - Regenerate as in the previous icon plans:
      `pnpm install --frozen-lockfile && pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`,
      then `rm -r crates/app/icons/ios crates/app/icons/android`.
    - Commit as `feat(app): replace the app icon with the film-frame design`.

- [ ] Step 2: Commit the icon generator under `tools/macos/app-icon/`
  - Done when:
    - `tools/macos/app-icon/gen.py` and `tools/macos/app-icon/photo.png`
      exist; `icon.svg` is not committed (it is derived).
    - `gen.py` has a header comment in the style of
      `tools/macos/export-menu-icons.swift`: what it produces, how to run
      it (`python3 tools/macos/app-icon/gen.py`), that it runs on macOS
      only (DIN Condensed Bold is a macOS system font; headless Chrome at
      `/Applications/Google Chrome.app/...` renders the SVG), that the
      committed PNGs are the source of truth and the rerun is only for a
      design change, that `photo.png` was generated with OpenAI gpt-image,
      and that `pnpm exec tauri icon` must be rerun afterwards (with the
      `ios/` / `android/` removal). All text is English.
    - `gen.py` resolves its paths from its own location (`Path(__file__)`):
      it reads `photo.png` next to itself (the SVG `href` is an absolute
      `file://` URL or the SVG is written next to `photo.png`), writes the
      SVG to a temporary location, runs Chrome, and writes the render to
      `crates/app/icons/source.png` relative to the repository root
      (`Path(__file__).resolve().parents[3]`). Running it from any cwd works.
    - The unused `stickerfill` and `stickershadow` defs are removed; the
      "8 per row" comment matches `HOLES = 10`; the comment referring to
      "the original icon" is reworded to describe the mark itself. The
      rendered SVG geometry is otherwise unchanged.
    - Running `python3 tools/macos/app-icon/gen.py` on this machine
      reproduces Step 1's `source.png`: compare with `cmp`; if the bytes
      differ (PNG encoder metadata), compare pixels with Pillow
      (`ImageChops.difference(a, b).getbbox(alpha_only=False)` is `None`)
      and record the result in `learnings.md`. If pixels differ, the
      committed `source.png` stays as Step 1 left it and the difference is
      recorded; do not overwrite it.
    - The "`tauri icon` also writes `ios/` and `android/` icons" entry in
      `../../agents/tauri-app.md` gains one sentence pointing at
      `tools/macos/app-icon/gen.py` as the generator of `source.png`.
    - `mise run ci` passes (lychee checks the new relative link in
      `tauri-app.md` if written as a link).
  - Implementation approach:
    - Step 1 is merged first; this step touches only `tools/` and
      `docs/agents/tauri-app.md`, not `crates/app/icons/`.
    - Keep `gen.py` dependency-free (standard library plus `subprocess` for
      Chrome); no Pillow at run time. The pixel comparison above is a
      one-off verification, not part of the script.
    - Place it under `tools/macos/` because, like `export-menu-icons.swift`,
      it only runs on macOS.
    - Commit as `chore(app): add the app icon generator`.

## Trade-offs and risks

- **Commit the generator (Step 2) vs. only `source.png`.** The user chose to
  commit it. The three earlier icon plans did not commit their Pillow scripts
  because they overwrote their own input and had no reuse value. This design
  is parametric, so a later tweak (hole count, text, sticker) becomes a
  constant change plus a rerun. Costs: `photo.png` adds 1.8 MB to the
  repository history permanently; the tool is macOS-only and depends on an
  installed Chrome; and Chrome / font rasterization may change between
  versions, so the render is not guaranteed byte-stable (hence "committed
  PNGs are the source of truth", the same stance the swift tool takes).
- **Whether to commit `icon.svg`**: not committed, since `gen.py` regenerates
  it and a committed copy would drift.
- **Reproducibility of the render**: Step 2 measures it. If Chrome produces
  pixel differences from the approved render, the approved bytes from Step 1
  stay, and the script is still useful for future tweaks (which would go
  through the same approval anyway).
- **Legibility at small sizes**: the edge print will not be readable at
  32x32; accepted as texture, as in the contact-sheet plan.
- **`60FPS` claim** is baked into the image (and into `gen.py`'s constant); a
  change needs a rerun, not a doc edit.
- `icon.ico` / `icon.icns` are only exercised by their platform builds; a
  malformed one would first surface in the release workflow, as in the
  previous plans.

## Progress

- (none yet)
