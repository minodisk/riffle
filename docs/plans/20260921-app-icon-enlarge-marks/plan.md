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

# Enlarge the dot and star on the app icon

## Purpose

The green dot and the yellow star on `crates/app/icons/source.png` read as
slightly small at Dock sizes. This work enlarges both marks by 1.25x in place,
keeping the rounded-square body, its gradient, corner radius and margins
untouched, and regenerates every bundled icon from the new source with
`tauri icon`, as `docs/plans/_archived/20260920-app-icon/` did.

Current state the plan is based on (measured with Pillow 12.3.0 on
`crates/app/icons/source.png`, 1024x1024 RGBA):

- Body: 824x824 rounded square at (100,100)-(924,924) with a ~185px corner
  radius (mask inset 2px, so the visible body spans 101-922), per the
  previous plan.
- Green dot: colour-key bbox `(182, 206)-(247, 271)` (66x66 px), mask
  centroid `(215.2, 239.4)`, colour ≈ `(112, 188, 96)`, flat fill, no
  shadow. Local background is dark navy ≈ `(1, 15-26, 65-103)`, a gentle
  gradient with film grain (per-channel sd ≈ 0.9 / 2.5 / 4.7 in a flat
  20x20 patch).
- Yellow star: colour-key bbox `(764, 202)-(842, 277)` (79x76 px), mask
  centroid `(804.0, 243.9)` (lower than the bbox centre because a 5-point
  star is asymmetric), colour ≈ `(250, 188, 71)` with a lighter rim
  `(255, 216, 117)` on the left edge, no drop shadow. Local background runs
  from purple `(106, 2, 123)` above to red `(240, 9, 86)` below.
- Colour keys that isolate each mark cleanly (alpha > 200 in both):
  dot `g > 150 and r < 140 and g - r > 50`;
  star `r > 180 and g > 140 and b < 140 and r - b > 80`.
  (Do not run the star key over the whole image: it also matches the orange
  wave in the centre. Restrict it to the star's bbox plus padding.)
- Because each mark is scaled about its own centroid, the enlarged mark is a
  superset of the original footprint (circle and star are both star-convex
  about their centre). Prototyped in scratch: with the mask dilated by 1px
  before resizing, zero pixels of the old footprint (dilated by 1px) are left
  with new-mask alpha < 250 at 1.2x and 1.25x, for both marks. So **no
  background fill / inpainting is needed**: the old mark is completely
  hidden by the new one, and the grainy gradient outside it is untouched.
  Without the dilation, 41 (1.2x) / 6 (1.25x) star pixels at the concave
  inner vertices had mask 227-249, i.e. faintly visible old edge.
- Enlarged extents stay well inside the body: at 1.25x the dot is pasted at
  about (166,190) size 98x98 and the star at about (746,184) size 114x110,
  i.e. ≥ 62px from the nearest body edge and clear of the corner arcs.
- `crates/app/icons/menu/` (`folder.png`, `gearshape.png`,
  `arrow.uturn.backward.png`, 36x36) are the macOS menu icons referenced
  from `crates/app/src/main.rs` and produced by
  `tools/macos/export-menu-icons.swift`; `tauri icon` does not write there.
  Leave them alone.
- `node_modules` is not installed in the worktree; run
  `pnpm install --frozen-lockfile` before `pnpm exec tauri icon`.
- `mise run ci` is `lint` + `test`; it does not inspect PNGs, so the visual
  checks below are manual.

## Steps

- [x] Step 1: Enlarge the dot and star on `source.png` and regenerate the bundled icons
  - Done when:
    - `crates/app/icons/source.png` is still 1024x1024 RGBA, the green dot
      and yellow star are 1.25x their previous size (dot ≈ 82px across, star
      ≈ 99x95px) and their mask centroids are within 1px of
      `(215.2, 239.4)` and `(804.0, 243.9)` respectively (re-measure with the
      colour keys above).
    - Every pixel outside the two pasted regions is byte-identical to the
      current `source.png` (compare with `ImageChops.difference` and check
      that its non-zero bbox lies within the union of the two paste boxes);
      in particular alpha, corners, margins and the gradient are unchanged.
    - No rectangular crop edge or colour mismatch is visible around either
      mark: `Read` a nearest-neighbour zoom of each mark (e.g. crop
      (150,180,290,300) and (730,170,880,300), scaled 3x) and the full icon
      at 256px.
    - All icons under `crates/app/icons/` (`32x32.png`, `64x64.png`,
      `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.icns`, `icon.ico`,
      `Square*Logo.png`, `StoreLogo.png`) are regenerated from `source.png`
      via `pnpm exec tauri icon`; `crates/app/icons/ios/` and
      `crates/app/icons/android/` do not exist; `crates/app/icons/menu/` is
      untouched (`git status` shows no change there).
    - `Read` `crates/app/icons/128x128@2x.png`: corners transparent, edge
      clean, dot and star visibly larger than before.
    - `crates/app/tauri.conf.json` has no diff.
    - `mise run ci` passes.
  - Implementation approach:
    - One-off Pillow script in the scratchpad (not committed, consistent
      with `20260920-app-icon`). No numpy; plain Pillow loops are fast
      enough at this size. For each mark:
      1. Crop the bbox padded by 6px: dot `(176, 200, 254, 278)`, star
         `(758, 196, 849, 284)`.
      2. Build an `L` mask from the colour key; record the mask centroid.
      3. Dilate the mask by 1px (`ImageFilter.MaxFilter(3)`) so the paste
         carries a 1px ring of the crop's own background, guaranteeing the
         old anti-aliased edge is covered.
      4. Resize crop and dilated mask by 1.25 with `Image.LANCZOS`.
      5. Paste with the resized mask at the offset that puts the scaled
         centroid on the original centroid:
         `offset = round(centroid_img - centroid_crop * 1.25)`.
      6. Save over `crates/app/icons/source.png`.
    - Verify in the same script: (a) old dilated footprint fully covered
      (new mask ≥ 250 everywhere on it), (b) difference bbox vs the
      original confined to the paste boxes, (c) re-measured bbox/centroid.
    - Regenerate from the repository root, as before:
      `pnpm install --frozen-lockfile && pnpm exec tauri icon crates/app/icons/source.png -o crates/app/icons`,
      then `rm -r crates/app/icons/ios crates/app/icons/android`.
    - Commit as `feat(app): enlarge the dot and star on the app icon`.

## Trade-offs and risks

- **Scale factor**: 1.25 chosen by the user (dot 66 -> 82px, ~8% -> ~10% of
  the body width). 1.2 was also verified clean but barely noticeable at Dock
  size; 1.3 starts to read as a different design.
- **No background fill**: relying on "new mark ⊇ old mark" instead of
  synthesising background is simpler and preserves the film grain exactly.
  It only holds because the marks are scaled up about their own centroid;
  if a mark is later moved or shrunk, a fill step (fit a per-channel linear
  gradient over the bbox from a surrounding ring, add matching noise) would
  be required. The 1px dilated ring pastes a scaled copy of the local
  background, which is invisible against the gradient and grain (checked
  visually at 3x); a smooth fill would have looked flatter.
- **Script not committed**: matches the previous app-icon work. Committing
  it has little reuse value because it overwrites its own input; the numbers
  needed to reproduce it are in this plan instead.
- The old mark is hidden rather than removed, so a future re-edit of the
  same marks must again scale up, not down, unless a fill step is added.
- `icon.ico` / `.icns` are only exercised by their platform builds; as in
  the previous plan, a malformed one would first surface in the release
  workflow.

## Progress

- (none yet)
