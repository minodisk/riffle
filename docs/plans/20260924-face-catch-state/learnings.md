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
