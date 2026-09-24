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

# Face-catch state and face boxes in the focus mark

## Purpose

The sharpness score is one cue per frame; it says how sharp the scored window
is, not whether the AF was on the subject's face at all. Culling a burst of
people shots needs that second cue: did the AF catch a face, or did it land on
the background or on the wrong person? Today the scan already runs YuNet on
non-eye-AF frames but throws the answer away, and it runs it on the whole
1616x1080 preview, where a 45-60 px face shrinks to ~10 px in the 320 px model
input and is missed.

Verified on 2134 ILCE-7M5 ARWs (details in the task brief; keep them in
`investigation.md` if the implementer wants them on record):

- Sony `AFTracking` takes only 0 (off), 1 (face tracking; on the α7 V also set
  when the AF sits on the back of a head), 2 (lock-on AF).
- Cropping the upright preview to 480x480 around the AF point at native
  resolution and giving the crop to `faces::detect` raises "AF inside a face"
  on tracking=1 frames from 56% to 73% at the same cost (~41 ms vs ~47 ms per
  call), and leaves fewer "no face near AF" misses than a 320 crop.
- `crates/core/src/sharpness.rs` `score` currently scores the eye window of
  the highest-confidence face when the AF point lies outside it; in 67 of 101
  such frames that face is a bystander (e.g. `_DSC1942` ranks high in its
  burst on a stranger's eyes). Which face the photographer wanted is not
  decidable by software; Riffle will not try.

Decided direction (agreed with the user):

1. A tri-state **face-catch state** per frame, `caught` / `missed` /
   `unknown`, computed at scan time and stored in the SQLite index, kept apart
   from the sharpness score (never folded into it):
   - Sony frame where `eye_af_frame` succeeds: `caught`, trusting the camera.
     No YuNet run at scan time there (it would flag back-of-head and upturned
     faces as misses and cost ~70% more scan time).
   - Otherwise, with a trusted AF point (`trusted_focus`): detect faces on a
     480x480 crop of the upright preview around the AF point. AF inside a
     detected face: `caught`; faces in the crop but the AF inside none:
     `missed`; no face in the crop: `unknown`.
   - No trusted AF point (manual focus, Leica DNG, SIGMA fp L): `unknown`.
2. Sharpness stops scoring another face's eye window when the AF point is off
   the face: with a trusted AF point the AF window is scored, whatever the
   detector found. Faces steer the score only when there is no AF point.
3. The `f` focus mark shows the state through its color; the meta pane's
   Riffle section gets a row.
4. The `f` mark also draws the detected faces: a rectangle per face box and a
   dot at the midpoint between its eyes, in a color distinct from the mark's
   own colors (cyan). Faces are detected **on demand at display time** by a
   Tauri command, cached in memory per file, never stored in the index and
   never run at scan time for eye-AF frames, so the scan cost and the index
   layout do not change for them. The detection region is the same one the
   scan-time state used (the 480 crop with a trusted AF point, the whole
   upright preview without one), through one shared core function, so the
   boxes never contradict the stored state; on eye-AF frames the crop
   detection runs for display only and a missing box there does not touch the
   state.

Constraints: the scan must not get slower on Sony files (the number of YuNet
calls per file at scan time must not increase); everything committed in
English, except `README.ja.md`, which every step that changes `README.md`
updates in Japanese in the same PR (see `CLAUDE.md`); `mise run ci` passes for
every step. Ground-truth files for a manual check are under
`/mnt/d/Photos/2026/2026-09-19` (`D:\Photos\2026\2026-09-19`):
`_DSC3113`, `_DSC3269`, `_DSC3562` (AF on the wanted face, found with the
crop), `_DSC2090` (upturned face, still missed), `_DSC2188` / `_DSC2266` /
`_DSC2354` / `_DSC2227` / `_DSC2367` (tracking=1 but the AF really off the
face), `_DSC1942` / `_DSC2289` / `_DSC2290` (tracking 0/2, AF on the wrong
person, so crop detection says `missed`), `_DSC1985` / `_DSC2992` (back of
head, tracking=1, `caught` by design).

## Steps

- [x] Step 1: Sharpness scores the AF window when the AF point is off the face
  - Done when:
    - `crates/core/src/sharpness.rs` `score`: with a trusted AF point
      (`focus` is `Some`) the window is always `window_at(w, h, p.0, p.1,
      WINDOW)` on the AF point (or the eye-AF-sized window when `frame` is
      `Some`, as today); `faces` are consulted only when `focus` is `None`
      (eye window of `chosen_face`, else `tile_max`). The module doc's four
      paths are rewritten to match: 0. eye-AF frame; 1. trusted AF point, faces
      ignored; 2. no AF point and a face at or above `FACE_CONFIDENCE`: between
      the eyes; 3. neither: the sharpest tile. The doc says why: the detector
      cannot tell the wanted face from a bystander, so an AF point off every
      face is trusted over the face.
    - Tests: `a_focus_point_outside_the_face_moves_to_the_eyes` is replaced by
      one asserting the score equals the AF-window score with no faces
      (`score_preview(&sharp, Some(background), None, &f) ==
      score_preview(&sharp, Some(background), None, &[])`);
      `a_focus_point_inside_the_face_keeps_the_af_window`,
      `a_face_scores_its_eyes_instead_of_the_sharpest_tile` (no AF point) and
      the rest still pass.
    - `crates/app/src/index.rs`: `EXTRACTOR_VERSION` is 3 and its doc comment
      lists what 3 changed (the score with an AF point off the face). No schema
      change.
    - `CLAUDE.md`'s layout sentence for `sharpness.rs` ("else between the eyes
      of a detected face (or on the AF point when it falls inside that face),
      else around the AF point, else from the sharpest tile") is reordered to
      the new precedence; `README.md` (and `README.ja.md`) "Sharpness cue" and
      `docs/usage.md` "Sharpness cue" say the eyes are used only when the
      camera recorded no AF point.
    - `mise run ci` passes.
  - Implementation approach:
    - Pure signature change is not needed: keep `score_preview(preview, focus,
      frame, faces)`; only the `match` in `score` changes. `scan.rs` keeps
      passing whole-image faces for now (Step 2 restructures it).
    - Follow `docs/agents/tauri-app.md` "Bump `EXTRACTOR_VERSION`, not
      `SCHEMA_VERSION`, when extraction output changes".

- [x] Step 2: Compute the face-catch state in `riffle-core` through one shared detection entry point
  - Done when:
    - `crates/core/src/faces.rs` (or a sibling module, the implementer's
      choice; keep the enum out of `lib.rs` unless a second consumer appears)
      has:
      - `pub enum FaceCatch { Caught, Missed, Unknown }` (`Copy`, `Debug`,
        `PartialEq`, `Eq`, `Default = Unknown`).
      - `pub const CATCH_CROP: usize = 480`, the crop side, and the confidence
        threshold the state uses (see the approach below) as one named
        constant, so Step 5 reads the same value.
      - A point mapping from stored to upright coordinates, the inverse of
        `to_stored`, for orientation 1 / 3 / 6 / 8: 6: `(x, y) -> (h-1-y, x)`;
        8: `(y, w-1-x)`; 3: `(w-1-x, h-1-y)` (`w` x `h` is the stored size).
        Unit test: mapping a point and applying `to_stored` to a face at the
        mapped point lands within 1 px of the original for all four
        orientations.
      - A crop helper that cuts a `CATCH_CROP` square (clamped inside the
        upright image; the whole image when it is smaller; reuse
        `sharpness::window_at` for the clamp) out of an upright RGB buffer and
        returns the sub-buffer with its origin. Unit test on a synthetic
        gradient: the crop's pixels equal the source pixels at the offset, and a
        corner point is clamped.
      - **The shared entry point** `pub fn detect_around(preview: &[u8],
        orientation: u16, focus: Option<FocusLocation>) -> Result<Detection>`
        (name at the implementer's discretion) that decodes and orients the
        preview (including the orientation-3 reversal `scan::detect_faces`
        does today, factored into an `upright_rgb` helper), picks the region
        (the crop around `partial::focus_point(w, h, Some(focus))` mapped to
        upright when `focus` is `Some`, the whole upright image when `None`),
        runs `faces::detect` exactly once, offsets crop results by the crop
        origin, applies the threshold, and returns `Detection { width,
        height, faces: Vec<Face> (stored coordinates via `to_stored`),
        point: Option<(usize, usize)> (the AF point in stored coordinates) }`.
        This is the only place that decides the region, so the scan (this
        step) and the display command (Step 5) cannot drift.
      - A pure classifier `pub fn face_catch(faces: &[Face], point: (usize,
        usize)) -> FaceCatch` (faces and point in the same coordinates):
        `Caught` when the point lies inside any face box, `Missed` when there
        are faces but the point is inside none, `Unknown` when there are none.
        Unit tests for the three outcomes and for a point on a box edge.
    - `crates/core/src/scan.rs`: `Entry` gains `pub face_catch: FaceCatch`.
      `extract` computes it: `eye_af_frame(&shot).is_some()` gives `Caught`
      with no detection; else it calls `detect_around(&preview,
      orientation, trusted_focus(&shot))` once: with a trusted AF point the
      state is `face_catch(&d.faces, d.point)` and `score_preview` gets
      `&[]` (Step 1 made faces irrelevant there); without one the state is
      `Unknown` and `d.faces` still go to `score_preview` so the eyes keep
      being scored on that path (see Trade-offs for the alternative). YuNet is
      called at most once per file on every path, exactly as today. Any
      detection failure or panic is `Unknown`, never an error for the file.
    - `crates/cli/src/main.rs` `faces` subcommand calls `detect_around` with
      the file's trusted AF point, prints the face-catch state, and draws the
      region and the faces found, so the ground-truth files can be checked by
      eye (`riffle-cli faces <file.ARW> <out.png>`).
    - `docs/performance.md` "Face detection cost" gains a short paragraph with
      `riffle-cli scan` before/after (same folder, runs alternated, threads
      and machine stated as the guide's "Write a performance number with its
      measurement conditions" requires) showing the per-file mean did not
      rise on the 2134-file Sony folder.
    - Manual check recorded in `learnings.md`: the state `riffle-cli faces`
      prints for the ground-truth files above (expected: `_DSC3113` /
      `_DSC3269` / `_DSC3562` caught; `_DSC1942` / `_DSC2289` / `_DSC2290`
      missed; `_DSC1985` / `_DSC2992` caught via eye-AF; `_DSC2090` unknown or
      missed).
    - `crates/app` compiles unchanged apart from `Entry`'s new field (the
      index does not store it yet; the extraction output the index sees is
      identical, so no `EXTRACTOR_VERSION` bump in this step).
    - `mise run ci` passes.
  - Implementation approach:
    - `decode_rgb` + `apply_orientation` already exist in `decode.rs`.
    - Which faces count: start with everything `faces::detect` returns
      (`SCORE_THRESHOLD` 0.6, what the verified numbers used); if the manual
      check shows false `missed` states from low-confidence boxes, raise it to
      `sharpness::FACE_CONFIDENCE` and record the reason. Do not decide this
      before measuring. Whatever is chosen is the one constant `detect_around`
      applies, so the display path inherits it.
    - Keep `faces::detect`'s signature; the crop is just a smaller image to it.

- [ ] Step 3: Persist the face-catch state in the index and hand it to the frontend
  - Done when:
    - `crates/app/src/index.rs`: `files` gains `face_catch INTEGER NOT NULL
      DEFAULT 0` (`0` unknown, `1` caught, `2` missed, coded like
      `ratings.flag`); `write_batch` writes `entry.face_catch` (`0` in the
      error-row insert); `entries` reads it back. `SCHEMA_VERSION` is 14 with
      its doc comment extended (a v10 to v13 database gains the column in place
      with `ALTER TABLE` and keeps `files`, `ratings`, `folders`); `13` joins
      the `prepare` whitelist; the new guard is `(10..14).contains(&version)`
      and the existing `(10..12)` / `(10..13)` guards stay. `EXTRACTOR_VERSION`
      is 4 (doc: 4 fills in the face-catch state).
    - The state reaches the frontend as a string `"caught" | "missed" |
      "unknown"` on the existing `Focus` struct (`face_catch`, serialized with
      a `serialize_with` function like `flag`), so a row without an AF point
      carries nothing extra. `src/main.ts`'s `Focus` interface and
      `src/focus.ts`'s `MarkFocus` gain the field (no behavior yet).
    - Index tests: a round trip writes entries with `Caught` / `Missed` /
      `Unknown` and reads the strings back; a v13-fixture migration test shows
      the column is gained in place and `files` / `ratings` survive (existing
      fixtures faking v10 to v13 also `DROP COLUMN face_catch`; asserts on
      `user_version == 13` move to 14).
    - `mise run ci` passes.
  - Implementation approach:
    - Follow `docs/plans/_archived/20260924-focus-mark-af-frame/` Step 1
      exactly (same shape of change: column + `ALTER` guard + extractor bump +
      fixture drops); its `learnings.md` records the fixture pitfall.
    - Audit every `!= SCHEMA_VERSION` guard while bumping, per the guide's
      "Bumping `SCHEMA_VERSION` can strand an old per-version column guard".

- [ ] Step 4: Color the `f` focus mark by the state and show it in the meta pane
  - Done when:
    - `crates/app/ui/src/focus.ts`: `FocusMark` gains `faceCatch` (or the
      color directly, via a pure `focusMarkColor(state)`), tested in
      `focus.test.ts` for the three states and for the manual-focus /
      no-focus `null` cases staying `null`.
    - `src/main.ts` `drawFocusMark` strokes the crosshair and the AF frame in
      the state's color over the same black outline: green (`#3f3`, today's
      color) for `caught`, orange for `missed`, white for `unknown`. Constants
      next to `FOCUS_MARK_ARM` / `FOCUS_MARK_GAP`. `drawZoom` and
      `drawCompare` are unchanged. The comment above `drawFocusMark` describes
      the colors.
    - `src/meta.ts` `metaGroups` takes the state and the Riffle section shows a
      `Face` row, `Caught` or `Missed`, omitted for `unknown` (a null row is
      already dropped by `section`); `meta.test.ts` covers the three cases;
      `renderMeta` passes `entries.get(files[index])?.focus?.face_catch`.
    - `README.md` (and `README.ja.md`) "Focus mark" and `docs/usage.md`
      "Focus mark" and the meta pane paragraph describe the colors and the
      row: green when the camera's face tracking or a face detected under the
      AF point says the AF caught a face, orange when faces were found near
      the AF point but it is on none of them, white when Riffle does not know
      (no AF point, manual focus, no face near the point). `CLAUDE.md`'s
      layout paragraph mentions the face-catch state next to `faces.rs` /
      `index.rs`. `todo.md`'s "face/eye-aware focus check" section: reword the
      "Store the face region in the SQLite index" item to what was decided
      (the state is stored, the faces are detected on demand in Step 5) and
      note that the off-face bystander scoring is gone.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the color decision in `focus.ts` (pure, tested); `main.ts` only
      reads it, matching how `focusMark` already carries the geometry.

- [ ] Step 5: Draw the detected faces in the `f` focus mark, detected on demand
  - Done when:
    - `crates/app/src/commands.rs`: a `faces_of(path: String)` command
      (registered in `crates/app/src/main.rs`'s `generate_handler!`) runs on
      `spawn_blocking` like `preview` / `focus_crop`: it reads the same preview
      tier `preview` serves (`riffle_core::reader::read_preview`), calls
      `faces::detect_around(&jpeg, arw.orientation, trusted_focus(&arw.shot))`
      from Step 2 (so the region and threshold are the scan's; on eye-AF
      frames this is the crop detection the scan skipped, for display only)
      and returns JSON `{ width, height, faces: [{ x, y, width, height, eye:
      { x, y } }] }` in the preview's stored pixel coordinates, `eye` being
      the midpoint of `left_eye` / `right_eye`. The core of it is a plain
      function `read_faces(path) -> Result<FacesResponse, String>` with a unit
      test on a synthetic TIFF fixture (as `scan.rs` tests build one) that
      checks the size round-trips and a non-JPEG preview is an `Err`, not a
      panic. The command does not touch the index or the stored state.
    - `crates/app/ui/src/main.ts`: an in-memory `Map<string, Faces>` keyed by
      path (cleared where `sharpness` is cleared: folder open, resync,
      clear-cache) with a `Map<string, Promise<...>>` of in-flight requests so
      a path is never detected twice. When the focus mark is on and the
      normal view draws a frame whose faces are not cached, the request is
      issued once; on resolve, if `current !== seq` the response is dropped
      (the `focus_crop` pattern at `main.ts` ~1223-1245), else it is cached
      and `draw()` is called again. Errors are logged, not shown, and cached
      as "no faces" so a bad file is not retried on every draw. No request is
      made while the mark is off, in the 1:1 view or in Compare.
    - `crates/app/ui/src/focus.ts`: a pure `faceMarks(faces, previewWidth,
      previewHeight, drawWidth, drawHeight)` returning, per face, a rectangle
      and the eye dot in the same centered unrotated preview coordinates
      `focusMark` uses (`x * drawWidth / previewWidth - drawWidth / 2`, same
      for `y`), so the rotation `draw()` already applied carries every
      orientation with no orientation-specific code. `focus.test.ts` covers
      a face at the origin, one at the far corner, and a scaled preview
      (`drawWidth != previewWidth`), with sizes that divide exactly so
      `toEqual` needs no tolerance.
    - `drawFocusMark` strokes each face rectangle and fills a small dot at
      the eye point after the AF mark, with the same black-outline-then-color
      two-pass style, in cyan (a constant next to the mark colors, e.g.
      `#3ff`). Only the normal view draws faces; `drawZoom` and `drawCompare`
      are unchanged (see Trade-offs).
    - A frontend unit test for the cache / stale logic where it is pure:
      extract the "which paths need a request / accept or drop this
      response" decision into a small tested function (e.g. in `focus.ts` or
      a new `faces.ts`), leaving only the `invoke` call in `main.ts`.
    - `README.md` (and `README.ja.md`) "Focus mark" and `docs/usage.md`
      "Focus mark" say the mark also draws the faces Riffle detects near the
      AF point (cyan box and a dot between the eyes), that they appear a
      moment after `f` because the detection runs when the frame is shown,
      and that on Sony face-tracked frames the boxes are informative only (the
      mark's green color comes from the camera). `CLAUDE.md`'s layout
      paragraph mentions the command in `commands.rs`. `docs/performance.md`
      notes the on-demand latency measured once on a real file (read +
      detect, ms, conditions).
    - `mise run ci` passes; a manual check on the ground-truth files shows
      the boxes agree with the mark color (`_DSC3113` green with a box under
      the crosshair, `_DSC1942` orange with the box beside it).
  - Implementation approach:
    - The drawn bitmap is the preview JPEG itself (`shown.bitmap`, possibly
      downscaled by the worker under the Linux pixel limit), and `drawWidth`
      / `drawHeight` always span the whole image, so mapping by ratio to the
      preview size the command returns is correct at any scale; do not use
      `bitmap.width` as the preview width.
    - Depends on Step 2 (`detect_around`) and Step 4 (mark colors). Assumes
      both are merged.
    - Follow the `docs/agents/tauri-app.md` items "Give the current folder
      one token, not one counter per feature" for the cache reset and
      "Synchronous commands run on the main thread" (hence `async` +
      `spawn_blocking`).

## Trade-offs and risks

- **Whole-image detection on the no-AF path: keep (planned) or drop.**
  Kept, YuNet still runs once at scan time on manual-focus Sony frames, Leica
  DNGs and SIGMA fp L files so their sharpness keeps scoring the eyes of the
  best face; the call count is unchanged and nothing visible changes for
  those users. Dropped, those files would score `tile_max`, the scan would
  get faster for DNG folders, and the face-aware sharpness path would shrink
  to nothing (the display path of Step 5 would still show faces). Recall on
  M11-P DNGs is already poor (no face in 29 of 36 samples), so the value kept
  is modest; the plan keeps it because dropping is a separate behavior change
  with its own measurement, not part of this feature. If the caller prefers
  dropping, Step 2's no-AF branch becomes "no detection, `Unknown`" and
  `score_preview` loses its `faces` parameter.
- **Confidence threshold for "faces in the crop".** The detector's own
  `SCORE_THRESHOLD` (0.6) is what the verified numbers used; the sharpness
  path uses `FACE_CONFIDENCE` (0.8). Using 0.6 finds more `missed`; 0.8 turns
  weak boxes into `unknown`. Step 2 measures on the ground-truth files before
  choosing; the display path inherits the choice through `detect_around`.
- **Face boxes on demand, not at scan time (decided).** Detecting when the
  frame is shown keeps the scan cost and the index layout unchanged (no YuNet
  at scan time on eye-AF frames, no box columns), at the price of a short
  delay (~40 ms detection plus the preview read) between pressing `f` or
  paging and the boxes appearing, and of a per-session in-memory cache that
  is lost on restart. Storing boxes in the index would remove the delay but
  cost four columns per face, a scan-time detection on every Sony eye-AF
  frame (~70% more scan time), and a schema change; rejected for that.
- **Boxes on Sony eye-AF frames.** Running the crop detection for display on
  frames whose state came from the camera can show a green mark with no box
  (back of a head, upturned face). Planned: draw whatever is found and let the
  docs explain that the color comes from the camera. Alternative: skip
  detection on eye-AF frames so the mark never looks inconsistent, losing the
  boxes on most Sony people shots. Step 5 goes with the former; flip it if the
  manual check finds the mismatch confusing.
- **Zoom / Compare views.** Planned: faces only in the normal view, like the
  AF mark today. Drawing them in the 1:1 view would need the crop's origin
  mapping (`placeholderRect` arithmetic) and in Compare the per-cell
  transform; both are larger changes for little culling value and can be a
  follow-up.
- **Cache eviction.** Planned: unbounded per session, cleared with the folder
  (a few hundred bytes per file). If a session opens tens of thousands of
  files this is still small; no eviction policy is planned.
- **Presentation of the state.** Option A (planned): the mark's color,
  green / orange / white. Cheap, works on both the eye-AF and the crop path,
  but makes every non-Sony body's mark white where it was green, and green
  vs orange is weak for red-green color-blind users. Option B: color plus a
  shape difference (e.g. the `missed` crosshair drawn as an X, or a double
  rectangle for `caught`) to be color-independent; slightly more drawing
  code. If B is wanted, only Step 4 changes.
- **What `unknown` looks like.** White (planned) tells the user the cue is
  absent. Keeping green for `unknown` would leave Leica users' mark as it is
  today but make `caught` indistinguishable from "no idea", which defeats the
  cue.
- **Two extractor bumps (Step 1 → 3, Step 3 → 4).** Users on the release
  between them re-extract twice. Merging Steps 1 and 3 avoids that at the cost
  of a larger review unit; the plan accepts the double rescan (a few seconds
  per folder) for smaller PRs.
- **`caught` on the back of a head.** By design, Sony tracking=1 is trusted
  (`_DSC1985`, `_DSC2992`). The docs say "the camera's face tracking" rather
  than "a face", so the wording is honest.
- **Risk: migration fixtures.** Faking v10 to v13 with a fixture built by
  `open` requires dropping `face_catch` too, or the in-place `ALTER` fails and
  the cache is discarded (recorded in the focus-mark-af-frame learnings).
- **Risk: previews smaller than the crop.** Tiny previews give a crop equal to
  the whole image; `window_at` already handles it, and the classifier does not
  care about the crop size.

## Progress

- (2026-09-24) Step 1 complete
