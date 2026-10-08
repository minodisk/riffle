# Learnings: mesh-eyes-index

## Step 1

- `eyes::more_closed_ear` / `ears` are generic over `P: AsRef<[f32]>`, so the
  scan computes the EAR on `Mesh.points` (`[f32; 2]`, stored coordinates)
  with the same code `judge_mesh` runs on the upright `[f32; 3]` points. No
  second copy of the geometry.
- Agreement of the stored-path EAR and `judge_mesh`'s: the synthetic test
  (orientations 1, 3, 6, 8 through `point_to_stored`, coordinates in the
  thousands of pixels) and the `#[ignore]`d real-image test both pass under
  a `1e-5` tolerance. Not asserted bit-exact: `point_to_stored` computes
  `h - x` / `w - y` in `f32`, which can round. On
  `D:\Photos\tests\2026-10-06-face-recall\step2\face.jpg` the real-image test
  passed (EAR 0.3895, orientation 6 stored path) with no measurable
  difference at that tolerance.
- The cue's assembly moved into a small `candidate::cue_of(face, detection,
  focus)` so the EAR / pose plumbing (and the under-the-floor `None`) can be
  tested without decoding a JPEG with a detectable face. `focus_cue_unless`
  is otherwise unchanged.
- The boxed test meshes in `candidate.rs` (`both_eyes`, `boxed_eyes`) put
  two of the six EAR points of an eye on the same corner, so their EAR is
  infinite and `more_closed_ear` gives `None`; the EAR test moves point 133
  to make it finite.
- `commands::EyesPose` (from #739) moved to `index::EyesPose` and is shared:
  `eyes_of`'s judgment, `Focus` and `FaceReady` serialize the same
  `{yaw, pitch, roll}` shape. The stored eye fields live in one
  `index::StoredEyes` flattened (`#[serde(flatten)]`) into both `Focus` and
  `FaceReady`, so the JSON carries `eyes_ear`, `eyes` (`open` / `closed` /
  `unknown`), `eyes_closed` and `pose` at the top level as the plan asks.
- The new columns sit after `faces_extractor` in `CREATE TABLE`, not right
  after `eye_focus`: `ALTER TABLE ADD COLUMN` always appends, so this keeps
  fresh and migrated databases in the same column order.
- Every migration test fixture from v10 to v16 now also drops the four new
  columns before setting its old `user_version`, else the v18 `ALTER TABLE
  ADD COLUMN` fails on a duplicate column. The `assert_eq!(version, 17)`
  lines became `assert_eq!(version, SCHEMA_VERSION)`.

## Step 2

- The stored eye fields are one `StoredEyes` interface (plus the `EyeState`
  type) in `eyes.ts`, next to `Pose`, mirroring `index::StoredEyes`;
  `MarkFocus` and `FaceReady` in `focus.ts` extend it, so `applyFaceReady`
  patches the four fields with no second shape to keep in sync. `Focus` in
  `main.ts` lists them explicitly like its other fields.
- `metaGroups` treats the stored value as one unit: when `eyes_closed` is
  stored, both `Eyes open` and `Head pose` come from the stored values (a
  stored `null` pose stays out rather than mixing in `eyes_of`'s pose of
  possibly another face); otherwise both come from the `eyes` judgment.
- `docs/humans/usage.md` / `usage.ja.md` also described `Eyes open` as
  "judged when the file is shown, not in the scan" and listed the filter
  menu's sections, so they were updated with README and performance.md,
  although the plan's Done-when names only those.
