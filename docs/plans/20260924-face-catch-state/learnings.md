# Learnings

## Step 1: Sharpness scores the AF window when the AF point is off the face

- With the AF-point-inside-face case gone, `sharpness::inside` had no caller
  and was removed to keep clippy's dead-code check green. Step 2's
  `face_catch` classifier needs the same point-in-box test; write it there
  (faces.rs) rather than resurrecting the private helper here.
- `todo.md` line ~315 still says sharpness is scored on the eyes "when the AF
  point is missing or off the face". Left untouched: Step 4 already rewords
  that section of `todo.md`, and this step does not edit `todo.md`.
- `README.md` / `README.ja.md` "What the camera records" (the AF position row)
  already described the new precedence (AF point first, eyes only without
  one), so only the "Sharpness cue" bullets needed rewording.

## Step 2: Compute the face-catch state in `riffle-core`

- `detect_around`, `face_catch`, `FaceCatch`, `CATCH_CROP`,
  `CATCH_CONFIDENCE`, `to_upright`, `upright_rgb` and `crop_rgb` live in
  `crates/core/src/faces.rs`. `faces.rs` now imports `sharpness::window_at`
  while `sharpness.rs` imports `faces::Face`; Rust is fine with the mutual
  module use.
- `to_stored` maps continuous coordinates (`h - x`) and `to_upright` maps
  pixel indices (`h - 1 - y`), as the plan specified, so a round trip is
  within 1 px, not exact. The test asserts that tolerance.
- In the scan, `Detection::point` being `Some` is the "trusted AF point"
  branch (it is `Some` exactly when `trusted_focus` was), so `extract` does
  not look at `focus` twice.
- Manual check (`riffle-cli faces`, α7 V, `/mnt/d/Photos/2026/2026-09-19`):
  - `_DSC3113`, `_DSC3269`, `_DSC3562`: caught, via camera face tracking.
    These three are tracking=1 eye-AF frames, so the scan never runs the crop
    detection on them. Running it anyway for display, the AF point lies
    inside a detected face in all three, so the Step 5 boxes will agree.
  - `_DSC1942`, `_DSC2289`, `_DSC2290`: missed. Each also has a face scoring
    at least 0.8, none of them under the AF point, so 0.8 would give the same
    result.
  - `_DSC1985`, `_DSC2992`: caught via camera face tracking (back of head),
    as designed.
  - `_DSC2090`: **caught via camera face tracking**, not the "unknown or
    missed" the plan expected. It is a tracking=1 frame with a valid AF
    frame, so the eye-AF rule decides before any detection runs. The crop
    detection alone would say missed: the AF point at (704,443) sits just
    left of the face box starting at x=711.
  - `_DSC2188`, `_DSC2266`, `_DSC2354`, `_DSC2227`, `_DSC2367`: caught via
    camera face tracking (tracking=1). The camera is trusted by design, even
    though the AF is really off the face here.
- Confidence threshold: measured on the 428 files of the folder that take the
  crop path (a trusted AF point, no eye-AF), comparing 0.6
  (`SCORE_THRESHOLD`) with 0.8 (`FACE_CONFIDENCE`):
  - The counts were 166 caught / 188 missed / 74 unknown at 0.6, against 141
    / 145 / 142 at 0.8. The state differs on 76 files.
  - At 0.8, 25 of the files caught at 0.6 are lost: 18 become unknown and 7
    become **missed**. In those 7 the face under the AF scores 0.6-0.8 (often
    upturned or in profile) while a bystander scores above 0.8, so 0.8 would
    create false misses.
  - A visual check of 10 of the 25 found the AF on a real face in all 10.
  - A visual check of 10 of the missed-at-0.6 files that become unknown at
    0.8 found real faces off the AF point in 9. Most had the AF on the back
    of a head, which is a genuine miss. The tenth (`_DSC3632`) was a box on
    a shirt logo.
  - Kept 0.6: `CATCH_CONFIDENCE = SCORE_THRESHOLD`.
- Scan cost, `riffle-cli scan` before (origin/main at 4bbdaaa) and after,
  runs alternated, same folder and machine (see `docs/performance.md`):
  - On one thread, the mean was 22.0-22.9 ms before and 21.8-22.1 ms after,
    apart from one 29.5 ms outlier.
  - On 12 threads the runs overlap at 35-42 ms, with drive noise in both
    directions.
  - The crop is smaller than the whole preview, so the downscale into the
    model input costs less. The upright rotation of the whole preview is
    still done, as before.

## Deferred issues (todo candidates)

- The "eye-AF frame is caught, else classify the crop" rule is written twice:
  in `crates/core/src/scan.rs` `extract` and in `crates/cli/src/main.rs`
  `faces`. A core helper taking the shot and a lazy detection would remove
  the copy. It was left as is because the scan must not detect on eye-AF
  frames while the CLI always does. Basis: Step 2 implementation.
- The crop detection puts false boxes on printed faces or logos
  (`_DSC3632`: a shirt logo scored 0.67), which yields a false `missed`.
  A size or aspect filter on crop boxes could be measured later. Related
  files: `crates/core/src/faces.rs` (`detect_around`). Basis: Step 2 manual
  check.
