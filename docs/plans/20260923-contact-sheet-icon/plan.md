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

# Replace the app icon with the contact-sheet design

## Purpose

The current app icon (rounded-square gradient body with a green dot and a
yellow star) is replaced by a design the user has already finalised as an
image: a single 2:3 portrait 35mm frame of the existing landscape on a black
film base, inside the same rounded-square footprint as today. The edge
printing is rotated like Kodak 35mm film (left edge: a hollow triangle with
`35A` mid-edge and `36` at the bottom; right edge: `RIFFLE 60FPS CULLER` at the
top and `36` at the bottom), and a green dot sticker straddles the frame's
top-left corner. The star is dropped. `60FPS` comes from a Windows Timing-logs
measurement on 2026-09-22 (`keypressToPixels` median 16.2 ms over 113 page
turns).

The finished 1024x1024 RGBA image lives outside the repository at

`/tmp/claude-1000/-home-minodisk--herdr-worktrees-riffle-worktree-silver-forest-185f/00065693-16cd-4cf9-b618-fd222838bddd/scratchpad/tall-v7.png`

(sha256 `f1bbba21df7274fb7d0f3cb5524d3164e081a36c1f1d93f25aa0598781a64fab`).
It was composited by a one-off Pillow script (`tall.py` in the same scratchpad
directory) from the current `crates/app/icons/source.png`. As with the two
previous icon plans (`docs/plans/_archived/20260920-app-icon/`,
`docs/plans/_archived/20260921-app-icon-enlarge-marks/`), the script is not
committed; the image is the artefact.

Current state the plan is based on:

- `crates/app/icons/` holds `source.png` plus the Tauri-generated set
  (`32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.png`,
  `icon.ico`, `icon.icns`, `Square{30,44,71,89,107,142,150,284,310}x*Logo.png`,
  `StoreLogo.png`) and `menu/`, the macOS menu icons produced by
  `tools/macos/export-menu-icons.swift` and referenced from
  `crates/app/src/main.rs`. `tauri icon` does not write into `menu/`.
- There is no mise task for icon generation. Both previous plans ran
  `pnpm install --frozen-lockfile && pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`
  from the repository root and then deleted the generated
  `crates/app/icons/ios/` and `crates/app/icons/android/` folders.
  `tauri icon` left `crates/app/tauri.conf.json` untouched.
- `README.md` and `docs/**/*.md` neither embed nor describe the app icon, so
  no documentation change is needed.
- `mise run ci` (`lint` + `test`) does not inspect PNGs; visual checks are
  manual.

## Steps

- [x] Step 1: Replace `source.png` with the contact-sheet image and regenerate the bundled icons
  - Done when:
    - `crates/app/icons/source.png` is byte-identical to the scratchpad image
      above (`cmp` exits 0; sha256 matches
      `f1bbba21df7274fb7d0f3cb5524d3164e081a36c1f1d93f25aa0598781a64fab`),
      and is 1024x1024 RGBA.
    - Every generated icon under `crates/app/icons/` (`32x32.png`,
      `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`,
      `icon.icns`, all `Square*Logo.png`, `StoreLogo.png`) is regenerated from
      the new `source.png` by `pnpm exec tauri icon`; `git status` shows each
      of them modified.
    - `crates/app/icons/ios/` and `crates/app/icons/android/` do not exist
      after the step (deleted if `tauri icon` created them).
    - `crates/app/icons/menu/` is untouched (`git status` shows no change
      there) and `crates/app/tauri.conf.json` has no diff.
    - `Read` `crates/app/icons/128x128@2x.png` and `32x32.png`: corners
      transparent, the film frame, edge printing and green dot are visible,
      no star remains.
    - `mise run ci` passes.
  - Implementation approach:
    - `cp` the scratchpad image over `crates/app/icons/source.png` (no
      re-encoding; do not open and re-save it with Pillow, which would change
      the bytes).
    - Regenerate from the repository root, as the previous icon plans did:
      `pnpm install --frozen-lockfile && pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`,
      then `rm -r crates/app/icons/ios crates/app/icons/android` if present.
    - Verify no stray files were added (`git status --porcelain
      crates/app/icons` should list only the expected modified files).
    - No README / docs change: nothing references the icon.
    - Commit as `feat(app): replace the app icon with the contact-sheet design`.

## Trade-offs and risks

- **Script not committed**: matches the two previous app-icon plans. The
  Pillow script (`tall.py`) only exists in the session scratchpad; the
  finished image is the artefact. If the design needs to be reproducible, the
  alternative is to commit the script under `tools/`. Not planned.
- **Scratchpad is ephemeral**: the source image lives in a session-specific
  temp directory. If it is gone when the step runs, the design cannot be
  recovered from the repository; the implementer must stop and ask rather
  than re-derive it.
- **Legibility at small sizes**: the edge printing (`RIFFLE 60FPS CULLER`,
  `35A`, `36`) will not be readable at 32x32; this is accepted as texture,
  as on real film. The `32x32.png` and `Square30x30Logo.png` check above is
  about the silhouette and the dot, not the text.
- **`60FPS` claim**: it is baked into the image, so it cannot be updated by
  a text change later; a future measurement that contradicts it needs a new
  image.
- `icon.ico` / `icon.icns` are only exercised by their platform builds; as in
  the previous plans, a malformed one would first surface in the release
  workflow.

## Progress

- 2026-09-23: Step 1 done — `source.png` replaced (sha256
  `f1bbba21df7274fb7d0f3cb5524d3164e081a36c1f1d93f25aa0598781a64fa`), icon set
  regenerated, `ios/` and `android/` removed.
