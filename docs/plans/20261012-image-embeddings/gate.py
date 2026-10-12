"""The Step 2 ranking gate: within-burst pick vs not-chosen pairs, a pairwise
logistic on the SigLIP 2 embedding (PCA 32) plus the technical features and
the position in the burst, with and without per-genre offsets, held out by
folder. The design and the criterion are pre-registered in gate.md.

Usage: python -I gate.py <gate-dir> [jobs]

<gate-dir> holds `features/<folder>.tsv` (+ `.output.txt`), `embed.tsv` and
`genre_text.tsv` (gate_dump.sh, genre_text.py). Writes `genres.tsv` (the
per-file sigmoid score of every label and the top-level softmax) there and
prints the result tables of gate.md (Markdown) on stdout.
"""

import csv
import math
import os
import re
import sys
from datetime import datetime, timezone

import numpy as np
from joblib import Parallel, delayed
from sklearn.decomposition import PCA
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import GroupKFold

GAP = 1000
SCALE = 111.8326
BIAS = -16.7669
PCA_DIMS = 32
CS = (0.01, 0.1, 1.0, 10.0)
OFFSET_SCALE = 1 / math.sqrt(10)
MIN_SPREAD_BURSTS = 20
AUC_BAR = 0.703
CURVE = (5, 10, 20)
CURVE_DRAWS = 3

GOOD_EYE_FOCUS = 0.9
GOOD_EYE_EAR = 0.25
GOOD_MAX_YAW = 35
GOOD_MAX_PITCH = 45
MAX_EYE_OFFSET = 0.1
MIN_EDGE_GAP = 0.02

TECH = [
    "ln rel sharpness",
    "`eye_focus` logit",
    "eyes open",
    "\\|yaw\\|",
    "\\|pitch\\|",
    "\\|roll\\|",
    "`photoTier` good",
    "no `eye_focus`",
    "no eyes judgment",
    "no pose",
]
POSITION = ["position", "first frame"]


# ---------------------------------------------------------------- data


def base(stem):
    return re.sub(r"-DxO_.*$", "", stem)


def num(v):
    return None if v in ("-", "err", "") else float(v)


def capture_ms(time, subsec):
    try:
        t = datetime.strptime(time, "%Y:%m:%d %H:%M:%S").replace(tzinfo=timezone.utc)
    except ValueError:
        return None
    s = subsec[:3].ljust(3, "0") if subsec != "-" else ""
    return int(t.timestamp()) * 1000 + (int(s) if re.fullmatch(r"\d{3}", s) else 0)


def tier(r):
    ef, ear, yaw, pitch = num(r["eye_focus"]), num(r["ear"]), num(r["yaw"]), num(r["pitch"])
    off, gap = num(r["eye_offset"]), num(r["edge_gap"])
    return (
        r["state"] == "Candidate"
        and ef is not None
        and ef >= GOOD_EYE_FOCUS
        and ear is not None
        and ear >= GOOD_EYE_EAR
        and yaw is not None
        and abs(yaw) <= GOOD_MAX_YAW
        and pitch is not None
        and abs(pitch) <= GOOD_MAX_PITCH
        and off is not None
        and off <= MAX_EYE_OFFSET
        and gap is not None
        and gap >= MIN_EDGE_GAP
    )


def load_embed(path):
    vecs = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            p = parts[0].replace("\\", "/").split("/")
            vecs[(p[-2], p[-1])] = np.array(parts[1:], dtype=np.float32)
    return vecs


def load_text(path):
    names, parents, vecs = [], [], []
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            names.append(parts[0])
            parents.append(None if parts[1] == "-" else parts[1])
            vecs.append(np.array(parts[2:], dtype=np.float64))
    return names, parents, np.array(vecs)


def load(gate):
    vecs = load_embed(os.path.join(gate, "embed.tsv"))
    feats = os.path.join(gate, "features")
    frames, missing = [], 0
    for name in sorted(os.listdir(feats)):
        if not name.endswith(".tsv"):
            continue
        folder = name[:-4]
        with open(os.path.join(feats, name), newline="", encoding="utf-8") as f:
            rows = list(csv.DictReader(f, delimiter="\t"))
        by = {}
        for r in rows:
            stem = os.path.splitext(r["file"])[0]
            b = base(stem)
            if b not in by or stem == b:
                by[b] = r
        copies = set()
        listing = os.path.join(feats, folder + ".output.txt")
        if os.path.exists(listing):
            with open(listing, encoding="utf-8") as f:
                for line in f:
                    s = base(line.strip())
                    m = re.match(r"^(.*)_\d+$", s)
                    if s and s not in by and m and m.group(1) in by:
                        copies.add(m.group(1))
        for b, r in by.items():
            v = vecs.get((folder, r["file"]))
            if v is None:
                missing += 1
                continue
            deciding = r["dop"] if r["dop"] != "-" else r["xmp"]
            frames.append(
                {
                    "folder": folder,
                    "file": r["file"],
                    "time": r["capture_time"],
                    "subsec": r["subsec"],
                    "ms": capture_ms(r["capture_time"], r["subsec"]),
                    "pick": r["dop"] == "Pick" or b in copies or (r["dop"] == "-" and r["xmp"] == "Pick"),
                    "reject": deciding == "Reject",
                    "sharp": num(r["sharpness"]),
                    "ef": num(r["eye_focus"]),
                    "eo": num(r["eyes_open"]),
                    "yaw": num(r["yaw"]),
                    "pitch": num(r["pitch"]),
                    "roll": num(r["roll"]),
                    "tier": tier(r),
                    "vec": v,
                }
            )
    return frames, missing


def order_key(f):
    gone = f["time"] in ("-", "err", "")
    sub = "" if f["subsec"] == "-" else f["subsec"]
    return (gone, "" if gone else f["time"], sub.ljust(16, "0"), f["file"])


def bursts_of(frames):
    groups, previous = [], None
    for f in sorted(frames, key=order_key):
        ms = f["ms"]
        if ms is None or previous is None or ms - previous > GAP:
            groups.append([f])
        else:
            groups[-1].append(f)
        previous = ms
    return groups


# ---------------------------------------------------------------- features


def logit(p):
    p = min(max(p, 1e-3), 1 - 1e-3)
    return math.log(p / (1 - p))


def tech_row(f, top):
    nan = math.nan
    return [
        math.log(max(f["sharp"] / top, 1e-3)) if f["sharp"] is not None and top else nan,
        nan if f["ef"] is None else logit(f["ef"]),
        nan if f["eo"] is None else f["eo"],
        nan if f["yaw"] is None else min(abs(f["yaw"]), 90),
        nan if f["pitch"] is None else min(abs(f["pitch"]), 60),
        nan if f["roll"] is None else min(abs(f["roll"]), 60),
        1.0 if f["tier"] else 0.0,
        1.0 if f["ef"] is None else 0.0,
        1.0 if f["eo"] is None else 0.0,
        1.0 if f["yaw"] is None else 0.0,
    ]


def build(frames, text):
    """Arrays over every frame, ordered by folder then burst."""
    names, parents, tvec = text
    top_idx = [i for i, p in enumerate(parents) if p is None]
    folders = sorted({f["folder"] for f in frames})
    rows, tech, pos, emb, burst_of, folder_of = [], [], [], [], [], []
    bursts = []
    for fi, folder in enumerate(folders):
        for g in bursts_of([f for f in frames if f["folder"] == folder]):
            sharp = [f["sharp"] for f in g if f["sharp"] is not None]
            top = max(sharp) if sharp else None
            start = len(rows)
            for i, f in enumerate(g):
                rows.append(f)
                tech.append(tech_row(f, top))
                pos.append([i / (len(g) - 1) if len(g) > 1 else 0.0, 1.0 if i == 0 else 0.0])
                emb.append(f.pop("vec"))
                burst_of.append(len(bursts))
                folder_of.append(fi)
            bursts.append((fi, start, len(rows)))
    emb = np.array(emb, dtype=np.float64)
    emb /= np.linalg.norm(emb, axis=1, keepdims=True)
    cos = emb @ tvec.T
    sig = 1 / (1 + np.exp(-(SCALE * cos + BIAS)))
    z = SCALE * cos[:, top_idx]
    soft = np.exp(z - z.max(axis=1, keepdims=True))
    soft /= soft.sum(axis=1, keepdims=True)
    burst_of = np.array(burst_of)
    genre = np.zeros_like(soft)
    for b, (_, s, e) in enumerate(bursts):
        genre[s:e] = soft[s:e].mean(axis=0)
    return {
        "rows": rows,
        "folders": folders,
        "tech": np.array(tech),
        "pos": np.array(pos),
        "emb": emb,
        "sig": sig,
        "soft": soft,
        "genre": genre,
        "genre_names": [names[i] for i in top_idx],
        "label_names": names,
        "burst_of": burst_of,
        "folder_of": np.array(folder_of),
        "bursts": bursts,
        "pick": np.array([f["pick"] for f in rows]),
        "reject": np.array([f["reject"] for f in rows]),
    }


def scorable(d):
    out = []
    for b, (fi, s, e) in enumerate(d["bursts"]):
        p = d["pick"][s:e]
        if e - s >= 2 and 0 < p.sum() < e - s:
            out.append(b)
    return out


def design(d, train_mask, variant):
    """Frame-level design matrix for one fold, standardized on the training
    folders' frames, PCA fit on them."""
    tech = d["tech"].copy()
    mean = np.nanmean(tech[train_mask], axis=0)
    idx = np.where(np.isnan(tech))
    tech[idx] = np.take(mean, idx[1])
    cols = []
    if variant != "emb":
        cols.append(tech)
        cols.append(d["pos"])
    if variant in ("shared", "genre", "emb"):
        pca = PCA(n_components=PCA_DIMS, whiten=True, random_state=0).fit(d["emb"][train_mask])
        cols.append(pca.transform(d["emb"]))
    x = np.hstack(cols)
    mu, sd = x[train_mask].mean(axis=0), x[train_mask].std(axis=0)
    sd[sd == 0] = 1
    x = (x - mu) / sd
    if variant == "genre":
        t = x[:, : len(TECH)]
        inter = [d["genre"][:, [g]] * t for g in range(d["genre"].shape[1])]
        x = np.hstack([x] + [c * OFFSET_SCALE for c in inter])
    return x


def pairs(d, bursts, x):
    """Pair differences, labels and weights: each burst's pairs total 1,
    every other pair flipped to (-d, 0), weight doubled, the same objective as
    entering every pair as (d, 1) and (-d, 0)."""
    diffs, ws = [], []
    for b in bursts:
        _, s, e = d["bursts"][b]
        p = np.arange(s, e)[d["pick"][s:e]]
        n = np.arange(s, e)[~d["pick"][s:e]]
        pi, ni = np.repeat(p, len(n)), np.tile(n, len(p))
        diffs.append(x[pi] - x[ni])
        ws.append(np.full(len(pi), 2.0 / len(pi)))
    diff = np.vstack(diffs)
    w = np.concatenate(ws)
    y = np.ones(len(diff), dtype=int)
    y[1::2] = 0
    diff[1::2] *= -1
    return diff, y, w


def fit(d, bursts, x, c):
    diff, y, w = pairs(d, bursts, x)
    m = LogisticRegression(C=c, fit_intercept=False, max_iter=2000)
    m.fit(diff, y, sample_weight=w)
    return m.coef_[0]


# ---------------------------------------------------------------- metrics


def burst_auc(score, pick):
    p, n = score[pick], score[~pick]
    if len(p) == 0 or len(n) == 0:
        return None, 0
    gt = (p[:, None] > n[None, :]).sum() + 0.5 * (p[:, None] == n[None, :]).sum()
    return gt / (len(p) * len(n)), len(p) * len(n)


def burst_metrics(d, bursts, score, tiebreak, negatives=None):
    """Per burst: (weighted AUC, pair wins, pairs, top-1, top-3)."""
    out = {}
    for b in bursts:
        _, s, e = d["bursts"][b]
        sc, pk = score[s:e], d["pick"][s:e]
        if negatives is not None:
            keep = pk | negatives[s:e]
            sc, pk, tb = sc[keep], pk[keep], tiebreak[s:e][keep]
        else:
            tb = tiebreak[s:e]
        a, npairs = burst_auc(sc, pk)
        if a is None:
            continue
        best = sc.max()
        tied = sc == best
        top1 = pk[tied].mean()
        order = np.lexsort((-tb, -sc))
        top3 = float(pk[order[:3]].any())
        out[b] = (a, a * npairs, npairs, top1, top3)
    return out


def summarize(vals):
    if not vals:
        return None
    v = np.array(list(vals.values()))
    return {
        "bursts": len(v),
        "wauc": v[:, 0].mean(),
        "uauc": v[:, 1].sum() / v[:, 2].sum(),
        "top1": v[:, 3].mean(),
        "top3": v[:, 4].mean(),
    }


def per_folder(d, vals, key):
    by = {}
    for b, v in vals.items():
        by.setdefault(d["bursts"][b][0], []).append(v)
    out = {}
    for fi, vs in by.items():
        vs = np.array(vs)
        out[fi] = (len(vs), vs[:, 0].mean() if key == "wauc" else vs[:, 3].mean())
    return out


def spread(pf):
    v = [x for n, x in pf.values() if n >= MIN_SPREAD_BURSTS]
    return float(np.std(v, ddof=1)) if len(v) > 1 else None


# ---------------------------------------------------------------- folds


def choose_c(d, train_bursts, x, folder_of_burst, tiebreak):
    groups = np.array([folder_of_burst[b] for b in train_bursts])
    k = min(5, len(set(groups)))
    best, best_c = -1, None
    for c in CS:
        vals = {}
        for tr, va in GroupKFold(n_splits=k).split(train_bursts, groups=groups):
            w = fit(d, [train_bursts[i] for i in tr], x, c)
            vals.update(burst_metrics(d, [train_bursts[i] for i in va], x @ w, tiebreak))
        s = summarize(vals)["wauc"]
        if s > best:
            best, best_c = s, c
    return best_c


def run_fold(d, sc, held, train_folders, variant, tiebreak):
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    train_mask = np.isin(d["folder_of"], list(train_folders))
    x = design(d, train_mask, variant)
    train = [b for b in sc if folder_of_burst[b] in train_folders]
    test = [b for b in sc if folder_of_burst[b] == held]
    c = choose_c(d, train, x, folder_of_burst, tiebreak)
    w = fit(d, train, x, c)
    test_mask = d["folder_of"] == held
    return held, c, x[test_mask] @ w, w


def lofo(d, sc, variant, tiebreak, jobs):
    folds = sorted({d["bursts"][b][0] for b in sc})
    allf = set(range(len(d["folders"])))
    res = Parallel(n_jobs=jobs)(
        delayed(run_fold)(d, sc, h, allf - {h}, variant, tiebreak) for h in folds
    )
    score = np.full(len(d["rows"]), np.nan)
    cs = {}
    for held, c, s, _ in res:
        score[d["folder_of"] == held] = s
        cs[held] = c
    return score, cs


def curve(d, sc, tiebreak, jobs):
    folds = sorted({d["bursts"][b][0] for b in sc})
    tasks = []
    for k in CURVE:
        for draw in range(CURVE_DRAWS):
            for h in folds:
                rng = np.random.default_rng(1000 * k + 100 * draw + h)
                others = [f for f in range(len(d["folders"])) if f != h]
                tasks.append((k, draw, h, set(rng.choice(others, size=k, replace=False).tolist())))
    res = Parallel(n_jobs=jobs)(
        delayed(run_fold)(d, sc, h, tr, "shared", tiebreak) for _, _, h, tr in tasks
    )
    out = {}
    for (k, draw, _, _), (held, _, s, _) in zip(tasks, res):
        score = out.setdefault((k, draw), np.full(len(d["rows"]), np.nan))
        score[d["folder_of"] == held] = s
    table = {}
    for k in CURVE:
        ms = [summarize(burst_metrics(d, sc, out[(k, draw)], tiebreak)) for draw in range(CURVE_DRAWS)]
        table[k] = (np.mean([m["wauc"] for m in ms]), np.mean([m["top1"] for m in ms]))
    return table


# ---------------------------------------------------------------- report


def f3(x):
    return "-" if x is None else f"{x:.3f}"


def pc(x):
    return "-" if x is None else f"{100 * x:.1f}%"


def table(head, rows):
    print(head)
    cols = head.count(" | ") + 1
    print("| " + " | ".join(["---"] + ["---:"] * (cols - 1)) + " |")
    for r in rows:
        print("| " + " | ".join(str(v) for v in r) + " |")
    print()


def write_genres(d, path):
    with open(path, "w", encoding="utf-8", newline="") as out:
        out.write("folder\tfile\t" + "\t".join(d["label_names"]) + "\t" + "\t".join("softmax " + n for n in d["genre_names"]) + "\n")
        for i, f in enumerate(d["rows"]):
            out.write(
                f"{f['folder']}\t{f['file']}\t"
                + "\t".join(f"{v:.4g}" for v in d["sig"][i])
                + "\t"
                + "\t".join(f"{v:.4f}" for v in d["soft"][i])
                + "\n"
            )


def main(gate, jobs):
    frames, missing = load(gate)
    text = load_text(os.path.join(gate, "genre_text.tsv"))
    d = build(frames, text)
    write_genres(d, os.path.join(gate, "genres.tsv"))
    sc = scorable(d)
    rng = np.random.default_rng(0)
    tiebreak = rng.random(len(d["rows"]))
    rand = np.random.default_rng(1).random(len(d["rows"]))

    print("## Data\n")
    pairs_n = sum(d["pick"][s:e].sum() * (~d["pick"][s:e]).sum() for b in sc for _, s, e in [d["bursts"][b]])
    rej_bursts = [b for b in sc if d["reject"][d["bursts"][b][1] : d["bursts"][b][2]].any()]
    table(
        "| Count | Value |",
        [
            ("Folders", len(d["folders"])),
            ("Frames (with a vector) / picks / explicit rejects", f"{len(d['rows'])} / {d['pick'].sum()} / {d['reject'].sum()}"),
            ("Frames without a vector (dropped)", missing),
            ("Bursts (gap 1000 ms)", len(d["bursts"])),
            ("Scorable bursts", len(sc)),
            ("Pick x not-chosen pairs in them", int(pairs_n)),
            ("Scorable bursts with an explicit reject", len(rej_bursts)),
        ],
    )

    tech = d["tech"]
    baselines = {
        "First frame": -d["pos"][:, 0],
        "`eye_focus` (missing last)": np.where(np.isnan(tech[:, 1]), -np.inf, tech[:, 1]),
        "Sharpness": np.where(np.isnan(tech[:, 0]), -np.inf, tech[:, 0]),
        "`photoTier` good": tech[:, 6],
        "Random (seed 1)": rand,
    }
    models, cs = {}, {}
    for name, variant in (
        ("Shared model (embedding + technical + position)", "shared"),
        ("Per-genre variant", "genre"),
        ("Without embedding (technical + position)", "noemb"),
        ("Embedding only", "emb"),
    ):
        models[name], cs[name] = lofo(d, sc, variant, tiebreak, jobs)

    allm = {**{k: (v, "baseline") for k, v in baselines.items()}, **{k: (v, "model") for k, v in models.items()}}
    vals = {k: burst_metrics(d, sc, s, tiebreak) for k, (s, _) in allm.items()}
    summ = {k: summarize(v) for k, v in vals.items()}

    print("## Held-out results (leave one folder out)\n")
    rows = []
    for k, (_, kind) in allm.items():
        m = summ[k]
        sp = spread(per_folder(d, vals[k], "wauc"))
        rows.append([k, m["bursts"], f3(m["wauc"]), f3(sp), f3(m["uauc"]), pc(m["top1"]), pc(spread(per_folder(d, vals[k], "top1"))), pc(m["top3"])])
    table("| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |", rows)

    print("`C` chosen per fold (count of folds):\n")
    table("| Model | " + " | ".join(str(c) for c in CS) + " |", [[k] + [sum(1 for v in cs[k].values() if v == c) for c in CS] for k in cs])

    shared, genre = "Shared model (embedding + technical + position)", "Per-genre variant"
    sp_shared = spread(per_folder(d, vals[shared], "wauc"))
    gain_genre = summ[genre]["wauc"] - summ[shared]["wauc"]
    chosen = genre if gain_genre > sp_shared else shared
    first = "First frame"
    sp_first = spread(per_folder(d, vals[first], "top1"))
    gain_top1 = summ[chosen]["top1"] - summ[first]["top1"]
    ok_auc = summ[chosen]["wauc"] >= AUC_BAR
    ok_top1 = gain_top1 > sp_first
    print("## Criterion\n")
    table(
        "| Check | Value | Bar | Met |",
        [
            ("Per-genre minus shared, weighted AUC", f3(gain_genre), f"> {f3(sp_shared)} (shared per-folder SD)", "yes" if gain_genre > sp_shared else "no"),
            ("Chosen variant", chosen, "-", "-"),
            ("Chosen weighted AUC", f3(summ[chosen]["wauc"]), f">= {AUC_BAR}", "yes" if ok_auc else "no"),
            ("Chosen Top-1 minus first frame Top-1", pc(gain_top1), f"> {pc(sp_first)} (first-frame per-folder SD)", "yes" if ok_top1 else "no"),
            ("Gate", "pass" if ok_auc and ok_top1 else "fail", "-", "-"),
        ],
    )

    print("## Per folder (held out)\n")
    pf = {k: (per_folder(d, vals[k], "wauc"), per_folder(d, vals[k], "top1")) for k in (first, shared, genre, "Without embedding (technical + position)")}
    rows = []
    for fi, folder in enumerate(d["folders"]):
        if fi not in pf[first][0]:
            continue
        r = [f"`{folder}`", pf[first][0][fi][0]]
        for k in pf:
            r += [f3(pf[k][0][fi][1]), pc(pf[k][1][fi][1])]
        rows.append(r)
    table("| Folder | Bursts | First frame AUC | Top-1 | Shared AUC | Top-1 | Per-genre AUC | Top-1 | No-embedding AUC | Top-1 |", rows)

    print("## Pick vs explicit reject\n")
    rows = []
    for k, (s, _) in allm.items():
        m = summarize(burst_metrics(d, rej_bursts, s, tiebreak, negatives=d["reject"]))
        rows.append([k, m["bursts"] if m else 0, f3(m["wauc"]) if m else "-", f3(m["uauc"]) if m else "-"])
    table("| Ranking | Bursts | Weighted AUC | Unweighted AUC |", rows)

    print("## Provisional genres\n")
    gname = d["genre_names"]
    bgenre = {b: int(np.argmax(d["genre"][d["bursts"][b][1]])) for b in sc}
    allg = [int(np.argmax(d["genre"][s])) for _, s, _ in d["bursts"]]
    rows = []
    for g, n in enumerate(gname):
        bs = [b for b in sc if bgenre[b] == g]
        r = [n, sum(1 for x in allg if x == g), len(bs)]
        for k in (first, shared, genre, "Without embedding (technical + position)"):
            m = summarize({b: vals[k][b] for b in bs if b in vals[k]})
            r += [f3(m["wauc"]) if m else "-", pc(m["top1"]) if m else "-"]
        rows.append(r)
    table("| Genre (burst argmax) | Bursts | Scorable bursts | First frame AUC | Top-1 | Shared AUC | Top-1 | Per-genre AUC | Top-1 | No-embedding AUC | Top-1 |", rows)
    top_frames = np.bincount(d["soft"].argmax(axis=1), minlength=len(gname))
    print("Frames per top-level genre (frame argmax): " + ", ".join(f"{n} {c}" for n, c in zip(gname, top_frames)) + ".\n")

    print("## Learning curve (shared model)\n")
    lc = curve(d, sc, tiebreak, jobs)
    rows = [[k, f3(lc[k][0]), pc(lc[k][1])] for k in CURVE]
    rows.append([26, f3(summ[shared]["wauc"]), pc(summ[shared]["top1"])])
    table("| Training folders | Weighted AUC | Top-1 |", rows)

    print("## Weights of the shared model fit on every folder\n")
    allf = set(range(len(d["folders"])))
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    x = design(d, np.ones(len(d["rows"]), dtype=bool), "shared")
    c = choose_c(d, sc, x, folder_of_burst, tiebreak)
    w = fit(d, sc, x, c)
    names = TECH + POSITION
    rows = [[n, f"{w[i]:+.3f}"] for i, n in enumerate(names)]
    rows.append(["PCA dims (L2 norm of the 32)", f"{np.linalg.norm(w[len(names):]):.3f}"])
    print(f"`C` = {c} ({len(allf)} folders); standardized columns.\n")
    table("| Column | Weight |", rows)


if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 8)
