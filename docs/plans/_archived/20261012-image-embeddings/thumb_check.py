"""Independent check of the thumb-vs-preview gap in onnxruntime + PIL: embed
each preview as is, and after the index's thumbnail treatment approximated in
PIL (a box downscale to a 404 px long edge, a q80 JPEG re-encode), without
the re-encode, and with q80 at 4:4:4 and q95 instead, and print the cosines.

Usage: python -I thumb_check.py <model-dir> <model.onnx> <spec> <list.tsv>
(`list.tsv` as for ref_embed.py)
"""

import io
import os
import sys

import numpy as np
import onnxruntime as ort
from PIL import Image

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ref_embed import SPECS, TRANSPOSE  # noqa: E402


def embed(session, img, spec):
    size, crop, resample, mean, std, output = spec
    w, h = img.size
    if crop:
        rw, rh = (size, int(size * h / w)) if w <= h else (int(size * w / h), size)
        img = img.resize((rw, rh), resample)
        left, top = (rw - size) // 2, (rh - size) // 2
        img = img.crop((left, top, left + size, top + size))
    else:
        img = img.resize((size, size), resample)
    x = (np.asarray(img, dtype=np.float32) / 255.0 - np.array(mean, np.float32)) / np.array(std, np.float32)
    (v,) = session.run([output], {session.get_inputs()[0].name: x.transpose(2, 0, 1)[None]})
    v = v.reshape(-1).astype(np.float64)
    return v / np.linalg.norm(v)


def thumb(img, reencode):
    w, h = img.size
    s = 404 / max(w, h)
    small = img.resize((round(w * s), round(h * s)), Image.Resampling.BOX)
    if reencode is None:
        return small
    buf = io.BytesIO()
    small.save(buf, "JPEG", **reencode)
    return Image.open(io.BytesIO(buf.getvalue())).convert("RGB")


def main(model_dir, model_file, spec_name, list_path):
    spec = SPECS[spec_name]
    session = ort.InferenceSession(os.path.join(model_dir, model_file), providers=["CPUExecutionProvider"])
    rows = [line.rstrip("\n").split("\t") for line in open(list_path, encoding="utf-8") if line.strip()]
    variants = {
        "404 px box": None,
        "+ q80 JPEG 4:2:0": {"quality": 80},
        "+ q80 JPEG 4:4:4": {"quality": 80, "subsampling": 0},
        "+ q95 JPEG 4:2:0": {"quality": 95},
    }
    cosines = {k: [] for k in variants}
    for _, path, orientation in rows:
        img = Image.open(path).convert("RGB")
        if int(orientation) in TRANSPOSE:
            img = img.transpose(TRANSPOSE[int(orientation)])
        full = embed(session, img, spec)
        for k, v in variants.items():
            cosines[k].append(float(full @ embed(session, thumb(img, v), spec)))
    for k, c in cosines.items():
        print(f"{spec_name}: {k:18} cosine min {min(c):.4f} mean {np.mean(c):.4f}")


if __name__ == "__main__":
    main(*sys.argv[1:])
