"""Step 2's labeling material: the cross-burst pairs of the pool folders,
stratified by SigLIP 2 cosine x capture gap, plus within-burst anchors, as
pre-registered in groups.md ("Pair sampling").

Usage: python -I scene_pairs.py <dump-dir> [<dump-dir>...] <out-dir>

Each <dump-dir> holds `embed.tsv` and `features/<folder>.tsv` (read by the
archived pairs.py `load`) and optionally `genres.tsv`. Needs numpy, Pillow
and `exiftool` on PATH. Writes into <out-dir>:

- `pairs.tsv`: id, folder, a, b, cosine, gap_s, cosine_bin, gap_bin
  (`burst` in both bins for the anchors).
- `img/<folder>_<stem>.jpg` and `label.html`, as pairs.py writes them, with
  the non-sports definition and its own localStorage key.

Prints the per-cell counts drawn, the shortfalls and the sports share.
"""

import csv
import json
import os
import random
import subprocess
import sys

import numpy as np
from PIL import Image, ImageOps

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "_archived", "20261012-image-embeddings"))
import pairs  # noqa: E402

SEED = 20261015
POOL = [
    "2026-08-16", "2026-08-15", "2026-08-08", "2026-08-02", "2026-07-31",
    "2026-07-30", "2026-07-26", "2026-07-24", "2026-07-23", "2026-07-22",
    "2026-07-21", "2026-07-11", "2026-07-02", "2026-09-05", "2026-08-29-l",
]
EXCLUDED_FROM = r"D:\Photos\tests\2026-10-12-similar-pairs\pairs.tsv"
EXCLUDED_FOLDER = "2026-08-08"
COS = [(0.70, 0.80), (0.80, 0.85), (0.85, 0.90), (0.90, 0.93), (0.93, 0.96), (0.96, 1.01)]
GAPS = [(None, 60), (60, 300), (300, 1800), (1800, None)]
GAP_NAMES = ["<=60s", "60-300s", "300-1800s", ">1800s"]
COUNTS = [
    [4, 4, 4, 4],
    [6, 6, 6, 6],
    [10, 10, 10, 10],
    [16, 12, 8, 4],
    [18, 12, 5, 5],
    [18, 6, 0, 1],
]
ANCHORS = 15
ROTATE = {"3": Image.Transpose.ROTATE_180, "6": Image.Transpose.ROTATE_270,
          "8": Image.Transpose.ROTATE_90}

DEFINITION = (
    "「同じ」の定義: 取捨選択で1枚だけ残す入れ替え可能な候補。"
    "同じ場所・同じ構図（風景や街並みを少しフレーミングし直した、露出を変えた、少し後に撮り直した）、"
    "または同じ被写体の同じポーズ・同じ動作（ポートレートの同じポーズ、同じ場所で同じことをしている子ども）。"
    "同じ場所の別の構図（引きと寄り）、別のポーズや動作、別の被写体、動きのある場面で両方残したい別の瞬間は「違う」。"
    "迷ったら「わからない」。"
)


def excluded():
    names = set()
    with open(EXCLUDED_FROM, newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            if r["folder"] == EXCLUDED_FOLDER:
                names.update(p.replace("\\", "/").split("/")[-1] for p in (r["a"], r["b"]))
    return names


def pool(dumps):
    folders = {}
    for d in dumps:
        folders.update(pairs.load(d))
    drop = excluded()
    out = {}
    for k in POOL:
        if k not in folders:
            raise SystemExit(f"{k}: not in the dumps")
        paths, vecs, bursts = folders[k]
        keep = [i for i, p in enumerate(paths) if k != EXCLUDED_FOLDER or p.split("/")[-1] not in drop]
        if k == EXCLUDED_FOLDER and len(paths) - len(keep) != len(drop):
            raise SystemExit(f"{k}: {len(paths) - len(keep)} of {len(drop)} excluded files found")
        out[k] = ([paths[i] for i in keep], vecs[keep].astype(np.float64), bursts[keep])
    return out


def times_of(dumps, folder, paths):
    for d in dumps:
        feats = os.path.join(d, "features", folder + ".tsv")
        if os.path.exists(feats):
            with open(feats, newline="", encoding="utf-8") as f:
                t = {r["file"]: pairs.capture_ms(r["capture_time"], r["subsec"])
                     for r in csv.DictReader(f, delimiter="\t")}
            return np.array([t[p.split("/")[-1]] for p in paths], dtype=np.int64)
    raise SystemExit(f"{folder}: no features")


def candidates(folders, dumps):
    cands, sims, gaps = {}, {}, {}
    for k, (paths, vecs, bursts) in folders.items():
        m = vecs @ vecs.T
        t = times_of(dumps, k, paths)
        i, j = np.triu_indices(len(paths), 1)
        c, g = m[i, j], np.abs(t[i] - t[j])
        cross = bursts[i] != bursts[j]
        for b, (lo, hi) in enumerate(COS):
            for w, (glo, ghi) in enumerate(GAPS):
                sel = cross & (c >= lo) & (c < hi)
                if glo is not None:
                    sel &= g > glo * 1000
                if ghi is not None:
                    sel &= g <= ghi * 1000
                cands[(k, b, w)] = np.stack([i[sel], j[sel]], axis=1)
        cands[(k, "burst")] = np.stack([i[~cross], j[~cross]], axis=1)
        sims[k], gaps[k] = m, (t[:, None] - t[None, :])
    return cands, sims, gaps


def draw(folders, cands, sims, gaps, rng):
    used = {k: np.zeros(len(p), dtype=bool) for k, (p, _, _) in folders.items()}
    drawn, short = [], {}

    def usable(key):
        out = {}
        for k in POOL:
            c = cands[(k,) + key]
            if len(c):
                c = c[~used[k][c[:, 0]] & ~used[k][c[:, 1]]]
                if len(c):
                    out[k] = c
        return out

    def take(key, n):
        for got in range(n):
            left = usable(key)
            if not left:
                short[key] = n - got
                return
            k = rng.choice(sorted(left))
            i, j = left[k][rng.randrange(len(left[k]))]
            used[k][[i, j]] = True
            paths = folders[k][0]
            drawn.append([k, paths[i], paths[j], float(sims[k][i, j]),
                          abs(int(gaps[k][i, j])) / 1000, key])

    for b in reversed(range(len(COS))):
        for w in reversed(range(len(GAPS))):
            if COUNTS[b][w]:
                take((b, w), COUNTS[b][w])
    take(("burst",), ANCHORS)
    rng.shuffle(drawn)
    for p in drawn:
        if rng.random() < 0.5:
            p[1], p[2] = p[2], p[1]
    return drawn, short


def dng_previews(paths, out):
    # exiftool's -PreviewImage of a Leica DNG is the 160 px thumbnail; its
    # 2112 px JpgFromRaw is the preview. pairs.previews skips existing files.
    os.makedirs(out, exist_ok=True)
    for path in paths:
        dst = os.path.join(out, pairs.img_name(path))
        if not path.lower().endswith(".dng") or os.path.exists(dst):
            continue
        tmp = dst + ".src"
        with open(tmp, "wb") as f:
            f.write(subprocess.run(["exiftool", "-b", "-JpgFromRaw", path],
                                   capture_output=True, check=True).stdout)
        orient = subprocess.run(["exiftool", "-n", "-s3", "-IFD0:Orientation", path],
                                capture_output=True, check=True, text=True).stdout.strip()
        with Image.open(tmp) as im:
            im = im.convert("RGB")
            if orient in ROTATE:
                im = im.transpose(ROTATE[orient])
            ImageOps.contain(im, (pairs.EDGE, pairs.EDGE),
                             Image.Resampling.LANCZOS).save(dst, quality=85)
        os.remove(tmp)


def top_genres(dumps):
    top = {}
    for d in dumps:
        path = os.path.join(d, "genres.tsv")
        if os.path.exists(path):
            with open(path, newline="", encoding="utf-8") as f:
                for r in csv.DictReader(f, delimiter="\t"):
                    soft = {c[8:]: float(v) for c, v in r.items() if c.startswith("softmax ")}
                    top[(r["folder"], r["file"])] = max(soft, key=soft.get)
    return top


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    dumps, out = sys.argv[1:-1], sys.argv[-1]
    rng = random.Random(SEED)
    folders = pool(dumps)
    cands, sims, gaps = candidates(folders, dumps)
    drawn, short = draw(folders, cands, sims, gaps, rng)
    os.makedirs(out, exist_ok=True)

    def bins(key):
        if key == ("burst",):
            return "burst", "burst"
        lo, hi = COS[key[0]]
        return f"{lo:.2f}-{min(hi, 1):.2f}", GAP_NAMES[key[1]]

    with open(os.path.join(out, "pairs.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("id\tfolder\ta\tb\tcosine\tgap_s\tcosine_bin\tgap_bin\n")
        for n, (k, a, b, c, g, key) in enumerate(drawn, 1):
            f.write(f"{n}\t{k}\t{a}\t{b}\t{c:.4f}\t{g:.3f}\t" + "\t".join(bins(key)) + "\n")
    files = sorted({p for _, a, b, _, _, _ in drawn for p in (a, b)})
    dng_previews(files, os.path.join(out, "img"))
    pairs.previews(files, os.path.join(out, "img"))
    data = [{"id": n, "folder": k, "a": pairs.img_name(a), "b": pairs.img_name(b)}
            for n, (k, a, b, _, _, _) in enumerate(drawn, 1)]
    page = (pairs.PAGE.replace("__PAIRS__", json.dumps(data))
            .replace('"riffle-similar-pairs-v1"', '"riffle-scene-pairs-v1"')
            .replace("類似ペアのラベル付け", "同じシーンのラベル付け")
            .replace("<title>Similar Pairs</title>", "<title>Scene Pairs</title>"))
    start = page.index('<span class="def">') + len('<span class="def">')
    page = page[:start] + DEFINITION + page[page.index("</span>", start):]
    with open(os.path.join(out, "label.html"), "w", encoding="utf-8", newline="\n") as f:
        f.write(page)

    counts = {}
    for *_, key in drawn:
        counts[key] = counts.get(key, 0) + 1
    print(f"{len(drawn)} pairs, {len(files)} files; pool "
          + ", ".join(f"{k} {len(folders[k][0])}" for k in POOL))
    print("cosine \\ gap\t" + "\t".join(GAP_NAMES) + "\ttotal")
    for b in range(len(COS)):
        row = [counts.get((b, w), 0) for w in range(len(GAPS))]
        print(f"{bins((b, 0))[0]}\t" + "\t".join(map(str, row)) + f"\t{sum(row)}")
    print(f"anchors\t{counts.get(('burst',), 0)}")
    for key, n in short.items():
        print(f"short {bins(key)}: {n}")
    print("candidates (pairs) per cell:")
    for b in range(len(COS)):
        print(f"{bins((b, 0))[0]}\t" + "\t".join(
            str(sum(len(cands[(k, b, w)]) for k in POOL)) for w in range(len(GAPS))))
    top = top_genres(dumps)
    kinds = {"sports": 0, "unknown": 0, "other": 0}
    for k, a, b, *_ in drawn:
        g = [top.get((k, p.split("/")[-1])) for p in (a, b)]
        kinds["sports" if "sports" in g else "unknown" if None in g else "other"] += 1
    print(f"sports pairs {kinds['sports']}, without genre scores {kinds['unknown']}, "
          f"other {kinds['other']}")
    per = {}
    for k, *_ in drawn:
        per[k] = per.get(k, 0) + 1
    print("pairs per folder: " + ", ".join(f"{k} {per.get(k, 0)}" for k in POOL))


if __name__ == "__main__":
    main()
