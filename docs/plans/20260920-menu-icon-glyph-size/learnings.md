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
- The script change is small because the old code already centred the
  configured symbol's reported `size` rect; only the rect's container had to
  become `canvasSize` while `pointSize` stayed the symbol configuration's.
