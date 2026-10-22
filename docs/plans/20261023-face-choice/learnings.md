# Learnings: face-choice

## Step 1: pre-registration

- The scan's cue lives in `crates/core/src/scan.rs` (`riffle_core::scan`),
  not in `crates/app`; the plan's `scan::extract_analysis` reference is to
  that module.
- "Missing last" in the archived reads means the lowest score: `-inf` in
  the per-folder cut (so a missing frame is removed first, which is why the
  stored `eye_focus`'s `PL` at q5, 4.44%, is close to random's) and the
  folder's lowest finite score minus 1 in the AUC. The pre-registration
  states it that way rather than as "sorted after every value".
- Choices the plan text left open, fixed in `facechoice.md`: the 0.8 floor
  applies to `largest`, `sharpest`, `score` and `mean` and to the no-AF
  fallback, while `af-whole` takes `nearest_face` over the whole set at the
  detector's 0.6 (the crop set's floor, so only the detection differs from
  `af`); `score` needs its top face at or above 0.8 (the face `chosen_face`
  would take); ties follow `max_by` (later face) and `min_by` (earlier
  face); the disagreement subset needs a face from both rules, frames with
  exactly one counted and left out; the multi-face subset is 2+ whole faces
  at or above 0.8; the share each rule leaves missing is printed with the
  face counts (a reading).
- The manifests print `eye_focus` to 6 significant digits, so the `af`
  reproduction against them is pre-registered at |difference| <= 1e-6 (the
  1e-9 tolerance is Step 2's check against `scan::extract_analysis`).
- Risk for Steps 2–3: the dump's stored `eye_focus` dates from 2026-10-12;
  `git log` shows no change to `candidate.rs`, `faces.rs`, `eyes.rs` or the
  cue path of `scan.rs` since then that should alter the value (#867 only
  carries every face's values to the UI), but if `af` does not reproduce,
  the read stops before any label is opened.

## Step 2: `riffle-cli facechoice` and the per-face dump

- The names the plan listed all exist on `main` as written:
  `reader::read_preview`, `sharpness::trusted_focus`, `decode::decode_rgb`,
  `faces::detect_around_rgb`, `faces::decode_whole` +
  `faces::detect_whole_upright`, `candidate::{luma, meshes_face,
  eye_focus, nearest_face}`, `eyes::{mesh_of, judged_face}` and
  `scan::extract_analysis(path).cue.eye_focus`. The subcommand lives in
  `crates/cli/src/facechoice.rs` and reuses `main.rs`'s private
  `raw_paths` and `summary` from the child module.
- Choices the plan text left open:
  - `af_x` / `af_y` are the crop detection's `Detection::point` (stored
    coordinates, `partial::focus_point` of the AF point); the whole set's
    `nearest` is taken to that same point, and without an AF point
    `nearest` is 0 on every row.
  - A frame without an AF point still gets its one crop row with `face`
    `-` (the crop set is empty there), so every file has at least two rows.
  - `scored` collapses `Sharper(e)` / `Only(e)` to the eye's side (`L` /
    `R`), `window` for the eye window. `mesh_ms` is `-` when
    `meshes_face` turns the face down (no mesh; `ear` and the pose are `-`
    too); a mesh that ran and failed has a time but `-` for `ear` / pose.
  - Floats are Rust's `Display`, the shortest string that round-trips: the
    box, score and landmarks at `f32`, `eye_focus`, `ear`, the pose and
    `mesh_ms` at `f64`. The landmark columns are `x,y`.
  - A file whose preview cannot be read, decoded or searched is reported on
    stderr and gets no rows (none happened).
  - `--check` runs `scan::extract_analysis` next to the per-face pass on
    the same worker and compares the `af` rule's value (`None` against
    `None` matches, else |difference| <= 1e-9).
- Run: `run.sh <threads> <folder>...` in
  `D:\Photos\tests\2026-10-23-face-choice\` (a copy of the release
  `riffle-cli.exe` next to it, so a rebuild cannot change a running job),
  `facechoice D:/photos/2026/<folder> 12 --check --out
  faces/<folder>.tsv.part`, renamed to `faces/<folder>.tsv` on exit 0; a
  folder whose `.tsv` exists is skipped. stderr in `logs/<folder>.err`, the
  wall times in `logs/wall.log`. The source folders are
  `D:/photos/2026/<folder>`, as the first line of the rank-app
  `dump.log` lists them; the library was only read.
- **Result**: all 27 folders, **32,459 files** (each folder's file set
  equal to its `features\<folder>.tsv`), **0 errors**, 63,387 crop faces
  and 126,054 whole faces, 93,046 of the 189,441 meshed. **`--check`: 0
  mismatches of 32,459 files** against `scan::extract_analysis`.
- **Wall time per folder** (12 threads, `--check` on, the machine
  otherwise near idle): 2026-06-05 50 s (1,337 files, the first folder,
  measured alone), 06-13 36, 06-14 62, 06-20 109, 07-02 11, 07-05 83,
  07-11 21, 07-18 60, 07-21 1, 07-22 8, 07-23 1, 07-24 8, 07-26 3, 07-30
  40, 07-31 13, 08-01 135, 08-02 7, 08-08 17, 08-15 43, 08-16 11, 08-22
  141, 08-29 140, 09-13-b 76, 09-19 87, 09-27-a 269, 09-27-b 59, 10-03 80;
  **1,571 s in all** (about 26 min, against the plan's 1.5–3 h guess).
- **Per-face mesh cost** (`eyes::mesh_of` alone): **24.4 ms mean** (median
  24.1, p95 26.5, max 33.6) on one thread over the first 150 files of
  2026-06-05 (263 meshed faces; 145 ms per file in all, without
  `--check`); **48.6 ms mean** over all 93,046 meshed faces at 12 threads
  (per-folder means 27.9–56.5 ms), the contention of 12 workers.
- No per-face value was read against `labels.tsv`; the face counts and
  every rule are Step 3's.
- `cargo` is not on Git Bash's `PATH` here; `mise exec -- cargo ...` works.
