# Eye-state model survey (Step 1)

Four candidates were run end to end on the faces of `eyes-truth.md`: one
eye-crop classifier and three landmark models with eyelid points. Measured
on 2026-10-07 on Windows 11 (Intel Core i7-13700, 24 hardware threads,
32 GB), release build, one thread, against `riffle-core` at this branch.

The measurement code is not in the repository: a scratch crate (`survey`,
depending on `riffle-core` by path and on `tract-onnx` 0.23 with
`default-features = false`) and the Python scripts that sampled and scored
the labels. A copy of both, the ONNX files below and the raw outputs is in
`D:\Photos\tests\2026-10-07-closed-eyes\` (`scratch\`, `survey-arw.tsv`,
`survey-dng.tsv`, `occsweep.tsv`, `timing40-*.tsv`, `ref-*.txt`).

## The files

| Candidate | File | Size | SHA-256 | Source | License |
|-----------|------|------|---------|--------|---------|
| Eye-crop classifier: Open Model Zoo `open-closed-eye-0001` | `open_closed_eye.onnx` (native ONNX, opset 9) | 46,164 B | `4daa100034482525a26c9afb9297c16580a531189e66e3d2b2ac7d32becfd593` | https://storage.openvinotoolkit.org/repositories/open_model_zoo/public/2022.1/open-closed-eye-0001/open_closed_eye.onnx (the SHA-384 matches the checksum in OMZ's `model.yml`) | Apache-2.0 (`openvino_training_extensions`); trained on the MRL Eye Dataset (infrared driver crops) |
| Face mesh v1 (MediaPipe Face Mesh, 468 points, 192x192) | `face_landmark.onnx`, converted here | 2,429,024 B (the TFLite is 1,242,398 B; tf2onnx widens its float16 weights to float32) | `0e754bfc97df1889dd29e91ec779e9e35075c1bb194d4dee00632e709fa77e9e` | https://storage.googleapis.com/mediapipe-assets/face_landmark.tflite (SHA-256 `1055cb9d4a9ca8b8c688902a3a5194311138ba256bcc94e336d8373a5f30c814`) | Apache-2.0 (MediaPipe) |
| Face mesh v2 (MediaPipe Face Landmarker, 478 points, 256x256) | `face_landmarks_detector.onnx`, converted here | 4,921,000 B (TFLite 2,553,590 B) | `cafab16c9132f1252e59f250b216831fa9e9c08f1424e7856e3e849771c05804` | `face_landmarks_detector.tflite` (SHA-256 `c7d54204ce0448474c7f3fa9af494787c0965cbdd6f20fc72867e43046bd43d5`) inside https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/latest/face_landmarker.task (SHA-256 `64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff`) | Apache-2.0 (MediaPipe) |
| MediaPipe Iris (71 eye contour + 5 iris points, 64x64 per eye) | `iris_landmark.onnx`, converted here | 2,642,058 B (TFLite 2,640,568 B) | `030a88812c723decac9e63b41d72e0655206b9fbf0a53e0f2b4a161a5d0fd306` | https://storage.googleapis.com/mediapipe-assets/iris_landmark.tflite (SHA-256 `d1744d2a09c25f501d39eba4faff47e53ecca8852c5ce19bce8eeac39357521f`) | Apache-2.0 (MediaPipe) |

Conversions (Python 3.13 venv): the two face meshes with
`python -m tf2onnx.convert --tflite <file>.tflite --output <file>.onnx --opset 13`
(tf2onnx 1.17.0 on TensorFlow 2.21.0); the iris model with tflite2onnx 0.4.1
(`tflite2onnx.convert`), which failed on `face_landmark.tflite` with an
`IndexError` in its layout propagation. A file adopted later would need this
conversion and its tool versions stated in its license file.

Dropped without a run:

- PFLD (98 WFLW points, 112x112): neither common implementation
  (`polarisZhao/PFLD-pytorch`, `guoqiangqi/PFLD`) carries a license, so the
  weights cannot be redistributed; WFLW itself is a research dataset.
- InsightFace `2d106det` / `1k3d68`: non-commercial (excluded by the plan).
- Third-party ONNX conversions of MediaPipe on Hugging Face
  (`Heliosoph/mediapipe-face-onnx`, `senty-au/face_landmarks_detector-ONNX`,
  `naklitechie/face-landmarks-onnx`, ...): converting Google's file here
  states the provenance instead.

## Loading in tract

All four load, type and optimize with `tract-onnx` 0.23,
`default-features = false`, `with_ignore_value_info(true)`,
`with_ignore_output_shapes(true)` and the input fact pinned (no unsupported
op; the ops are Conv, PRelu, MaxPool, Pad, Add, Reshape, Transpose, Sigmoid,
and Relu / Div / Exp / ReduceSum for the classifier). Plan build time on the
first call: 7 ms, 67 ms, 207 ms and 139 ms.

| Candidate | Input fact | Input | Outputs by outlet label |
|-----------|------------|-------|-------------------------|
| Classifier | `[1, 3, 32, 32]` | BGR NCHW, `(v - 127) / 255` (OMZ's `--mean_values` / `--scale_values`) | `19`: `[1, 2, 1, 1]` softmax. Index 0 is **closed**, index 1 open (measured: 0.000 on wide-open eyes, 1.000 on shut ones; OMZ's README says `[open, closed]`, but its gaze demo treats `out[0] < out[1]` as open, which agrees with the measurement) |
| Face mesh v1 | `[1, 192, 192, 3]` | RGB NHWC, 0..1 (`face_landmark_cpu.pbtxt`: `output_tensor_float_range 0..1`) | `conv2d_21`: 468 x (x, y, z) in input pixels; `conv2d_31`: face presence logit |
| Face mesh v2 | `[1, 256, 256, 3]` | RGB NHWC, 0..1 | `Identity`: 478 x (x, y, z); `Identity_1`: face presence logit; `Identity_2` unused |
| Iris | `[1, 3, 64, 64]` | RGB NCHW, 0..1 (`zero_center: false`); the image-right eye mirrored, as MediaPipe feeds its right eye | `output_eyes_contours_and_brows`: 71 x (x, y, z), the first 16 the eyelid contour; `output_iris` unused |

## Crops

All crops are cut from the full-size upright preview (`faces::upright_rgb`)
with `faces::crop_rgb` (clamped inside the image) and resized bilinearly to
the model input, no roll correction. The geometry is the one
`riffle-cli eyecrops` writes:

- Eye crop (classifier): a square centered on YuNet's eye point, side
  0.8 x the inter-ocular distance (about 1.8 x the eye's corner-to-corner
  width, the box OMZ's demo cuts), at least 16 px.
- Face crop (face meshes): the YuNet box squared on its longer side and grown
  1.25x (YuNet's box runs from the forehead to the chin, so this is close to
  MediaPipe's 1.5x of its tighter detector box).
- Iris crop: a square on the eye point, 1.05 x the inter-ocular distance
  (MediaPipe's 2.3x of the eye-corner distance).

Scores: the classifier's closed probability; the EAR of the face meshes'
six points per eye (`33, 160, 158, 133, 153, 144` and
`362, 385, 387, 263, 373, 380`); for the iris model, the height over the width
of the 16 eyelid points' bounding box. Lower EAR / openness is more closed;
a face scores the more closed of its two eyes.

## Results

Per-face latency: one thread, the crop and resize included (both eyes for
the classifier and the iris model), over the 1952 faces judged in the
2134-ARW folder, then over the 40-ARW sample (every 53rd file) in three
runs alternated with `riffle-cli detect` on the same files:

| Candidate | 1952 faces: mean / median / p95 | 40-ARW runs: mean | Per-file pass-2 cost added |
|-----------|----------------------------------|-------------------|----------------------------|
| Classifier (2 runs) | 0.52 / 0.48 / 0.79 ms | 0.50-0.55 ms | +1% |
| Face mesh v1 | 15.4 / 15.4 / 21.5 ms | 15.2-16.1 ms | +33-35% |
| Face mesh v2 | 50.9 / 49.1 / 68.6 ms | 48.4-50.1 ms | +105-109% |
| Iris (2 runs) | 18.6 / 18.2 / 26.4 ms | 18.5-18.7 ms | +40% |

The reference in the same session: `riffle-cli detect` on the 40 ARWs,
decode 13.4-13.6 ms + detection 31.8-32.7 ms, about 46 ms per file. The
machine ran slower than in the recall plan (10 + 24 ms then), so the budget
is taken as a ratio: +25% is about +11.5 ms per file today, one face per
file (only the face nearest the AF point is judged).

Accuracy on the labeled set (`eyes-truth.md`; 463 eyes / 253 faces of ARW
and DNG with an `o` or `c` label, 109 / 57 closed). "Best" is the threshold
with the highest F1 of `closed`, per face:

| Candidate | Eye AUC | Face AUC | Face at the best threshold: accuracy / precision / recall | ARW random sample only: eye / face AUC |
|-----------|---------|----------|-------------------------------------------------------------|----------------------------------------|
| Classifier | 0.718 | 0.716 | 0.585 / 0.35 / 0.95 (at 0.5: 0.644 / 0.35 / 0.70) | 0.719 / 0.745 |
| Face mesh v1 | 0.817 | 0.886 | 0.842 / 0.62 / 0.79 (EAR <= 0.214) | 0.797 / 0.865 |
| Face mesh v2 | 0.940 | 0.974 | 0.957 / 0.94 / 0.86 (EAR <= 0.137) | 0.864 / 0.918 |
| Iris | 0.860 | 0.863 | 0.838 / 0.59 / 0.89 (openness <= 0.215) | 0.818 / 0.871 |

- The classifier's output is nearly binary (0.000 or 1.000) and wrong often
  in both directions on these RGB visible-light previews; it was trained on
  infrared crops. Varying its crop (0.4-1.2 x the inter-ocular distance,
  shifted down by 0-10% of it, gray or color input) moved the AUC between
  0.65 and 0.735 per eye and 0.64 and 0.775 per face; the default crop is
  within that spread. No crop rescues it.
- Face mesh v2 is the only candidate accurate enough for a filter (9 in 10
  flagged faces really closed at a recall of 0.86), at about 4x the budget.
- Tilted heads: 8 labelable ARW faces have a roll over 15°, too few to
  measure; all four models were run without roll correction.
- The 3/8 decode of the whole-image path (classifier only, 20 DNG faces):
  eye AUC 0.588 against 0.776 on the full-size crops of the same faces.

## Verdict

No candidate fits both the budget and the accuracy: the classifier fits the
budget at +1% but cannot tell the eye state apart (AUC 0.72); every landmark
model is accurate enough only above the budget (+33% for face mesh v1 at a
face AUC of 0.89, +105% for face mesh v2 at 0.97). See the Decision section
of `plan.md`.
