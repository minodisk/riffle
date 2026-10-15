# Candidate 2: the detector shortlist

Measured 2026-10-15 on this machine (Windows 11, 24 logical cores). Each
download sits in its own folder under `D:\Photos\tests\2026-10-14-detectors\`
and was treated as untrusted data: only read as a model by onnxruntime and
tract, scripts kept in this plan folder and the session scratchpad, Python
run with `-I -B`.

## Licenses (code and weights), checked at the source

| Model | Code | Weights | Checked at |
| --- | --- | --- | --- |
| YOLOX Nano / Tiny / S | Apache-2.0 | Apache-2.0 (release assets of the same repository, no separate terms) | <https://github.com/Megvii-BaseDetection/YOLOX/blob/main/LICENSE>, release `0.1.1rc0` |
| NanoDet-Plus m 416 | Apache-2.0 | Apache-2.0 (release assets of the same repository, no separate terms) | <https://github.com/RangiLyu/nanodet/blob/main/LICENSE>, release `v1.0.0-alpha-1` |
| RTMDet tiny / s | Apache-2.0 (MMDetection) | Apache-2.0 (MMDetection model zoo) | <https://github.com/open-mmlab/mmdetection/blob/main/LICENSE> |
| D-FINE N / S | Apache-2.0 | Apache-2.0 (`ustc-community/dfine-nano-coco`, `dfine-small-coco` model cards); ONNX by onnx-community from those | <https://github.com/Peterande/D-FINE/blob/master/LICENSE>, <https://huggingface.co/ustc-community/dfine-nano-coco> |
| RT-DETRv2 R18 | Apache-2.0 | Apache-2.0 (`PekingU/rtdetr_v2_r18vd`, and the onnx-community export's card) | <https://github.com/lyuwenyu/RT-DETR/blob/main/LICENSE>, <https://huggingface.co/PekingU/rtdetr_v2_r18vd> |
| DEIM | Apache-2.0 (its `LICENSE` file; GitHub's API reports `NOASSERTION` because of a copyright header above the text) | not fetched | <https://github.com/ShihuaHuang95/DEIM/blob/main/LICENSE> |

None of the shortlist is AGPL / GPL / non-commercial; the Ultralytics YOLO
and YOLO-NAS families stay excluded as the plan says.

## Pinned files

| Model | URL | Size (bytes) | SHA-256 |
| --- | --- | ---: | --- |
| YOLOX Nano | `https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_nano.onnx` | 3,659,407 | `c789161ed43c8269fcd4e67c67eeeb4e80c622da2eb296a20bc6007bd18a0b7d` |
| YOLOX Tiny | `https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_tiny.onnx` | 20,219,662 | `427cc366d34e27ff7a03e2899b5e3671425c262ea2291f88bb942bc1cc70b0f7` |
| YOLOX S | `https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_s.onnx` | 35,858,002 | `c5c2d13e59ae883e6af3b45daea64af4833a4951c92d116ec270d9ddbe998063` |
| NanoDet-Plus m 416 | `https://github.com/RangiLyu/nanodet/releases/download/v1.0.0-alpha-1/nanodet-plus-m_416.onnx` | 4,793,616 | `59d2f166088889c902f523bf08079391993491324f0d84847e3c4016a8f7cc3d` |
| D-FINE N | `https://huggingface.co/onnx-community/dfine_n_coco-ONNX/resolve/380d2839c327efaf65dd0fe0c2c10ab7fadd5473/onnx/model.onnx` | 15,258,358 | `0f684f409618ee8a822410e754a29caa817d1aa16283ce89cad936d0a48e2f35` |
| D-FINE S | `https://huggingface.co/onnx-community/dfine_s_coco-ONNX/resolve/a3cf03147a9b86c78475139115c8ac142577352d/onnx/model.onnx` | 41,535,197 | `cd8a49a945feda6d28c6304ae8ae85c2759ba1d78a5a83a22c5ce8db82ef7238` |
| RT-DETRv2 R18 | `https://huggingface.co/onnx-community/rtdetr_v2_r18vd-ONNX/resolve/936f90b6a476c6da4dfe053fc521af55285976ba/onnx/model.onnx` | 81,057,510 | `583a236ac21c95a7fd94f284fc21485e42355bfef82c27011ba78fbc09ee87e2` |

**Not measured:**

- **RTMDet tiny / s**: no ONNX is published (OpenMMLab ships PyTorch
  checkpoints; the ONNX comes out of an mmdeploy export that needs PyTorch,
  mmcv and mmdeploy, none of which is set up here). Not exported; recorded in
  `learnings.md` as a deviation.
- **DEIM**: no ONNX of its COCO weights was found (onnx-community has none),
  and it shares D-FINE's decoder, whose cost in tract (below) already rules
  the family out at this bar.

## Input, ops, cost

The input is the upright preview of `riffle-cli previews --near 640`
(808 x 540 or 540 x 808, the 4/8 DCT scale of a 1616-px ARW preview),
preprocessed as each model's own pipeline does (`detect_eval.py`'s
`preprocess`, the reference for a Rust port):

- **YOLOX**: fit inside the square input, top-left, padded with 114, BGR,
  raw 0..255, OpenCV's linear resize (half-pixel centers, no antialias);
  the output is the raw head (sigmoid objectness and classes, box offsets
  and log sizes per grid cell of strides 8 / 16 / 32), decoded as
  `demo_postprocess`, score = objectness x class, class-wise NMS at IoU 0.45.
- **NanoDet-Plus**: stretched to 416 x 416 (`keep_ratio: False`, a
  `warpPerspective` of a scale matrix, so the linear resize without the
  half-pixel offset), BGR, `(x - [103.53, 116.28, 123.675]) / [57.375,
  57.12, 58.395]`; the output (3598 x 112) is the 80 sigmoid class scores
  and 4 x 8 distance bins per prior of strides 8 / 16 / 32 / 64 (52, 26,
  13 and 7 cells a side, centers at `cell * stride`), the distances the
  softmax expectation of the bins times the stride, class-wise NMS at IoU 0.6.
- **D-FINE / RT-DETRv2** (onnx-community exports): PIL bilinear to 640 x 640,
  RGB, 0..1; 300 queries, sigmoid logits, normalized `cx, cy, w, h` boxes,
  no NMS.

The default confidence is each model's own display threshold (YOLOX's ONNX
demo `--score_thr` 0.3, NanoDet's demo 0.35, the transformers pipeline's
0.5). Detections are kept from 0.05 up.

| Model | Input | ONNX ops (beyond `Conv` and elementwise) | tract 0.23 load (`default-features = false`) | tract warm ms/image, 1 thread | onnxruntime warm ms/image, 1 thread | Size bar (<= 100 MB) | Cost bar (<= 132 ms) |
| --- | ---: | --- | --- | ---: | ---: | --- | --- |
| YOLOX Nano | 416 | `MaxPool`, `Resize`, `Slice`, `Concat`, `Reshape`, `Transpose` | loads | 53.1 | 14.5 | yes | yes |
| YOLOX Tiny | 416 | same | loads | 100.5 | 50.9 | yes | yes |
| YOLOX S | 640 | same | loads | 393.6 | 200.2 | yes | **no** |
| NanoDet-Plus m 416 | 416 | `LeakyRelu`, `MaxPool`, `Resize`, `Split`, `Slice` (initializers listed as graph inputs) | loads | 73.5 | 34.9 | yes | yes |
| D-FINE N | 640 | `GridSample` x 6, `TopK`, `ScatterND`, `GatherElements`, `MatMul`, `Softmax`, `Erf` | **loads** (GridSample included) | 1008.0 | 85.4 | yes | **no** |
| D-FINE S | 640 | `GridSample` x 9, same family | loads | 2429.9 | 254.3 | yes | **no** |
| RT-DETRv2 R18 | 640 | `GridSample` x 9, same family | loads | 2819.3 | 464.2 | yes | **no** |

tract's numbers are `detector_loads_and_runs_in_tract`
(`crates/core/tests/detector_tract.rs`, release build, 20 warm runs on a
constant tensor, the machine otherwise idle). The plan expected the
transformer heads' `GridSample` to fail the load without
`tract-transformers`; it did not: all three load and run, but at 1.0-2.8 s
per image, 8-21x the bar, so they are no-ship on cost.

## The labeled frames (detect_eval.py)

240 frames with a `ball` label (137 visible / 103 none). The ball columns
read each frame's top `sports ball` confidence (0 when none at 0.05 or
above); the best-F1 confidence is one of those values, ties to the higher.

| Model | Input | Top ball confidence AUC vs `ball` | Default confidence | Recall | Precision | F1 | Best-F1 confidence | Recall | Precision | F1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| YOLOX Nano | 416 | 0.672 | 0.3 | 0.197 | 1.000 | 0.329 | 0.050 | 0.482 | 0.786 | 0.597 |
| YOLOX Tiny | 416 | 0.748 | 0.3 | 0.438 | 0.952 | 0.600 | 0.053 | 0.628 | 0.811 | 0.708 |
| YOLOX S | 640 | 0.874 | 0.3 | 0.657 | 0.957 | 0.779 | 0.143 | 0.766 | 0.946 | 0.847 |
| NanoDet-Plus m 416 | 416 | 0.759 | 0.35 | 0.307 | 0.933 | 0.462 | 0.136 | 0.818 | 0.696 | 0.752 |
| D-FINE N | 640 | 0.855 | 0.5 | 0.175 | 1.000 | 0.298 | 0.085 | 0.891 | 0.782 | 0.833 |
| D-FINE S | 640 | 0.928 | 0.5 | 0.825 | 0.904 | 0.863 | 0.413 | 0.883 | 0.896 | 0.890 |
| RT-DETRv2 R18 | 640 | 0.897 | 0.5 | 0.730 | 0.870 | 0.794 | 0.256 | 0.934 | 0.780 | 0.850 |

Persons at the default confidence on the labeled frames:

| Model | Confidence | Mean | Median | 0 | 1 | 2 | 3-5 | 6-10 | > 10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| YOLOX Nano | 0.3 | 5.60 | 5 | 0 | 9 | 20 | 105 | 94 | 12 |
| YOLOX Tiny | 0.3 | 5.85 | 5 | 0 | 8 | 20 | 106 | 85 | 21 |
| YOLOX S | 0.3 | 6.71 | 6 | 0 | 5 | 21 | 87 | 88 | 39 |
| NanoDet-Plus m 416 | 0.35 | 6.60 | 6 | 0 | 8 | 16 | 87 | 94 | 35 |
| D-FINE N | 0.5 | 0.45 | 0 | 182 | 35 | 11 | 11 | 1 | 0 |
| D-FINE S | 0.5 | 7.07 | 6 | 0 | 6 | 20 | 81 | 87 | 46 |
| RT-DETRv2 R18 | 0.5 | 7.29 | 6 | 0 | 7 | 21 | 74 | 81 | 57 |

(D-FINE N's confidences sit lower overall; at 0.5 it reports almost no
person. It is no-ship on cost either way.)

## Choice

Shippable (license, size, the one-thread tract cost): YOLOX Nano, YOLOX
Tiny and NanoDet-Plus m 416. The best ball F1 among them is
**NanoDet-Plus m 416: F1 0.752 at confidence 0.13574** (recall 0.818,
precision 0.696; the top-confidence AUC 0.759), against YOLOX Tiny's 0.708
and Nano's 0.597. It is the chosen detector; its persons are counted at its
default 0.35.

The better ball detectors (YOLOX S 0.847, D-FINE S 0.890, RT-DETRv2 0.850,
D-FINE N 0.833) all fail the cost bar in tract, 3x to 21x over it. A
smaller ball detector that ships and misses small balls is the trade the
plan anticipated.
