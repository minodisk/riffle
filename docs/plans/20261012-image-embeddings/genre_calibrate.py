"""Per-label thresholds of the genre data file, calibrated on the hand labels.

Usage: python -I genre_calibrate.py <genres.json> <label-dir> <embed.tsv>...

<genres.json> is genre_vectors.py's output (thresholds may be null).
<label-dir> holds genre_sample.py's `frames.tsv` (id -> path) and the
labeling page's `labels.tsv` (id, comma-separated label names, `none` for a
frame whose subject cannot be told). The ground truth of Unknown is a frame
none of the file's labels applies to: `none`, and `other` or a dropped
genre when the file does not list it.
Each <embed.tsv> is `riffle-cli embed --out` output (path, 768 floats); the
labeled frames are looked up in all of them.

A label is on when its cosine reaches its threshold or one of its
sub-genres is on (genre.rs's rule); Unknown is on when no label is. Each
sub-genre's threshold is chosen first, then each parent's given its
sub-genres, as the one maximizing F1 over the labeled frames (ties: the
higher threshold), printed rounded down to 4 decimals for the file. Prints,
as Markdown:

- the label counts;
- per label, at the threshold the file holds (if any) and at the chosen
  one: precision, recall, F1, and the same under leave-one-folder-out (each
  folder's frames judged at the threshold chosen on the other folders), so
  the optimism of choosing on the same 300 frames is visible;
- with `--library`, how many frames of all the given embed.tsv files each
  label is on for at the file's thresholds.
"""

import json
import math
import sys
from collections import Counter

import numpy as np


def load_labels(label_dir):
    paths = {}
    with open(f"{label_dir}/frames.tsv", encoding="utf-8") as f:
        next(f)
        for line in f:
            parts = line.rstrip("\n").split("\t")
            paths[parts[0]] = parts[3]
    rows = []
    with open(f"{label_dir}/labels.tsv", encoding="utf-8") as f:
        next(f)
        for line in f:
            if not line.strip():
                continue
            fid, names = line.rstrip("\n").split("\t")
            rows.append((paths[fid], set(names.split(","))))
    return rows


def load_vectors(files, wanted):
    """The vectors of `wanted` paths, or of every path when `wanted` is None."""
    out = {}
    for name in files:
        with open(name, encoding="utf-8") as f:
            for line in f:
                path, rest = line.split("\t", 1)
                path = path.replace("\\", "/")
                if wanted is None or path in wanted:
                    out[path] = np.array(rest.split("\t"), dtype=np.float32)
    return out


def metrics(pred, truth):
    tp = int(np.sum(pred & truth))
    fp = int(np.sum(pred & ~truth))
    fn = int(np.sum(~pred & truth))
    p = tp / (tp + fp) if tp + fp else float("nan")
    r = tp / (tp + fn) if tp + fn else float("nan")
    f1 = 2 * tp / (2 * tp + fp + fn) if tp else 0.0
    return p, r, f1, tp, fp


class Scorer:
    def __init__(self, data, cos):
        self.labels = [e["label"] for e in data["labels"]]
        self.parent = {e["label"]: e["parent"] for e in data["labels"]}
        self.children = {l: [c for c in self.labels if self.parent[c] == l] for l in self.labels}
        self.cos = cos

    def on(self, label, th, rows):
        """Whether `label` is on per row, given thresholds `th`."""
        pred = self.cos[rows, self.labels.index(label)] >= th[label]
        for c in self.children[label]:
            pred = pred | self.on(c, th, rows)
        return pred

    def order(self):
        """Sub-genres before their parents."""
        return [l for l in self.labels if self.parent[l]] + [
            l for l in self.labels if not self.parent[l]
        ]

    def choose(self, truth, rows):
        th = {}
        for label in self.order():
            col = self.cos[rows, self.labels.index(label)]
            best = (-1.0, None)
            for t in sorted(set(col.tolist()) | {1.0}, reverse=True):
                th[label] = t
                f1 = metrics(self.on(label, th, rows), truth[label][rows])[2]
                if f1 > best[0]:
                    best = (f1, t)
            th[label] = best[1]
        return th


def main(argv):
    library = "--library" in argv
    argv = [a for a in argv if a != "--library"]
    genres, label_dir, embeds = argv[0], argv[1], argv[2:]
    with open(genres, encoding="utf-8") as f:
        data = json.load(f)
    rows = load_labels(label_dir)
    vec = load_vectors(embeds, {p for p, _ in rows})
    missing = [p for p, _ in rows if p not in vec]
    if missing:
        raise SystemExit(f"no vector for {missing[:5]} ({len(missing)})")
    e = np.stack([vec[p] for p, _ in rows])
    e /= np.linalg.norm(e, axis=1, keepdims=True)
    t = np.array([x["vector"] for x in data["labels"]], dtype=np.float32)
    s = Scorer(data, e @ t.T)
    n = len(rows)
    every = np.arange(n)
    folders = np.array([p.split("/")[-2] for p, _ in rows])
    truth = {l: np.array([l in names for _, names in rows]) for l in s.labels}
    unknown = ~np.any([truth[l] for l in s.labels], axis=0)

    counts = Counter(x for _, names in rows for x in names)
    print(f"## Labels\n\n{n} frames from {len(set(folders))} folders.\n")
    print("| Label | Frames |\n|---|---|")
    for name, c in counts.most_common():
        print(f"| {name} | {c} |")
    for name in s.labels:
        if name not in counts:
            print(f"| {name} | 0 |")

    held = {l: np.zeros(n, dtype=bool) for l in s.labels}
    held_unknown = np.zeros(n, dtype=bool)
    for folder in sorted(set(folders)):
        test = np.flatnonzero(folders == folder)
        th = s.choose(truth, np.flatnonzero(folders != folder))
        for l in s.labels:
            held[l][test] = s.on(l, th, test)
        any_on = np.zeros(len(test), dtype=bool)
        for l in s.labels:
            any_on |= held[l][test]
        held_unknown[test] = ~any_on

    fixed = None
    if all(x["threshold"] is not None for x in data["labels"]):
        fixed = {x["label"]: x["threshold"] for x in data["labels"]}
    chosen = s.choose(truth, every)

    def row(name, th_col, pred, true, fixed_pred, held_pred):
        p, r, f1, tp, fp = metrics(pred, true)
        hp, hr, hf1, _, _ = metrics(held_pred, true)
        out = f"| {name} | {int(true.sum())} | {th_col} | {p:.2f} | {r:.2f} | {f1:.2f} | {tp} / {fp} | {hp:.2f} | {hr:.2f} | {hf1:.2f} |"
        if fixed_pred is not None:
            fp_, fr, ff1, _, _ = metrics(fixed_pred, true)
            out += f" {fp_:.2f} | {fr:.2f} | {ff1:.2f} |"
        return out

    print("\n## Per label\n")
    head = "| Label | Positives | Chosen threshold | P | R | F1 | TP / FP | P (LOFO) | R (LOFO) | F1 (LOFO) |"
    sep = "|---|---|---|---|---|---|---|---|---|---|"
    if fixed:
        head += " P (file) | R (file) | F1 (file) |"
        sep += "---|---|---|"
    print(head + "\n" + sep)
    any_on = np.zeros(n, dtype=bool)
    any_fixed = np.zeros(n, dtype=bool)
    for l in s.labels:
        pred = s.on(l, chosen, every)
        any_on |= pred
        fixed_pred = s.on(l, fixed, every) if fixed else None
        if fixed:
            any_fixed |= fixed_pred
        print(row(l, f"{math.floor(chosen[l] * 1e4) / 1e4:.4f}", pred, truth[l], fixed_pred, held[l]))
    print(row("Unknown", "-", ~any_on, unknown, ~any_fixed if fixed else None, held_unknown))

    if library and fixed:
        print("\n## Library\n")
        allv = load_vectors(embeds, None)
        m = np.stack(list(allv.values()))
        m /= np.linalg.norm(m, axis=1, keepdims=True)
        lib = Scorer(data, m @ t.T)
        idx = np.arange(len(m))
        any_lib = np.zeros(len(m), dtype=bool)
        print(f"{len(m)} frames.\n\n| Label | Frames on | Share |\n|---|---|---|")
        for l in s.labels:
            pred = lib.on(l, fixed, idx)
            any_lib |= pred
            print(f"| {l} | {int(pred.sum())} | {pred.mean():.1%} |")
        print(f"| Unknown | {int((~any_lib).sum())} | {(~any_lib).mean():.1%} |")


if __name__ == "__main__":
    if len(sys.argv) < 4:
        raise SystemExit(__doc__)
    main(sys.argv[1:])
