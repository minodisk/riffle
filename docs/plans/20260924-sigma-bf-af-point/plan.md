<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Sigma BF AF point from the Sigma MakerNote

## Purpose

A Sigma BF DNG records where the camera focused in the Sigma MakerNote tag
0x0147 (two SHORTs, e.g. `386 323`), but `riffle_core::arw::parse` reads a
focus point only out of a Sony MakerNote (`FocusLocation`, 0x2027), so every
BF frame has `Shot::focus == None`: the focus mark (`f`) draws nothing, the
1:1 focus check (`z`) centers on the image, and the sharpness cue falls back
to face detection or the sharpest tile. Filling `Shot::focus` from 0x0147
makes all three work on the BF the way they do on a Sony body, with no change
downstream: `partial::focus_point`, `sharpness::score`, the index
(`files.focus_w/h/x/y`), `IndexedFile::focus`, `drawFocusMark` and `zoom.ts`
all take a `FocusLocation` in "sensor" units and scale it to the JPEG at hand.

Investigation results (from exiftool 13.59 on the 11 sample files under
`/mnt/c/Users/daisu/Downloads/AmazonPhotos/BF_*.DNG`, all landscape,
`Orientation=1`, AF-C, firmware `Sigma BF Ver1.03.3771`):

- The Sigma MakerNote is the plain ExifIFD tag 0x927c (`undef[8594]`), not
  the `DNGPrivateData` block (0xc634, which is a separate
  `SIGMA_EXTH_SPPA` blob). The note starts with the 10-byte header
  `SIGMA\0\0\0` + 2 version bytes (`01 04` here; exiftool's
  `MakerNoteSigma` uses `Start => $valuePtr + 10`), followed by a normal
  little-endian IFD (84 entries) whose value offsets are absolute to the TIFF
  start (verified: `SerialNumber` entry points at 0x1d62 = note IFD start
  0x1970 + 2 + 84*12). So `read_ifd(buf, at + 10)` and the existing `ascii` /
  `shorts` / `integer` helpers work as they do for Sony.
- Tag 0x0147 is `int16u[2]` (4 bytes, so the value rides inside the entry:
  `82 01 43 01` = `386 323`). Values across the samples: x 317..572,
  y 261..425. Neighbors 0x0146 and 0x0148 are `int8u` 0 on every sample; no
  area/frame size tag was found.
- Raw is 6080x4042, `DefaultCropSize` 6016x4012 (3:2), preview strip JPEG
  1620x1080, 1:1 JPEG 6016x4012. The point was overlaid by the requester and
  lands on the subject when read as `x/1000` of the width and `y/667` of the
  height (a 3:2 frame normalized to 1000 wide); `y/1000` misses. A second
  check on `BF_06156` was inconclusive (the 1000x667 reading lands on a prop
  next to the center child, not on a face), so it is treated as an
  assumption to verify in Step 1, not a fact.
- Sigma fp L DNGs do not carry 0x0147. Sigma also writes `FocusSetting`
  (0x0006, ASCII, `AF` on every sample) and `AFMode` (0x0005, `AF-C`).
- `riffle-cli info` already parses the sample (`preview` and `full` found,
  `focus: -`), and `maker_note_ifd` already returns `None` for a non-Sony
  make, so nothing else in the DNG path needs to change.

## Steps

- [x] Step 1: Parse the Sigma BF AF point into `Shot::focus` and document it
  - Done when:
    - `crates/core/src/arw.rs` reads Sigma MakerNote tag 0x0147
      (`TYPE_SHORT`, count 2, value inline in the entry) into
      `Shot::focus` as `FocusLocation { sensor_w: 1000, sensor_h: 667, x, y }`,
      only when `Make` starts with `SIGMA` case-insensitively and `Model`
      is exactly `Sigma BF`, and the MakerNote starts with `SIGMA\0\0\0`
      (skip the 10-byte header before `read_ifd`). Any other make/model
      (Leica, Sigma fp L, Sony) leaves the Sigma path untouched and
      `Shot::focus` exactly as before
    - The 1000x667 denominators are named constants with a doc comment that
      states the assumption (3:2 frame normalized to 1000 wide, unrotated
      sensor coordinates like Sony's `FocusLocation`, no area size known,
      derived from 11 landscape samples, portrait unverified) so a later
      correction is a one-line change
    - `focus_mode`, `af_tracking`, `focus_frame`, `focus_distance_mm` stay
      `None` for a Sigma note (no eye-AF path, no manual-focus gate; see
      Trade-offs)
    - A point outside the 1000x667 range is still stored (documented);
      `partial::focus_point` already clamps into the JPEG. A wrong type or
      count yields `None`, not an error; an offset past the buffer is an
      error, matching `shorts`/`ascii`
    - Unit tests in `arw.rs`, in the style of `tiff_with_leica_maker_note` /
      `leica_note` (a `sigma_note(entries)` builder that prefixes
      `SIGMA\0\0\0\x01\x04` and a `tiff_with_sigma_maker_note(model, note)`
      that sets `Make = "Sigma"` and `Model`): the tag present ->
      `Some(FocusLocation { 1000, 667, x, y })`; the tag absent -> `None`;
      `Model = "Sigma fp L"` with the tag present -> `None`; wrong
      type/count -> `None`; a Sigma note leaves `focus_mode`, `af_tracking`,
      `focus_frame`, `focus_distance_mm` `None`; the existing Sony and Leica
      tests are unchanged and green
    - Manual verification, recorded in `learnings.md`: `riffle-cli info` on
      the 11 sample files prints `focus: 1000 667 x y` matching
      `exiftool -u -Sigma_0x0147`; `riffle-cli scan` over the sample folder
      yields a sharpness for each. If the point is visibly off, note it
      (candidates: the point scaled against `DefaultCropSize`, or `y/1000`)
      in `learnings.md` and in the constants' doc comment
    - `README.md` "RAW formats and cameras" lists `Sigma BF` under DNG as
      verified, keeping the list's format. `CLAUDE.md` is updated only if
      warranted (a new helper inside `arw.rs` does not qualify)
    - `mise run ci` passes
  - Implementation approach:
    - Keep the change inside `exif()`: after the Sony `maker` block, add a
      `sigma_af_point(buf, &exif_ifd, make, model) -> Result<Option<FocusLocation>>`
      sibling of `leica_focus_distance` (same shape: find 0x927c, check the
      header bytes and length, `read_ifd(buf, at + SIGMA_HEADER_LEN)`, find
      the tag). Assign it to `shot.focus` only when the Sony path left it
      `None` (`shot.focus = shot.focus.or(...)`), so the Sony code path is
      untouched
    - Constants next to the Leica ones: `TAG_SIGMA_AF_POINT: u16 = 0x0147`,
      `SIGMA_HEADER: &[u8] = b"SIGMA\0\0\0"`, `SIGMA_HEADER_LEN: usize = 10`,
      `SIGMA_BF_MODEL: &str = "Sigma BF"`, and the two denominators
      (e.g. `SIGMA_BF_AF_GRID_W: u16 = 1000`, `SIGMA_BF_AF_GRID_H: u16 = 667`)
    - Reading a count-2 SHORT: the value is inline (`value.to_le_bytes()`
      split into two `u16le`), like `ascii`'s `count <= 4` branch; the
      existing `shorts::<N>` helper is for `N > 2` (value at offset) and
      must not be reused as is
    - Do not touch `maker_note_ifd` (its Sony header/`make` gate stays), the
      index schema (`focus_w/h/x/y` columns already hold any `FocusLocation`;
      no `SCHEMA_VERSION` bump), `commands.rs`, or the frontend
    - `crates/cli/src/main.rs` `info` and `focusbox` print `shot.focus` and
      need no change; they are the manual-verification tool

## Trade-offs and risks

- **Scale of the point (1000x667) is an assumption.** The requester's overlay
  supports it; a second check on `BF_06156` was inconclusive. The constants
  make any switch a one-line change.
- **Portrait frames are unverified.** All 11 samples are `Orientation=1`.
  The point is treated as unrotated sensor coordinates like Sony's
  `FocusLocation`, which `drawFocusMark`/`zoom.ts` rotate together with the
  JPEG. If the BF writes the point in the rotated (display) frame instead,
  portrait marks will be off; that needs a portrait sample.
- **Manual-focus gate.** Sony frames drop the point in MF via
  `trusted_focus` (`focus_mode == 0`). Sigma writes `FocusSetting` (0x0006,
  ASCII `AF`; the MF spelling is unknown). Taken: do not read it, the BF point
  is always trusted. Revisit with an MF sample.
- **No AF area size.** Only the point is known, so the BF takes the existing
  point-only path in `sharpness::score`. No eye-AF-frame path.
- **Gate strictness.** Gating on `Model == "Sigma BF"` means a future Sigma
  body writing 0x0147 is ignored until added; the alternative risks a wrong
  scale on a body with a different grid.
- **Behavior change on BF frames.** Already-indexed BF frames keep the old
  sharpness until `Clear Cache` or a file change. Worth a line in the PR
  description.
- **Single step.** The original plan split the README line into a Step 2; the
  user chose to fold it into Step 1 and run single-PR mode.

## Progress

- Step 1: Read the Sigma BF AF point (`TAG_SIGMA_AF_POINT`, tag `0x0147`) out
  of the Sigma MakerNote into `Shot.focus`, gated on `Make` starting with
  `SIGMA` (case-insensitive) and `Model == "Sigma BF"`.
