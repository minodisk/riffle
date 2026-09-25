# What the camera records

Some features depend on what the camera records in the RAW file. For the
tested cameras, see [Compatibility](../README.md#compatibility); for the
features themselves, see [usage.md](./usage.md).

| Camera | AF point | AF frame size | Face tracking | Sub-second capture time |
|---|---|---|---|---|
| Sony α7 V | ✓ | ✓ | ✓ | ✓ |
| SIGMA BF | ✓ | – | – | – |
| SIGMA fp L | – | – | – | – |
| Leica M11-P | – | – | – | – |

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
