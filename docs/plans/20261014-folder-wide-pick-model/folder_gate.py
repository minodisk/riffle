"""The Step 2 folder-wide pick gate: a per-frame logistic of "the user picks
this frame" on the SigLIP 2 embedding (PCA 32), the technical columns and the
burst context, held out by folder, against the non-embedding baselines. The
design and the criterion are pre-registered in gate.md.

Usage: python -I folder_gate.py <app-dump-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump (`features/<folder>.tsv` +
`.output.txt`, `embed.tsv`, `genre_text.tsv`, `prior_scores.tsv`), read by
the archived gate.py's `load` and `build`. Prints the result tables of
gate.md (Markdown) on stdout.
"""

import json
import math
import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.decomposition import PCA
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import GroupKFold

try:
    from scipy.stats import wilcoxon
except ImportError:
    wilcoxon = None

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "_archived", "20261012-image-embeddings"))
import gate  # noqa: E402

GENRES = os.path.join(HERE, "..", "..", "..", "crates", "core", "genres", "siglip2.json")
CS = gate.CS
PCA_DIMS = gate.PCA_DIMS
MIN_FRAMES = 50
MIN_PICKS = 5
MIN_SINGLES = 5
LIFT_BAR = 2.0
ALPHA = 0.05
CURVE = (5, 10, 20)
CURVE_DRAWS = 3

TECH = gate.TECH + ["ln sharpness"]
CONTEXT = ["ln burst size", "position", "first frame"]
GROUPS = [
    ("Look", None),
    ("Burst context", ["ln burst size", "position", "first frame"]),
    ("Sharpness", ["ln rel sharpness", "ln sharpness"]),
    ("AF eye focus", ["`eye_focus` logit", "no `eye_focus`"]),
    ("Eyes open", ["eyes open", "no eyes judgment"]),
    ("Head pose", ["\\|yaw\\|", "\\|pitch\\|", "\\|roll\\|", "no pose"]),
    ("Good photo", ["`photoTier` good"]),
]

FULL = "Full model (embedding + technical + context)"
GENRE = "Full + genre"
NOEMB = "Without embedding (technical + context)"
EMB = "Embedding only"
CONTEXT_ONLY = "Context only"
CENTERED = "Folder-centered embedding"
VARIANTS = [(FULL, "full"), (GENRE, "genre"), (NOEMB, "noemb"), (EMB, "emb"), (CONTEXT_ONLY, "context"), (CENTERED, "centered")]

RANDOM = "Random (seed 0)"
EARLIER = "Capture order, earlier first"
LATER = "Capture order, later first"
SIZE = "Burst size, larger first"
SHARP = "Absolute sharpness"
EYE = "`eye_focus` (missing last)"
TIER = "`photoTier` good"
PRIOR = "In-burst prior's score"
NON_EMBEDDING = [RANDOM, EARLIER, LATER, SIZE, SHARP, EYE, TIER, NOEMB, CONTEXT_ONLY]


# ---------------------------------------------------------------- data


def extend(d, gate_dir):
    n = len(d["rows"])
    size = np.zeros(n)
    single = np.zeros(n, dtype=bool)
    order = np.zeros(n)
    for _, s, e in d["bursts"]:
        size[s:e] = e - s
        single[s:e] = e - s == 1
    for fi in range(len(d["folders"])):
        idx = np.where(d["folder_of"] == fi)[0]
        order[idx] = np.arange(len(idx))
    sharp = np.array([math.log(f["sharp"]) if f["sharp"] is not None else math.nan for f in d["rows"]])
    d["tech"] = np.hstack([d["tech"], sharp[:, None]])
    d["context"] = np.hstack([np.log(size)[:, None], d["pos"]])
    d["size"] = size
    d["single"] = single
    d["order"] = order
    d["sharp"] = sharp

    centered = d["emb"].copy()
    for fi in range(len(d["folders"])):
        m = d["folder_of"] == fi
        centered[m] -= centered[m].mean(axis=0)
    d["centered"] = centered

    with open(GENRES, encoding="utf-8") as f:
        labels = json.load(f)["labels"]
    on = np.stack([d["emb"] @ np.array(lab["vector"], dtype=np.float64) >= lab["threshold"] for lab in labels], axis=1)
    names = [lab["label"] for lab in labels]
    for i, lab in enumerate(labels):
        if lab.get("parent"):
            on[:, names.index(lab["parent"])] |= on[:, i]
    d["genres"] = on.astype(np.float64)
    d["genre_labels"] = names

    prior = {}
    with open(os.path.join(gate_dir, "prior_scores.tsv"), encoding="utf-8") as f:
        next(f)
        for line in f:
            folder, file, score = line.rstrip("\n").split("\t")
            prior[(folder, file)] = float(score)
    d["prior"] = np.array([prior.get((f["folder"], f["file"]), -np.inf) for f in d["rows"]])

    w = np.zeros(n)
    pick = d["pick"]
    for fi in range(len(d["folders"])):
        m = d["folder_of"] == fi
        p, q = (m & pick).sum(), (m & ~pick).sum()
        if p == 0 or q == 0:
            w[m] = 1 / m.sum()
        else:
            w[m & pick] = 1 / (2 * p)
            w[m & ~pick] = 1 / (2 * q)
    d["weight"] = w


# ---------------------------------------------------------------- model


def design(d, train_mask, variant):
    tech = d["tech"].copy()
    mean = np.nanmean(tech[train_mask], axis=0)
    idx = np.where(np.isnan(tech))
    tech[idx] = np.take(mean, idx[1])
    cols = []
    if variant in ("full", "genre", "noemb", "centered"):
        cols += [tech, d["context"]]
    if variant == "context":
        cols.append(d["context"])
    if variant in ("full", "genre", "emb", "centered"):
        emb = d["centered"] if variant == "centered" else d["emb"]
        pca = PCA(n_components=PCA_DIMS, whiten=True, random_state=0).fit(emb[train_mask])
        cols.append(pca.transform(emb))
    if variant == "genre":
        cols.append(d["genres"])
    x = np.hstack(cols)
    mu, sd = x[train_mask].mean(axis=0), x[train_mask].std(axis=0)
    sd[sd == 0] = 1
    return (x - mu) / sd


def fit(d, idx, x, c):
    m = LogisticRegression(C=c, fit_intercept=True, max_iter=2000)
    m.fit(x[idx], d["pick"][idx].astype(int), sample_weight=d["weight"][idx])
    return m.coef_[0], m.intercept_[0]


def choose_c(d, train_mask, x):
    idx = np.where(train_mask)[0]
    groups = d["folder_of"][idx]
    k = min(5, len(set(groups.tolist())))
    splits = list(GroupKFold(n_splits=k).split(idx, groups=groups))
    best, best_c = -1, None
    for c in CS:
        vals = []
        for tr, va in splits:
            w, b = fit(d, idx[tr], x, c)
            s = x[idx[va]] @ w + b
            for fi in sorted(set(groups[va].tolist())):
                m = groups[va] == fi
                a = auc(s[m], d["pick"][idx[va]][m])
                if a is not None:
                    vals.append(a)
        v = float(np.mean(vals))
        if v > best:
            best, best_c = v, c
    return best_c


def run_fold(d, held, train_folders, variant):
    train_mask = np.isin(d["folder_of"], sorted(train_folders))
    x = design(d, train_mask, variant)
    c = choose_c(d, train_mask, x)
    w, b = fit(d, np.where(train_mask)[0], x, c)
    test = d["folder_of"] == held
    return held, c, x[test] @ w + b


def lofo(d, variant, jobs):
    allf = set(range(len(d["folders"])))
    res = Parallel(n_jobs=jobs)(delayed(run_fold)(d, h, allf - {h}, variant) for h in sorted(allf))
    score = np.full(len(d["rows"]), np.nan)
    cs = {}
    for held, c, s in res:
        score[d["folder_of"] == held] = s
        cs[held] = c
    return score, cs


# ---------------------------------------------------------------- metrics


def auc(score, label):
    p, n = label.sum(), (~label).sum()
    if p == 0 or n == 0:
        return None
    _, inv, counts = np.unique(score, return_inverse=True, return_counts=True)
    ends = np.cumsum(counts)
    ranks = (ends - (counts - 1) / 2)[inv]
    return float((ranks[label].sum() - p * (p + 1) / 2) / (p * n))


def folder_metrics(d, score, tiebreak):
    out = {}
    for fi in range(len(d["folders"])):
        m = np.where(d["folder_of"] == fi)[0]
        sc, pk = score[m], d["pick"][m]
        frame = auc(sc, pk)
        units, labels = [], []
        for bi in sorted(set(d["burst_of"][m].tolist())):
            _, s, e = d["bursts"][bi]
            units.append(score[s:e].max())
            labels.append(d["pick"][s:e].any())
        burst = auc(np.array(units), np.array(labels))
        sg = d["single"][m]
        singles = None
        if pk[sg].sum() >= MIN_SINGLES and (~pk[sg]).sum() >= MIN_SINGLES:
            singles = auc(sc[sg], pk[sg])
        lift = None
        if pk.sum() > 0:
            top = np.lexsort((-tiebreak[m], -sc))[: math.ceil(len(m) / 10)]
            lift = pk[top].mean() / pk.mean()
        rej = d["reject"][m] & ~pk
        reject = auc(np.concatenate([sc[pk], sc[rej]]), np.concatenate([np.ones(pk.sum(), bool), np.zeros(rej.sum(), bool)])) if rej.any() else None
        out[fi] = {"frame": frame, "burst": burst, "singles": singles, "lift": lift, "reject": reject}
    return out


def values(pf, key, folders):
    return {fi: pf[fi][key] for fi in folders if pf[fi][key] is not None}


def mean_sd(v):
    a = np.array(list(v.values()))
    if len(a) == 0:
        return None, None, 0
    return float(a.mean()), float(a.std(ddof=1)) if len(a) > 1 else None, len(a)


def signed_rank(diff):
    """One-sided (greater) Wilcoxon signed-rank test, zeros dropped:
    `scipy.stats.wilcoxon` when scipy is there, else the exact null
    distribution with mid-ranks for ties."""
    d = np.array([x for x in diff if x != 0])
    if len(d) == 0:
        return 0.0, 1.0, 0
    if wilcoxon is not None:
        r = wilcoxon(d, zero_method="wilcox", alternative="greater", method="auto")
        return float(r.statistic), float(r.pvalue), len(d)
    a = np.abs(d)
    _, inv, counts = np.unique(a, return_inverse=True, return_counts=True)
    ends = np.cumsum(counts)
    ranks = (ends - (counts - 1) / 2)[inv]
    twice = np.rint(2 * ranks).astype(int)
    t = float(ranks[d > 0].sum())
    dist = np.zeros(twice.sum() + 1)
    dist[0] = 1.0
    for r in twice:
        shifted = np.zeros_like(dist)
        shifted[r:] = dist[:-r]
        dist = (dist + shifted) / 2
    p = float(dist[int(round(2 * t)) :].sum())
    return t, p, len(d)


def paired(a, b):
    keys = sorted(set(a) & set(b))
    return [a[k] - b[k] for k in keys]


# ---------------------------------------------------------------- report


f3 = gate.f3
table = gate.table


def pm(m, s):
    return "-" if m is None else f"{m:.3f}" + ("" if s is None else f" ({s:.3f})")


def curve(d, jobs):
    tasks = []
    for k in CURVE:
        for draw in range(CURVE_DRAWS):
            for h in range(len(d["folders"])):
                rng = np.random.default_rng(1000 * k + 100 * draw + h)
                others = [f for f in range(len(d["folders"])) if f != h]
                tasks.append((k, draw, h, set(rng.choice(others, size=k, replace=False).tolist())))
    res = Parallel(n_jobs=jobs)(delayed(run_fold)(d, h, tr, "full") for _, _, h, tr in tasks)
    out = {}
    for (k, draw, _, _), (held, _, s) in zip(tasks, res):
        score = out.setdefault((k, draw), np.full(len(d["rows"]), np.nan))
        score[d["folder_of"] == held] = s
    return out


def main(gate_dir, jobs):
    start = time.time()
    frames, missing = gate.load(gate_dir)
    d = gate.build(frames, gate.load_text(os.path.join(gate_dir, "genre_text.tsv")))
    extend(d, gate_dir)
    n = len(d["rows"])
    tiebreak = np.random.default_rng(0).random(n)
    pick, folder_of = d["pick"], d["folder_of"]
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(len(d["folders"]))}
    picks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(len(d["folders"]))}
    qual = [fi for fi in range(len(d["folders"])) if sizes[fi] >= MIN_FRAMES and picks[fi] >= MIN_PICKS]

    print("## Data\n")
    rates = [picks[fi] / sizes[fi] for fi in range(len(d["folders"]))]
    table(
        "| Count | Value |",
        [
            ("Folders / qualifying (>= 50 frames, >= 5 picks)", f"{len(d['folders'])} / {len(qual)}"),
            ("Frames (with a vector) / picks / explicit rejects", f"{n} / {pick.sum()} / {d['reject'].sum()}"),
            ("Explicit rejects that are also picks (a virtual copy)", int((d["reject"] & pick).sum())),
            ("Frames without a vector (dropped)", missing),
            ("Bursts (gap 1000 ms), singles included", len(d["bursts"])),
            ("Singles (bursts of one) / of them picks", f"{int(d['single'].sum())} / {int((d['single'] & pick).sum())}"),
            ("Bursts holding a pick", sum(1 for _, s, e in d["bursts"] if pick[s:e].any())),
            ("Folder pick rate min / median / max", f"{100 * min(rates):.1f}% / {100 * float(np.median(rates)):.1f}% / {100 * max(rates):.1f}%"),
            ("Frames with an on genre label / `Unknown`", f"{int(d['genres'].any(axis=1).sum())} / {int((~d['genres'].any(axis=1)).sum())}"),
        ],
    )

    tech = d["tech"]
    baselines = {
        RANDOM: tiebreak,
        EARLIER: -d["order"],
        LATER: d["order"],
        SIZE: d["size"],
        SHARP: np.where(np.isnan(d["sharp"]), -np.inf, d["sharp"]),
        EYE: np.where(np.isnan(tech[:, 1]), -np.inf, tech[:, 1]),
        TIER: tech[:, 6],
        PRIOR: d["prior"],
    }
    models, cs = {}, {}
    for name, variant in VARIANTS:
        models[name], cs[name] = lofo(d, variant, jobs)
    scores = {**models, **baselines}
    pf = {k: folder_metrics(d, s, tiebreak) for k, s in scores.items()}

    print("## Held-out results (leave one folder out, qualifying folders)\n")
    print("Each cell: the mean over the qualifying folders (per-folder SD in brackets); `n` the folders it is over.\n")
    rows = []
    for k in scores:
        r = [k]
        for key in ("frame", "burst", "singles", "lift"):
            m, s, c = mean_sd(values(pf[k], key, qual))
            r += [pm(m, s)] + ([c] if key in ("burst", "singles") else [])
        rows.append(r)
    table("| Score | Frame AUC | Burst AUC | n | Singles AUC | n | Lift at top tenth |", rows)

    print("`C` chosen per fold (count of folds):\n")
    table("| Model | " + " | ".join(str(c) for c in CS) + " |", [[k] + [sum(1 for v in cs[k].values() if v == c) for c in CS] for k in cs])

    frame = {k: values(pf[k], "frame", qual) for k in scores}
    burst = {k: values(pf[k], "burst", qual) for k in scores}
    t_g, p_g, n_g = signed_rank(paired(frame[GENRE], frame[FULL]))
    chosen = GENRE if p_g < ALPHA else FULL
    best = max(NON_EMBEDDING, key=lambda k: mean_sd(frame[k])[0])
    t1, p1, n1 = signed_rank(paired(frame[chosen], frame[best]))
    t2, p2, n2 = signed_rank(paired(burst[chosen], burst[SIZE]))
    lift = mean_sd(values(pf[chosen], "lift", qual))[0]
    ok1, ok2, ok3 = p1 < ALPHA, p2 < ALPHA, lift >= LIFT_BAR
    g1 = mean_sd(frame[chosen])[0] - mean_sd(frame[best])[0]
    sd1 = mean_sd(frame[best])[1]
    g2 = mean_sd(burst[chosen])[0] - mean_sd(burst[SIZE])[0]
    sd2 = mean_sd(burst[SIZE])[1]
    yes = lambda b: "met" if b else "not met"  # noqa: E731

    print("## Criterion\n")
    print(f"Best non-embedding comparison by mean frame AUC: {best} ({f3(mean_sd(frame[best])[0])}).\n")
    table(
        "| Check | Statistic | p (one-sided) | Bar | Result |",
        [
            (f"Genre variant: {GENRE} minus full, frame AUC (n = {n_g})", f"W+ = {t_g:g}, mean gain {f3(mean_sd(frame[GENRE])[0] - mean_sd(frame[FULL])[0])}", f"{p_g:.4f}", "p < 0.05", "adopted" if chosen == GENRE else "not adopted"),
            ("Chosen model", chosen, "-", "-", "-"),
            (f"1. Frame AUC, chosen minus {best} (n = {n1})", f"W+ = {t1:g}, mean gain {f3(g1)}", f"{p1:.4f}", "p < 0.05", yes(ok1)),
            (f"2. Burst AUC, chosen minus {SIZE} (n = {n2})", f"W+ = {t2:g}, mean gain {f3(g2)}", f"{p2:.4f}", "p < 0.05", yes(ok2)),
            ("3. Mean lift at the top tenth", f3(lift), "-", ">= 2.0", yes(ok3)),
            ("Gate", "pass" if ok1 and ok2 and ok3 else "fail", "-", "-", "-"),
        ],
    )
    print("SD-gap reading (the earlier gates' rule; for reading only, does not decide):\n")
    table(
        "| Check | Mean gain | Comparison's per-folder SD | Gain > SD |",
        [
            (f"1. Frame AUC vs {best}", f3(g1), f3(sd1), "yes" if sd1 is not None and g1 > sd1 else "no"),
            (f"2. Burst AUC vs {SIZE}", f3(g2), f3(sd2), "yes" if sd2 is not None and g2 > sd2 else "no"),
        ],
    )

    print("## Per folder (held out)\n")
    rows = []
    for fi, folder in enumerate(d["folders"]):
        r = [f"`{folder}`", sizes[fi], picks[fi], f"{100 * picks[fi] / sizes[fi]:.1f}%", "yes" if fi in qual else "no"]
        for k in (FULL, NOEMB, SIZE):
            r += [f3(pf[k][fi]["frame"]), f3(pf[k][fi]["burst"])]
        rows.append(r)
    table("| Folder | Frames | Picks | Pick rate | Qualifying | Full frame AUC | Full burst AUC | No-embedding frame AUC | No-embedding burst AUC | Burst size frame AUC | Burst size burst AUC |", rows)

    print("## Pick vs explicit reject\n")
    rows = []
    for k in scores:
        m, _, c = mean_sd({fi: v["reject"] for fi, v in pf[k].items() if v["reject"] is not None})
        rows.append([k, c, f3(m)])
    table("| Score | Folders | Mean AUC |", rows)

    print("## Learning curve (full model)\n")
    lc = curve(d, jobs)
    rows = []
    for k in CURVE:
        fr, bu = [], []
        for draw in range(CURVE_DRAWS):
            p = folder_metrics(d, lc[(k, draw)], tiebreak)
            fr.append(mean_sd(values(p, "frame", qual))[0])
            bu.append(mean_sd(values(p, "burst", qual))[0])
        rows.append([k, f3(float(np.mean(fr))), f3(float(np.mean(bu)))])
    rows.append([26, f3(mean_sd(frame[FULL])[0]), f3(mean_sd(burst[FULL])[0])])
    table("| Training folders | Frame AUC | Burst AUC |", rows)

    print("## Weights of the full model fit on every folder\n")
    every = np.ones(n, dtype=bool)
    x = design(d, every, "full")
    c = choose_c(d, every, x)
    w, b = fit(d, np.where(every)[0], x, c)
    names = TECH + CONTEXT
    print(f"`C` = {c} ({len(d['folders'])} folders); standardized columns; intercept {b:+.3f}.\n")
    table("| Column | Weight |", [[nm, f"{w[i]:+.3f}"] for i, nm in enumerate(names)] + [["PCA dims (L2 norm of the 32)", f"{np.linalg.norm(w[len(names):]):.3f}"]])
    rows = []
    for g, cols in GROUPS:
        v = w[len(names) :] if cols is None else np.array([w[names.index(cn)] for cn in cols])
        rows.append([g, len(v), f"{np.linalg.norm(v):.3f}"])
    table("| Reason group | Columns | L2 norm |", rows)

    print(f"Wall time {time.time() - start:.0f} s on {jobs} jobs.")


if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 8)
