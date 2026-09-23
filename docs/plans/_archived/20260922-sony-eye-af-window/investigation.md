# Investigation (planning, 2026-09-22)

Verified on real α7 V ARWs with the ExifTool 13.59 perl library.

- `AFTracking` = Sony MakerNote tag 0x2021, BYTE, plaintext. 0 Off,
  1 Face tracking, 2 Lock On AF.
- `FocusFrameSize` = tag 0x2037, 3 x SHORT, plaintext: `w h valid`. exiftool
  prints `n/a` when the third value is 0 (seen: `5519 3864 0`); valid frames
  carry 257 in the third slot.
- `FocusLocation` = 0x2027 and `FocusMode` = 0x201b, both already parsed.
- `AFAreaMode` is not plaintext: the value shown as "Human Eye Tracking" (21)
  is byte 0x17 of the enciphered `Tag9402` block (tag 0x9402; substitution
  `c = b^3 mod 249`, `Sony.pm` `Decipher`). The plaintext 0x201c is only
  `AFAreaModeSetting` (the menu setting). On the sampled folders it is noisy:
  `AFAreaMode=21` occurs with `AFTracking=0` and center + 832x740, and
  `AFTracking=1` occurs with `AFAreaMode` 12 or 0 and a moved small frame.
  `AFTracking` is the "engaged" signal.
- Sample of 57 files from `2026-08-29` (every 60th): with `AFTracking=1`,
  frames were 153x154 .. 569x570 (one 394x263) off-center, plus 3 files at
  center 3504 2336 with 832x740 (not engaged). With `AFTracking=0`, spot/zone
  AF also writes small frames (153x156, 175x176, 350x351) at or near center.
  One engaged file sits at x = 3504 exactly but y = 2297, so the not-engaged
  test must require both coordinates at the sensor center.
- `_DSC2565.ARW` (`2026-07-11`) is engaged with a 1533x1535 frame (close
  subject), so the frame side must be clamped to `WINDOW`.
- `Shot` literals without `..Default::default()`: `crates/app/src/index.rs`
  tests and `crates/core/src/sharpness.rs` tests.
