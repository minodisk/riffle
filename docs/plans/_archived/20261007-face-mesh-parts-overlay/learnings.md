# Learnings: face-mesh-parts-overlay

## Step 1: Draw the face parts outline and the irises

- Tables generated, not hand-typed. Source:
  `https://raw.githubusercontent.com/google-ai-edge/mediapipe/212f110c65818db7f2d08c42a9539ecb2b6c0e1d/mediapipe/python/solutions/face_mesh_connections.py`,
  downloaded to its own scratchpad directory only (not committed). The
  generator (in a separate scratchpad directory, run as
  `python -I gen.py <path to face_mesh_connections.py>`) reads the
  `frozenset([...])` literals with `ast` (no import of the downloaded module),
  maps each pair to `(min, max)`, dedupes, sorts and prints flat lists.
- Counts as expected by the plan: contours (`FACEMESH_LIPS`,
  `FACEMESH_LEFT_EYE`, `FACEMESH_LEFT_EYEBROW`, `FACEMESH_RIGHT_EYE`,
  `FACEMESH_RIGHT_EYEBROW`, `FACEMESH_FACE_OVAL`) 124 directed / 124
  undirected, max index 466; `FACEMESH_NOSE` 25 / 25, indices 1..=440; no
  shared edge, union 149; irises 8 / 8, indices 469..=477.
- Chose the plan's preferred shape: one merged `FACE_OUTLINE_EDGES` (298
  values), `FACE_IRIS_EDGES` (16 values), `FACE_IRIS_CENTERS = [468, 473]`,
  and `meshEdges(state)` returning `{ edges, dots }`. Line widths set to 3 /
  1.5 as the plan chose; the constant names were kept since they still mean
  the judged face's mesh lines.
- Tripped up: a Windows path with backslashes written through a Python
  string inside a Bash heredoc came out mangled (`\202` read as an octal
  escape somewhere on the way), so `todo.md` got a control character until
  it was rebuilt with `chr(92)`. Check paths with backslashes after a
  scripted edit.

## Deferred issues (todo candidates)

- **Pending manual check** (Step 1 was ticked on the automated criteria; no
  GUI here). Updated in place as the existing `todo.md` item
  `### App: real-device check of the face mesh overlay on the judged face`.
  On Windows or macOS, with `f` on: (1) an upright ARW with an open-eyed face
  of 60 px or more shows the cyan face parts outline plus both iris rings and
  center dots on the judged face, a moment after the `Eyes` row; (2) a
  portrait ARW (orientation 6 or 8) shows the same rotated with the image and
  scaling with the window; (3) `D:\photos\2026\2026-09-19\_DSC1889.ARW` /
  `_DSC1890.ARW` (labeled closed) show the outline only, no iris ring or dot;
  (4) a face below 60 px shows nothing and logs no error in `Riffle.log`.
  Line widths (3 / 1.5) and the iris dot radius (`FACE_MARK_EYE_RADIUS`)
  were chosen without a GUI and get tuned there. Files:
  `crates/app/ui/src/main.ts` (`drawFaceMesh`), `crates/app/ui/src/facemesh.ts`.
