"""Run one model of detect_eval.py's shortlist (onnxruntime, its
preprocessing and decoding) on every preview of a `riffle-cli previews`
export and write `detections.tsv`.

Usage: python -I -B detect_all.py <detectors-dir> <model> <previews-dir> <detections.tsv> [--jobs n]

<previews-dir> holds the export's `list.tsv` (`path\\tjpeg\\twidth\\theight`).
`detections.tsv` has a header and one line per detection,
`folder\\tfile\\twidth\\theight\\tclass\\tconfidence\\tx0\\ty0\\tx1\\ty1` (class
`person` or `sports ball`, confidence at or above detect_eval's `KEEP`, the
box in preview pixels), and one line with class `-` and empty values for a
frame without any, so every frame of the export is in the file.
"""

import csv
import os
import sys
import time
from multiprocessing import Pool

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import detect_eval  # noqa: E402

_state = {}


def init(base, name):
    spec = detect_eval.MODELS[name]
    _state["spec"] = spec
    _state["sess"] = detect_eval.session(base, spec)


def work(row):
    path, jpeg = row
    dets, _ = detect_eval.run(_state["sess"], _state["spec"], detect_eval.load_rgb(jpeg))
    return path, dets


def main(argv):
    opts = {"--jobs": "8"}
    pos, it = [], iter(argv)
    for a in it:
        if a in opts:
            opts[a] = next(it)
        else:
            pos.append(a)
    if len(pos) != 4:
        raise SystemExit(__doc__)
    base, name, previews, out = pos
    with open(os.path.join(previews, "list.tsv"), newline="", encoding="utf-8") as f:
        rows = [(r[0], r[1], r[2], r[3]) for r in csv.reader(f, delimiter="\t")]
    size = {p: (w, h) for p, _, w, h in rows}
    start = time.time()
    n = 0
    with Pool(int(opts["--jobs"]), initializer=init, initargs=(base, name)) as pool, open(out, "w", encoding="utf-8", newline="") as f:
        f.write("folder\tfile\twidth\theight\tclass\tconfidence\tx0\ty0\tx1\ty1\n")
        for path, dets in pool.imap(work, [(p, j) for p, j, _, _ in rows], chunksize=16):
            folder, file = os.path.basename(os.path.dirname(path)), os.path.basename(path)
            w, h = size[path]
            head = f"{folder}\t{file}\t{w}\t{h}"
            if not dets:
                f.write(f"{head}\t-\t\t\t\t\t\n")
            for c, conf, x0, y0, x1, y1 in dets:
                f.write(f"{head}\t{detect_eval.CLASSES[c]}\t{conf:.5f}\t{x0:.1f}\t{y0:.1f}\t{x1:.1f}\t{y1:.1f}\n")
            n += 1
            if n % 2000 == 0:
                print(f"{n} / {len(rows)} frames, {time.time() - start:.0f} s", file=sys.stderr, flush=True)
    print(f"{n} frames with {name} in {time.time() - start:.0f} s", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv[1:])
