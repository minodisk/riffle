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

# App icon

## Purpose

`crates/app/icons/` still holds the Tauri default icons, so the released
Riffle app ships with the generic Tauri logo. The user has supplied an icon
design (a rounded-square gradient with a green dot and a yellow star) as
`/Users/mino/Downloads/ChatGPT Image 2026年9月20日 00_40_44.png`. This work
turns that image into a clean 1024x1024 transparent source and regenerates
every bundled icon from it with `tauri icon`.

Current state the plan is based on:

- The supplied PNG is 1254x1254 RGBA with transparency already applied, but
  its edge is ragged (colour fringes on the left and right sides, e.g. around
  x≈1200–1224 in the middle rows) and it is off-centre: the alpha bounding box
  is `(0, 8, 1224, 1254)`, touching the left and bottom edges with no margin.
- macOS does not round icons for `.icns` apps: a square icon stays square,
  and on macOS 26 non-conforming legacy icons are shrunk onto a plate. The
  icon must therefore carry its own rounded shape and the macOS grid margin.
- Python 3 with Pillow is available (ImageMagick is not).
  `pnpm exec tauri icon` (tauri-cli 2.x) works after
  `pnpm install --frozen-lockfile`, and also emits `ios/` and `android/`
  subfolders.
- `crates/app/tauri.conf.json` `bundle.icon` references files `tauri icon`
  produces, so the config does not change.

## Steps

- [x] Step 1: Prepare the 1024x1024 transparent source and regenerate the app icons
  - Done when:
    - `crates/app/icons/source.png` is 1024x1024 RGBA; the rounded-square body
      is 824x824 and centred (100px transparent margin on each side), with a
      macOS-style corner radius of ~185px (≈22.5% of the body); every pixel
      outside the rounded square has alpha 0 and the edge is smooth with no
      colour fringe (spot-check edge alpha and RGB with Pillow).
    - All icons under `crates/app/icons/` (`32x32.png`, `64x64.png`,
      `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.icns`, `icon.ico`,
      `Square*Logo.png`, `StoreLogo.png`) are regenerated from `source.png`
      via `pnpm exec tauri icon`; the generated `ios/` and `android/` folders
      are removed (no mobile targets).
    - `tauri.conf.json` is unchanged.
    - `mise run ci` passes and the app crate still builds.
    - Visually check `128x128@2x.png` (`Read` it) for transparent corners,
      a clean edge, and the dot/star intact.
  - Implementation approach:
    - One-off Pillow script in a scratch location (not committed):
      1. Take the opaque body of the supplied image: find the region of the
         rounded square ignoring the ragged fringe (e.g. the bbox of pixels
         with alpha 255, or trim a few px on each side), and crop a square
         around it so the artwork (dot, star, wave) stays intact.
      2. Resize the crop to 824x824 with LANCZOS, and discard its original
         alpha (treat as opaque RGB, filling any transparent pixels inside the
         crop from neighbouring colour so no dark/transparent fringe bleeds in).
      3. Build a geometric alpha mask: `ImageDraw.rounded_rectangle` with a
         ~185px radius, rendered at 4x and downsampled (LANCZOS) for smooth
         anti-aliasing, inset by ~1–2px so no original fringe survives.
      4. Paste centred on a transparent 1024x1024 canvas at (100, 100) and
         save as `crates/app/icons/source.png`.
    - Regenerate from the repository root:
      `pnpm install --frozen-lockfile && pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`,
      then delete `crates/app/icons/ios/` and `crates/app/icons/android/`.
    - Commit as `feat(app): set the Riffle app icon`.

## Trade-offs and risks

- Corner radius follows the macOS grid (~22.5%) rather than the supplied
  image's own curve, so the icon lines up with neighbouring Dock icons (user
  agreed).
- Only `source.png` is committed, not the original 1254px PNG.
- `icon.ico` is only exercised by Windows builds; a malformed one would first
  surface in the release workflow, hence the local build check.

## Progress

- (none yet)
