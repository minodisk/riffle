# What the camera records

Some features depend on what the camera records in the RAW file. For the
tested cameras, see [Compatibility](../README.md#compatibility); for the
features themselves, see [usage.md](./usage.md); for why these differ by body
even within one RAW format, see [How RAW files differ](./raw-formats.md).

| Camera | AF point | AF frame size | Face tracking | Sub-second capture time |
|---|---|---|---|---|
| Sony α1 | ✓ | ✓ | – | ✓ |
| Sony α9 III | – | – | – | ✓ |
| Sony α7 V | ✓ | ✓ | ✓ | ✓ |
| Sony α7 IV | ✓ | ✓ | – | ✓ |
| Sony α7R V | ✓ | ✓ | – | ✓ |
| Sony α7S III | ✓ | ✓ | – | ✓ |
| Sony α7C II | ✓ | ✓ | – | ✓ |
| Sony α7CR | – | – | – | ✓ |
| Sony α6700 | ✓ | ✓ | – | ✓ |
| Sony ZV-E1 | ✓ | ✓ | – | ✓ |
| SIGMA BF | ✓ | – | – | – |
| SIGMA fp L | – | – | – | – |
| Leica M11-P | – | – | – | – |
| Canon EOS R | ✓ | ✓ | – | ✓ |
| Canon EOS RP | ✓ | ✓ | – | ✓ |
| Canon EOS R3 | ✓ | ✓ | – | ✓ |
| Canon EOS R5 | ✓ | ✓ | – | ✓ |
| Canon EOS R5 Mark II | ✓ | ✓ | – | ✓ |
| Canon EOS R6 | – | – | – | ✓ |
| Canon EOS R6 Mark II | ✓ | ✓ | – | ✓ |
| Canon EOS R6 Mark III | ✓ | ✓ | – | ✓ |
| Canon EOS R7 | ✓ | ✓ | – | ✓ |
| Canon EOS R10 | ✓ | ✓ | – | ✓ |
| Canon EOS R50 | ✓ | ✓ | – | ✓ |
| Canon EOS R50 V | ✓ | ✓ | – | ✓ |
| Canon EOS R100 | ✓ | ✓ | – | ✓ |
| Nikon Z 9 | ✓ | ✓ | – | ✓ |
| Nikon Z 8 | – | – | – | ✓ |
| Nikon Z 7II | ✓ | ✓ | – | ✓ |
| Nikon Z 6II | ✓ | ✓ | – | ✓ |
| Nikon Z 6 | ✓ | ✓ | – | ✓ |
| Nikon Z 5 | ✓ | ✓ | – | ✓ |
| Nikon Z f | ✓ | ✓ | – | ✓ |
| Nikon Z fc | ✓ | ✓ | – | ✓ |
| Nikon Z 50 | ✓ | ✓ | – | ✓ |
| Nikon Z 30 | ✓ | ✓ | – | ✓ |
| Nikon D850 | – | – | – | ✓ |
| Nikon D500 | – | – | – | ✓ |

On the Canon and Nikon bodies, Riffle reads the AF point from the MakerNote
(`AFInfo2`); with several AF points in focus, the mark covers them all. The
Nikon Z 8 and Canon EOS R6 samples carried no AF position (an automatic area
that never locked, and manual focus), so those two are unconfirmed; the Nikon
D850 and D500 write an older `AFInfo2` that Riffle does not read.

The Sony α9 III and α7CR samples were all shot in manual focus, which Riffle
treats as having no AF point, so those two are unconfirmed. On the Sony bodies
other than the α7 V, `–` under Face tracking means face tracking was not
recorded on the sample, whose subjects hold no face; it does not mean the body
lacks it.

- **AF point**: the focus mark is drawn there, the 1:1 focus check opens
  centered on it, and sharpness is scored around it. Faces are then ignored,
  even when the point lies outside every face. Without one, there is no focus
  mark, the 1:1 focus check opens at the frame center, and sharpness is scored
  between the eyes of a detected face, else on the sharpest region. A
  manual-focus shot on a body that records the focus mode (Sony) is treated as
  having no AF point; DMF shots and bodies that record no focus mode are not.
- **AF frame size and face tracking**: with both, sharpness is scored on the
  camera's eye-AF frame, which is the most reliable because it does not rely
  on face detection. On the α7 V, `Face tracking` is also recorded when the
  AF sits on the back of a head: it means the camera recognized a person's
  head, not strictly a face or an eye. The meta pane's `AF tracking` row
  shows this value.
- **Sub-second capture time**: frames within 1 s of the previous one form a
  burst. Without it, frames are grouped by whole seconds, so a shot taken up
  to about 2 s after the previous frame can still join its burst.
