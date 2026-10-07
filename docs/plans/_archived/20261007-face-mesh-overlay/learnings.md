# Learnings: face-mesh-overlay

## Step 1

- `eyes::ear_of` split into `landmarks_of` (crop, model, `to_full` mapping)
  and `more_closed_ear`; `judge` is now `judge_mesh` minus the points. The
  `catch_unwind` wraps `landmarks_of` only: `ears` indexes fixed points below
  `LANDMARKS` and cannot panic, so the guarantees of `judge` hold.
- `faces::point_to_stored` takes the stored size, like `to_stored`; in
  `read_eyes` that is the `(w, h)` from `decode_rgb`, not `upright_rgb`'s
  `(uw, uh)`.
- The response nests the points as `eyes.mesh { width, height, points }`
  (`EyesMesh` inside `EyesJudgment`), the first shape Step 2's plan offers
  for the frontend `Eyes` type.
- The ignored `times_eyes_of_on_real_files` printed `r.eyes` with `{:?}`;
  it now prints only `(state, probability)` so 478 points do not flood each
  line.

## Step 2

- The table came from
  `https://raw.githubusercontent.com/google-ai-edge/mediapipe/master/mediapipe/python/solutions/face_mesh_connections.py`,
  whose last commit touching the file is
  `212f110c65818db7f2d08c42a9539ecb2b6c0e1d` (2023-04-27). Parsing
  `FACEMESH_TESSELATION`: 2556 directed pairs, 1322 unique as
  `(min, max)`, highest index 467, no self-loop, matching the plan. The
  Python file was downloaded to the scratchpad only; the generator emitted a
  flat `number[]` of 2644 values into `crates/app/ui/src/facemesh.ts`.
- `node` is not on the Git Bash PATH here; `mise exec -- node` works.
- The mapping helper is `meshPoints` in `facemesh.ts` (same scale-and-center
  rule as `faceMarks`), and `drawFaceMesh` in `main.ts` runs right after
  `drawFaceMarks` with the same gates; `draw()` already returns early for
  zoom and Compare, and a `null` / `undefined` cache entry has no `mesh`.
- Line widths picked without a GUI: outline 1.5 px at 0.8 black, then 0.5 px
  `FACE_MARK_COLOR` (`FACE_MESH_OUTLINE_WIDTH` / `FACE_MESH_LINE_WIDTH`).

## Deferred issues (todo candidates)

- Pending manual check (Step 1, from plan.md Step 1 "Done when", the
  `eyes` timing line): on a real machine (any platform), open a folder of RAW
  files with a face of 60 px or more, turn on the debug log, show such a file
  and read the `eyes total=... ipc=...` line in `Riffle.log`. Expected: the
  stages still add up to about `total`, and `ipc=` stays within a few ms of
  what it was before this change (478 points, ~8-10 KB of JSON). The Step 1
  checkbox was ticked on the automated criteria; this IPC figure was not
  measured here. Files: `crates/app/src/commands.rs` (`read_eyes`),
  `crates/app/ui/src/main.ts` (`requestEyes`).
- Pending manual check (Step 2, from plan.md Step 2 "Done when", the
  real-file check): on a real machine (any platform), with `f` on, show (1)
  an upright ARW with a face of 60 px or more, (2) a portrait ARW
  (orientation 6 or 8) with such a face, and (3) a file whose only face is
  below 60 px on the preview. Expected: in (1) and (2) the cyan tessellation
  sits on the judged face a moment after the `Eyes` row appears, rotated with
  the image and scaling with the window, and stays readable (not a solid
  cyan blob); in (3) no mesh and no error in `Riffle.log`. If the mesh buries
  the face, tune `FACE_MESH_OUTLINE_WIDTH` / `FACE_MESH_LINE_WIDTH` or fall
  back to `FACEMESH_CONTOURS` (plan's Trade-offs). The Step 2 checkbox was
  ticked on the automated criteria. Files: `crates/app/ui/src/main.ts`
  (`drawFaceMesh`), `crates/app/ui/src/facemesh.ts`.
