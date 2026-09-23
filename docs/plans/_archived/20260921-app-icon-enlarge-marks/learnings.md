# Learnings

## Step 1: Enlarge the dot and star on `source.png`

- Pillow's `Image.getbbox()` defaults to `alpha_only=True` for RGBA images
  (Pillow >= 9.5). The first verification run reported "no difference" between
  the original and the edited `source.png` because only the alpha channel was
  compared, and alpha is indeed untouched. Use
  `ImageChops.difference(a, b).getbbox(alpha_only=False)` — or, better for this
  check, iterate the pixels and assert every non-zero one falls inside the paste
  boxes, since a single bbox spanning both marks says nothing useful.
- Measuring the enlarged mark must use a window larger than the plan's 6px-padded
  crop boxes. The 1.25x dot is 83px across inside a 78x78 crop box, so measuring
  within that box clips the bbox to the box itself. This is only a measurement
  artifact: the paste itself is unclipped, because the crop is resized to 98x98
  first. Windows `(160,184,270,294)` / `(740,178,868,300)` measure cleanly.
- Measured results match the plan: dot 66 -> 83px, star 79x76 -> 99x96; centroids
  `(214.76, 239.06)` and `(804.21, 243.32)`, both within 1px of the plan's
  values. With the 1px dilation, 0 pixels of either old dilated footprint are
  left with new-mask alpha < 250, so no background fill was needed, and the
  alpha channel is byte-identical.
- `pnpm exec tauri icon` always writes `ios/` and `android/`; they are removed
  afterwards, as in `20260920-app-icon`. `crates/app/icons/menu/` and
  `crates/app/tauri.conf.json` show no diff.

## Deferred issues (todo candidates)

- **Decide whether the Pillow pixel-diff verification pitfalls deserve a guide**
  - Change: if the pattern recurs, add a short "verifying pixel edits" note under
    `docs/agents/` (a new file, or a section in `tauri-app.md`), covering:
    (1) `Image.getbbox()` defaults to `alpha_only=True` on RGBA (Pillow >= 9.5),
    so a "no difference" check over `ImageChops.difference` can silently pass
    while the RGB channels differ — use `getbbox(alpha_only=False)` or a
    per-pixel scan; (2) measuring a resized/pasted mark inside the padded crop
    box that produced it clips the bbox and understates the mark's size — the
    measurement window must be wider than the source crop.
  - Rationale: both pitfalls actually bit this work, but only once. Filing a
    general guide off a single occurrence risks untested general advice.
  - Done when: the next plan doing Pillow-based pixel verification is checked
    for the same pitfalls; if they recur, the guide is written, linking
    `docs/plans/_archived/20260921-app-icon-enlarge-marks/learnings.md` for the
    concrete numbers. If they do not, close this out.
