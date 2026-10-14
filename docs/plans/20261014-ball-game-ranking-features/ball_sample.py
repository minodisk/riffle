"""Bar 4's hand-label sample: one frame from each of 240 scorable bursts of a
gate directory, 200 from sports bursts and 40 from the rest, for the user to
label `ball` and `contest` (plan.md, Pre-registered criteria).

Usage: python -I ball_sample.py <gate-dir> <out-dir>

<gate-dir> is read by the archived gate.py's own `load` / `build`, so the
bursts (gap 1000 ms), the scorable ones and the burst's genre (the argmax of
its frames' mean top-level softmax, the `genres.tsv` scores) are the gate's.
The bursts and the frame within each are drawn with a fixed seed; ids are
given after a seeded shuffle, so the id does not tell the stratum. Needs
`exiftool` on PATH. Writes into <out-dir>:

- `frames.tsv`: id, folder, file, path, stratum, the burst's sports
  probability.
- `img/<folder>_<stem>.jpg`: each frame's embedded preview, upright, 1600 px
  on the long edge (the archived genre_sample.py's `previews`).
- `labels.tsv`: the template, `id\\tball\\tcontest` with empty labels
  (written only when absent, so labels already entered are kept).
- `label.html`: the labeling page (file://), which shows the frames in id
  order without any score or file name and exports `labels.tsv` (ball
  `visible` / `none` / `unsure`, contest `yes` / `no` / `na`).
"""

import json
import os
import sys

import numpy as np

sys.dont_write_bytecode = True
sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_archived", "20261012-image-embeddings"),
)
import gate  # noqa: E402
import genre_sample  # noqa: E402

SEED = 20261014
SPORTS = 200
OTHER = 40
EDGE = 1600


def paths_of(gate_dir):
    out = {}
    with open(os.path.join(gate_dir, "embed.tsv"), encoding="utf-8") as f:
        for line in f:
            path = line.split("\t", 1)[0].removeprefix("\\\\?\\").replace("\\", "/")
            p = path.split("/")
            out[(p[-2], p[-1])] = path
    return out


PAGE = """<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ball Labels</title>
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
  background: var(--card); border-radius: 6px; display: block; cursor: zoom-in; }
#photo.zoom { object-fit: none; cursor: zoom-out; }
.meta { margin: 8px 0; color: var(--muted); display: flex; gap: 16px; flex-wrap: wrap; }
#current { font-weight: 600; color: var(--fg); }
#warn { color: var(--warn); }
h3 { margin: 4px 0 6px; font-size: 14px; }
.q { display: flex; flex-direction: column; gap: 4px; margin-bottom: 14px; }
.q label { display: grid; grid-template-columns: 22px 1fr auto; gap: 2px 8px; align-items: start;
  padding: 6px 8px; border: 1px solid var(--line); border-radius: 6px; cursor: pointer; background: var(--card); }
.q label.on { border-color: var(--on); background: #0d2a4a; }
.q input { margin: 3px 0 0; }
.q .name { font-weight: 600; }
.q .def { grid-column: 2 / 4; color: var(--muted); font-size: 12px; }
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
  <strong>ボールと競り合いのラベル付け</strong>
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
    <div id="questions"></div>
    <div class="buttons">
      <button id="back">戻る <kbd>&larr;</kbd></button>
      <button id="next" class="primary">決定して次へ <kbd>Enter</kbd></button>
    </div>
    <p class="help">両方の質問に 1 つずつ答えて <kbd>Enter</kbd> で次へ進みます。写真をクリックすると等倍表示になります（小さなボールの確認用）。<kbd>&rarr;</kbd> は答えずに次へ進みます。ラベルはブラウザに自動保存されます。最後に「TSV を書き出す」で labels.tsv を保存し、このフォルダーの labels.tsv を置き換えてください。</p>
  </div>
</main>
<div id="done">全フレームのラベル付けが終わりました。「TSV を書き出す」で labels.tsv を保存してください。<br><br>
  <button id="review">見直す</button></div>
<script>
const FRAMES = __FRAMES__;
const QUESTIONS = [
  { id: "ball", title: "ボール", options: [
    { id: "visible", name: "写っている", key: "z", def: "競技のボールがフレーム内にある（手の中・空中・床の上、大きさやブレは問わない）。一部でも見えれば「写っている」。" },
    { id: "none", name: "写っていない", key: "x", def: "ボールがフレーム内にない。" },
    { id: "unsure", name: "判断できない", key: "c", def: "ボールらしきものはあるが確信が持てない。" },
  ] },
  { id: "contest", title: "競り合い", options: [
    { id: "yes", name: "あり", key: "a", def: "ボールを持っている（操っている）選手と、相手チームの選手が少なくとも 1 人、両方フレーム内にいる。" },
    { id: "no", name: "なし", key: "s", def: "球技の場面だが、ボール保持者がいない・写っていない、または保持者の相手選手が写っていない。" },
    { id: "na", name: "該当しない", key: "d", def: "球技の場面ではない、またはチームや保持者が判断できない。" },
  ] },
];
const KEY = "riffle-ball-labels-v1";
let labels = {};
try { labels = JSON.parse(localStorage.getItem(KEY) || "{}") || {}; } catch (e) { labels = {}; }
function save() { try { localStorage.setItem(KEY, JSON.stringify(labels)); } catch (e) {} }
const done = id => labels[id] && labels[id].ball && labels[id].contest;
const firstUnlabeled = () => { const i = FRAMES.findIndex(f => !done(f.id)); return i < 0 ? FRAMES.length : i; };
let pos = firstUnlabeled();
const $ = id => document.getElementById(id);
const boxes = {};
for (const q of QUESTIONS) {
  const h = document.createElement("h3"); h.textContent = q.title; $("questions").appendChild(h);
  const box = document.createElement("div"); box.className = "q"; $("questions").appendChild(box);
  for (const o of q.options) {
    const el = document.createElement("label");
    const input = document.createElement("input");
    input.type = "radio"; input.name = q.id;
    input.onchange = () => choose(q.id, o.id);
    const name = document.createElement("span"); name.className = "name"; name.textContent = o.name;
    const key = document.createElement("kbd"); key.textContent = o.key.toUpperCase();
    const def = document.createElement("span"); def.className = "def"; def.textContent = o.def;
    el.append(input, name, key, def);
    box.appendChild(el);
    boxes[q.id + ":" + o.id] = { input, el };
  }
}
function choose(q, o) {
  if (pos >= FRAMES.length) return;
  const id = FRAMES[pos].id;
  labels[id] = Object.assign({}, labels[id], { [q]: o });
  save(); $("warn").textContent = ""; render();
}
function render() {
  const n = FRAMES.filter(f => done(f.id)).length;
  $("progress").textContent = `ラベル済み ${n} / ${FRAMES.length}`;
  const end = pos >= FRAMES.length;
  $("main").style.display = end ? "none" : "";
  $("done").style.display = end ? "block" : "none";
  if (end) return;
  const f = FRAMES[pos];
  $("photo").src = "img/" + encodeURIComponent(f.img);
  $("photo").classList.remove("zoom");
  $("current").textContent = `#${f.id}（${pos + 1} 件目）`;
  $("state").textContent = done(f.id) ? "ラベル済み" : "未ラベル";
  const cur = labels[f.id] || {};
  for (const q of QUESTIONS) for (const o of q.options) {
    const b = boxes[q.id + ":" + o.id]; const on = cur[q.id] === o.id;
    b.input.checked = on; b.el.classList.toggle("on", on);
  }
  for (const k of [1, 2]) { const q = FRAMES[pos + k]; if (q) new Image().src = "img/" + encodeURIComponent(q.img); }
}
function go(d) { pos = Math.max(0, Math.min(FRAMES.length, pos + d)); $("warn").textContent = ""; render(); }
function commit() {
  if (pos >= FRAMES.length) return;
  if (!done(FRAMES[pos].id)) { $("warn").textContent = "両方の質問に答えてください"; return; }
  go(1);
}
function exportTsv() {
  const rows = ["id\\tball\\tcontest"].concat(FRAMES.map(f => { const l = labels[f.id] || {}; return f.id + "\\t" + (l.ball || "") + "\\t" + (l.contest || ""); }));
  const url = URL.createObjectURL(new Blob([rows.join("\\n") + "\\n"], { type: "text/tab-separated-values" }));
  const a = document.createElement("a"); a.href = url; a.download = "labels.tsv";
  document.body.appendChild(a); a.click(); a.remove(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}
$("photo").onclick = () => $("photo").classList.toggle("zoom");
$("back").onclick = () => go(-1); $("next").onclick = commit;
$("first").onclick = () => { pos = firstUnlabeled(); $("warn").textContent = ""; render(); };
$("review").onclick = () => { pos = 0; render(); };
$("export").onclick = exportTsv;
document.addEventListener("keydown", e => {
  if (e.ctrlKey || e.metaKey || e.altKey) return;
  const k = e.key.toLowerCase();
  for (const q of QUESTIONS) for (const o of q.options) if (o.key === k) { e.preventDefault(); choose(q.id, o.id); return; }
  if (k === "enter") { e.preventDefault(); commit(); }
  else if (k === "arrowleft" || k === "backspace") { e.preventDefault(); go(-1); }
  else if (k === "arrowright") { e.preventDefault(); go(1); }
});
render();
</script>
</body>
</html>
"""


def main(gate_dir, out):
    frames, _ = gate.load(gate_dir)
    d = gate.build(frames, gate.load_text(os.path.join(gate_dir, "genre_text.tsv")))
    paths = paths_of(gate_dir)
    si = d["genre_names"].index("sports")
    sc = gate.scorable(d)
    sports = [b for b in sc if int(np.argmax(d["genre"][d["bursts"][b][1]])) == si]
    other = [b for b in sc if int(np.argmax(d["genre"][d["bursts"][b][1]])) != si]
    rng = np.random.default_rng(SEED)
    chosen = []
    for stratum, pool, n in (("sports", sports, SPORTS), ("other", other, OTHER)):
        if len(pool) < n:
            raise SystemExit(f"{stratum}: only {len(pool)} scorable bursts for {n}")
        for b in rng.choice(pool, size=n, replace=False):
            _, s, e = d["bursts"][b]
            chosen.append((int(rng.integers(s, e)), stratum))
    rng.shuffle(chosen)

    os.makedirs(out, exist_ok=True)
    rows = []
    with open(os.path.join(out, "frames.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("id\tfolder\tfile\tpath\tstratum\tsports\n")
        for n, (i, stratum) in enumerate(chosen, 1):
            r = d["rows"][i]
            path = paths[(r["folder"], r["file"])]
            rows.append(path)
            f.write(f"{n}\t{r['folder']}\t{r['file']}\t{path}\t{stratum}\t{d['genre'][i, si]:.4f}\n")
    genre_sample.EDGE = EDGE
    genre_sample.previews(rows, os.path.join(out, "img"))
    template = os.path.join(out, "labels.tsv")
    if not os.path.exists(template):
        with open(template, "w", encoding="utf-8", newline="") as f:
            f.write("id\tball\tcontest\n" + "".join(f"{n}\t\t\n" for n in range(1, len(rows) + 1)))
    data = [{"id": n, "img": genre_sample.img_name(p)} for n, p in enumerate(rows, 1)]
    with open(os.path.join(out, "label.html"), "w", encoding="utf-8", newline="\n") as f:
        f.write(PAGE.replace("__FRAMES__", json.dumps(data)))
    folders = {}
    for i, _ in chosen:
        folders[d["rows"][i]["folder"]] = folders.get(d["rows"][i]["folder"], 0) + 1
    print(f"{len(rows)} frames ({SPORTS} sports, {OTHER} other) from {len(folders)} folders, of {len(sports)} sports / {len(other)} other scorable bursts")
    print("folders: " + ", ".join(f"{k} {v}" for k, v in sorted(folders.items())))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main(*sys.argv[1:])
