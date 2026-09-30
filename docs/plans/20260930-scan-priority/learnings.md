# Learnings

## Step 1: the analysis pass in `riffle-core`

- `extract_faces` / `extract_faces_all` were renamed to `extract_analysis` /
  `extract_analysis_all`, returning `Analysis { cue, sharpness }`. The app
  (`crates/app/src/index.rs`, `run_faces_scan`) had to follow the rename in
  this step to keep building, although the plan listed only `scan.rs` and the
  CLI; it reads `.cue.eye_focus` and ignores the score until Step 2. Stale
  mentions of `extract_faces` in `docs/agents/tauri-app.md` and
  `docs/agents/tract-onnx-inference.md` were renamed too.
- To keep the score bit-identical, the whole-image face search and the
  guarded `score_preview` call were pulled out of `extract_unless` into two
  private helpers (`whole_image_faces`, `score`) that both `extract_unless`
  and `extract_analysis_unless` call. Step 2 deletes the call from
  `extract_unless` and the helpers stay with the analysis pass.
- Interim cost until Step 2: the second pass now also scores the rows it
  runs on (those with a trusted AF point; `faces_todo` still marks no-AF rows
  done), about 3 ms per file on top of the cue, and the result is dropped.
- The equality test (`the_analysis_score_is_the_one_extract_gives`) covers
  the no-AF-point branch only (a flat and a textured 64x48 preview): the
  `scan.rs` fixture is a bare TIFF shell with no MakerNote, so it carries no
  AF point. The AF-point branch shares the same `score` helper and passes the
  same `trusted_focus` / `eye_af_frame` / empty faces, so it is identical by
  construction.
- A JPEG file returns `Analysis::default()` before `read_preview`, so a
  missing `.jpg` path is `Ok` (asserted in the JPEG test).

## Step 2: the score moves to the second pass in the app

- `faces_todo`'s JPEG rule is matched in SQL (`path LIKE '%.jpg' OR path
  LIKE '%.jpeg'`); SQLite's `LIKE` ignores ASCII case, so `.JPG` / `.JPEG`
  match too, the way `is_jpeg_file` does. The same `UPDATE` now also clears
  `sharpness`, so an error row or a JPEG row is marked done with neither
  value.
- Step 1's equality test (`the_analysis_score_is_the_one_extract_gives`)
  compared `extract` with `extract_analysis`; with the score gone from
  `extract` it had nothing left to compare and was removed. The app's
  `the_faces_pass_writes_and_reports_every_path` now checks that the first
  pass leaves every score `NULL` and the second fills in exactly
  `extract_analysis`'s score, and the new
  `a_row_the_old_first_pass_scored_keeps_its_score_and_a_new_one_gets_it_from_the_faces_pass`
  covers the no-bump migration.
- `run_faces_scan` now keeps the whole `Analysis` per pending file instead
  of just the probability; an unreadable file is stored as
  `Analysis::default()` (no probability, no score).
- `riffle-cli scan` no longer includes the score (nor the whole-image face
  search for no-AF files) in its per-file time, so its numbers are not
  comparable with the "scan" figures in `docs/performance.md` measured
  before this step; Step 7 rewrites that section.
- The frontend ignores `FaceReady.sharpness` until Step 3; until then the
  scores show at once when `faces-done` re-reads the entries
  (`refreshOnFacesDone`), not file by file.
