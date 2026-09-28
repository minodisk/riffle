# Learnings

## Step 1

- Checked against the current code before writing: the Sigma inline check is
  about the MakerNote tag's own `count` (an `UNDEFINED` note of `count <= 4`
  sits in the value field; `count <= SIGMA_HEADER_LEN` leaves no room for an
  IFD), not about the AF-point entry. The AF point `SHORT[2]` is inline too
  and is read with `value.to_le_bytes()`. The guide words both.
- `leica_focus_distance` returns `None` on an out-of-range note, while
  `sigma_af_point` bails on one past its size check; the guide only states
  the shared order (size check first), not identical error handling.
- The count-1 `SHORT` padding broke `strip_jpeg` (via `integer()`), not the
  MakerNote readers; the archived plan names the symptoms (wrong full-size
  JPEG on one SIGMA fp L file, no JPEG at all on another).
- The tract guide also names `scan::extract_faces` as a rayon caller of the
  shared detector, since the focus-candidate pass was added after the
  face-aware-sharpness learnings were written.
- The face-aware-sharpness learnings' Step 1 heading has a suffix, so the
  guide links that file without a fragment.
