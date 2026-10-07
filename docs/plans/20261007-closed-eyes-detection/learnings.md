# Learnings: closed-eyes detection

## Step 1: eye-state model survey

- **Outcome: none fits pass 2; Face Landmarker v2 adopted on demand.** The
  measured verdict stands: no candidate fits the second scan pass's budget.
  The adoption is on demand for the shown file only, outside the scan. The
  decision, the numbers and the new Steps 2-4 are in `plan.md`
  ("Decision"), `model-survey.md` and `eyes-truth.md`.
- **Plan change (user intervention, 2026-10-07).** Step 1 was first
  committed (`fecff8a2`) with the decision "none" and Steps 2-5 struck. The
  user replaced it before the PR opened: Face Landmarker v2 runs on demand
  for the shown file (the way `faces_of` detects the focus mark's faces),
  its result in the meta pane only, while the scan stays as it is. The same
  branch then reworded the `todo.md` item (kept open, closed by Step 4),
  the last paragraph of "Eye-state model survey (Windows 11)" in
  `docs/humans/performance.md` / `.ja.md`, and this file.
- **Reading the budget.** The plan's Trade-offs turned +25% of the per-file
  `detect` time into "4-5 ms per face" assuming ~1.8 faces per file, but the
  approved "which face" default judges only the face nearest the AF point,
  so the budget is per file, one face. Either reading rules out every
  landmark model; only the classifier fits, and it does not work.
- **The machine was slower than in the recall plan**: `riffle-cli detect` on
  40 ARWs took decode 12.7-15.5 ms + detection 31.8-39.5 ms per file (10 +
  24 ms in the recall plan), and `candidates` on the 2134-ARW folder at 24
  threads 19.22 / 19.64 / 14.82 s (12-15 s then). The model latencies were
  therefore compared with `detect` run alternately in the same session, and
  the cost is reported as a ratio.
- **Downcast eyes look closed.** Most ARW faces labeled `c` are children
  looking down (at a ball, a desk), not mid-blinks, and the DNG ones are
  mostly smiles and squints. A single frame cannot tell them apart; the
  labels follow what is visible (no iris = closed). The OMZ classifier,
  trained on infrared driver crops, seems to call many of them open, which
  is part of its low AUC; face mesh v2's EAR follows the visible lids.
- **The OMZ classifier's output index**: index 0 is closed, index 1 open
  (measured), although its README lists `[open, closed]`; its gaze demo's
  `out[0] < out[1]` = open agrees with the measurement. The output is close
  to binary (0.000 / 1.000) on these previews, so its threshold barely
  matters.
- **YuNet's eye points sit a little above the pupil** (2-3 preview px on a
  240 px face, drawn on `_DSC2250` / `_DSC3723`); shifting the classifier's
  crop down by 5-10% of the inter-ocular distance made it worse, not better.
- **Profile faces collapse the inter-ocular distance** (eye points 4-10 px
  apart on a 160-300 px face), so an eye crop sized from it falls to the
  16 px floor. A later attempt should size eye crops from the face box as
  well, not only from the eye distance.
- **The crop geometry stayed in the CLI.** The plan allowed a public helper
  in `faces.rs` only if a later core module needed it too; with no model
  adopted there is none, so `face_to_upright`, `eye_squares`, `face_square`
  and their constants live next to `eyecrops` in `crates/cli/src/main.rs`.
  Moving them out of core gave byte-identical `eyecrops` output on the
  146-DNG folder. With the on-demand adoption, Step 2 moves `face_square`
  / `face_to_upright` into `riffle-core` after all.
- **Converting MediaPipe's TFLite files**: tflite2onnx 0.4.1 converts
  `iris_landmark.tflite` but fails on `face_landmark.tflite` (an
  `IndexError` in its layout propagation); tf2onnx 1.17.0 with TensorFlow
  2.21.0 (`--tflite ... --opset 13`) converts both face meshes, widening
  the float16 weights to float32 (1.2 MB -> 2.4 MB, 2.6 MB -> 4.9 MB). All
  four ONNX files load in tract with `default-features = false`.
- **Windows tooling**: a Python venv under the session scratchpad failed to
  install numpy with `WinError 206` (path too long); a venv at
  `%TEMP%\cesvenv` worked. `mise x -- cargo` must run with the worktree as
  the working directory (the scratchpad has no `mise.toml`, so `mise` finds
  no Rust); build a scratch crate with `--manifest-path`. A `mise x --
  python - <<EOF` heredoc hung in Git Bash; `python3` (on the PATH) did not.
- **Labeling at scale**: 288x128 eye-band tiles, 24 to a sheet, were
  readable down to about a 60 px face; per-eye two-letter codes per tile,
  checked against the sheet's file list, kept 504 labels aligned. Four in
  ten random faces could not be labeled at all (no eye to judge, or too
  small / blurred / dark).

## Step 2: the `eyes` module in `riffle-core`

- **The AUC reproduces exactly.** `riffle-cli eyes` on the 504 labeled files
  (`labeled-paths.txt`) gives, for every file, the same face and the same
  EAR to 4 decimals as the survey's `fm2` columns (max difference 0.0000).
  On `eyes-truth.md`: face AUC 0.974 over the 253 open / closed faces (57
  closed), accuracy 0.957, precision 0.94, recall 0.86 at EAR <= 0.137
  (tp 49, fp 3, fn 8, tn 193), as in `model-survey.md`. Over the 246 faces
  at or above the 60 px floor: AUC 0.974, accuracy 0.955, same precision /
  recall (the 7 faces below it are all open). ARW random sample 0.918, DNG
  0.986. Three runs give identical lines apart from the times. The outputs
  are `D:\Photos\tests\2026-10-07-closed-eyes\eyes-step2-{1,2,3}.txt`;
  the scoring script `scratch\auc.py` there.
- **The logistic's slope.** `EYES_LOGIT_SLOPE` = 29.0: the
  maximum-likelihood slope of `sigmoid(k * (0.137 - EAR))` with the
  midpoint held at 0.137, on the per-face min EAR of `survey-arw.tsv` /
  `survey-dng.tsv` against the 253 labeled faces: k = 28.996 (log-likelihood
  -46.55). A free two-parameter fit gives intercept 4.885, slope -31.53, so a
  midpoint of 0.155; the midpoint was held at the best-F1 threshold so the
  percentage and the word agree. EAR 0.10 -> 75% closed, 0.12 -> 62%,
  0.15 -> 41%, 0.20 -> 14%, 0.30 -> 1%. The script is
  `D:\Photos\tests\2026-10-07-closed-eyes\scratch\fit.py` (kept out of
  `main`).
- **Per-face latency** of `eyes::ear_of` (crop, resize, model, EAR), one
  thread, release, over the 504 labeled faces, three runs: mean 34.5 / 34.6
  / 35.5 ms, median 34.0 / 33.8 / 35.2 ms, p95 41.4 / 42.5 / 42.5 ms. On the
  40-ARW sample 29.8 ms mean, against `riffle-cli detect` decode 10.9 ms +
  detection 26.1 ms on the same files (the survey day: 13.5 + 32.4 ms). The
  machine ran ~20% faster than on the survey day, but the model ran ~35%
  faster than the survey's 49-51 ms, so part of the gap is the survey
  itself: it ran three other models on each face between the face mesh
  calls (Inferred: cache pressure). `riffle-cli bench` row 6 on 10 ARWs:
  39.2 ms mean; on 3 DNG faces 37.3 ms.
- **First call (plan build + one run)**: 169.1, 155.8 and 161.6 ms, so
  ~120-135 ms of plan build (the survey measured 207 ms for the build).
- **Binary sizes** (release, Windows 11, same day):

  | Binary | Before | After |
  |--------|--------|-------|
  | `riffle-cli` | 24,591,872 B (24.6 MB) | 29,557,248 B (29.6 MB) |
  | `riffle-app` | 47,491,584 B (47.5 MB) | 47,499,776 B (47.5 MB) |

  `riffle-cli` grew by 4.97 MB, about the ONNX file. `riffle-app` grew by
  8 KB only: it does not call `eyes` yet, and the linker drops the unused
  `include_bytes!` static. The app's real cost shows once Step 3 calls it;
  measure it there (Step 4 records it). The incremental release rebuild of
  `riffle-core` + `riffle-cli` with the model took 15.9 s; a cold build was
  not timed (nothing suggested a change: the model is a byte array, not
  code).
- **`eyecrops` is byte-identical** on the 146-DNG folder before / after
  moving `face_to_upright` (to `faces.rs`, next to `to_stored`) and
  `face_square` / `FACE_CROP` (to `eyes.rs`, the only core user):
  `diff -r` of the two output folders (505 files, 168 faces) is empty.
- **API shape.** Besides `judge` (the floor, the `catch_unwind` boundary,
  `Option<Eyes>`), `eyes` exposes `ear_of` (the EAR with no floor and no
  panic guard) and `Eyes::from_ear`, so `riffle-cli eyes` can print the EAR
  of faces below the floor too and the AUC is taken over every labeled face
  with one model run per face. `Eyes` holds only `probability` and `state`,
  as planned.
- **"The crop leaves the image"** is read as the face box's center lying
  outside the image (or a buffer too short for the size): `ear_of` returns
  `None` then. A crop that only overhangs an edge is clamped inside the
  image by `faces::crop_rgb`, as the survey cut it, so the measured faces
  and the judged ones are the same set.
- **The `#[ignore]`d real-image test** passed on
  `D:\photos\samples\JPG\Canon_EOS_R5_Official_portrait_of_Liz_Truss.jpg`
  (`RIFFLE_FACE_JPEG=... cargo test --release -p riffle-core eyes --
  --include-ignored`): 478 points inside 0..256 and a probability in 0..1.
- **CLAUDE.md** gained `src/eyes.rs` in the layout paragraph, which the
  plan's file list did not name; the module map would otherwise miss it.
- **Editing Markdown through a Python heredoc**: Windows paths in ordinary
  Python string literals lost their backslashes to escapes (`\t`, `\a`,
  `\f` became control characters in this file once); build such text with
  raw strings or `chr(92)`, and grep the result for control characters.
- **CI (local)**: the first `mise run ci` failed on `clippy::type_complexity`
  for a closure returning `Result<Option<(bool, Face, Option<f64>, f64)>>`
  in `riffle-cli eyes`; a `Judged` struct and a `judge_file` function (the
  `Detected` pattern of `detect`) fixed it.

## Deferred issues (todo candidates)

- **Pending manual check (user): review the faces labeled closed.** The
  agent labeled every face of `eyes-truth.md`; the approved default is that
  the user reviews only the `closed` ones. Platform: any (image viewer).
  Steps: for each file in the "Faces with a closed eye" list of
  `docs/plans/20261007-closed-eyes-detection/eyes-truth.md`, open its tile
  `D:\Photos\tests\2026-10-07-closed-eyes\tiles\<stem>.png` (or the crops
  `arw\<stem>-<n>-face.png` / `-l.png` / `-r.png`) and confirm that the eye
  marked `c` shows no iris; note any that are open. Expected: the large
  majority confirmed; a change would move the AUCs in `model-survey.md`
  and the EAR threshold Step 2 takes from them, since both come from these
  labels.
  Step 1's checkbox was ticked on the automated criteria (the measurements
  and the decision do not wait for this review).

- **Repoint the tract guide's source link when the plan is archived.**
  Step 2 added a link from `docs/agents/tract-onnx-inference.md` ("The face
  mesh: RGB NHWC 0..1, points by `Identity`") to
  `../plans/20261007-closed-eyes-detection/learnings.md`. The wrap-up's
  move to `docs/plans/_archived/` breaks it for lychee unless it is changed
  to `../plans/_archived/20261007-closed-eyes-detection/learnings.md` in the
  same change. Basis: Step 2 implementation.
- **Measure `riffle-app`'s size with the model actually called.** Step 2's
  after-size of `riffle-app` (+8 KB) does not include the 4.9 MB model
  because nothing in the app calls `eyes` yet; Step 3's build is the first
  where it lands. Step 4's size table must take the app size from a build
  with Step 3 in it. Basis: Step 2 binary sizes; files
  `crates/core/src/eyes.rs`, `docs/humans/performance.md`.
