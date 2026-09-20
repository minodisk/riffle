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
