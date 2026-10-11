"""Python reference embeddings for `riffle-cli embed`: each model's own
preprocessing (PIL, per its processor config) and onnxruntime.

Usage:
  python -I ref_embed.py <model-dir> <model.onnx> <spec> <list.tsv> <out.tsv> [threads]

`list.tsv` is the one `riffle-cli embed --previews <dir>` writes:
`source path\tpreview jpeg\torientation` per line. The output is
`source path\tvec...`, L2-normalized, the format `riffle-cli embed-compare`
reads. `<model.onnx>` is relative to `<model-dir>` (the download directory,
which this script only reads).
"""

import os
import sys
import time

import numpy as np
import onnxruntime as ort
from PIL import Image

CLIP_MEAN = (0.48145466, 0.4578275, 0.40821073)
CLIP_STD = (0.26862954, 0.26130258, 0.27577711)

# The processor configs of the downloads, as in `embed::SPECS`:
# (size, short side then center crop, PIL filter, mean, std, output name).
SPECS = {
    # preprocessor_config.json: SiglipImageProcessor, 256x256, resample 2.
    "siglip2": (256, False, Image.Resampling.BILINEAR, (0.5,) * 3, (0.5,) * 3, "pooler_output"),
    # visual/preprocess_cfg.json: shortest 224, bicubic, CLIP mean / std.
    "openclip": (224, True, Image.Resampling.BICUBIC, CLIP_MEAN, CLIP_STD, "embedding"),
    # preprocessor_config.json: shortest 224, crop 224, resample 3.
    "tinyclip": (224, True, Image.Resampling.BICUBIC, CLIP_MEAN, CLIP_STD, "image_embeds"),
    # preprocessor_config.json: shortest 256, crop 256, resample 2, no normalize.
    "mobileclip": (256, True, Image.Resampling.BILINEAR, (0.0,) * 3, (1.0,) * 3, "image_embeds"),
}

TRANSPOSE = {
    3: Image.Transpose.ROTATE_180,
    6: Image.Transpose.ROTATE_270,
    8: Image.Transpose.ROTATE_90,
}


def preprocess(path, orientation, spec):
    size, crop, resample, mean, std, _ = spec
    img = Image.open(path).convert("RGB")
    if orientation in TRANSPOSE:
        img = img.transpose(TRANSPOSE[orientation])
    w, h = img.size
    if crop:
        # HF's and immich's rule: the short side to `size`, the long side
        # truncated, then the centered square with a floored offset.
        rw, rh = (size, int(size * h / w)) if w <= h else (int(size * w / h), size)
        img = img.resize((rw, rh), resample)
        left, top = (rw - size) // 2, (rh - size) // 2
        img = img.crop((left, top, left + size, top + size))
    else:
        img = img.resize((size, size), resample)
    x = np.asarray(img, dtype=np.float32) / 255.0
    x = (x - np.array(mean, dtype=np.float32)) / np.array(std, dtype=np.float32)
    return x.transpose(2, 0, 1)[None]


def main(model_dir, model_file, spec_name, list_path, out_path, threads="1"):
    spec = SPECS[spec_name]
    opts = ort.SessionOptions()
    opts.intra_op_num_threads = int(threads)
    session = ort.InferenceSession(
        os.path.join(model_dir, model_file), opts, providers=["CPUExecutionProvider"]
    )
    input_name = session.get_inputs()[0].name
    rows = [line.rstrip("\n").split("\t") for line in open(list_path, encoding="utf-8") if line.strip()]
    start = time.perf_counter()
    with open(out_path, "w", encoding="utf-8") as out:
        for source, jpeg, orientation in rows:
            x = preprocess(jpeg, int(orientation), spec)
            (v,) = session.run([spec[5]], {input_name: x})
            v = v.reshape(-1).astype(np.float64)
            v /= np.linalg.norm(v)
            out.write(source + "".join(f"\t{c:.8g}" for c in v) + "\n")
    elapsed = time.perf_counter() - start
    print(f"{len(rows)} files, {elapsed * 1000 / max(len(rows), 1):.1f} ms/image, ort {ort.__version__}")


if __name__ == "__main__":
    main(*sys.argv[1:])
