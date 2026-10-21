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

# Face detection values on the focus mark

## Purpose

While the `f` focus mark is on, every face box it draws (not only the AF
face) gets YuNet's own numbers beside it, permanently: the detection score,
the box size and the distance between the eyes, plus the five YuNet landmarks
(two eyes, nose tip, two mouth corners) as dots. The user wants these on
screen as material for a later decision about which face the analysis should
target when no AF point is recorded (today sharpness takes the best-scoring
face, `chosen_face` in `crates/core/src/sharpness.rs`, and the face analysis
the largest face). Which face is chosen does not change here.

To tell YuNet's output from the face mesh's at a glance, everything YuNet
gives (box, label, five dots) stays in the current cyan (`#3ff`) and the face
mesh drawing (the face parts outline and iris edges from `facemesh.ts`)
switches to magenta (`#f3f`).

## Decisions (by the user, at planning time)

- Show the score, the box size (px), the eye distance (px), the two eye dots
  and the nose / mouth-corner dots.
- Drop the existing dot between the eyes (the midpoint); the two eye dots
  replace it. Remove the `eye` midpoint field from `FaceBox` / `faceMarks` if
  nothing else reads it.
- YuNet in cyan `#3ff`, the face mesh in magenta `#f3f`.
- Single-PR mode (one step).

## Steps

- [x] Step 1: Keep YuNet's nose and mouth landmarks in `Face`, carry every
      face's score and landmarks through `faces_of`, draw the values and the
      five dots beside each face box, and draw the face mesh in magenta
  - Done when:
    - `riffle_core::faces::Face` holds the nose tip and the two mouth corners
      next to `left_eye` / `right_eye`, decoded from `kps[i*10+4..i*10+10]`
      (YuNet's keypoint order is right eye, left eye, nose tip, right mouth
      corner, left mouth corner, in image terms; keep the existing "left of
      the image first" swap for the eyes and order the mouth corners the same
      way, left of the image first), and every coordinate mapper carries them
      (`detect_at`'s `/ scale`, `translate`, `to_stored`, `face_to_upright`,
      `scaled_to_stored`).
    - Unit tests in `faces.rs` cover the new points:
      `decodes_a_cell_into_input_coordinates` fills all 10 keypoint values and
      asserts the nose and mouth positions;
      `maps_an_upright_face_back_to_the_stored_image`,
      `face_to_upright_is_the_inverse_of_to_stored` and
      `a_face_on_a_scaled_decode_maps_back_to_the_stored_preview` assert them
      alongside the eyes.
    - `faces_of` (`FaceBox` in `crates/app/src/commands.rs`) also returns
      `score`, `left_eye`, `right_eye`, `nose`, `mouth_left`, `mouth_right`
      (each a `FacePoint`), in the same stored preview px as the box; the
      `eye` midpoint field is removed if nothing else reads it.
    - With the focus mark on, each detected face's box has a label beside it
      with the score (two decimals, `0.93`), the box size in stored preview
      px (`212×254`, integers) and the eye distance in stored preview px
      (`eyes 61`, integer), and the five landmark dots are drawn in cyan. The
      midpoint dot between the eyes is no longer drawn. The label reads
      upright on portrait (orientation 6 / 8) and half-turned (3) frames and
      stays inside the drawn image.
    - The face mesh (parts outline, iris edges) is drawn in magenta `#f3f`;
      the YuNet box, label and dots stay cyan `#3ff`.
    - The 1:1 view and Compare still draw no faces (unchanged).
    - New pure frontend logic (label text, upright placement) has unit tests
      in `crates/app/ui/src/focus.test.ts`; `faceMarks`'s tests cover the
      new points.
    - No stored or computed value changes: `EXTRACTOR_VERSION` is not bumped
      (the index never stores the landmarks and no consumer reads them).
    - CLAUDE.md's Layout entry for `src/faces.rs`, `faces_of` and `focus.ts`
      mentions the five landmarks and the label; README.md / README.ja.md and
      `docs/humans/usage.md` / `usage.ja.md` describe the box's label, the
      dots and the mesh's magenta where they describe the cyan box and the
      dot between the eyes today.
    - `meta.ts` is not touched.
    - `mise run ci` passes.
  - Implementation approach:
    - Core (`crates/core/src/faces.rs`): add `nose: (f32, f32)`,
      `mouth_left: (f32, f32)`, `mouth_right: (f32, f32)` to `Face` (plain
      fields, like the eyes; no new struct). `Face` has no `Default`, so the
      test literals outside `faces.rs` (`crates/cli/src/main.rs`,
      `crates/core/src/candidate.rs`, `crates/core/src/eyes.rs`,
      `crates/core/src/scan.rs`, `crates/core/src/sharpness.rs`) and the
      ones in `faces.rs`'s tests need the fields; fill them with `(0.0, 0.0)`
      or a value next to the eyes. Each file's test helper (`face(...)`,
      `upright()`, `scored(...)`) absorbs most of them. Do not touch what
      those tests assert.
    - `commands.rs`: add the new fields as `FacePoint`s; drop the `eye`
      midpoint once `main.ts` / `focus.ts` no longer read it. Update the doc
      comment on `FacesResponse` / `read_faces`.
    - Frontend (`crates/app/ui/src/focus.ts`): extend the `Faces` interface
      and `FaceMark` (`faceMarks` scales the new points like `eye` did). Add
      pure, tested helpers: `faceLabel(face)` returning the label text
      (score `toFixed(2)`, `Math.round` of width / height / eye distance,
      the eye distance from `left_eye` / `right_eye` in preview px, not
      drawn px), and an upright placement helper. `draw()` in `main.ts`
      rotates the context for orientation 3 / 6 / 8 before `drawFaceMarks`,
      so text drawn there would be rotated. Draw the labels in a separate
      `drawFaceLabels(drawWidth, drawHeight, orientation)` called after the
      rotated block (its own `save` / `translate(center)` / `scale(dpr)`,
      no rotation), mapping each mark's rect, which is centered on the
      origin, to upright coordinates with a pure rotation
      (6: `(x, y) -> (-y, x)`, 8: `(x, y) -> (y, -x)`, 3: `(x, y) -> (-x, -y)`,
      the inverse of `faces.rs`'s `to_stored`), then anchoring the label
      above the upright box's top-left and dropping it just inside the box
      when the box touches the drawn image's top. Test the rotation and the
      flip on all four orientations.
    - Dots: in `drawFaceMarks`, draw the five landmarks with the same
      outline-then-fill passes and `FACE_MARK_EYE_RADIUS` the midpoint dot
      used, and remove the midpoint dot.
    - Mesh color: add a `FACE_MESH_COLOR = "#f3f"` next to `FACE_MARK_COLOR`
      in `main.ts` and use it for the face mesh drawing (the parts outline
      and iris edges), keeping the black halo pass.
    - Styling (`docs/agents/ui-styling.md`): text in `FACE_MARK_COLOR` with
      the `rgba(0, 0, 0, 0.8)` halo via `strokeText` then `fillText`, font
      `11px system-ui, sans-serif` as the other canvas text uses; no
      `margin`. If the guide lists the canvas mark colors as exceptions, add
      `#f3f` there next to `#3ff`. Run the guide's color greps and layout
      greps before the PR.
    - Why stored preview px: the box and the landmarks come back from
      `faces_of` in the stored preview's coordinates (`Detection.width` /
      `height` is the stored preview size), which is also what
      `sharpness::FACE_CONFIDENCE`, the 24 px eye region floor and the
      "about 60 px" face floor are measured in, so the numbers compare
      directly to the thresholds; original sensor px would need a scale the
      response does not carry. Say this in the label's code comment and in
      the usage docs ("in preview pixels").
    - Docs: README.md / README.ja.md ("cyan box with a dot between the
      eyes"), `docs/humans/usage.md` and the matching sentence in
      `usage.ja.md`; CLAUDE.md's `faces.rs` description ("the YuNet face/eye
      detector") and the `faces_of` and `focus.ts` entries.

## Trade-offs and risks

- **Label collisions.** Two faces close together (a group photo) will have
  overlapping labels. Out of scope to lay them out; the label is debugging
  material.
- **Face struct shape.** Three tuple fields were chosen over a
  `landmarks: [(f32, f32); 5]` array to keep `left_eye` / `right_eye` as the
  consumers use them (`candidate.rs`, `eyes.rs`, `sharpness.rs`, `scan.rs`'s
  `eye_offset`) and avoid touching their reads.
- **No `EXTRACTOR_VERSION` bump.** Only `faces_of` (on demand, not indexed)
  exposes the landmarks; `run_faces_scan` stores nothing from them. Confirm
  during implementation by grepping the consumers of `Face` for any use
  beyond `x`/`y`/`width`/`height`/`score`/eyes.

## Progress

- Step 1 done: `Face` keeps YuNet's nose and mouth landmarks, `faces_of` carries
  every face's score and landmarks, and the focus mark draws the values beside
  each box (the label's size swaps width and height on a quarter turn so it
  reads as drawn upright). The manual Windows check recorded in `learnings.md`
  is still pending.
- (2026-10-22) Step 1 complete
