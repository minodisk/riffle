# Learnings: mesh-roll

## Step 1: the opt-in rotated mesh and `riffle-cli meshfit`

- **burst-keep-score Step 5 merged before Step 1 started** (#752, 2026-10-10:
  `SCHEMA_VERSION` 19 / `FACES_VERSION` 8 on `main`). So the CLI reads
  `candidate::mesh_eye_offset` directly instead of reproducing `eye_offset`
  locally as the plan first said, and the merge half of the gate before
  Step 3 already holds; only the user's approval of the Decision remains.
  `plan.md`'s Purpose, "What is known", Step 1, the gate and the ordering
  risk were reworded to match.
- **The rotated crop** (`eyes::rotated_crop`): the window is the one
  `crop_rgb` / `mesh_of` already cut (`window_at` of `face_square`, clamped
  inside the image), turned about **the window's center**, not the face
  center, so a roll of 0 is the existing crop pixel for pixel (pinned by a
  test). The two differ only for a face whose window is clamped at an image
  edge. Sampling is **bilinear**, a sample outside the image takes the
  **nearest edge pixel** (edge clamp, no black wedges). At roll 0 every
  sample lands on a pixel center with exact f32 arithmetic, so the bilinear
  weights are exactly 0 / 1 and the bytes match.
- **One deviation from the Done-when**: `meshfit` runs both meshes on every
  judged face, also under `EYES_MIN_FACE`, because the closed-eyes and head
  pose truth sets were measured over every labeled face (`riffle-cli eyes`
  prints them below the floor too). The cue columns still score the mesh
  only when `meshes_face` accepts the face, as the scan does, so the
  focus-sample numbers stay reproducible; filter on `side >= 60` for the
  scan's view.
- **The `before` columns reproduce Step 5's dump exactly**: over the six
  folders, all 7261 AF frames with a mesh `eye_offset` in
  `step5\dump\*.tsv` have the same value in `before_eye_offset` (within
  0.001). `meshfit` counts 7267 such frames (AF, side >= 60, an offset).
  `_DSC2638` reads `before` EAR 0.3601 / roll 11.8 / offset 0.1247 as in
  the plan, `after` EAR 0.2602 / roll 5.1 / offset 0.0918, `yunet_roll`
  -10.7.
- **The dump** (`D:\Photos\tests\2026-10-09-mesh-roll\dump\`, made by
  `dump.sh` next to it with the release `riffle-cli.exe` kept there): one
  `<set>.tsv`, `<set>.log` (the stderr totals and the wall time) and
  `<set>\` of overlays per set: the six folders, `2026-02-01` (the DNG
  half of the eyes / pose truth sets; `2026-09-19` is the other half), the
  nine `*-focus-sample` folders and `good-mark-2026-10-09` (180 RAW files in
  the folder now, not 60). 0 errors in any set.
- **The `after` mesh never failed**: 0 of 8915 faces in the six folders,
  0 of 1058 in the labeled sets (nor did `before`).
- **Mesh time before / after** (per call, 24 rayon threads busy at once, so
  inflated by contention; release build): six folders, 8915 faces, mean
  70.4 ms before and 71.3 ms after (+1.3%; per folder from 60.8 / 61.3 on
  `2026-09-27-a` to 85.1 / 89.6 on `2026-07-11`). On the 100-file labeled
  folders `after` reads 3-8% *faster*, which is the order (`before` runs
  first on each file) rather than the sampler. The rotation's cost is in
  the noise; Step 3 measures it again at 24 threads on `2026-09-19`.
- **A first look for Step 2 (not the analysis):** over those 7267 frames
  the share over 0.10 is 28.7% before and 28.7% after (2083 / 2082); on the
  657 with |`yunet_roll`| >= 15 it goes from 479 to 506. The overlays show
  why on the large rolls: on profile and far-turned faces YuNet's two eye
  points sit together on the visible eye or the nose (`2026-09-19__DSC2251`,
  `yunet_roll` -53, offset 0.096 -> 0.331), so the angle is bogus and the
  rotation makes a fitting mesh worse, while a frontal tilted face improves
  (`2026-09-27-a__DSC5737`, +34, 0.183 -> 0.059). On the frames whose
  `before` mesh is on (offset under 0.05), `yunet_roll` and the mesh's roll
  correlate only at 0.29. This is the guard the plan's trade-offs name;
  Step 2's by-yaw split and labels decide it.
- Writing the CLI test through a Python heredoc turned its `\t` escapes
  into literal tabs (the Git Bash heredoc plus Python string escapes);
  `perl -i` silently did nothing on Windows. A small Python script that
  replaces tabs on the affected lines fixed it.
