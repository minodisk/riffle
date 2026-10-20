"""Step 1: the burst-relative feature set `B` on every dump frame, label-free.

Usage: python -I -B burstfeat.py <app-dump-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`. Writes `burst_features.tsv`
(`folder`, `file`, one column per feature, NaN where undefined) into
<out-dir>, prints the label-free facts (Markdown) and saves them as
`burstfeat.out.md` there. Reads no label and no model score.
"""

import csv
import math
import os
import sys
import time

import numpy as np

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

rf, gate, folder_gate = bf.rf, bf.gate, bf.folder_gate

COLUMNS = ["nn_cos", "sharp_z", "sharp_rank", "ef_gap", "ef_rank", "ear_gap", "ear_rank"]
QUANTILES = (0, 5, 25, 50, 75, 95, 100)


def gap_rank(v, s, e):
    """Per frame of the burst `s:e`: its value minus the highest of the other frames, and the share of the others higher."""
    gap, rank = np.full(e - s, np.nan), np.full(e - s, np.nan)
    x = v[s:e]
    for i in range(e - s):
        if np.isnan(x[i]):
            continue
        others = np.delete(x, i)
        others = others[~np.isnan(others)]
        if len(others):
            gap[i] = x[i] - others.max()
            rank[i] = (others > x[i]).mean()
    return gap, rank


def features(d, ear):
    """The 7 `B` columns of every dump frame, in `d["rows"]` order."""
    n = len(d["rows"])
    out = np.full((n, len(COLUMNS)), np.nan)
    sharp = d["sharp"]
    ef = np.array([np.nan if f["ef"] is None else gate.logit(f["ef"]) for f in d["rows"]])
    for _, s, e in d["bursts"]:
        if e - s < 2:
            continue
        cos = d["emb"][s:e] @ d["emb"][s:e].T
        np.fill_diagonal(cos, -np.inf)
        out[s:e, 0] = cos.max(axis=1)
        x = sharp[s:e]
        have = ~np.isnan(x)
        if have.sum() >= 2:
            sd = x[have].std()
            out[s:e, 1] = np.where(have, (x - x[have].mean()) / sd if sd > 0 else 0.0, np.nan)
        _, out[s:e, 2] = gap_rank(sharp, s, e)
        out[s:e, 3], out[s:e, 4] = gap_rank(ef, s, e)
        out[s:e, 5], out[s:e, 6] = gap_rank(ear, s, e)
    return out


def ranks(x):
    """Average ranks, ties sharing their mean rank."""
    order = np.argsort(x, kind="stable")
    r = np.empty(len(x))
    r[order] = np.arange(len(x))
    _, inv, cnt = np.unique(x, return_inverse=True, return_counts=True)
    sums = np.bincount(inv, weights=r)
    return (sums / cnt)[inv]


def spearman(x, y):
    return float(np.corrcoef(ranks(x), ranks(y))[0, 1])


def fmt(x):
    return "NaN" if math.isnan(x) else f"{x:.4f}"


def main(dump, out_dir):
    start = time.time()
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    ear = rf.load_ear(dump, d)
    rows, n = d["rows"], len(d["rows"])
    b = features(d, ear)

    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "burst_features.tsv"), "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f, delimiter="\t", lineterminator="\n")
        w.writerow(["folder", "file"] + COLUMNS)
        for r, v in zip(rows, b):
            w.writerow([r["folder"], r["file"]] + [repr(float(x)) if not np.isnan(x) else "NaN" for x in v])

    print("# Burst-relative features `B`\n")
    print(f"{n} dump frames ({missing} without an embedding dropped by `gate.load`), {len(d['bursts'])} bursts, "
          f"{int(d['single'].sum())} singles.\n")

    print("## Frames with a value\n")
    gate.table("| Column | Frames with a value | Share % | NaN |",
               [(f"`{c}`", int((~np.isnan(b[:, j])).sum()), f"{100 * (~np.isnan(b[:, j])).mean():.1f}",
                 int(np.isnan(b[:, j]).sum())) for j, c in enumerate(COLUMNS)])

    print("## Quantiles over the dump\n")
    head = "| Column | " + " | ".join(f"q{q}" for q in QUANTILES) + " | Mean |"
    qrows = []
    for j, c in enumerate(COLUMNS):
        v = b[~np.isnan(b[:, j]), j]
        qrows.append([f"`{c}`"] + [fmt(x) for x in np.percentile(v, QUANTILES)] + [fmt(v.mean())])
    gate.table(head, qrows)
    gate.table("| Column | Share = 0 % | Share > 0 % |",
               [(f"`{c}`", f"{100 * (b[~np.isnan(b[:, j]), j] == 0).mean():.1f}",
                 f"{100 * (b[~np.isnan(b[:, j]), j] > 0).mean():.1f}") for j, c in enumerate(COLUMNS) if c.endswith("_rank")])

    print("## Spearman correlation with the burst-relative base columns of `G0`\n")
    base = [("`tech[:, 0]` log(sharp / burst top)", d["tech"][:, 0]), ("`context[:, 0]` log burst size", d["context"][:, 0]),
            ("`context[:, 1]` position", d["context"][:, 1]), ("`context[:, 2]` first-of-burst", d["context"][:, 2])]
    srows = []
    for j, c in enumerate(COLUMNS):
        row = [f"`{c}`"]
        for _, y in base:
            ok = ~np.isnan(b[:, j]) & ~np.isnan(y)
            row.append(f"{spearman(b[ok, j], y[ok]):+.3f}")
        srows.append(row)
    gate.table("| Column | " + " | ".join(k for k, _ in base) + " |", srows)
    print("Over the frames where both values exist.\n")

    print("## Checks\n")
    keys = [(r["folder"], r["file"]) for r in rows]
    with open(os.path.join(out_dir, "burst_features.tsv"), encoding="utf-8", newline="") as f:
        written = [(r["folder"], r["file"]) for r in csv.DictReader(f, delimiter="\t")]
    sr = b[:, 2]
    have = ~np.isnan(sr)
    top = d["tech"][:, 0] == 0
    checks = [
        ("Every (`folder`, `file`) of the dump has one row", len(written) == n and set(written) == set(keys) and len(set(written)) == n),
        ("`sharp_rank` is 0 exactly where `tech[:, 0]` is 0 (frames with a `sharp_rank`)", bool(np.array_equal(sr[have] == 0, top[have]))),
        ("`nn_cos` is NaN exactly on the singles", bool(np.array_equal(np.isnan(b[:, 0]), d["single"]))),
    ]
    gate.table("| Check | Result |", [(k, "pass" if v else "FAIL") for k, v in checks])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "burstfeat.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    if not all(v for _, v in checks):
        raise SystemExit("a check failed")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:3])
