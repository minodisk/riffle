"""Step 2: the folder-wide evaluation of the shipped within-burst ranking,
folder-wide training (pointwise and pairwise) and the genre / Candidate 1 /
Candidate 2 cues, held out by folder. The design, the metrics and the bars
are pre-registered in folderwide.md.

Usage: python -I folderwide.py <app-dump-dir> <ball-labels-dir> <out-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <ball-labels-dir> holds
`extras_c1.tsv` and `extras_c2.tsv`. Writes `scores.tsv` (folder, file, one
held-out score column per variant) into <out-dir> and prints the result
tables (Markdown) on stdout.
"""

import math
import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import GroupKFold

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
ARCHIVED = os.path.join(HERE, "..", "_archived")
for sub in (
    "20261012-image-embeddings",
    "20261014-folder-wide-pick-model",
    "20261014-ball-game-ranking-features",
    "20261014-genre-ranking-offsets",
):
    sys.path.insert(0, os.path.join(ARCHIVED, sub))
import gate  # noqa: E402
import folder_gate  # noqa: E402
import ballgate  # noqa: E402
import genre_offsets  # noqa: E402

GENRES = os.path.join(HERE, "..", "..", "..", "crates", "core", "genres", "siglip2.json")
folder_gate.GENRES = GENRES

OFFSET_SHRINK = 30
PAIR_CAP = 20000
BOOT = 1000
MIN_FRAMES = folder_gate.MIN_FRAMES
MIN_PICKS = folder_gate.MIN_PICKS
BAR_AUC = 0.010
BAR_RP = 2.0
BAR_FOLDERS = 16
ALPHA = 0.05

W0 = "W0"
W0_OFF = "W0+off"
F0 = "F0"
F0_CTX = "F0+ctx"
P0 = "P0"
CUES = ["+genre cols", "+genre off", "+c1", "+c2", "+c1+c2"]
FOLDER_WIDE = [F0, F0_CTX, P0]

RANDOM = "Random (seed 0)"
EARLIER = "Capture order, earlier first"
LATER = "Capture order, later first"
SIZE = "Burst size, larger first"
SHARP = "Absolute sharpness"
EYE = "`eye_focus` (missing last)"
TIER = "`photoTier` good"
CENTERED = "W0 burst-centered (what the app displays)"


# ---------------------------------------------------------------- pointwise


def append_block(x, block, train_mask):
    e = block.copy()
    mean = np.nanmean(e[train_mask], axis=0)
    idx = np.where(np.isnan(e))
    e[idx] = np.take(mean, idx[1])
    mu, sd = e[train_mask].mean(axis=0), e[train_mask].std(axis=0)
    sd[sd == 0] = 1
    return np.hstack([x, (e - mu) / sd])


def offsets(x, genre_of_row, tops):
    s = 1 / math.sqrt(OFFSET_SHRINK)
    return np.hstack([x] + [(genre_of_row == g)[:, None] * x * s for g in tops])


def point_design(d, dw, train_mask, base, block, genre_of_row, tops):
    x = gate.design(dw, train_mask, "shared") if base == F0 else folder_gate.design(d, train_mask, "full")
    if genre_of_row is not None:
        return offsets(x, genre_of_row, tops)
    if block is not None:
        return append_block(x, block, train_mask)
    return x


def point_fold(d, dw, held, base, block, genre_of_row, tops):
    train_mask = d["folder_of"] != held
    x = point_design(d, dw, train_mask, base, block, genre_of_row, tops)
    c = folder_gate.choose_c(d, train_mask, x)
    w, b = folder_gate.fit(d, np.where(train_mask)[0], x, c)
    return held, c, x[d["folder_of"] == held] @ w + b


def point_lofo(d, dw, base, jobs, block=None, genre_of_row=None, tops=None):
    res = Parallel(n_jobs=jobs)(
        delayed(point_fold)(d, dw, h, base, block, genre_of_row, tops) for h in range(len(d["folders"]))
    )
    return collect(d, res)


def collect(d, res):
    score = np.full(len(d["rows"]), np.nan)
    cs = {}
    for held, c, s in res:
        score[d["folder_of"] == held] = s
        cs[held] = c
    return score, cs


# ---------------------------------------------------------------- folder-wide pairwise


def pair_index(d):
    """Per folder: the pick and non-pick frame indices of up to PAIR_CAP
    pick x non-pick pairs, drawn without replacement (seed = folder index)."""
    out = []
    for fi in range(len(d["folders"])):
        m = np.where(d["folder_of"] == fi)[0]
        p, n = m[d["pick"][m]], m[~d["pick"][m]]
        t = len(p) * len(n)
        if t == 0:
            out.append((np.zeros(0, dtype=int), np.zeros(0, dtype=int)))
            continue
        k = np.arange(t) if t <= PAIR_CAP else np.sort(np.random.default_rng(fi).choice(t, PAIR_CAP, replace=False))
        out.append((p[k // len(n)], n[k % len(n)]))
    return out


def pair_fit(pidx, folders, x, c):
    diffs, ws = [], []
    for fi in folders:
        pi, ni = pidx[fi]
        if len(pi) == 0:
            continue
        diffs.append(x[pi] - x[ni])
        ws.append(np.full(len(pi), 2.0 / len(pi)))
    diff = np.vstack(diffs)
    w = np.concatenate(ws)
    y = np.ones(len(diff), dtype=int)
    y[1::2] = 0
    diff[1::2] *= -1
    m = LogisticRegression(C=c, fit_intercept=False, max_iter=2000)
    m.fit(diff, y, sample_weight=w)
    return m.coef_[0]


def pair_choose_c(d, pidx, train_mask, x):
    idx = np.where(train_mask)[0]
    groups = d["folder_of"][idx]
    k = min(5, len(set(groups.tolist())))
    splits = list(GroupKFold(n_splits=k).split(idx, groups=groups))
    best, best_c = -1, None
    for c in gate.CS:
        vals = []
        for tr, va in splits:
            w = pair_fit(pidx, sorted(set(groups[tr].tolist())), x, c)
            s = x[idx[va]] @ w
            for fi in sorted(set(groups[va].tolist())):
                m = groups[va] == fi
                a = folder_gate.auc(s[m], d["pick"][idx[va]][m])
                if a is not None:
                    vals.append(a)
        v = float(np.mean(vals))
        if v > best:
            best, best_c = v, c
    return best_c


def pair_fold(d, dw, pidx, held):
    train_mask = d["folder_of"] != held
    x = gate.design(dw, train_mask, "shared")
    c = pair_choose_c(d, pidx, train_mask, x)
    w = pair_fit(pidx, [f for f in range(len(d["folders"])) if f != held], x, c)
    return held, c, x[d["folder_of"] == held] @ w


def pair_lofo(d, dw, pidx, jobs):
    res = Parallel(n_jobs=jobs)(delayed(pair_fold)(d, dw, pidx, h) for h in range(len(d["folders"])))
    return collect(d, res)


# ---------------------------------------------------------------- metrics


def r_precision(score, pick, tiebreak):
    k = int(pick.sum())
    if k == 0 or np.isnan(score).any():
        return None
    top = np.lexsort((-tiebreak, -score))[:k]
    return float(pick[top].mean())


def metrics(d, score, tiebreak, sc):
    pf = folder_gate.folder_metrics(d, score, tiebreak)
    for fi in range(len(d["folders"])):
        m = d["folder_of"] == fi
        rp = r_precision(score[m], d["pick"][m], tiebreak[m])
        pf[fi]["rp"] = None if rp is None else 100 * rp
        if np.isnan(score[m]).any():
            for key in ("frame", "burst", "singles", "lift", "reject"):
                pf[fi][key] = None
    vals = gate.burst_metrics(d, sc, score, tiebreak)
    return pf, vals


def pooled(pf, key, qual, sizes):
    ks = [fi for fi in qual if pf[fi][key] is not None]
    if not ks:
        return None
    return float(np.average([pf[fi][key] for fi in ks], weights=[sizes[fi] for fi in ks]))


def boot_folders(diff):
    diff = np.array(diff)
    rng = np.random.default_rng(0)
    draws = rng.integers(0, len(diff), size=(BOOT, len(diff)))
    means = diff[draws].mean(axis=1)
    return float(diff.mean()), float(np.quantile(means, 0.025)), float(np.quantile(means, 0.975))


def pm(m, s, scale=1.0, digits=3):
    if m is None:
        return "-"
    return f"{scale * m:.{digits}f}" + ("" if s is None else f" ({scale * s:.{digits}f})")


def ci(m, lo, hi, digits=3):
    return f"{m:+.{digits}f} [{lo:+.{digits}f}, {hi:+.{digits}f}]"


# ---------------------------------------------------------------- main


def main(dump, labels_dir, out_dir, jobs):
    start = time.time()
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    dw = dict(d)
    dw["tech"] = d["tech"].copy()
    folder_gate.extend(d, dump)
    sc = gate.scorable(d)
    n = len(d["rows"])
    tiebreak = np.random.default_rng(0).random(n)
    pick, folder_of = d["pick"], d["folder_of"]
    nf = len(d["folders"])
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    picks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(nf)}
    qual = [fi for fi in range(nf) if sizes[fi] >= MIN_FRAMES and picks[fi] >= MIN_PICKS]

    tops, bgenre = genre_offsets.burst_genres(d, GENRES)
    genre_of_row = np.empty(n, dtype=object)
    for b, (_, s, e) in enumerate(d["bursts"]):
        genre_of_row[s:e] = bgenre[b]
    c1_names, c1 = ballgate.load_extras([os.path.join(labels_dir, "extras_c1.tsv")], d)
    c2_names, c2 = ballgate.load_extras([os.path.join(labels_dir, "extras_c2.tsv")], d)
    blocks = {"+genre cols": d["genres"], "+c1": c1, "+c2": c2, "+c1+c2": np.hstack([c1, c2])}
    pidx = pair_index(d)
    npairs = sum(len(p) for p, _ in pidx)
    capped = sum(1 for fi in range(nf) if picks[fi] * (sizes[fi] - picks[fi]) > PAIR_CAP)
    sc_folders = sorted({d["bursts"][b][0] for b in sc})

    print("## Data\n")
    rates = [picks[fi] / sizes[fi] for fi in range(nf)]
    gate.table(
        "| Count | Value |",
        [
            ("Folders / qualifying (>= 50 frames, >= 5 picks)", f"{nf} / {len(qual)}"),
            ("Frames (with a vector) / picks / explicit rejects", f"{n} / {pick.sum()} / {d['reject'].sum()}"),
            ("Frames without a vector (dropped)", missing),
            ("Bursts (gap 1000 ms), singles included / scorable", f"{len(d['bursts'])} / {len(sc)}"),
            ("Singles / of them picks", f"{int(d['single'].sum())} / {int((d['single'] & pick).sum())}"),
            ("Folders with a scorable burst (`W0`-family folds)", len(sc_folders)),
            ("Folder pick rate min / median / max", f"{100 * min(rates):.1f}% / {100 * float(np.median(rates)):.1f}% / {100 * max(rates):.1f}%"),
            ("Bursts per genre (shipped rule)", ", ".join(f"{g} {sum(1 for x in bgenre if x == g)}" for g in tops + [genre_offsets.UNKNOWN])),
            ("`c1` columns (frames without a row)", f"{', '.join(c1_names)} ({int(np.isnan(c1).all(axis=1).sum())})"),
            ("`c2` columns (frames without a row)", f"{', '.join(c2_names)} ({int(np.isnan(c2).all(axis=1).sum())})"),
            ("`P0` pairs / folders capped at 20,000", f"{npairs} / {capped}"),
            ("`P0` all-folder pair design (pairs x 44 x 8 bytes)", f"{npairs * 44 * 8 / 2**20:.0f} MiB"),
        ],
    )

    models, cs, times = {}, {}, {}

    def run(name, f):
        t = time.time()
        models[name], cs[name] = f()
        times[name] = time.time() - t
        print(f"{name}: {times[name]:.0f} s", file=sys.stderr, flush=True)

    folds = sc_folders
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    trains = {h: [b for b in sc if folder_of_burst[b] != h] for h in folds}
    run(W0, lambda: gate.lofo(dw, sc, "shared", tiebreak, jobs))
    run(W0_OFF, lambda: genre_offsets.lofo(dw, sc, folds, trains, genre_of_row, tops, OFFSET_SHRINK, "all", tiebreak, jobs))
    run(F0, lambda: point_lofo(d, dw, F0, jobs))
    run(F0_CTX, lambda: point_lofo(d, dw, F0_CTX, jobs))
    run(P0, lambda: pair_lofo(d, dw, pidx, jobs))
    for cue in CUES:
        if cue == "+genre off":
            run(F0_CTX + " " + cue, lambda: point_lofo(d, dw, F0_CTX, jobs, genre_of_row=genre_of_row, tops=tops))
        else:
            run(F0_CTX + " " + cue, lambda: point_lofo(d, dw, F0_CTX, jobs, block=blocks[cue]))
    for cue in CUES:
        if cue != "+genre off":
            run(W0 + " " + cue, lambda: ballgate.lofo(dw, sc, blocks[cue], tiebreak, jobs))

    def cue_name(base, cue):
        return W0_OFF if base == W0 and cue == "+genre off" else f"{base} {cue}"

    w0 = models[W0]
    centered = w0.copy()
    for _, s, e in d["bursts"]:
        centered[s:e] -= w0[s:e].mean()
    tech = d["tech"]
    baselines = {
        CENTERED: centered,
        RANDOM: tiebreak,
        EARLIER: -d["order"],
        LATER: d["order"],
        SIZE: d["size"],
        SHARP: np.where(np.isnan(d["sharp"]), -np.inf, d["sharp"]),
        EYE: np.array([-np.inf if f["ef"] is None else f["ef"] for f in d["rows"]]),
        TIER: tech[:, 6],
    }
    scores = {**models, **baselines}
    res = {k: metrics(d, s, tiebreak, sc) for k, s in scores.items()}
    pf = {k: r[0] for k, r in res.items()}
    wb = {k: r[1] for k, r in res.items()}
    summ = {k: gate.summarize(v) for k, v in wb.items()}

    with open(os.path.join(out_dir, "scores.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("folder\tfile\t" + "\t".join(models) + "\n")
        for i, f in enumerate(d["rows"]):
            out.write(f"{f['folder']}\t{f['file']}\t" + "\t".join("nan" if np.isnan(models[k][i]) else f"{models[k][i]:.6g}" for k in models) + "\n")

    def mean(k, key):
        return folder_gate.mean_sd(folder_gate.values(pf[k], key, qual))

    print("## Reference checks\n")
    gate.table(
        "| Check | Archived | This run |",
        [
            ("`W0` within-burst weighted AUC / Top-1", "0.710 / 54.9%", f"{gate.f3(summ[W0]['wauc'])} / {gate.pc(summ[W0]['top1'])}"),
            ("`F0+ctx` mean frame AUC (qualifying folders)", "0.666", gate.f3(mean(F0_CTX, "frame")[0])),
            ("`W0+off` within-burst weighted AUC", "0.709", gate.f3(summ[W0_OFF]["wauc"])),
        ],
    )

    print("## Held-out results (leave one folder out)\n")
    chance = float(np.mean([100 * picks[fi] / sizes[fi] for fi in qual]))
    print(
        f"Frame AUC and R-precision: the mean over the {len(qual)} qualifying folders (per-folder SD in brackets) "
        f"and the frame-weighted mean beside it (pooled). R-precision chance (mean pick rate) {chance:.1f}%. "
        f"Within-burst: weighted pairwise AUC and Top-1 over the scorable bursts. Singles AUC over the folders "
        f"with >= 5 single picks and 5 single non-picks (`n`).\n"
    )
    rows = []
    for k in scores:
        fm, fs, _ = mean(k, "frame")
        rm, rs, _ = mean(k, "rp")
        sm, ss, sn = mean(k, "singles")
        lm, ls, _ = mean(k, "lift")
        bm, bs, _ = mean(k, "burst")
        rows.append(
            [k, pm(fm, fs), gate.f3(pooled(pf[k], "frame", qual, sizes)), pm(rm, rs, digits=1), pm(pooled(pf[k], "rp", qual, sizes), None, digits=1),
             gate.f3(summ[k]["wauc"]), gate.pc(summ[k]["top1"]), pm(sm, ss), sn, pm(lm, ls, digits=2), pm(bm, bs)]
        )
    gate.table("| Score | Frame AUC | Pooled | R-precision % | Pooled | Within-burst wAUC | Top-1 | Singles AUC | n | Lift at top tenth | Burst AUC |", rows)

    comparisons = [(k, W0) for k in FOLDER_WIDE]
    comparisons += [(cue_name(F0_CTX, c), F0_CTX) for c in CUES]
    comparisons += [(cue_name(W0, c), W0) for c in CUES]

    print("## Paired differences (variant minus comparison)\n")
    print(
        f"Over the {len(qual)} qualifying folders: mean paired per-folder difference with the 1,000-draw folder "
        "bootstrap 95% interval (seed 0); the one-sided Wilcoxon signed-rank p (zeros dropped); folders where the "
        "variant's frame AUC is at or above the comparison's. Within-burst: the per-burst weighted AUC difference "
        "with `genre_offsets.boot`'s per-burst bootstrap, and the Top-1 difference. Bars: frame AUC >= +0.010, "
        "R-precision >= +2.0 points, >= 16 folders at or above, p < 0.05 on both.\n"
    )
    rows = []
    for v, base in comparisons:
        fa = folder_gate.paired(folder_gate.values(pf[v], "frame", qual), folder_gate.values(pf[base], "frame", qual))
        ra = folder_gate.paired(folder_gate.values(pf[v], "rp", qual), folder_gate.values(pf[base], "rp", qual))
        fbm, flo, fhi = boot_folders(fa)
        rbm, rlo, rhi = boot_folders(ra)
        _, pa, _ = folder_gate.signed_rank(fa)
        _, pr, _ = folder_gate.signed_rank(ra)
        above = sum(1 for x in fa if x >= 0)
        wm, wlo, whi = genre_offsets.boot(wb[v], wb[base], sc, np.random.default_rng(0))
        dt = 100 * (summ[v]["top1"] - summ[base]["top1"])
        checks = [fbm >= BAR_AUC, rbm >= BAR_RP, above >= BAR_FOLDERS, pa < ALPHA, pr < ALPHA]
        rows.append(
            [f"`{v}` - `{base}`", ci(fbm, flo, fhi), f"{pa:.4f}", ci(rbm, rlo, rhi, 1), f"{pr:.4f}", f"{above} of {len(fa)}",
             ci(wm, wlo, whi), f"{dt:+.1f}", f"{sum(checks)} of 5", "pass" if all(checks) else "fail"]
        )
    gate.table("| Comparison | Frame AUC | p | R-precision, points | p | Folders at or above | Within-burst wAUC | Top-1, points | Bars met | Result |", rows)

    best_fw = max(FOLDER_WIDE, key=lambda k: mean(k, "frame")[0])
    cue_variants = [cue_name(b, c) for b in (F0_CTX, W0) for c in CUES]
    best_cue = max(cue_variants, key=lambda k: mean(k, "frame")[0])
    print("## Per folder (held out)\n")
    print(f"Best folder-wide by mean frame AUC: `{best_fw}`; best cue variant: `{best_cue}`.\n")
    rows = []
    for fi, folder in enumerate(d["folders"]):
        r = [f"`{folder}`", sizes[fi], picks[fi], f"{100 * picks[fi] / sizes[fi]:.1f}%", "yes" if fi in qual else "no"]
        for k in (W0, best_fw, best_cue):
            r += [gate.f3(pf[k][fi]["frame"]), "-" if pf[k][fi]["rp"] is None else f"{pf[k][fi]['rp']:.1f}"]
        rows.append(r)
    gate.table(
        f"| Folder | Frames | Picks | Pick rate | Qualifying | `{W0}` frame AUC | R-prec % | `{best_fw}` frame AUC | R-prec % | `{best_cue}` frame AUC | R-prec % |",
        rows,
    )

    print("`C` chosen per fold (count of folds):\n")
    gate.table("| Model | " + " | ".join(str(c) for c in gate.CS) + " |", [[k] + [sum(1 for v in cs[k].values() if v == c) for c in gate.CS] for k in cs])

    print("## Pick vs explicit reject\n")
    rows = []
    for k in scores:
        m, _, c = folder_gate.mean_sd({fi: v["reject"] for fi, v in pf[k].items() if v["reject"] is not None})
        rows.append([k, c, gate.f3(m)])
    gate.table("| Score | Folders | Mean AUC |", rows)

    print("## Wall time\n")
    gate.table("| Model | Seconds |", [[k, f"{v:.0f}"] for k, v in times.items()])
    print(f"Total {time.time() - start:.0f} s on {jobs} jobs.")


if __name__ == "__main__":
    if len(sys.argv) not in (4, 5):
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]) if len(sys.argv) > 4 else 8)
