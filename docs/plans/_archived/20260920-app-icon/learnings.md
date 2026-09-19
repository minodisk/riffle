# Learnings

## Step 1

- The supplied PNG's near-opaque body has alpha 251-253, not 255, so a
  "bbox of alpha 255" finds nothing useful; the `alpha > 200` bbox is
  `(45, 49, 1210, 1204)`. The source was cropped at `(53, 52, 1203, 1202)`
  (1150px square, a few px inside the fringe).
- The supplied image's own corner radius is smaller than the macOS ~22.5%
  radius, so only 68 pixels inside the new mask were semi-transparent after
  the crop; they were filled with the mean of nearby opaque pixels. numpy is
  not installed, so this was done in plain Python with Pillow.
- The mask is inset by 2px, so the visible body spans x/y 101-922 on the
  1024 canvas.
- `tauri icon` did not touch `tauri.conf.json`; the `ios/` and `android/`
  output was deleted.
