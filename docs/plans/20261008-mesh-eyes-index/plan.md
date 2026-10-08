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

# Store the mesh eye state and head pose in the index

## Purpose

The scan's second pass already runs the face mesh on the face nearest a
trusted AF point (`candidate::focus_cue_unless` -> `scored_face` ->
`eyes::mesh_of`, since `mesh-eye-focus`), and `Mesh` carries the 478 points
and the head pose. Yet the meta pane's `Eyes open` and `Head pose` rows still
come from the on-demand `eyes_of` command, which runs the same model again
for the shown file only and stores nothing, so the strip cannot filter on
closed eyes. This work stores the EAR of the more closed eye (the closed-eye
judgment's input) and the yaw / pitch / roll of that face in the index from
pass 2, at no extra model run, lets the meta pane read them, and adds an
`Eyes` section (open / closed / unknown) to the strip's filter menu. The pose
is stored only; a looking-away filter needs thresholds that are not labeled
yet and is out of scope.

Scope decided with the user:

- Store only for the face pass 2 already meshes (a trusted AF point and a
  face at or above `eyes::EYES_MIN_FACE`, the `candidate::meshes_face` gate
  merged in #737). No mesh run is added for files without an AF point, so
  pass 2 time is essentially unchanged.
- The stored value must agree with `eyes::judge_mesh` on the same face: same
  crop (`face_square`), same floor (60 px, now the gate), the EAR on the
  points `mesh_of` already produced.
- Keep `eyes_of`: it still draws the mesh overlay and is the fallback for a
  file with no stored value (no AF point -> the largest confident face).
- Do not change the focus cue's coefficients / thresholds / margins /
  `EYE_REGION_MIN`, the closed-eye constants (`EYES_CLOSED_EAR`,
  `EYES_LOGIT_SLOPE`), or the face crop. Do not pick a looking-away
  threshold.

Start from `origin/main`: that commit (#737, `candidate::meshes_face`) is
what gives the stored value its 60 px floor.

## Steps

- [x] Step 1: Carry the EAR and the pose out of the cue and store them in the index (Rust: core, index, events)
  - Done when:
    - `candidate::Cue` carries the EAR of the more closed eye and the pose of
      the meshed face (both `None` without a mesh), taken from the
      `Mesh` the cue already has, with no second model run and no change to
      the cue's numbers (`eye_focus` / `state` identical before and after on
      the existing tests).
    - A unit test shows the EAR on `Mesh.points` (stored 2D coordinates)
      equals `eyes::more_closed_ear` on the upright 3D points for
      orientations 1, 3, 6 and 8 (synthetic points through
      `faces::point_to_stored`), and the `#[ignore]`d
      `judges_a_face_in_a_real_image` test in `crates/core/src/eyes.rs` also
      asserts the stored-path EAR equals `judge_mesh`'s `Eyes::from_ear`
      input on the same face. A test shows a face under `EYES_MIN_FACE`
      yields `None` EAR and pose from the cue (through the existing gate, not
      a new floor).
    - `SCHEMA_VERSION` is 18 with four nullable `REAL` columns on `files`
      next to `eye_focus` (the EAR, yaw, pitch, roll; `NULL` = none),
      migrated in place with `ALTER TABLE ADD COLUMN` for v10..v17 databases
      (and the doc comment on `SCHEMA_VERSION` extended in the existing
      style). A test opens a v17-shaped database and checks the rows survive
      and the columns exist.
    - `FACES_VERSION` is 7 (doc comment: `7` re-runs pass 2 to fill the new
      columns), so an existing folder re-runs pass 2 only; a test shows a row
      at `faces_extractor = 6` is listed by `faces_todo` and its thumbnail /
      metadata are untouched. `faces_todo`'s reset for JPEG / error rows also
      NULLs the new columns.
    - `write_faces` stores the four values, `indexed_file` reads them,
      `Focus` and `FaceReady` carry them plus derived fields, and the
      `faces-progress` event payload includes them (a serialization test, as
      the existing `Focus` / `FaceReady` tests do).
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/core/src/eyes.rs`: make the EAR usable on 2D points without a
      second copy of the geometry, e.g. `ears` / `more_closed_ear` generic
      over `P: AsRef<[f32]>` so both `[f32; 3]` and `[f32; 2]` slices work
      (`ears` only reads `[0]` and `[1]`). Keep the docs noting z is unused
      so the stored-coordinate EAR is the upright one.
    - `crates/core/src/candidate.rs` (keep the diff small; #737 touched
      `scored_face` / `focus_cue_unless`): add `ear: Option<f64>` to
      `EyeMeasures` (computed in `mesh_eye_measures` from `mesh.points`, next
      to the `pose` it already copies), and two fields on `Cue`, e.g.
      `eyes_ear: Option<f64>` and `pose: Option<Pose>`, filled in
      `focus_cue_unless` from `focus.and_then(|f| f.mesh)`. Update the
      `Cue { .. }` literal in `crates/cli/src/main.rs` tests
      (around line 1668) and anything else that builds a `Cue`. Adding the
      EAR / pose columns to `riffle-cli candidates`' dump is optional; not
      required.
    - `crates/app/src/index.rs`: column names of the `eye_focus` family,
      e.g. `eyes_ear REAL, pose_yaw REAL, pose_pitch REAL, pose_roll REAL`;
      a migration block `if (10..18).contains(&version)` following the
      v13 / v15 blocks; `write_faces` grows from a 3-tuple to a row struct
      (or tuple) with the four values; `INDEXED_FILE` adds the columns
      after `eye_focus` (column indices shift only for the new ones).
    - Derive the eye state in Rust like `candidate(eye_focus)`: `Focus` and
      `FaceReady` get `eyes_ear: Option<f64>`, a serialized
      `eyes: "open" | "closed" | "unknown"` derived from the EAR with
      `eyes::Eyes::from_ear` (`None` -> `unknown`), `eyes_closed:
      Option<f64>` (the closed probability, `closed_probability(ear)`, so the
      frontend needs none of the constants), and `pose: Option<{yaw, pitch,
      roll}>` (serialize `pose::Pose`; a serde derive on it or a small
      serializable struct like `commands::EyesPose`, which could move to
      `index.rs` or be shared).
    - `run_faces_scan`'s `flush` maps `analysis.cue.eyes_ear` / `cue.pose`
      into the rows and the `FaceReady`s.
    - Docs in this step: `docs/agents/tauri-app.md` (the `FACES_VERSION`
      bullet: the columns pass 2 fills now include the EAR and the pose),
      `CLAUDE.md` layout text for `index.rs` (the columns pass 2 fills).
- [x] Step 2: Read the stored values in the meta pane and add the `Eyes` filter (frontend and user docs)
  - Done when:
    - `Focus` in `main.ts` and `FaceReady` in `focus.ts` carry the new
      fields; `applyFaceReady` patches them in place as it does `eye_focus`.
    - `metaGroups` shows `Eyes open` from the stored closed probability
      (`1 - eyes_closed`) and `Head pose` from the stored pose when present,
      and falls back to the `eyes_of` judgment (`eyes` argument) otherwise;
      `meta.test.ts` covers stored-only, fallback-only, both (stored wins)
      and neither. The mesh overlay keeps working: `requestEyes` and
      `drawFaceMesh` are untouched.
    - `filter.ts` has an `eyes` set in `FilterState` with values `open` /
      `closed` / `unknown`, `passes` takes the file's stored `eyes` state
      (`unknown` when there is no focus or no value, like `candidate`), and
      `filter.test.ts` covers the three values, nothing checked, OR within
      the group and AND with the other groups.
    - `index.html` has an `Eyes` section (separator, label, `Open` /
      `Closed` / `Unknown` `menuitemcheckbox` items with `data-eyes`) after
      the `AF eye` section, wired in `main.ts` like `data-candidate`
      (`filterItems` selector, `filterChanged`, the click handler,
      `filterActive`, the reset, and the `faces-progress` refilter when the
      eyes set is non-empty, as the candidate filter fills in while pass 2
      runs).
    - `README.md` / `README.ja.md` describe the `Eyes` filter and that
      `Eyes open` / `Head pose` come from the scan for AF faces (judged when
      shown otherwise); `docs/humans/performance.md` /
      `performance.ja.md` "Closed-eyes judgment on demand" and the summary
      table row say the EAR and the pose are stored by pass 2 for the AF
      face and `eyes_of` stays for the overlay and the no-AF fallback;
      `CLAUDE.md` layout text for `meta.ts` and `filter.ts` mentions the
      stored values and the eyes section.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (the event and the `folder_entries` rows
      carry the fields).
    - Keep `meta.ts` free of the closed-eye constants: the state and the
      closed probability arrive derived from Rust. `FocusCue` in `meta.ts`
      grows with `eyes_closed: number | null` and `pose: Pose | null`
      (reuse the `Pose` type from `eyes.ts`).
    - Match the existing filter plumbing exactly (a `shownEyes` set next to
      `shownCandidates`; the `data-*` attribute dispatch chain). The
      `companion.ts` view state and the MCP tools are not changed.

## Trade-offs and risks

- **What to store: the EAR or the closed probability.** The plan stores the
  EAR (the raw measure) and derives the probability and the state in Rust,
  mirroring `eye_focus` -> `candidate()`. If `EYES_CLOSED_EAR` /
  `EYES_LOGIT_SLOPE` are ever refitted, stored EARs stay valid and only the
  derivation changes (no `FACES_VERSION` bump).
- **Pose stored but unused by the filter.** Yaw / pitch / roll are stored
  and shown only; a looking-away filter is left out until the thresholds are
  labeled (the open todo item).
- **`eyes_of` still runs for every shown file**, including files whose
  value is now stored, because the overlay needs the mesh points. Skipping
  it is the separate "reuse the scan's mesh in `eyes_of`" todo item.
- **Agreement of the stored and the on-demand value.** `ears` uses x and y
  only, and `mesh_of` and `judge_mesh` share the crop and the model input,
  so they agree up to float rounding of `point_to_stored`; the planned unit
  test and the extended ignored real-image test pin this. A measurable
  difference goes in `learnings.md` rather than a silent tolerance.
- **A file with an AF point but no stored value** (face under 60 px, no
  face near the point, mesh failure) shows the `eyes_of` fallback in the
  meta pane but counts as `unknown` in the filter, consistent with the
  `AF eye` filter's `unknown`.
- **Migration.** v18 is purely additive (`ALTER TABLE ADD COLUMN`); the
  `FACES_VERSION` bump re-runs pass 2 on every indexed folder once (about
  21 s on the 2134-ARW folder at 24 threads per `performance.md`), the
  agreed cost.

## Progress

- (2026-10-09) Step 1 complete (SCHEMA_VERSION 18, FACES_VERSION 7; the EAR and pose carried on Cue and stored by pass 2; see learnings.md)
