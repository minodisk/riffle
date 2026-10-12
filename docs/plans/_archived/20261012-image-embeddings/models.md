# Step 1 measurements: candidate embedding models in tract

Measured 2026-10-12 on the Windows 11 machine (13th Gen Intel Core i7-13700,
24 logical processors, 32 GB), release build of `riffle-cli` at `tract-onnx`
0.23.8 with `default-features = false`. The Python reference is
onnxruntime 1.31.0 (CPU) with Pillow 12.3.0 and numpy 2.5.4 in a venv
outside the repository, every script run with `python -I` from `%TEMP%`.

## Files and commands

- Downloads (untrusted data, nothing committed) under
  `D:\Photos\tests\2026-10-12-embed-models\<model>\`:
  - `siglip2-b32-256`: `onnx-community/siglip2-base-patch32-256-ONNX` at
    revision `7efccb86b5b6601bfe9e14326e1c11b40b68d44c`,
    `onnx/vision_model.onnx` (SHA-256
    `0ddb7507cfcb54c2cd503029949add9dc6586de67164bb69c2c09588cdcc9719`),
    `vision_model_fp16.onnx`, `vision_model_int8.onnx`,
    `vision_model_uint8.onnx`; derived here by `weight_only.py`:
    `vision_model_wint8.onnx`, `vision_model_wfp16.onnx`.
  - `openclip-b32-laion2b`: `immich-app/ViT-B-32__laion2b-s34b-b79k` at
    `19a057a0fb927cf239541300e74237cd56c7ffe2`, `visual/model.onnx`.
  - `tinyclip-40m-32`: `onnx-community/TinyCLIP-ViT-40M-32-Text-19M-LAION400M-ONNX`
    at `737108a175dc6c043d9e64cf738baa91a272f7cb`, `onnx/model.onnx` (text
    and vision in one graph); `vision_model.onnx` is its image tower cut out
    by `extract_vision.py`.
  - `mobileclip-s2`: `Xenova/mobileclip_s2` at
    `0c311c620b36a2270b851db7bef9135f3eaae5d7`, `onnx/vision_model.onnx`.
- Reference set: 12 previews from four folders (orientation 1 x6, 6 x1,
  8 x5), listed in `D:\Photos\tests\2026-10-12-embed-ref\files.txt`:
  `_DSC1796` / `_DSC1907` / `_DSC2129` of `2026-09-19`, three each of
  `2026-08-15`, `2026-06-21` and `2026-10-03`.
- 200-file set: every 10th ARW of `D:\photos\2026\2026-09-19` (214 files),
  `D:\Photos\tests\2026-10-12-embed-runs\files.txt`; dumps and logs
  (`runs.log`, `near.log`, `mem.log`) next to it.
- Commands (`$B` the models directory, `$R` the reference directory):

  ```sh
  riffle-cli embed $B/siglip2-b32-256/vision_model.onnx siglip2 $(cat $R/files.txt) --out $R/siglip2-fp32.rust.tsv --previews $R/previews
  python -I ref_embed.py $B/siglip2-b32-256 vision_model.onnx siglip2 $R/previews/list.tsv $R/siglip2-fp32.ref.tsv
  riffle-cli embed-compare $R/siglip2-fp32.rust.tsv $R/siglip2-fp32.ref.tsv
  riffle-cli embed <model> <spec> $(cat files.txt) 1  --out <model>.preview.t1.tsv
  riffle-cli embed <model> <spec> $(cat files.txt) 24 --out <model>.preview.tsv
  riffle-cli embed <model> <spec> $(cat files.txt) 24 --thumb --out <model>.thumb.tsv
  riffle-cli embed <model> <spec> $(cat files.txt) 24 --near 404 --out <model>.near404.tsv
  riffle-cli embed-compare <model>.thumb.tsv <model>.preview.tsv
  python -I thumb_check.py $B/<model-dir> <file> <spec> $R/previews/list.tsv
  ```

## Summary

| Model | License | Ship | Loads (`default-features = false`) | Cosine vs reference, 12 previews (min) | Warm ms/image, 1 thread | ms/image wall, 24 threads | Plan build | File | Working set: after load / run 1 thr / run 24 thr / peak |
|---|---|---|---|---|---|---|---|---|---|
| SigLIP 2 B/32 @256 FP32 | Apache-2.0 | ship | yes | 0.999997 | 132.3 | 16.6-17.6 | 443-486 ms | 378.5 MB | 395 / 416 / 637 / 1145 MB |
| SigLIP 2 fp16 export | Apache-2.0 | - | **no** (head `Gemm`: F16 / F32 mismatch) | ORT only: 0.999993 vs FP32 | - | - | - | 189.4 MB | - |
| SigLIP 2 int8 export | Apache-2.0 | - | **no** (`ConvInteger`: U8 vs I8) | ORT only: **0.638** vs FP32 | - | - | - | 95.9 MB | - |
| SigLIP 2 uint8 export | Apache-2.0 | - | yes | 0.980 vs ORT uint8; 0.659 vs FP32 | 8381.7 | - | 693 ms | 95.9 MB | 299 MB peak |
| SigLIP 2 weight-only int8 (`weight_only.py`) | Apache-2.0 | - | yes | 0.999546 vs ORT same file; 0.999049 vs FP32 | 250.1 | 85.3 | 1248-1264 ms | 95.8 MB | - / - / - / 332 (1 thr), 673 MB (24 thr) |
| SigLIP 2 weight-only fp16 (`weight_only.py`) | Apache-2.0 | - | yes | 0.999997 vs ORT same file; 0.999998 vs FP32 | 275.1 | 75.9 | 1287-1324 ms | 189.7 MB | - / - / - / 580 (1 thr), 739 MB (24 thr) |
| OpenCLIP ViT-B/32 laion2b-s34b-b79k (immich export) | MIT | ship | yes | 1.000000 | 116.1 | 14.4-14.6 | 402-555 ms | 351.6 MB | 372 / 393 / 637 / 1066 MB |
| TinyCLIP ViT-40M/32 (image tower) | MIT | ship | yes (after `extract_vision.py`) | 1.000000 | 62.0 | 7.7-8.2 | 201-218 ms | 159.0 MB (337.2 MB combined) | 176 / 196 / 447 / 486 MB |
| MobileCLIP-S2 (reference) | Apple AMLR research-only weights | **no-ship** | yes | 1.000000 | 327.6 | 36.2-36.6 | 509-540 ms | 143.0 MB | 170 / 192 / 440 / 443 MB |

Published ImageNet-1k zero-shot (not measured here): SigLIP 2 B/32 @256
74.0, OpenCLIP B/32 laion2b-s34b-b79k 66.6, TinyCLIP 40M/32 59.8,
MobileCLIP-S2 74.4. Output width: SigLIP 2 768, the others 512.

Notes on the columns:

- "Warm ms/image, 1 thread" is the mean of `embed::embed_with`
  (preprocessing plus inference) over the 214 files after one warm-up call,
  one rayon thread. The preview decode is separate (10.5-13 ms per file
  warm; 51 ms on the very first, cold-cache run).
- "ms/image wall, 24 threads" is the 214-file run's wall time divided by
  214, reading and decoding each preview included. One call then takes
  314-354 ms (SigLIP 2, OpenCLIP), the 24 workers sharing the cores. The
  1-thread and 24-thread dumps are identical (cosine 1.000000, top-5
  agreement 1.000).
- Working set: "after load" is the current working set once
  `Model::load` returned and the file's bytes were dropped; "run" is the
  largest current working set sampled every 10 ms during the 214-file run;
  "peak" is the process's peak working set, which `Model::load` sets: the
  file's bytes, the parsed protobuf and the optimized plan are alive
  together (about 3x the file).
- Cosine of the quantized and weight-only rows: against the onnxruntime
  run of the same file ("vs ORT same file") and against the FP32 reference
  ("vs FP32"). On the 214 files the weight-only int8 file's SigLIP 2
  vectors are at cosine min 0.995483 / mean 0.999375 against FP32 with a
  top-5 agreement of 0.890; the weight-only fp16 file's at min 0.999997,
  top-5 agreement 0.996.

## Loading with `default-features = false`

- SigLIP 2 FP32, OpenCLIP, TinyCLIP (image tower), MobileCLIP-S2: load,
  optimize and run with `with_ignore_value_info(true)`,
  `with_ignore_output_shapes(true)` and the pinned `[1, 3, size, size]` input
  fact. No op needed `tract-transformers`; the exports use plain
  `MatMul` / `Softmax` / `Erf` / `LayerNormalization` (OpenCLIP) or the
  decomposed `ReduceMean` / `Pow` / `Sqrt` layer norm (SigLIP 2, TinyCLIP).
- TinyCLIP: onnx-community ships only the combined text + vision graph; the
  plain re-export of its image tower (`extract_vision.py`,
  `pixel_values` -> `image_embeds`) loads.
- SigLIP 2 fp16 export: fails in translation, `Translating node #917
  "/vision_model/head/attention/Gemm" Gemm ToTypedTranslator: Output mismatch
  after rewiring expansion for output #0: expected 1,768,F16 got
  1,768,F32`. The weight-only fp16 re-export (`weight_only.py`, float16
  initializers widened by `Cast`) fixes the load.
- SigLIP 2 int8 export: fails in analysis, `"/vision_model/embeddings/patch_embedding/Conv_quant"
  ConvHir: Impossible to unify U8 with I8`. The uint8 export loads. Both are
  dynamic quantizations whose vectors are far from FP32's in onnxruntime as
  well (cosine min 0.638 / 0.663, mean 0.784 / 0.790): unusable whatever the
  runtime. The weight-only int8 re-export (`DequantizeLinear` per output
  channel) loads and is accurate.

## Thumb vs preview input

For the 214 files, the index thumbnail (`scan::extract`: the preview decoded
at 2/8, 404x270, q80 JPEG) decoded again and rotated, against the full
preview, 24 threads:

| Model | Cosine min | Cosine mean | Top-5 agreement |
|---|---|---|---|
| SigLIP 2 FP32 | 0.907008 | 0.960570 | 0.519 |
| SigLIP 2 weight-only int8 | 0.908875 | 0.964033 | 0.531 |
| SigLIP 2 weight-only fp16 | 0.906974 | 0.960580 | 0.518 |
| OpenCLIP | 0.853920 | 0.911082 | 0.600 |
| TinyCLIP | 0.866401 | 0.910036 | 0.610 |
| MobileCLIP-S2 | 0.956944 | 0.974955 | 0.694 |

The thumbnail input fails the bar (mean cosine >= 0.98 and top-5 agreement
>= 0.8) for every model, so **pass 3 must decode the preview**.

It is the q80 re-encode, not the 404 px size. `thumb_check.py`
(onnxruntime + PIL, the 12 reference previews) embeds the preview after a box
downscale to 404 px with and without a JPEG re-encode:

| Model | 404 px box | + q80 JPEG 4:2:0 | + q80 4:4:4 | + q95 4:2:0 |
|---|---|---|---|---|
| SigLIP 2 (min / mean) | 0.9976 / 0.9991 | 0.9044 / 0.9424 | 0.9730 / 0.9913 | 0.9707 / 0.9852 |
| OpenCLIP (min / mean) | 0.9834 / 0.9922 | 0.8699 / 0.9036 | 0.9198 / 0.9491 | 0.9370 / 0.9501 |

So the preview decoded at the DCT scale nearest 404 px, with no re-encode
(`riffle-cli embed --near 404`, `decode::decode_upright_near`), is the
cheaper input to try. On the 214 files against the full preview, 24 threads:

| Model | Cosine min | Cosine mean | Top-5 agreement | Decode ms (24 thr) | ms/image wall |
|---|---|---|---|---|---|
| SigLIP 2 FP32 | 0.992958 | 0.998452 | 0.843 | 9.0 (full: 26.1) | 14.4 |
| OpenCLIP | 0.943650 | 0.982749 | 0.710 | 9.4 | 12.3 |
| TinyCLIP | 0.967180 | 0.994125 | 0.799 | 8.8 | 5.8 |
| MobileCLIP-S2 | 0.986061 | 0.997341 | 0.838 | 8.9 | 34.6 |

SigLIP 2 at `--near 404` clears the same bar (0.998 / 0.843). For scale:
the weight-only int8 file, at mean cosine 0.999375 against FP32, agrees on
the top 5 at 0.890 only, so top-5 agreement within one shooting day is a
strict measure.

## Binary size if bundled

The release `riffle-cli` grew from 29,754,880 B to 408,233,984 B
(+378,479,104 B, the file's 378,478,689 B) with the SigLIP 2 FP32 file
`include_bytes!`'d and read by `embed` (a throwaway build, not committed),
as the face mesh did (`tract-onnx-inference.md`). The same holds for any
binary that reaches it, so `riffle-app` would grow by the same 378 MB. Its
working set after load was 774 MB against 395 MB loading from a file
(the static's pages plus a copy for `Model::load`). For the other
candidates, the estimate is the file size: OpenCLIP +352 MB, TinyCLIP +159 MB,
the weight-only int8 file +96 MB.

The FP32 weights barely compress: `gzip -6` takes the SigLIP 2 file from
378,478,689 B to 351,217,425 B (-7%), so an installer carrying it as a
resource would grow by about 350 MB.
