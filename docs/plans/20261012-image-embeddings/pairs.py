"""Step 4's labeling material: within-folder file pairs, stratified by the
SigLIP 2 cosine, for the user to label same / different scene, from which
the Show Similar threshold is chosen (similarity.md).

Usage: python -I pairs.py <gate-dir> <out-dir>

<gate-dir> is Step 2's gate folder: `embed.tsv` (path, then the 768
L2-normalized floats, tab-separated) and `features/<folder>.tsv` (the
capture time and subsec per file). Needs numpy, Pillow and `exiftool` on
PATH. Writes into <out-dir>:

- `pairs.tsv`: id, folder, a, b, cosine, bin (`burst` for the anchors).
- `img/<folder>_<stem>.jpg`: each file's embedded preview, upright, 800 px
  on the long edge.
- `label.html`: the labeling page (file://), which shows the pairs in id
  order without the cosine or the file names and exports `labels.tsv`
  (id, label: same / different / unsure).

Pairs never cross folders and never reuse a file. Most exclude two frames of
the same burst (capture gap <= GAP ms, chained), which are trivially the
same scene; ANCHORS within-burst pairs are kept as a check. Ids are given
after a seeded shuffle and the sides are swapped at random, so neither the
id nor the side tells the bin.
"""

import csv
import json
import os
import random
import re
import subprocess
import sys
from datetime import datetime, timezone

import numpy as np
from PIL import Image, ImageOps

SEED = 20261012
GAP = 1000
MIN_FILES = 150
FOLDERS = 8
# [lo, hi) cosine bins and the pairs drawn from each; denser where the
# threshold is likely (the within-folder median is about 0.88).
BINS = [
    (0.70, 0.80, 12),
    (0.80, 0.85, 20),
    (0.85, 0.90, 28),
    (0.90, 0.93, 30),
    (0.93, 0.96, 30),
    (0.96, 1.01, 15),
]
ANCHORS = 15
EDGE = 800


def capture_ms(time, subsec):
    try:
        t = datetime.strptime(time, "%Y:%m:%d %H:%M:%S").replace(tzinfo=timezone.utc)
    except ValueError:
        return None
    s = subsec[:3].ljust(3, "0") if subsec != "-" else ""
    return int(t.timestamp()) * 1000 + (int(s) if re.fullmatch(r"\d{3}", s) else 0)


def load(gate):
    by_folder = {}
    with open(os.path.join(gate, "embed.tsv"), encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            path = parts[0].replace("\\", "/")
            folder = path.split("/")[-2]
            by_folder.setdefault(folder, []).append((path, np.array(parts[1:], dtype=np.float32)))
    folders = {}
    for folder, items in by_folder.items():
        times = {}
        feats = os.path.join(gate, "features", folder + ".tsv")
        if not os.path.exists(feats):
            continue
        with open(feats, newline="", encoding="utf-8") as f:
            for r in csv.DictReader(f, delimiter="\t"):
                times[r["file"]] = capture_ms(r["capture_time"], r["subsec"])
        items = [(p, v, times.get(p.split("/")[-1])) for p, v in items]
        items = [x for x in items if x[2] is not None]
        items.sort(key=lambda x: (x[2], x[0]))
        burst, prev, bursts = -1, None, []
        for _, _, t in items:
            if prev is None or t - prev > GAP:
                burst += 1
            bursts.append(burst)
            prev = t
        folders[folder] = (
            [p for p, _, _ in items],
            np.stack([v for _, v, _ in items]),
            np.array(bursts),
        )
    return folders


def sample(folders, rng):
    eligible = sorted(k for k, (paths, _, _) in folders.items() if len(paths) >= MIN_FILES)
    chosen = sorted(rng.sample(eligible, FOLDERS))
    cands = {}  # (folder, bin index) -> array of (i, j)
    sims = {}
    for k in chosen:
        _, vecs, bursts = folders[k]
        m = vecs @ vecs.T
        sims[k] = m
        i, j = np.triu_indices(len(vecs), 1)
        c = m[i, j]
        cross = bursts[i] != bursts[j]
        for b, (lo, hi, _) in enumerate(BINS):
            sel = cross & (c >= lo) & (c < hi)
            cands[(k, b)] = np.stack([i[sel], j[sel]], axis=1)
        same = ~cross
        cands[(k, "burst")] = np.stack([i[same], j[same]], axis=1)
    used = set()
    pairs = []

    def draw(key, n):
        got = 0
        while got < n:
            ks = [k for k in chosen if len(cands[(k, key)])]
            if not ks:
                raise SystemExit(f"bin {key}: no candidates left")
            k = rng.choice(ks)
            pool = cands[(k, key)]
            for _ in range(50):
                i, j = pool[rng.randrange(len(pool))]
                if (k, i) not in used and (k, j) not in used:
                    break
            else:
                continue
            used.update({(k, i), (k, j)})
            paths = folders[k][0]
            pairs.append([k, paths[i], paths[j], float(sims[k][i, j]), key])
            got += 1

    for b, (_, _, n) in enumerate(BINS):
        draw(b, n)
    draw("burst", ANCHORS)
    rng.shuffle(pairs)
    for p in pairs:
        if rng.random() < 0.5:
            p[1], p[2] = p[2], p[1]
    return chosen, pairs


def img_name(path):
    parts = path.split("/")
    return f"{parts[-2]}_{os.path.splitext(parts[-1])[0]}.jpg"


def previews(paths, out):
    os.makedirs(out, exist_ok=True)
    for path in paths:
        dst = os.path.join(out, img_name(path))
        if os.path.exists(dst):
            continue
        jpeg = subprocess.run(
            ["exiftool", "-b", "-PreviewImage", path], capture_output=True, check=True
        ).stdout
        orient = subprocess.run(
            ["exiftool", "-n", "-s3", "-Orientation", path],
            capture_output=True,
            check=True,
            text=True,
        ).stdout.strip()
        tmp = dst + ".src"
        with open(tmp, "wb") as f:
            f.write(jpeg)
        with Image.open(tmp) as im:
            im = im.convert("RGB")
            # The preview carries no orientation of its own; apply the RAW's.
            ops = {"3": Image.Transpose.ROTATE_180, "6": Image.Transpose.ROTATE_270,
                   "8": Image.Transpose.ROTATE_90}
            if orient in ops:
                im = im.transpose(ops[orient])
            im = ImageOps.contain(im, (EDGE, EDGE), Image.Resampling.LANCZOS)
            im.save(dst, quality=85)
        os.remove(tmp)


PAGE = """<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Similar Pairs</title>
<style>
:root { --bg: #111; --fg: #eee; --muted: #999; --card: #1c1c1c; --line: #333;
  --same: #2e7d32; --diff: #c62828; --unsure: #8d6e00; }
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg);
  font: 14px/1.5 system-ui, "Segoe UI", "Hiragino Sans", "Yu Gothic UI", sans-serif; }
header { padding: 10px 16px; border-bottom: 1px solid var(--line); display: flex;
  flex-wrap: wrap; gap: 8px 24px; align-items: center; }
header .def { color: var(--muted); flex: 1 1 400px; }
#progress { font-variant-numeric: tabular-nums; }
main { padding: 12px 16px; }
.pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.pair img { width: 100%; height: calc(100vh - 210px); object-fit: contain;
  background: var(--card); border-radius: 6px; display: block; }
.meta { margin: 8px 0; color: var(--muted); display: flex; gap: 16px; flex-wrap: wrap; }
#current { font-weight: 600; }
.buttons { display: flex; gap: 8px; flex-wrap: wrap; }
button { font: inherit; color: var(--fg); background: var(--card); border: 1px solid var(--line);
  border-radius: 6px; padding: 8px 14px; cursor: pointer; }
button:hover { border-color: var(--muted); }
button.same { background: var(--same); } button.diff { background: var(--diff); }
button.unsure { background: var(--unsure); }
button.on { outline: 3px solid var(--fg); }
kbd { font: 12px ui-monospace, monospace; border: 1px solid var(--muted); border-radius: 3px;
  padding: 0 4px; margin-left: 4px; }
#done { display: none; padding: 40px 16px; text-align: center; font-size: 16px; }
@media (max-width: 700px) { .pair { grid-template-columns: 1fr; }
  .pair img { height: 40vh; } }
</style>
</head>
<body>
<header>
  <strong>類似ペアのラベル付け</strong>
  <span class="def">「同じ」の定義: 取捨選択で1枚だけ残す入れ替え可能な候補（同じ選手の同じプレー、または同じ種類の動き）。同じ試合の別のプレーは「違う」。迷ったら「わからない」。</span>
  <span id="progress"></span>
  <button id="export">TSV を書き出す</button>
</header>
<main id="main">
  <div class="pair"><img id="a" alt="左"><img id="b" alt="右"></div>
  <div class="meta"><span id="current"></span><span id="folder"></span><span id="label"></span></div>
  <div class="buttons">
    <button class="same" data-label="same">同じシーン<kbd>S</kbd></button>
    <button class="diff" data-label="different">違うシーン<kbd>D</kbd></button>
    <button class="unsure" data-label="unsure">わからない<kbd>U</kbd></button>
    <button id="back">戻る<kbd>&larr;</kbd><kbd>Backspace</kbd></button>
    <button id="next">次へ<kbd>&rarr;</kbd></button>
    <button id="first">最初の未ラベルへ</button>
  </div>
</main>
<div id="done">全ペアのラベル付けが終わりました。「TSV を書き出す」で labels.tsv を保存してください。<br><br>
  <button id="review">見直す</button></div>
<script>
const PAIRS = __PAIRS__;
const KEY = "riffle-similar-pairs-v1";
const NAMES = { same: "同じシーン", different: "違うシーン", unsure: "わからない" };
let labels = {};
try { labels = JSON.parse(localStorage.getItem(KEY) || "{}") || {}; } catch (e) { labels = {}; }
function save() { try { localStorage.setItem(KEY, JSON.stringify(labels)); } catch (e) {} }
const firstUnlabeled = () => { const i = PAIRS.findIndex(p => !labels[p.id]); return i < 0 ? PAIRS.length : i; };
let pos = firstUnlabeled();
const $ = id => document.getElementById(id);
function render() {
  const n = Object.keys(labels).filter(id => PAIRS.some(p => String(p.id) === id)).length;
  $("progress").textContent = `ラベル済み ${n} / ${PAIRS.length}`;
  const done = pos >= PAIRS.length;
  $("main").style.display = done ? "none" : "";
  $("done").style.display = done ? "block" : "none";
  if (done) return;
  const p = PAIRS[pos];
  $("a").src = "img/" + p.a; $("b").src = "img/" + p.b;
  $("current").textContent = `#${p.id}（${pos + 1} 件目）`;
  $("folder").textContent = p.folder;
  $("label").textContent = labels[p.id] ? "現在: " + NAMES[labels[p.id]] : "未ラベル";
  document.querySelectorAll("button[data-label]").forEach(b =>
    b.classList.toggle("on", b.dataset.label === labels[p.id]));
  for (const k of [1, 2]) { const q = PAIRS[pos + k]; if (q) { new Image().src = "img/" + q.a; new Image().src = "img/" + q.b; } }
}
function mark(label) {
  if (pos >= PAIRS.length) return;
  labels[PAIRS[pos].id] = label; save(); pos++; render();
}
function go(d) { pos = Math.max(0, Math.min(PAIRS.length, pos + d)); render(); }
function exportTsv() {
  const rows = ["id\\tlabel"].concat(PAIRS.filter(p => labels[p.id]).map(p => p.id + "\\t" + labels[p.id]));
  const url = URL.createObjectURL(new Blob([rows.join("\\n") + "\\n"], { type: "text/tab-separated-values" }));
  const a = document.createElement("a"); a.href = url; a.download = "labels.tsv";
  document.body.appendChild(a); a.click(); a.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}
document.querySelectorAll("button[data-label]").forEach(b => b.onclick = () => mark(b.dataset.label));
$("back").onclick = () => go(-1); $("next").onclick = () => go(1);
$("first").onclick = () => { pos = firstUnlabeled(); render(); };
$("review").onclick = () => { pos = 0; render(); };
$("export").onclick = exportTsv;
document.addEventListener("keydown", e => {
  if (e.ctrlKey || e.metaKey || e.altKey) return;
  const k = e.key.toLowerCase();
  if (k === "s") mark("same"); else if (k === "d") mark("different"); else if (k === "u") mark("unsure");
  else if (k === "arrowleft" || k === "backspace") { e.preventDefault(); go(-1); }
  else if (k === "arrowright") go(1);
});
render();
</script>
</body>
</html>
"""


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    gate, out = sys.argv[1], sys.argv[2]
    rng = random.Random(SEED)
    chosen, pairs = sample(load(gate), rng)
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "pairs.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("id\tfolder\ta\tb\tcosine\tbin\n")
        for n, (k, a, b, c, key) in enumerate(pairs, 1):
            label = "burst" if key == "burst" else f"{BINS[key][0]:.2f}-{min(BINS[key][1], 1):.2f}"
            f.write(f"{n}\t{k}\t{a}\t{b}\t{c:.4f}\t{label}\n")
    previews(sorted({p for _, a, b, _, _ in pairs for p in (a, b)}), os.path.join(out, "img"))
    data = [
        {"id": n, "folder": k, "a": img_name(a), "b": img_name(b)}
        for n, (k, a, b, _, _) in enumerate(pairs, 1)
    ]
    with open(os.path.join(out, "label.html"), "w", encoding="utf-8", newline="\n") as f:
        f.write(PAGE.replace("__PAIRS__", json.dumps(data)))
    counts = {}
    for _, _, _, _, key in pairs:
        counts[key] = counts.get(key, 0) + 1
    print(f"{len(pairs)} pairs from {len(chosen)} folders: {', '.join(chosen)}")
    for b, (lo, hi, _) in enumerate(BINS):
        print(f"  [{lo:.2f}, {min(hi, 1):.2f}): {counts.get(b, 0)}")
    print(f"  burst anchors: {counts.get('burst', 0)}")


if __name__ == "__main__":
    main()
