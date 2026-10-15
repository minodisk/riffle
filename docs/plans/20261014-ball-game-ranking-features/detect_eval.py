"""Candidate 2's detector shortlist on the hand-labeled frames: each model
(onnxruntime) on the upright previews of the labeled frames, the `sports
ball` recall / precision at the model's default confidence and at the
best-F1 confidence against the `ball` label, the person count distribution,
and the warm one-thread ms/image. The preprocessing and the decoding here
are the reference a Rust port mirrors; `detect_all.py` imports them.

Usage: python -I -B detect_eval.py <detectors-dir> <labels.tsv> <previews-dir> [--models a,b]
       python -I -B detect_eval.py <detectors-dir> <labels.tsv> <previews-dir> --model m --tensors <dir>
       python -I -B detect_eval.py <detectors-dir> <labels.tsv> <previews-dir> --model m --compare <dir> --conf <c>

<detectors-dir> holds one folder per model (`MODELS`' `file`); <labels.tsv>
has the `frames.tsv` of ball_sample.py next to it; <previews-dir> is the
`riffle-cli previews` export (`<folder>/<stem>.jpg`). --tensors writes each
labeled frame's input tensor (`<id>.f32`) and `meta.tsv` for the riffle-core
test `detector_loads_and_runs_in_tract`, which writes its outputs next to
them; --compare decodes those outputs and onnxruntime's and reports the box
agreement at the confidence given (same class, IoU >= 0.9).
"""

import csv
import os
import sys
import time

import numpy as np
import onnxruntime as ort
from PIL import Image
from sklearn.metrics import roc_auc_score

PERSON, BALL = 0, 32
CLASSES = {PERSON: "person", BALL: "sports ball"}
# The lowest confidence a detection is kept at; the thresholds are chosen above it.
KEEP = 0.05
AGREE_IOU = 0.9

# `conf` is the model's own default display confidence: YOLOX's
# demo/ONNXRuntime `--score_thr`, NanoDet's demo `visualize`, and the
# transformers object-detection pipeline's `threshold` for the DETR exports.
MODELS = {
    "yolox_nano": {"kind": "yolox", "file": "yolox_nano/yolox_nano.onnx", "size": 416, "conf": 0.3},
    "yolox_tiny": {"kind": "yolox", "file": "yolox_tiny/yolox_tiny.onnx", "size": 416, "conf": 0.3},
    "yolox_s": {"kind": "yolox", "file": "yolox_s/yolox_s.onnx", "size": 640, "conf": 0.3},
    "nanodet_plus_m_416": {"kind": "nanodet", "file": "nanodet_plus_m_416/nanodet-plus-m_416.onnx", "size": 416, "conf": 0.35},
    "dfine_n": {"kind": "detr", "file": "dfine_n/model.onnx", "size": 640, "conf": 0.5},
    "dfine_s": {"kind": "detr", "file": "dfine_s/model.onnx", "size": 640, "conf": 0.5},
    "rtdetr_v2_r18": {"kind": "detr", "file": "rtdetr_v2_r18/model.onnx", "size": 640, "conf": 0.5},
}
NMS_IOU = {"yolox": 0.45, "nanodet": 0.6}
NANODET_STRIDES = (8, 16, 32, 64)
NANODET_REG_MAX = 7
NANODET_MEAN = np.array([103.53, 116.28, 123.675], dtype=np.float32)
NANODET_STD = np.array([57.375, 57.12, 58.395], dtype=np.float32)
YOLOX_STRIDES = (8, 16, 32)


def resize_linear(rgb, ow, oh, half_pixel):
    """OpenCV's linear resize (`cv2.resize` INTER_LINEAR with half-pixel
    centers, `cv2.warpAffine` of a scale matrix without), no antialiasing,
    the source clamped at the edges, rounded back to uint8."""
    h, w = rgb.shape[:2]

    def axis(n_out, n_in):
        o = np.arange(n_out, dtype=np.float64)
        s = (o + 0.5) * n_in / n_out - 0.5 if half_pixel else o * n_in / n_out
        s = np.clip(s, 0, n_in - 1)
        i0 = np.floor(s).astype(np.int64)
        i1 = np.minimum(i0 + 1, n_in - 1)
        return i0, i1, (s - i0).astype(np.float32)

    y0, y1, fy = axis(oh, h)
    x0, x1, fx = axis(ow, w)
    img = rgb.astype(np.float32)
    top = img[y0][:, x0] * (1 - fx)[None, :, None] + img[y0][:, x1] * fx[None, :, None]
    bot = img[y1][:, x0] * (1 - fx)[None, :, None] + img[y1][:, x1] * fx[None, :, None]
    out = top * (1 - fy)[:, None, None] + bot * fy[:, None, None]
    return np.clip(np.rint(out), 0, 255).astype(np.uint8)


def preprocess(spec, rgb):
    """The model's input tensor [1, 3, S, S] and the scale (sx, sy) from
    input pixels back to the preview's."""
    s = spec["size"]
    h, w = rgb.shape[:2]
    if spec["kind"] == "yolox":
        # yolox.data.data_augment.preproc: fit inside S x S, top-left, pad 114, BGR 0..255.
        r = min(s / h, s / w)
        rw, rh = int(w * r), int(h * r)
        canvas = np.full((s, s, 3), 114, dtype=np.uint8)
        canvas[:rh, :rw] = resize_linear(rgb, rw, rh, True)
        x = canvas[:, :, ::-1].astype(np.float32)
        return x.transpose(2, 0, 1)[None].copy(), 1 / r, 1 / r
    if spec["kind"] == "nanodet":
        # keep_ratio False: warpPerspective of a scale matrix to S x S, BGR, mean / std.
        x = resize_linear(rgb, s, s, False)[:, :, ::-1].astype(np.float32)
        x = (x - NANODET_MEAN) / NANODET_STD
        return x.transpose(2, 0, 1)[None].astype(np.float32), w / s, h / s
    # RTDetrImageProcessor: PIL bilinear to S x S, RGB, rescale 1/255, no normalization.
    x = np.asarray(Image.fromarray(rgb).resize((s, s), Image.BILINEAR), dtype=np.float32) / 255.0
    return x.transpose(2, 0, 1)[None].copy(), w / s, h / s


def nms(boxes, scores, iou):
    order = np.argsort(-scores, kind="stable")
    keep = []
    while order.size:
        i = order[0]
        keep.append(i)
        rest = order[1:]
        xx0 = np.maximum(boxes[i, 0], boxes[rest, 0])
        yy0 = np.maximum(boxes[i, 1], boxes[rest, 1])
        xx1 = np.minimum(boxes[i, 2], boxes[rest, 2])
        yy1 = np.minimum(boxes[i, 3], boxes[rest, 3])
        inter = np.clip(xx1 - xx0, 0, None) * np.clip(yy1 - yy0, 0, None)
        area = lambda b: (b[..., 2] - b[..., 0]) * (b[..., 3] - b[..., 1])
        order = rest[inter / (area(boxes[i]) + area(boxes[rest]) - inter + 1e-9) <= iou]
    return keep


def decode(spec, outputs, sx, sy):
    """Person and sports ball detections at or above KEEP, after the model's
    NMS, as (class, confidence, x0, y0, x1, y1) in preview pixels."""
    s = spec["size"]
    if spec["kind"] == "detr":
        logits, boxes = outputs
        scores = 1 / (1 + np.exp(-logits[0]))
        cx, cy, bw, bh = boxes[0].T
        xyxy = np.stack([cx - bw / 2, cy - bh / 2, cx + bw / 2, cy + bh / 2], axis=1) * s
        per_class = {c: (xyxy, scores[:, c], None) for c in CLASSES}
    elif spec["kind"] == "yolox":
        out = outputs[0][0].astype(np.float64)
        grids, strides = [], []
        for st in YOLOX_STRIDES:
            n = s // st
            yv, xv = np.meshgrid(np.arange(n), np.arange(n), indexing="ij")
            grids.append(np.stack([xv, yv], axis=2).reshape(-1, 2))
            strides.append(np.full((n * n, 1), st))
        grid, stride = np.concatenate(grids), np.concatenate(strides)
        xy = (out[:, :2] + grid) * stride
        wh = np.exp(out[:, 2:4]) * stride
        xyxy = np.concatenate([xy - wh / 2, xy + wh / 2], axis=1)
        per_class = {c: (xyxy, out[:, 4] * out[:, 5 + c], NMS_IOU["yolox"]) for c in CLASSES}
    else:
        out = outputs[0][0].astype(np.float64)
        centers, strides = [], []
        for st in NANODET_STRIDES:
            n = -(-s // st)
            yv, xv = np.meshgrid(np.arange(n), np.arange(n), indexing="ij")
            centers.append(np.stack([xv, yv], axis=2).reshape(-1, 2) * st)
            strides.append(np.full((n * n, 1), st))
        center, stride = np.concatenate(centers), np.concatenate(strides)
        reg = out[:, 80:].reshape(-1, 4, NANODET_REG_MAX + 1)
        reg = np.exp(reg - reg.max(axis=2, keepdims=True))
        dist = (reg / reg.sum(axis=2, keepdims=True)) @ np.arange(NANODET_REG_MAX + 1) * stride
        xyxy = np.clip(np.concatenate([center - dist[:, :2], center + dist[:, 2:]], axis=1), 0, s)
        per_class = {c: (xyxy, out[:, c], NMS_IOU["nanodet"]) for c in CLASSES}
    dets = []
    for c, (b, sc, iou) in per_class.items():
        idx = np.where(sc >= KEEP)[0]
        if idx.size and iou is not None:
            idx = idx[nms(b[idx], sc[idx], iou)]
        for i in idx[np.argsort(-sc[idx], kind="stable")]:
            dets.append((c, float(sc[i]), b[i, 0] * sx, b[i, 1] * sy, b[i, 2] * sx, b[i, 3] * sy))
    return dets


def session(base, spec, threads=1):
    opts = ort.SessionOptions()
    opts.intra_op_num_threads = threads
    opts.inter_op_num_threads = 1
    # NanoDet's export lists its initializers as graph inputs, which onnxruntime warns about per weight.
    opts.log_severity_level = 3
    return ort.InferenceSession(os.path.join(base, spec["file"]), opts, providers=["CPUExecutionProvider"])


def load_rgb(path):
    with Image.open(path) as im:
        return np.asarray(im.convert("RGB"))


def run(sess, spec, rgb):
    x, sx, sy = preprocess(spec, rgb)
    t = time.perf_counter()
    outputs = sess.run(None, {sess.get_inputs()[0].name: x})
    return decode(spec, outputs, sx, sy), (time.perf_counter() - t) * 1000


def labeled(labels_path, previews):
    """(id, preview path, ball label) per labeled frame with a `ball` label."""
    with open(os.path.join(os.path.dirname(os.path.abspath(labels_path)), "frames.tsv"), newline="", encoding="utf-8") as f:
        frame = {r["id"]: (r["folder"], r["file"]) for r in csv.DictReader(f, delimiter="\t")}
    out = []
    with open(labels_path, newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            ball = (r.get("ball") or "").strip()
            if ball in ("visible", "none"):
                folder, file = frame[r["id"]]
                out.append((r["id"], os.path.join(previews, folder, os.path.splitext(file)[0] + ".jpg"), ball == "visible"))
    return out


def prf(top, y, t):
    pred = top >= t
    tp = int((pred & y).sum())
    recall = tp / max(int(y.sum()), 1)
    precision = tp / max(int(pred.sum()), 1)
    f1 = 2 * precision * recall / max(precision + recall, 1e-9)
    return recall, precision, f1


def best_f1(top, y):
    """The confidence (one of the frames' top ball confidences) with the best F1, ties to the higher one."""
    best = (KEEP, *prf(top, y, KEEP))
    for t in sorted(set(top[top >= KEEP].tolist())):
        r = prf(top, y, t)
        if r[2] >= best[3]:
            best = (t, *r)
    return best


def evaluate(base, names, frames):
    y = np.array([b for _, _, b in frames])
    print(f"{len(frames)} labeled frames ({int(y.sum())} ball visible / {int((~y).sum())} none).\n")
    rows, dist_rows = [], []
    for name in names:
        spec = MODELS[name]
        sess = session(base, spec)
        run(sess, spec, load_rgb(frames[0][1]))
        top, persons, ms = [], [], []
        for _, path, _ in frames:
            dets, t = run(sess, spec, load_rgb(path))
            ms.append(t)
            top.append(max((c for k, c, *_ in dets if k == BALL), default=0.0))
            persons.append(sum(1 for k, c, *_ in dets if k == PERSON and c >= spec["conf"]))
        top, persons = np.array(top), np.array(persons)
        r, p, f = prf(top, y, spec["conf"])
        t, br, bp, bf = best_f1(top, y)
        auc = roc_auc_score(y, top)
        rows.append([f"`{name}`", spec["size"], f"{np.mean(ms):.1f}", f"{auc:.3f}", spec["conf"], f"{r:.3f}", f"{p:.3f}", f"{f:.3f}", f"{t:.3f}", f"{br:.3f}", f"{bp:.3f}", f"{bf:.3f}"])
        bins = [(persons == 0).sum(), (persons == 1).sum(), (persons == 2).sum(), ((persons >= 3) & (persons <= 5)).sum(), ((persons >= 6) & (persons <= 10)).sum(), (persons > 10).sum()]
        dist_rows.append([f"`{name}`", spec["conf"], f"{persons.mean():.2f}", int(np.median(persons))] + [int(b) for b in bins])
        print(f"{name}: done", file=sys.stderr)
    print("| Model | Input | onnxruntime ms/image (1 thread, warm) | Top ball confidence AUC vs `ball` | Default confidence | Recall | Precision | F1 | Best-F1 confidence | Recall | Precision | F1 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for r in rows:
        print("| " + " | ".join(str(v) for v in r) + " |")
    print("\nPersons at the default confidence on the labeled frames:\n")
    print("| Model | Confidence | Mean | Median | 0 | 1 | 2 | 3-5 | 6-10 | > 10 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for r in dist_rows:
        print("| " + " | ".join(str(v) for v in r) + " |")


def write_tensors(spec, frames, out):
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "meta.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("id\tpreview\tsx\tsy\n")
        for fid, path, _ in frames:
            x, sx, sy = preprocess(spec, load_rgb(path))
            x.astype("<f4").tofile(os.path.join(out, f"{int(fid):04d}.f32"))
            f.write(f"{int(fid):04d}\t{path}\t{sx!r}\t{sy!r}\n")


def iou(a, b):
    ix = max(0.0, min(a[2], b[2]) - max(a[0], b[0]))
    iy = max(0.0, min(a[3], b[3]) - max(a[1], b[1]))
    inter = ix * iy
    union = (a[2] - a[0]) * (a[3] - a[1]) + (b[2] - b[0]) * (b[3] - b[1]) - inter
    return inter / union if union > 0 else 0.0


def compare(base, spec, out, conf):
    sess = session(base, spec)
    labels = [o.name for o in sess.get_outputs()]
    shapes = [o.shape for o in sess.get_outputs()]
    frames = agree = boxes = matched = 0
    worst = 1.0
    with open(os.path.join(out, "meta.tsv"), newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            x = np.fromfile(os.path.join(out, r["id"] + ".f32"), dtype="<f4").reshape(1, 3, spec["size"], spec["size"])
            ref = sess.run(None, {sess.get_inputs()[0].name: x})
            got = [np.fromfile(os.path.join(out, f"{r['id']}.{l}.f32"), dtype="<f4").reshape(ref[i].shape) for i, l in enumerate(labels)]
            sx, sy = float(r["sx"]), float(r["sy"])
            a = [d for d in decode(spec, ref, sx, sy) if d[1] >= conf]
            b = [d for d in decode(spec, got, sx, sy) if d[1] >= conf]
            used, ok = set(), len(a) == len(b)
            for d in a:
                cands = [(iou(d[2:], e[2:]), j) for j, e in enumerate(b) if e[0] == d[0] and j not in used]
                best = max(cands, default=(0.0, None))
                if best[1] is not None and best[0] >= AGREE_IOU:
                    used.add(best[1])
                    matched += 1
                    worst = min(worst, best[0])
                else:
                    ok = False
            frames += 1
            boxes += len(a)
            agree += ok
    print(f"outputs {labels} {shapes}")
    print(f"{frames} frames at confidence {conf}: {agree} with the same boxes; {matched} of {boxes} onnxruntime boxes matched by tract at IoU >= {AGREE_IOU} (lowest matched IoU {worst:.4f})")


def main(argv):
    opts = {"--models": ",".join(MODELS), "--model": None, "--tensors": None, "--compare": None, "--conf": None}
    pos, it = [], iter(argv)
    for a in it:
        if a in opts:
            opts[a] = next(it)
        else:
            pos.append(a)
    if len(pos) != 3:
        raise SystemExit(__doc__)
    base, labels_path, previews = pos
    frames = labeled(labels_path, previews)
    if opts["--tensors"]:
        write_tensors(MODELS[opts["--model"]], frames, opts["--tensors"])
    elif opts["--compare"]:
        compare(base, MODELS[opts["--model"]], opts["--compare"], float(opts["--conf"]))
    else:
        evaluate(base, opts["--models"].split(","), frames)


if __name__ == "__main__":
    main(sys.argv[1:])
