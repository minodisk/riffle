"""Step 5's labeling material: about 300 frames, stratified by the
provisional SigLIP 2 genre scores, for the user to label with genres, from
which the per-label thresholds and `Unknown` are calibrated (genres.md).

Usage: python -I genre_sample.py <genre_prompts.json> <genre_text.tsv> <out-dir> <vec-dir>...

Each <vec-dir> holds `embed.tsv` (path, then the 768 L2-normalized floats,
tab-separated) and `features/<folder>.tsv` (the capture time and subsec per
file), the layout of Step 2's gate folder (gate_dump.sh). Folders outside
the gate's 27 get the same layout from `riffle-cli features <dir>` and
`riffle-cli embed <model> siglip2 <dir>... --near 404 --out embed.tsv`.
`genre_text.tsv` is genre_text.py's output. Needs numpy, Pillow and
`exiftool` on PATH. Writes into <out-dir>:

- `frames.tsv`: id, folder, file, path, the provisional top-level genre and
  its per-frame softmax, the highest-sigmoid label and its sigmoid, and the
  stratum it was drawn for.
- `img/<folder>_<stem>.jpg`: each frame's embedded preview, upright, 800 px
  on the long edge.
- `label.html`: the labeling page (file://), which shows the frames in id
  order without any score or file name and exports `labels.tsv` (id, the
  comma-separated label ids of genre_prompts.json, or `none`).

The scores are those of gate.py (SCALE * cosine + BIAS, sigmoid per label;
softmax over the top-level labels), per frame rather than burst-averaged.
About 88% of the gate frames are sports, so each non-sports label gets its
own quota drawn from the frames ranked highest by that label's sigmoid
(widening the rank window until the quota fills), then a share of
low-confidence frames (Unknown candidates) and a random share of sports.
A frame is taken only if its burst (capture gap <= GAP ms, chained) has no
frame yet, it is not a near duplicate (cosine > NEAR) of a frame already
taken from its folder, and its folder has fewer than FOLDER_CAP frames in
the stratum. Ids are given after a seeded shuffle, so the id does not tell
the stratum.
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
NEAR = 0.95
SCALE = 111.8326
BIAS = -16.7669
EDGE = 800
FOLDER_CAP = 4
WINDOW = 5  # the rank window starts at WINDOW * quota and widens by as much
LOW = 0.5  # top-level softmax below this is a low-confidence frame
# (stratum, quota), drawn in this order so the rarest labels pick first.
QUOTAS = [
    ("birds / animals", 22),
    ("pets", 22),
    ("macro", 22),
    ("landscape", 22),
    ("camping", 22),
    ("soccer", 22),
    ("group photo", 22),
    ("other", 22),
    ("portrait", 25),
    ("snap", 32),
    ("low confidence", 22),
    ("sports (random)", 45),
]
NAMES = {
    "sports": "スポーツ",
    "basketball": "バスケットボール",
    "soccer": "サッカー",
    "portrait": "ポートレート",
    "group photo": "集合写真",
    "landscape": "風景",
    "birds / animals": "野鳥・動物",
    "pets": "ペット",
    "snap": "スナップ",
    "camping": "キャンプ",
    "macro": "マクロ",
    "other": "その他",
}
DEFS = {
    "sports": "競技・試合・練習の場面。選手のプレーが主役。",
    "basketball": "バスケットボールの試合・練習。",
    "soccer": "サッカーの試合・練習。",
    "portrait": "特定の人物を主役にした写真（ポーズの有無は問わない）。",
    "group photo": "複数人がカメラに向かって並んだ・集まった記念写真。",
    "landscape": "山・海・空・街並みなど景色そのものが主役。",
    "birds / animals": "野鳥や動物（動物園・野生）が主役。",
    "pets": "飼っている犬・猫などのペットが主役。",
    "snap": "日常や街の何気ない場面。人や出来事を気軽に切り取ったもの。",
    "camping": "テント・焚き火・キャンプ場などキャンプの場面。",
    "macro": "花・虫・小物などに極端に寄った接写。",
    "other": "上のどれでもないが被写体ははっきりある（料理、物、建物、乗り物、書類など）。",
}
KEYS = "1234567890qw"


def capture_ms(time, subsec):
    try:
        t = datetime.strptime(time, "%Y:%m:%d %H:%M:%S").replace(tzinfo=timezone.utc)
    except ValueError:
        return None
    s = subsec[:3].ljust(3, "0") if subsec != "-" else ""
    return int(t.timestamp()) * 1000 + (int(s) if re.fullmatch(r"\d{3}", s) else 0)


def load(dirs):
    """Returns paths, folders, the stacked vectors and a burst id per frame."""
    paths, folders, vecs, bursts = [], [], [], []
    for d in dirs:
        by_folder = {}
        with open(os.path.join(d, "embed.tsv"), encoding="utf-8") as f:
            for line in f:
                parts = line.rstrip("\n").split("\t")
                path = parts[0].replace("\\", "/")
                by_folder.setdefault(path.split("/")[-2], []).append(
                    (path, np.array(parts[1:], dtype=np.float32))
                )
        for folder, items in sorted(by_folder.items()):
            times = {}
            feats = os.path.join(d, "features", folder + ".tsv")
            if os.path.exists(feats):
                with open(feats, newline="", encoding="utf-8") as f:
                    for r in csv.DictReader(f, delimiter="\t"):
                        times[r["file"]] = capture_ms(r["capture_time"], r["subsec"])
            items = [(p, v, times.get(p.split("/")[-1])) for p, v in items]
            # Frames without a capture time sort last, each its own burst.
            items.sort(key=lambda x: (x[2] is None, x[2] or 0, x[0]))
            prev = None
            for p, v, t in items:
                if t is None or prev is None or t - prev > GAP:
                    bursts.append(len(bursts) and bursts[-1] + 1)
                else:
                    bursts.append(bursts[-1])
                prev = t
                paths.append(p)
                folders.append(folder)
                vecs.append(v)
    vecs = np.stack(vecs)
    vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)
    return paths, folders, vecs, np.array(bursts)


def load_text(path, labels):
    vec = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            vec[parts[0]] = np.array(parts[2:], dtype=np.float64)
    missing = [l for l in labels if l not in vec]
    if missing:
        raise SystemExit(f"{path}: no vector for {missing}")
    t = np.stack([vec[l] for l in labels])
    return t / np.linalg.norm(t, axis=1, keepdims=True)


def sample(paths, folders, vecs, bursts, labels, parents, tvec, rng):
    cos = vecs.astype(np.float64) @ tvec.T
    sig = 1 / (1 + np.exp(-(SCALE * cos + BIAS)))
    top_idx = [i for i, l in enumerate(labels) if parents[l] is None]
    z = SCALE * cos[:, top_idx]
    soft = np.exp(z - z.max(axis=1, keepdims=True))
    soft /= soft.sum(axis=1, keepdims=True)
    top = np.array([top_idx[i] for i in soft.argmax(axis=1)])
    top_p = soft.max(axis=1)
    sports = labels.index("sports")

    used, used_bursts, picked_in = set(), set(), {}
    chosen = []

    def try_take(i, stratum, cap):
        if i in used or bursts[i] in used_bursts:
            return False
        if sum(1 for j, s in chosen if s == stratum and folders[j] == folders[i]) >= cap:
            return False
        same = picked_in.get(folders[i], [])
        if same and float(np.max(vecs[same] @ vecs[i])) > NEAR:
            return False
        used.add(i)
        used_bursts.add(bursts[i])
        picked_in.setdefault(folders[i], []).append(i)
        chosen.append((i, stratum))
        return True

    for stratum, quota in QUOTAS:
        if stratum == "low confidence":
            pool = [int(i) for i in np.flatnonzero(top_p < LOW)]
            order = [pool]
        elif stratum == "sports (random)":
            pool = [int(i) for i in np.flatnonzero(top == sports)]
            order = [pool]
        else:
            ranked = [int(i) for i in np.argsort(-sig[:, labels.index(stratum)], kind="stable")]
            w = WINDOW * quota
            order = [ranked[s : s + w] for s in range(0, len(ranked), w)]
        got = 0
        for chunk in order:
            chunk = chunk[:]
            rng.shuffle(chunk)
            for i in chunk:
                if got == quota:
                    break
                if try_take(i, stratum, FOLDER_CAP):
                    got += 1
            if got == quota:
                break
        if got < quota:
            raise SystemExit(f"{stratum}: only {got} of {quota}")
    rng.shuffle(chosen)
    rows = []
    for i, stratum in chosen:
        best = int(np.argmax(sig[i]))
        rows.append(
            {
                "folder": folders[i],
                "file": paths[i].split("/")[-1],
                "path": paths[i],
                "top": labels[top[i]],
                "top_softmax": float(top_p[i]),
                "best": labels[best],
                "best_sigmoid": float(sig[i, best]),
                "stratum": stratum,
            }
        )
    return rows


def img_name(path):
    parts = path.split("/")
    return f"{parts[-2]}_{os.path.splitext(parts[-1])[0]}.jpg"


def previews(paths, out):
    os.makedirs(out, exist_ok=True)
    for path in paths:
        dst = os.path.join(out, img_name(path))
        if os.path.exists(dst):
            continue
        jpeg = b""
        for tag in ("-PreviewImage", "-JpgFromRaw", "-OtherImage"):
            jpeg = subprocess.run(["exiftool", "-b", tag, path], capture_output=True).stdout
            if jpeg:
                break
        if not jpeg:
            raise SystemExit(f"{path}: no embedded preview")
        orient = subprocess.run(
            ["exiftool", "-n", "-s3", "-IFD0:Orientation", path],
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
<title>Genre Labels</title>
<style>
:root { --bg: #111; --fg: #eee; --muted: #999; --card: #1c1c1c; --line: #333; --on: #1565c0; --warn: #c62828; }
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg);
  font: 14px/1.5 system-ui, "Segoe UI", "Hiragino Sans", "Yu Gothic UI", sans-serif; }
header { padding: 10px 16px; border-bottom: 1px solid var(--line); display: flex;
  flex-wrap: wrap; gap: 8px 24px; align-items: center; }
#progress { font-variant-numeric: tabular-nums; }
main { padding: 12px 16px; display: grid; grid-template-columns: minmax(0, 1fr) 360px; gap: 16px; }
#photo { width: 100%; height: calc(100vh - 140px); object-fit: contain;
  background: var(--card); border-radius: 6px; display: block; }
.meta { margin: 8px 0; color: var(--muted); display: flex; gap: 16px; flex-wrap: wrap; }
#current { font-weight: 600; color: var(--fg); }
#warn { color: var(--warn); }
.labels { display: flex; flex-direction: column; gap: 4px; }
.labels label { display: grid; grid-template-columns: 22px 1fr auto; gap: 2px 8px; align-items: start;
  padding: 6px 8px; border: 1px solid var(--line); border-radius: 6px; cursor: pointer; background: var(--card); }
.labels label.sub { margin-left: 24px; }
.labels label.on { border-color: var(--on); background: #0d2a4a; }
.labels label.none { margin-top: 8px; }
.labels input { margin: 3px 0 0; }
.labels .name { font-weight: 600; }
.labels .def { grid-column: 2 / 4; color: var(--muted); font-size: 12px; }
.buttons { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 12px; }
button { font: inherit; color: var(--fg); background: var(--card); border: 1px solid var(--line);
  border-radius: 6px; padding: 8px 14px; cursor: pointer; }
button:hover { border-color: var(--muted); }
button.primary { background: var(--on); }
kbd { font: 12px ui-monospace, monospace; border: 1px solid var(--muted); border-radius: 3px; padding: 0 4px; }
.help { color: var(--muted); font-size: 12px; margin-top: 12px; }
#done { display: none; padding: 40px 16px; text-align: center; font-size: 16px; }
@media (max-width: 800px) { main { grid-template-columns: 1fr; } #photo { height: 50vh; } }
</style>
</head>
<body>
<header>
  <strong>ジャンルのラベル付け</strong>
  <span id="progress"></span>
  <button id="first">最初の未ラベルへ</button>
  <button id="export">TSV を書き出す</button>
</header>
<main id="main">
  <div>
    <img id="photo" alt="写真">
    <div class="meta"><span id="current"></span><span id="state"></span><span id="warn"></span></div>
  </div>
  <div>
    <div class="labels" id="labels"></div>
    <div class="buttons">
      <button id="back">戻る <kbd>&larr;</kbd></button>
      <button id="next" class="primary">決定して次へ <kbd>Enter</kbd></button>
    </div>
    <p class="help">当てはまるものをすべて選びます（複数可）。サブジャンル（字下げされた項目）を選ぶと親ジャンルにも自動で印が付きます。親を外すとそのサブジャンルも外れます。「その他」は被写体ははっきりしているが上のどれでもない写真、「どれにも当てはまらない」は何が写っているか判別できない写真（真っ暗・激しいブレ・誤シャッターなど）です。<kbd>&rarr;</kbd> は選ばずに次へ進みます。</p>
  </div>
</main>
<div id="done">全フレームのラベル付けが終わりました。「TSV を書き出す」で labels.tsv を保存してください。<br><br>
  <button id="review">見直す</button></div>
<script>
const FRAMES = __FRAMES__;
const LABELS = __LABELS__;
const NONE = { id: "none", name: "どれにも当てはまらない", def: "何が写っているか判別できない（真っ暗、激しいブレ、誤シャッター、地面だけなど）。", key: "n", parent: null };
const ALL = LABELS.concat([NONE]);
const KEY = "riffle-genre-labels-v1";
let labels = {};
try { labels = JSON.parse(localStorage.getItem(KEY) || "{}") || {}; } catch (e) { labels = {}; }
function save() { try { localStorage.setItem(KEY, JSON.stringify(labels)); } catch (e) {} }
const has = id => Array.isArray(labels[id]) && labels[id].length > 0;
const firstUnlabeled = () => { const i = FRAMES.findIndex(f => !has(f.id)); return i < 0 ? FRAMES.length : i; };
let pos = firstUnlabeled();
const $ = id => document.getElementById(id);
const boxes = {};
for (const l of ALL) {
  const el = document.createElement("label");
  el.className = (l.parent ? "sub" : "") + (l.id === "none" ? " none" : "");
  const box = document.createElement("input");
  box.type = "checkbox";
  box.onchange = () => toggle(l.id, box.checked);
  const name = document.createElement("span"); name.className = "name"; name.textContent = l.name;
  const key = document.createElement("kbd"); key.textContent = l.key.toUpperCase();
  const def = document.createElement("span"); def.className = "def"; def.textContent = l.def;
  el.append(box, name, key, def);
  $("labels").appendChild(el);
  boxes[l.id] = { box, el };
}
function toggle(label, on) {
  if (pos >= FRAMES.length) return;
  const id = FRAMES[pos].id;
  const set = new Set(labels[id] || []);
  const entry = ALL.find(l => l.id === label);
  if (on) {
    set.add(label);
    if (label === "none") { for (const l of LABELS) set.delete(l.id); }
    else { set.delete("none"); if (entry.parent) set.add(entry.parent); }
  } else {
    set.delete(label);
    for (const l of LABELS) if (l.parent === label) set.delete(l.id);
  }
  const order = ALL.map(l => l.id).filter(x => set.has(x));
  if (order.length) labels[id] = order; else delete labels[id];
  save(); $("warn").textContent = ""; render();
}
function render() {
  const n = FRAMES.filter(f => has(f.id)).length;
  $("progress").textContent = `ラベル済み ${n} / ${FRAMES.length}`;
  const done = pos >= FRAMES.length;
  $("main").style.display = done ? "none" : "";
  $("done").style.display = done ? "block" : "none";
  if (done) return;
  const f = FRAMES[pos];
  $("photo").src = "img/" + encodeURIComponent(f.img);
  $("current").textContent = `#${f.id}（${pos + 1} 件目）`;
  $("state").textContent = has(f.id) ? "ラベル済み" : "未ラベル";
  const set = new Set(labels[f.id] || []);
  for (const l of ALL) { const b = boxes[l.id]; b.box.checked = set.has(l.id); b.el.classList.toggle("on", set.has(l.id)); }
  for (const k of [1, 2]) { const q = FRAMES[pos + k]; if (q) new Image().src = "img/" + encodeURIComponent(q.img); }
}
function go(d) { pos = Math.max(0, Math.min(FRAMES.length, pos + d)); $("warn").textContent = ""; render(); }
function commit() {
  if (pos >= FRAMES.length) return;
  if (!has(FRAMES[pos].id)) { $("warn").textContent = "1 つ以上選んでください"; return; }
  go(1);
}
function exportTsv() {
  const rows = ["id\\tlabels"].concat(FRAMES.filter(f => has(f.id)).map(f => f.id + "\\t" + labels[f.id].join(",")));
  const url = URL.createObjectURL(new Blob([rows.join("\\n") + "\\n"], { type: "text/tab-separated-values" }));
  const a = document.createElement("a"); a.href = url; a.download = "labels.tsv";
  document.body.appendChild(a); a.click(); a.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}
$("back").onclick = () => go(-1); $("next").onclick = commit;
$("first").onclick = () => { pos = firstUnlabeled(); $("warn").textContent = ""; render(); };
$("review").onclick = () => { pos = 0; render(); };
$("export").onclick = exportTsv;
document.addEventListener("keydown", e => {
  if (e.ctrlKey || e.metaKey || e.altKey) return;
  const k = e.key.toLowerCase();
  const l = ALL.find(x => x.key === k);
  if (l) { e.preventDefault(); toggle(l.id, !(labels[FRAMES[pos] && FRAMES[pos].id] || []).includes(l.id)); }
  else if (k === "enter") { e.preventDefault(); commit(); }
  else if (k === "arrowleft" || k === "backspace") { e.preventDefault(); go(-1); }
  else if (k === "arrowright") { e.preventDefault(); go(1); }
});
render();
</script>
</body>
</html>
"""


def main():
    if len(sys.argv) < 5:
        raise SystemExit(__doc__)
    prompts, text, out, dirs = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
    with open(prompts, encoding="utf-8") as f:
        entries = json.load(f)["labels"]
    labels = [e["label"] for e in entries]
    parents = {e["label"]: e["parent"] for e in entries}
    unnamed = [l for l in labels if l not in NAMES or l not in DEFS]
    if unnamed or len(labels) > len(KEYS):
        raise SystemExit(f"no Japanese name / definition / key for {unnamed}")
    rng = random.Random(SEED)
    paths, folders, vecs, bursts = load(dirs)
    rows = sample(paths, folders, vecs, bursts, labels, parents, load_text(text, labels), rng)
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "frames.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("id\tfolder\tfile\tpath\ttop\ttop_softmax\tbest\tbest_sigmoid\tstratum\n")
        for n, r in enumerate(rows, 1):
            f.write(
                f"{n}\t{r['folder']}\t{r['file']}\t{r['path']}\t{r['top']}\t{r['top_softmax']:.4f}"
                f"\t{r['best']}\t{r['best_sigmoid']:.4g}\t{r['stratum']}\n"
            )
    previews([r["path"] for r in rows], os.path.join(out, "img"))
    # Parents first, each followed by its sub-genres.
    order = [l for l in labels if parents[l] is None]
    order = [x for l in order for x in [l] + [s for s in labels if parents[s] == l]]
    page_labels = [
        {"id": l, "name": NAMES[l], "def": DEFS[l], "key": KEYS[i], "parent": parents[l]}
        for i, l in enumerate(order)
    ]
    data = [{"id": n, "img": img_name(r["path"])} for n, r in enumerate(rows, 1)]
    with open(os.path.join(out, "label.html"), "w", encoding="utf-8", newline="\n") as f:
        f.write(
            PAGE.replace("__FRAMES__", json.dumps(data))
            .replace("__LABELS__", json.dumps(page_labels, ensure_ascii=False))
        )
    print(f"{len(rows)} frames from {len(set(r['folder'] for r in rows))} folders")
    for key, title in (("stratum", "stratum"), ("top", "provisional top-level genre"), ("best", "highest-sigmoid label")):
        counts = {}
        for r in rows:
            counts[r[key]] = counts.get(r[key], 0) + 1
        print(f"by {title}: " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items(), key=lambda x: -x[1])))
    folder_counts = {}
    for r in rows:
        folder_counts[r["folder"]] = folder_counts.get(r["folder"], 0) + 1
    print("folders: " + ", ".join(f"{k} {v}" for k, v in sorted(folder_counts.items())))


if __name__ == "__main__":
    main()
