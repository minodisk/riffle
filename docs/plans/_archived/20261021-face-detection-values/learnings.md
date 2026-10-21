# Learnings

## Step 1

- `Face` has no `Default`, so every literal without a `..base` spread needed
  the three new fields: 6 in `crates/cli/src/main.rs`, 2 in `eyes.rs`, one
  each in `candidate.rs`, `scan.rs` and `sharpness.rs`, plus the ones in
  `faces.rs`'s tests. The literals built with `..upright()` / `..face(...)`
  needed nothing.
- `to_stored` and `face_to_upright` map each landmark without swapping the
  left / right pair after a turn, as they already did for the eyes; the
  "left of the image first" order holds on the upright image YuNet ran on.
- The only reader of the removed `eye` midpoint besides `main.ts` was the
  `found` fixture in `crates/app/ui/src/faces.test.ts`, which got the new
  fields.
- The face labels are drawn by `drawFaceLabels` after `draw()` restores the
  rotated context; `faceLabelAt` maps the mark's rect with `uprightPoint`
  (the same rotation `context.rotate` applies) and clamps the label inside
  the upright drawn image horizontally, given the measured text width.

## Deferred issues (todo candidates)

- Pending manual check (the step's checkbox was ticked on the automated
  criteria): on Windows, open a folder with faces in landscape and portrait
  (orientation 6 / 8) frames and, if available, a half-turned (3) one, press
  `f`, and confirm that each face box has a cyan label like
  `0.93  212×254  eyes 61` reading upright above the box (just inside it
  when the box touches the image's top), five cyan dots on the eyes, nose
  tip and mouth corners, no dot between the eyes, and the face mesh outline
  and irises in magenta. The 1:1 view (`z`) and Compare should draw no faces.
  Basis: plan Step 1 Done-when (drawing on a real preview cannot be checked
  in CI). Files: `crates/app/ui/src/main.ts`, `crates/app/ui/src/focus.ts`.
