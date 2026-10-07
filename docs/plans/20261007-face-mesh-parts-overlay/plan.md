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

# Face parts outline instead of the full face mesh

## Purpose

`face-mesh-overlay` (`docs/plans/_archived/20261007-face-mesh-overlay/plan.md`,
PRs #717 / #718) draws MediaPipe's full `FACEMESH_TESSELATION` (1322 edges,
`crates/app/ui/src/facemesh.ts`, stroked by `drawFaceMesh` in
`crates/app/ui/src/main.ts`) over the judged face while `f` is on. On real
files it buries the face: the plan's own fallback was the contours subset.
This work replaces the tessellation with the parts a person reads at a
glance, the face oval, eyes, eyebrows, lips (`FACEMESH_CONTOURS`), the nose
(`FACEMESH_NOSE`) and the two iris rings with a dot at each iris center
(`FACEMESH_IRISES` plus points 468 and 473), and leaves the irises out when
the `Eyes` judgment is `closed`, since the model places iris points on a
closed eye too. Everything else (the `eyes_of` response, the gates, the
rotation and scaling, the EAR) is untouched; no new setting.

Verified 2026-10-07 from `mediapipe/python/solutions/face_mesh_connections.py`
at commit `212f110c65818db7f2d08c42a9539ecb2b6c0e1d` (the commit the current
table cites; Apache-2.0, Copyright 2021 The MediaPipe Authors):

- `FACEMESH_CONTOURS` = union of `FACEMESH_LIPS`, `FACEMESH_LEFT_EYE`,
  `FACEMESH_LEFT_EYEBROW`, `FACEMESH_RIGHT_EYE`, `FACEMESH_RIGHT_EYEBROW`,
  `FACEMESH_FACE_OVAL`: 124 directed pairs, 124 undirected edges, indices
  0..=466 (128 points).
- `FACEMESH_NOSE` exists in that file: 25 directed pairs, 25 undirected
  edges, indices 1..=440 (24 points). It shares no edge with the contours,
  so contours + nose = 149 undirected edges, all below 468.
- `FACEMESH_IRISES` = `FACEMESH_LEFT_IRIS` + `FACEMESH_RIGHT_IRIS`: 8
  undirected edges, two closed rings `469-470-471-472` (around center 468)
  and `474-475-476-477` (around center 473). The centers 468 and 473 are in
  no edge, so they are drawn as dots.
- The frontend already has all 478 points: `Eyes.mesh.points` in
  `crates/app/ui/src/eyes.ts` mirrors `EyesMesh` in
  `crates/app/src/commands.rs` (`read_eyes` maps every point of
  `eyes::judge_mesh`). No Rust change.

## Steps

- [x] Step 1: Draw the face parts outline and the irises instead of the tessellation
  - Done when:
    - With `f` on, a file whose `Eyes` row has a judgment shows over the
      judged face only the contours + nose edges (149) and, when
      `eyes.state === "open"`, the two iris rings (8 edges) and a dot at
      points 468 and 473; when `eyes.state === "closed"`, no iris ring and no
      iris dot, the rest unchanged. Same gates (`showFocus`, `viewOnly`,
      `shown.seq === seq`, cache `undefined` / `null`), same stored-preview
      coordinates, rotation and `drawWidth / previewWidth` scaling as before
      (`meshPoints` stays as is).
    - `crates/app/ui/src/facemesh.ts` no longer holds `FACE_MESH_EDGES`
      (the 2644-value tessellation array is removed, not kept unused). It
      holds the new tables generated from the MediaPipe source, deduped
      undirected `(lower, higher)` edges, sorted, as flat index arrays, with
      the header stating source file, commit, which MediaPipe sets each
      table comes from and the counts (124 + 25 = 149; 8), plus the two iris
      center indices, and a pure function that decides what to draw for a
      judgment state.
    - `crates/app/ui/src/facemesh.test.ts`: each table is well-formed (every
      index an integer, contours / nose edges in `0..468`, iris edges in
      `468..478`, no self-loop, no edge twice in either direction, exactly
      149 and 8 edges; the iris centers are 468 and 473 and appear in no
      iris edge); a pure test that the `closed` state yields no iris
      segments and no dots while `open` yields the 8 iris edges and 2 dots,
      and that both states yield the 149 outline edges; the existing
      `meshPoints` tests stay. `eyes.test.ts` / `meta.test.ts` fixtures need
      no change (the `Eyes` type is unchanged).
    - `crates/core/models/LICENSE-mediapipe`: the provenance paragraph for
      the table names the new sets (`FACEMESH_CONTOURS`, `FACEMESH_NOSE`,
      `FACEMESH_IRISES`), the same commit, and the new counts and
      modification; it no longer says `FACEMESH_TESSELATION` / 1322.
    - Docs wording, each pair in the same PR: `README.md` (line ~126 "also
      gets its face mesh") + `README.ja.md` (line ~57 "顔メッシュも表示します"),
      `docs/humans/usage.md` (line ~202 "that face also gets the face mesh
      the judgment looked at, drawn as thin cyan lines") +
      `docs/humans/usage.ja.md` (line 14 "判定に使った顔メッシュも ... 細いシアンの線で表示します")
      now say the outline of the face parts (face oval, eyes, brows, nose,
      lips) and, when the eyes are judged open, the irises. The "face mesh"
      mentions that describe the Face Landmarker model itself (`README.md`
      line ~141, `README.ja.md` line 59, `usage.md` line ~258, `usage.ja.md`
      line 19) stay. `CLAUDE.md` line ~118: `src/facemesh.ts` "holds
      MediaPipe's face mesh tesselation the focus mark draws over the judged
      face" becomes the parts outline and iris edges it draws; the
      `commands.rs` sentence (line ~62, "returns the face mesh points it
      judged on") still holds, leave it.
    - `todo.md` `### App: real-device check of the face mesh overlay on the judged face`
      (line ~610) is updated in place, not duplicated: its background says
      the tessellation was seen to bury the face and this plan replaced it
      with the parts outline + irises, its Files line and the TODO item name
      the new constants / tables, and the check becomes: (1) upright ARW
      with an open-eyed face of 60 px or more: outline + both iris rings and
      dots sit on the face; (2) a portrait (orientation 6 or 8) one: rotated
      with the image, scales with the window; (3) a file labeled closed
      (`D:\photos\2026\2026-09-19\_DSC1889.ARW` / `_DSC1890.ARW` per the
      closed-eyes todo above it): outline only, no iris; (4) a face below
      60 px: nothing, no error in `Riffle.log`. Record the line widths as
      chosen without a GUI again. The same check is listed as pending in
      `learnings.md`.
    - `mise run ci` passes (`pnpm exec vp fmt` / `check` / `test`, lychee).
  - Implementation approach:
    - Generate, do not hand-type: download the Python file at the pinned
      commit into its own scratchpad directory (not committed), parse the
      `frozenset([...])` literals of the six contour sets, `FACEMESH_NOSE`,
      `FACEMESH_LEFT_IRIS` / `FACEMESH_RIGHT_IRIS` (or import the module
      with `python -I` from outside that directory), map each pair to
      `(min, max)`, dedupe, sort, emit flat `number[]` literals. Expected:
      contours 124, nose 25, union 149, max index 466 / 440; irises 8, all in
      469..=477. Record source, commit, counts and the generator command in
      `learnings.md` as the previous feature did.
    - Table shape in `facemesh.ts`: one `FACE_OUTLINE_EDGES` (contours +
      nose merged, 298 values) and one `FACE_IRIS_EDGES` (16 values), plus
      `FACE_IRIS_CENTERS = [468, 473]`. Keeping contours and nose as one
      array is enough since they are always drawn together; if the
      implementer prefers two arrays for traceability to the MediaPipe
      names, the test counts become 124 / 25 and the header says so.
    - The pure decision, so the closed / open rule is testable without a
      canvas: e.g. `meshEdges(state: Eyes["state"]): { edges: readonly number[]; dots: readonly number[] }`
      (outline edges plus, for `"open"`, the iris edges and the two centers;
      for `"closed"`, the outline only and no dots), or two functions. Keep
      it index-based; `drawFaceMesh` then maps indices through the
      `meshPoints` result and strokes. `Eyes` is imported type-only from
      `./eyes.js`.
    - `main.ts` `drawFaceMesh`: `const eyes = eyesCache.get(files[index]); if (eyes === undefined || eyes === null) return;`
      (today it reads `?.mesh` and checks `undefined`; `null` also has no
      `mesh`, keep that behaviour explicit), then one `beginPath` of
      `moveTo` / `lineTo` for the edges with the outline-then-color passes,
      then, if there are dots, a second path of `arc`s filled like the eye
      dot in `drawFaceMarks` (`FACE_MARK_EYE_RADIUS`, black 0.8 stroke then
      `FACE_MARK_COLOR` fill). Update the function's comment. Colors are
      literal per `docs/agents/ui-styling.md` (canvas), reuse
      `FACE_MARK_COLOR`.
    - Line widths, chosen: with ~157 edges instead of 1322 the lines can be
      heavier than the 1.5 / 0.5 that was picked to thin out the blob, but
      lighter than the 4 / 2 of the face boxes so the box stays the outer
      frame: `FACE_MESH_OUTLINE_WIDTH = 3`, `FACE_MESH_LINE_WIDTH = 1.5`
      (same 2:1 ratio as the boxes). Rename them only if the identifiers
      change meaning (e.g. `FACE_PARTS_*`); update their comment ("~1300
      edges" no longer applies). The real-file check in `todo.md` is where
      they get tuned.
    - Draw order stays box, dot, then the parts (so the iris dots and the
      outline sit above the box); `draw()` already returns early for zoom
      and Compare, and `requestEyes`'s settle branch already calls `draw()`.
    - Remove whatever the change orphans (`FACE_MESH_EDGES` and its import);
      nothing else in the frontend uses it (grep: only `main.ts` and the
      test).

## Trade-offs and risks

- **Iris point reliability.** The irises are drawn only for `open`; a face
  judged open with one closed eye (the EAR is taken on the more closed eye,
  so that is already `closed`) is not a case. A downcast open eye reads as
  `closed` (documented in usage.md), so it gets no iris, which is right
  since the iris is hidden there too.
- **One table or three.** Merging contours + nose into one outline array is
  the smaller code; keeping `FACE_CONTOUR_EDGES` / `FACE_NOSE_EDGES` apart
  mirrors MediaPipe's names in the tests at the cost of one more array and
  concatenation at draw time. The plan prefers one merged outline array
  (they are never drawn separately); the implementer may choose the other
  if it makes the provenance clearer, adjusting the test counts.
- **Line widths.** 3 / 1.5 is a guess made without a GUI, like the previous
  values. The alternative is to keep the boxes' 4 / 2 for one fewer
  constant pair; rejected because two equal-weight cyan outlines (box and
  oval) compete. The real-file todo item is the place to settle it.
- **Iris dot size.** Reusing `FACE_MARK_EYE_RADIUS` (2.5 px) keeps one
  constant; on a large face at fit-to-window the ring is larger than the
  dot, on a 60 px face the ring may collapse onto the dot. Acceptable; a
  dedicated radius is a one-line change later if the check shows it.
- **Docs reach.** `CHANGELOG.md` is generated by release-please; do not edit
  it.

## Progress

- (none yet)
