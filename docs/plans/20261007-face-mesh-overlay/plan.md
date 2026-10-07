<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Face mesh overlay on the judged face

## Purpose

The focus mark (`f`) draws the faces Riffle detects as a cyan box with a dot
between the eyes (`drawFaceMarks` in `crates/app/ui/src/main.ts`), and the
meta pane's `Eyes` row says whether the eyes of the judged face are closed.
That judgment already runs the MediaPipe Face Landmarker v2 face mesh on the
judged face (`eyes::ear_of` in `crates/core/src/eyes.rs`), maps its 478 points
back to the full-size upright preview, takes the eye aspect ratio and throws
the points away. Drawing the mesh over that one face lets the user see what
the `Eyes` judgment looked at (where the lids landed, whether the mesh sits on
the face at all), at no new inference: the points ride back on the `eyes_of`
response and the frontend draws the mesh's edges in the same overlay as the
face boxes.

Scope, decided up front:

- Only the judged face gets a mesh (the one `eyes::judged_face` picks); the
  other boxes stay box-and-dot.
- The mesh appears when `eyes_of` answers, so a moment after the `Eyes` row,
  and under the existing focus mark toggle (`f`): no new setting. A face below
  `EYES_MIN_FACE`, a superseded request, a JPEG / view-only folder, the 1:1
  view and Compare get no mesh, exactly when they get no `Eyes` row or no face
  boxes today.
- The edges drawn are MediaPipe's `FACEMESH_TESSELATION` over the first 468
  points (verified 2026-10-07 from
  `mediapipe/python/solutions/face_mesh_connections.py` at `master`,
  Apache-2.0, Copyright 2021 The MediaPipe Authors: 2556 directed pairs that
  dedupe to 1322 undirected edges, indices 0..=467; the ten iris points
  468..=477 are not in it). The table must be embedded, since the model only
  outputs points.
- The EAR, the constants, the crop and the scan are untouched (see the module
  doc of `eyes.rs`: a change there is a new model).

## Steps

- [x] Step 1: Return the mesh points from `eyes_of` in stored preview coordinates
  - Done when:
    - `riffle_core::eyes` exposes the 478 points, in full-size upright preview
      pixels, next to the judgment, without changing what `ear_of` / `judge`
      compute. `riffle-cli` (`crates/cli/src/main.rs` lines ~729 and ~865 call
      `ear_of`) still compiles and its EARs are unchanged.
    - `riffle_core::faces` has a point mapping from upright to stored
      coordinates for `f32` points, sharing its arithmetic with `to_stored`
      (which maps a whole `Face`), with a unit test against `face_to_upright`
      / `to_stored` for orientations 1, 3, 6 and 8.
    - The `eyes_of` response (`EyesResponse` / `EyesJudgment` in
      `crates/app/src/commands.rs`) carries the points in the stored
      (unrotated) preview's coordinates plus that preview's `width` /
      `height`, the same space `faces_of` (`FacesResponse`) reports faces in,
      so the frontend scales them with the same `drawWidth / previewWidth`
      rule as `faceMarks`.
    - Tests: a core test that the window-to-full mapping of the model's points
      is the one `ear_of` applies (factor it out of `ear_of` into a pure
      function so it is testable without running the model); the `faces`
      point-mapping round trip above; the existing `read_eyes` tests keep
      passing (`eyes_without_a_face_are_unknown_and_a_jpeg_is_refused`,
      `a_superseded_eyes_request_stops_before_the_detection`), and the
      judgment-less responses carry no points. `mise run ci` passes.
    - The `eyes` timing line (`debugLog` in `requestEyes`) still adds up; the
      IPC cost of 478 points (~8 KB of JSON) is negligible, confirm with the
      `ipc=` figure once on a real file.
  - Implementation approach:
    - `crates/core/src/eyes.rs`: split `ear_of` into the points stage and the
      EAR stage. One shape that keeps the public API: a new
      `pub fn landmarks_of(rgb, width, height, face) -> Result<Option<Vec<(f32, f32)>>>`
      (the crop, the model, the mapping `win.x + p[0] * kx`), `ear_of` becomes
      `landmarks_of` + `ears`, and a new `pub fn judge_mesh(...) -> Option<(Eyes, Vec<(f32, f32)>)>`
      (or a small `Judged { eyes, points }` struct) with the same
      `EYES_MIN_FACE` floor and `catch_unwind` as `judge`, which becomes a
      wrapper that drops the points. Keep `judge`'s doc comment's guarantees.
    - `crates/core/src/faces.rs`: lift the `point` closure of `to_stored` into
      `pub fn point_to_stored((x, y): (f32, f32), orientation, width, height) -> (f32, f32)`
      and call it from `to_stored`, so the two cannot drift. Note `to_upright`
      is the `usize` pixel inverse (`height - 1 - y`); the `f32` mapping has
      no `- 1`, same as `to_stored` / `face_to_upright` today.
    - `crates/app/src/commands.rs` `read_eyes`: after `eyes::judge_mesh`, map
      every point with `faces::point_to_stored(p, orientation, w, h)` where
      `(w, h)` are the stored preview's from `decode_rgb` (the `uw, uh` from
      `upright_rgb` are the upright ones; `point_to_stored` takes the stored
      size like `to_stored` does, check its test). Add `width`, `height` and
      the points to the response. Put the points in `EyesJudgment` (they
      exist exactly when the judgment does), serialized as `[[x, y], ...]`
      (`Vec<[f32; 2]>`), so the frontend's `Eyes` type grows one field.
    - Do not change the `superseded` / request-id flow (see
      `docs/agents/tauri-app.md`, "An on-demand per-file command takes its
      request id from the frontend").
    - Update the layout note in `CLAUDE.md` only if a module's description
      changes (`src/eyes.rs` "not called by the scan" still holds; add that
      `eyes_of` also returns the mesh points).
- [ ] Step 2: Draw the mesh in the focus mark overlay
  - Done when:
    - With `f` on, a file whose `Eyes` row has a judgment shows the
      tessellation edges over the judged face, on the stored preview
      coordinates like the face boxes, so `draw()`'s rotation carries them for
      orientations 1, 3, 6 and 8 and they scale with the window like the
      boxes. The mesh appears when `eyes_of` settles for the current file
      (the settle path calls `draw()` as well as `renderMeta()`), and nothing
      is drawn when `eyesCache.get(path)` is `undefined` or `null`, in
      `viewOnly`, zoomed or comparing (the existing `drawFaceMarks` gates).
    - The tessellation table lives in one new DOM-free module
      (`crates/app/ui/src/facemesh.ts`), 1322 undirected edges as a flat
      index array, with a header stating its origin (MediaPipe
      `face_mesh_connections.py`, `FACEMESH_TESSELATION`, Apache-2.0,
      Copyright 2021 The MediaPipe Authors) the way `icons.ts` attributes
      Lucide, and the license text pointed at.
    - A pure helper (in `focus.ts` next to `faceMarks`, or in `facemesh.ts`)
      maps the points onto the drawn preview (`drawWidth / previewWidth`
      scale, centered on the origin) so the canvas code only strokes
      segments.
    - Frontend tests: the table is well-formed (exactly 1322 edges, every
      index an integer in `0..468`, no self-loop, no duplicate edge in either
      direction); the mapping helper scales and centers like `faceMarks`
      (same style as `focus.test.ts`); `meta.test.ts` / `eyes.test.ts`
      fixtures updated for the new `Eyes` field. `mise run ci` passes
      (`pnpm exec vp fmt` / `check` / `test`).
    - Docs: the focus mark paragraph in `README.md` + `README.ja.md` and in
      `docs/humans/usage.md` (line ~196) + `docs/humans/usage.ja.md` (line 14)
      gain one sentence: the judged face also gets the face mesh, a moment
      after the `Eyes` row. `CLAUDE.md`'s frontend layout gets `facemesh.ts`.
    - A manual check on a real file is listed in `learnings.md` as done or
      pending: an upright ARW with a face, a portrait (orientation 6 or 8)
      one, and a face below 60 px (no mesh, no error).
  - Implementation approach:
    - Build the table once from the downloaded `face_mesh_connections.py`
      (do not commit the Python file): parse `FACEMESH_TESSELATION`, take
      each pair as `(min, max)`, dedupe, sort, emit as a flat
      `number[]` / `Uint16Array` literal of 2644 values. Record the source
      URL, the commit and the counts in the module header and in
      `learnings.md`. Edge count check: 2556 directed pairs, 1322 unique.
    - License: point the header at the existing
      `crates/core/models/LICENSE-mediapipe` and add a "Provenance" paragraph
      there for the table, so one notice covers both MediaPipe artifacts.
    - Colors: `docs/agents/ui-styling.md` says canvas drawing uses literal
      colors read once, not `var()`; follow `drawFaceMarks`'s
      outline-then-color passes (black 0.8 outline, then the color) but
      thinner, e.g. outline `lineWidth` 2 and color 1, or a single
      half-alpha `FACE_MARK_COLOR` stroke, so ~1300 edges do not bury the
      face. Decide while looking at a real file; keep the constants next to
      `FACE_MARK_COLOR` / `FACE_MARK_EYE_RADIUS` in `main.ts`.
    - Draw order: box and dot first, mesh after, in the same `context.save()`
      block or a sibling `drawFaceMesh(drawWidth, drawHeight)` called from
      `draw()` right after `drawFaceMarks`. One `beginPath` with 1322
      `moveTo` / `lineTo` pairs and one `stroke` per pass is enough; do not
      stroke per edge.
    - `requestEyes`'s settle branch: `if (eyesCache.settle(...) && files[index] === path) { renderMeta(); if (showFocus) draw(); }`.
    - `eyes.ts`: `Eyes` gains `mesh: { width: number; height: number; points: [number, number][] }`
      (or `points` plus `width` / `height` flat), mirroring `EyesJudgment`.
      `EyesCache` itself needs no change.
    - The `faces_of` boxes and the `eyes_of` face are detected
      independently (`read_faces` and `read_eyes` both call the scan's
      detection), so the mesh may sit on a face slightly off its box; that is
      expected, do not try to match them.

## Trade-offs and risks

- **Where the points are serialized.** Inside `EyesJudgment` (chosen: the
  points exist exactly when the judgment does, one `null` check in the
  frontend) versus a sibling `mesh: Option<...>` on `EyesResponse` (keeps
  `Eyes` / `EyesCache` untouched but adds a second cache or a combined
  value). The first means the frontend `Eyes` type and its test fixtures
  change.
- **Alternative: send the edge table from Rust.** Embedding the table in core
  and shipping edges per response would be ~2600 extra numbers per call for
  a constant; rejected, the table is static and belongs with the drawing.
- **Irises (468..=477).** Not in the tessellation; drawing them as dots or
  rings is a small addition but not asked for. Default: not drawn.
- **Visual density.** 1322 edges on a 60-120 px face at fit-to-window scale
  may read as a cyan blob; the line widths are to be chosen on a real file,
  and if it stays unreadable the fallback is to draw only the
  `FACEMESH_CONTOURS` subset (eyes, brows, lips, oval; a separate, smaller
  MediaPipe table). The plan embeds the full tessellation as asked.
- **IPC size.** 478 points as JSON floats is ~8-10 KB per `eyes_of`; the
  existing `eyes` timing line's `ipc=` figure will show if that matters (it
  should not; `faces_of` and the RGBA crops are far larger).
- **Face mismatch.** The mesh face comes from `read_eyes`'s own detection,
  the boxes from `read_faces`'s; with a trusted AF point both run
  `detect_around` on the same crop and agree, without one `read_eyes` picks
  the largest face of a whole-image detection, which may be a face the
  boxes also show but not necessarily pixel-identical.

## Progress

- (2026-10-07) Step 1 complete
