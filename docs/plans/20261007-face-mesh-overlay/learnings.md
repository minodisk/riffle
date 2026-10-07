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
