# Learnings: closed-eyes detection

## Step 1: eye-state model survey

- **Outcome: no model adopted.** The decision, the numbers and the "none"
  branch (Steps 2-5 struck, `todo.md` item rewritten) are in `plan.md`
  ("Decision"), `model-survey.md` and `eyes-truth.md`.
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
  146-DNG folder.
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

## Deferred issues (todo candidates)

- **Pending manual check (user): review the faces labeled closed.** The
  agent labeled every face of `eyes-truth.md`; the approved default is that
  the user reviews only the `closed` ones. Platform: any (image viewer).
  Steps: for each file in the "Faces with a closed eye" list of
  `docs/plans/20261007-closed-eyes-detection/eyes-truth.md`, open its tile
  `D:\Photos\tests\2026-10-07-closed-eyes\tiles\<stem>.png` (or the crops
  `arw\<stem>-<n>-face.png` / `-l.png` / `-r.png`) and confirm that the eye
  marked `c` shows no iris; note any that are open. Expected: the large
  majority confirmed; a change matters only if Step 1's "none" is ever
  revisited, since the AUCs in `model-survey.md` come from these labels.
  Step 1's checkbox was ticked on the automated criteria (the measurements
  and the decision do not wait for this review).
