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
  artefact: the paste itself is unclipped, because the crop is resized to 98x98
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

- (none)
