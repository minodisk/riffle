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

## Step 3: Persist the face-catch state in the index

- `SCHEMA_VERSION` was still 13 and `EXTRACTOR_VERSION` still 3 on the
  branch base (after #398), so the planned 14 / 4 held without renumbering.
- Every migration test asserts `user_version` after `Index::open`, including
  the v2 to v9 ones that drop `files`: all nine asserts moved from 13 to 14,
  not only the v10 to v13 fixtures. The v10 / v11 / v12 fixtures also drop
  `face_catch`, or the in-place `ALTER` would fail and the cache be discarded.
- The v13 fixture sets `extractor = 3` so the test shows the row survives
  the `ALTER` and is re-extracted by the extractor bump alone.
- `FaceCatch` moved from the test module's imports to the top of `index.rs`
  (Step 2 had imported it only for the `entry()` fixture).
- The round-trip test reads the state through `serde_json::to_value` of
  `Focus`, so it checks the strings the frontend receives, not just the enum.

## Step 4: Color the `f` focus mark by the state

- `FocusMark` carries the state (`faceCatch`) rather than a color, and
  `main.ts` maps it through `FOCUS_MARK_COLORS` next to `FOCUS_MARK_ARM` /
  `FOCUS_MARK_GAP`, satisfying both the "decision in `focus.ts`" and the
  "constants next to the arm and gap" wording of the plan. Orange is `#f93`.
- `metaGroups` gained an optional third argument so the existing call sites
  in the tests stay unchanged; `unknown` maps to a null row, which `section`
  already drops.
- The eye-AF-first scoring rule made the `todo.md` section intro stale since
  Step 1; it was rewritten here as Step 1 had deferred.

## Step 5: Draw the detected faces in the `f` focus mark

- `faces_of` / `read_faces` / `FacesResponse` live in
  `crates/app/src/commands.rs`; `read_faces` relies on `decode_rgb`'s own
  `catch_unwind`, so a non-JPEG preview comes back as `Err` without another
  guard (tested with a synthetic TIFF whose preview is `b"not a jpeg"`).
- Deviation from the plan's frontend wording: a `faces_of` response is **not**
  dropped when `current !== seq`. The cache is keyed by path and the request
  is marked in flight, so dropping it on a page turn would lose the only
  request for that path: paging away and back before it resolves finds the
  path still in flight, issues nothing, and the dropped response then leaves
  the frame without boxes until something else redraws. Instead the response
  is kept for its path and `draw()` runs only when that path is still
  current. Folder staleness is handled by a generation token in `FaceCache`
  (`crates/app/ui/src/faces.ts`): `clear()` (folder open, which covers
  clear-cache and the format switch, and every `refreshEntries`, which runs
  after a resync's scan) bumps it, and a response from before the clear is
  dropped without touching the new generation's in-flight entry.
- The in-flight set is a `Set<string>` inside `FaceCache` rather than a
  `Map<string, Promise>`: nothing awaits the promise a second time, so only
  membership matters.
- Faces are drawn by a separate `drawFaceMarks` called after `drawFocusMark`
  in `draw()`'s normal path, so a manual-focus or no-AF frame (no AF mark)
  still shows the faces of the whole-preview detection.
- Verified: `read_faces` on `_DSC3113` and `_DSC1942` (temporary `#[ignore]`d
  release test, removed before committing) returns the same boxes as
  `riffle-cli faces`. `_DSC3113` (caught, orientation 8): AF point (929,607)
  lies inside the face box (895,584) 57x47. `_DSC1942` (missed): AF point
  (762,449) on a jersey number, all three boxes beside it. The CLI's PNGs were
  checked by eye. Latency: ~30ms mean read + detect, warm cache, recorded in
  `docs/performance.md`.
- Not verified: the GUI itself (the canvas drawing, the cyan color over the
  mark, the redraw after the response, and the rotation carrying the boxes
  on portrait frames) could not be run here. It needs a manual check in the
  app with `f` on `_DSC3113` (green mark, box under the crosshair) and
  `_DSC1942` (orange mark, boxes beside it).

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
