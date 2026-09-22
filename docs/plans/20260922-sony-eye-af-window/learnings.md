# Learnings

## Step 1

- Real α7 V files store `FocusFrameSize` (0x2037) as `UNDEFINED[6]` (type 7,
  count 6), not `SHORT[3]`; exiftool only reinterprets it as `int16u[3]`. A
  reader accepting just `SHORT[3]` returned `None` on `_DSC3590.ARW`. The
  parser now treats `UNDEFINED[6]` as `SHORT[3]` (value at offset either way)
  and still accepts `SHORT[3]`. Verified on `_DSC3590.ARW`: `AFTracking` 0,
  frame 832x740, validity bytes `01 01` (257).
- `shorts4` was generalised to `shorts::<N>` for the 3-SHORT read.
