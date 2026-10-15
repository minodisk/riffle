"""Step 3: the pre-registered column and group ablations of `F0+ctx` and
`W0`, held out by folder, against the unablated held-out scores of Step 2's
`scores.tsv`, each group alone, and the weights of both models fit on all
folders. The design and the bars are pre-registered in folderwide.md.

Usage: python -I -B ablate.py <app-dump-dir> <ball-labels-dir> <out-dir> [jobs]

<app-dump-dir> and <ball-labels-dir> as for folderwide.py; <out-dir> holds
Step 2's `scores.tsv`. Prints the ablation tables (Markdown) on stdout.
"""

import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import folderwide as fw  # noqa: E402

gate = fw.gate
folder_gate = fw.folder_gate
ballgate = fw.ballgate
genre_offsets = fw.genre_offsets

W0, F0_CTX = fw.W0, fw.F0_CTX
BASES = [F0_CTX, W0]
NT = len(gate.TECH)
NP = len(gate.POSITION)
W_PCA = list(range(NT + NP, NT + NP + gate.PCA_DIMS))
F_PCA = list(range(NT + 1 + 3, NT + 1 + 3 + gate.PCA_DIMS))
F_LN_SHARP, F_LN_SIZE = NT, NT + 1

# Column indices into each base's design: `W0` is gate.design's TECH (10),
# POSITION (2), PCA (32); `F0+ctx` is folder_gate.design's TECH (10) + ln
# sharpness, CONTEXT (ln burst size, position, first frame), PCA (32). The
# missing-value indicators leave with their group, as rank.rs's MISSING.
GROUPS = {
    "Look": {W0: W_PCA, F0_CTX: F_PCA},
    "Burst position": {W0: [NT, NT + 1], F0_CTX: [F_LN_SIZE + 1, F_LN_SIZE + 2]},
    "Sharpness": {W0: [0], F0_CTX: [0, F_LN_SHARP]},
    "AF eye focus": {W0: [1, 7], F0_CTX: [1, 7]},
    "Eyes open": {W0: [2, 8], F0_CTX: [2, 8]},
    "Head pose": {W0: [3, 4, 5, 9], F0_CTX: [3, 4, 5, 9]},
    "Good photo": {W0: [6], F0_CTX: [6]},
}
DROPS = list(GROUPS) + ["\\|yaw\\|", "\\|pitch\\|", "\\|roll\\|", "ln burst size"]
SINGLE = {"\\|yaw\\|": 3, "\\|pitch\\|": 4, "\\|roll\\|": 5}
ALONE = [
    ("Head pose", "Head pose"),
    ("Sharpness", "Sharpness"),
    ("AF eye focus", "AF eye focus"),
    ("Eyes open", "Eyes open"),
    ("Good photo", "Good photo"),
    ("Burst position", "Burst position"),
    ("Embedding (Look)", "Look"),
    ("c1", "c1"),
    ("c2", "c2"),
    ("Genre labels", "genres"),
]
FULL = "Unablated (rerun)"


def width(base):
    return NT + NP + gate.PCA_DIMS if base == W0 else NT + 1 + 3 + gate.PCA_DIMS


def drop_cols(base, name):
    if name == "ln burst size":
        drop = [F_LN_SIZE]
    elif name in SINGLE:
        drop = [SINGLE[name]]
    else:
        drop = GROUPS[name][base]
    return [i for i in range(width(base)) if i not in drop]


def specs(base):
    """(label, kind, payload): kind "cols" masks the base's design, "block"
    is a model of one external block alone."""
    out = [(FULL, "cols", list(range(width(base))))]
    for name in DROPS:
        if name == "ln burst size" and base == W0:
            continue
        out.append(("drop " + name, "cols", drop_cols(base, name)))
    for label, key in ALONE:
        if key in GROUPS:
            out.append(("alone " + label, "cols", GROUPS[key][base]))
        else:
            out.append(("alone " + label, "block", key))
    return out


def task(d, dw, sc, blocks, tiebreak, base, label, kind, payload, held):
    train_mask = d["folder_of"] != held
    if kind == "block":
        x = fw.append_block(np.zeros((len(d["rows"]), 0)), blocks[payload], train_mask)
    elif base == W0:
        x = gate.design(dw, train_mask, "shared")[:, payload]
    else:
        x = folder_gate.design(d, train_mask, "full")[:, payload]
    test = d["folder_of"] == held
    if base == W0:
        fob = {b: d["bursts"][b][0] for b in sc}
        train = [b for b in sc if fob[b] != held]
        c = gate.choose_c(dw, train, x, fob, tiebreak)
        return base, label, held, c, x[test] @ gate.fit(dw, train, x, c)
    c = folder_gate.choose_c(d, train_mask, x)
    w, b = folder_gate.fit(d, np.where(train_mask)[0], x, c)
    return base, label, held, c, x[test] @ w + b


def fit_all(d, dw, sc, tiebreak, base):
    every = np.ones(len(d["rows"]), dtype=bool)
    if base == W0:
        x = gate.design(dw, every, "shared")
        fob = {b: d["bursts"][b][0] for b in sc}
        c = gate.choose_c(dw, sc, x, fob, tiebreak)
        return base, c, gate.fit(dw, sc, x, c), None
    x = folder_gate.design(d, every, "full")
    c = folder_gate.choose_c(d, every, x)
    w, b = folder_gate.fit(d, np.where(every)[0], x, c)
    return base, c, w, b


def load_scores(path, d):
    with open(path, encoding="utf-8") as f:
        head = f.readline().rstrip("\n").split("\t")
        by = {}
        for line in f:
            r = line.rstrip("\n").split("\t")
            by[(r[0], r[1])] = r[2:]
    out = {}
    for name in BASES:
        j = head.index(name) - 2
        out[name] = np.array([float(by[(f["folder"], f["file"])][j]) for f in d["rows"]])
    return out


def yes(hi):
    return "yes" if hi < 0 else "no"


def main(dump, labels_dir, out_dir, jobs):
    start = time.time()
    frames, _ = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    dw = dict(d)
    dw["tech"] = d["tech"].copy()
    folder_gate.extend(d, dump)
    sc = gate.scorable(d)
    n, nf = len(d["rows"]), len(d["folders"])
    tiebreak = np.random.default_rng(0).random(n)
    pick, folder_of = d["pick"], d["folder_of"]
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    picks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(nf)}
    qual = [fi for fi in range(nf) if sizes[fi] >= fw.MIN_FRAMES and picks[fi] >= fw.MIN_PICKS]
    _, c1 = ballgate.load_extras([os.path.join(labels_dir, "extras_c1.tsv")], d)
    _, c2 = ballgate.load_extras([os.path.join(labels_dir, "extras_c2.tsv")], d)
    blocks = {"c1": c1, "c2": c2, "genres": d["genres"]}
    ref = load_scores(os.path.join(out_dir, "scores.tsv"), d)

    folds = {W0: sorted({d["bursts"][b][0] for b in sc}), F0_CTX: list(range(nf))}
    work = [(base, label, kind, payload, h) for base in BASES for label, kind, payload in specs(base) for h in folds[base]]
    print(f"{len(work)} fits on {jobs} jobs", file=sys.stderr, flush=True)
    t = time.time()
    res = Parallel(n_jobs=jobs, verbose=5)(delayed(task)(d, dw, sc, blocks, tiebreak, *w) for w in work)
    fold_time = time.time() - t
    models, cs = {}, {}
    for base, label, held, c, s in res:
        key = (base, label)
        if key not in models:
            models[key] = np.full(n, np.nan)
            cs[key] = {}
        models[key][folder_of == held] = s
        cs[key][held] = c
    t = time.time()
    fitted = {b: (c, w, i) for b, c, w, i in Parallel(n_jobs=2)(delayed(fit_all)(d, dw, sc, tiebreak, b) for b in BASES)}
    weight_time = time.time() - t

    res = {k: fw.metrics(d, s, tiebreak, sc) for k, s in models.items()}
    for b in BASES:
        res[(b, "ref")] = fw.metrics(d, ref[b], tiebreak, sc)
    pf = {k: r[0] for k, r in res.items()}
    wb = {k: r[1] for k, r in res.items()}
    summ = {k: gate.summarize(v) for k, v in wb.items()}

    def mean(k, key):
        return folder_gate.mean_sd(folder_gate.values(pf[k], key, qual))

    print("## Check: the unablated rerun against Step 2's `scores.tsv`\n")
    rows = []
    for b in BASES:
        s, r = models[(b, FULL)], ref[b]
        ok = ~np.isnan(r)
        rows.append([f"`{b}`", f"{np.abs(s[ok] - r[ok]).max():.2e}", gate.f3(mean((b, "ref"), "frame")[0]), gate.f3(mean((b, FULL), "frame")[0]),
                     gate.f3(summ[(b, "ref")]["wauc"]), gate.f3(summ[(b, FULL)]["wauc"])])
    gate.table("| Base | Max abs score difference | Frame AUC, `scores.tsv` | Frame AUC, rerun | Within-burst wAUC, `scores.tsv` | Within-burst wAUC, rerun |", rows)

    print("## Drop one (ablated minus unablated)\n")
    print(
        f"Each row is a full held-out rerun (leave one folder out, the same `C` grid and inner CV) without the named columns, "
        f"against the unablated model's held-out score in Step 2's `scores.tsv`. Frame AUC and R-precision: the mean paired "
        f"per-folder difference over the {len(qual)} qualifying folders with the 1,000-draw folder bootstrap 95% interval "
        "(seed 0). Within-burst: the per-burst weighted AUC difference with `genre_offsets.boot`'s per-burst bootstrap, and "
        "the Top-1 difference. *Contributes* when the interval lies entirely below zero.\n"
    )
    for b in BASES:
        print(f"### `{b}`\n")
        rows = []
        for label, _, _ in specs(b):
            if not label.startswith("drop "):
                continue
            k = (b, label)
            fa = folder_gate.paired(folder_gate.values(pf[k], "frame", qual), folder_gate.values(pf[(b, "ref")], "frame", qual))
            ra = folder_gate.paired(folder_gate.values(pf[k], "rp", qual), folder_gate.values(pf[(b, "ref")], "rp", qual))
            fm, flo, fhi = fw.boot_folders(fa)
            rm, rlo, rhi = fw.boot_folders(ra)
            wm, wlo, whi = genre_offsets.boot(wb[k], wb[(b, "ref")], sc, np.random.default_rng(0))
            dt = 100 * (summ[k]["top1"] - summ[(b, "ref")]["top1"])
            rows.append([label[5:], fw.ci(fm, flo, fhi), yes(fhi), fw.ci(rm, rlo, rhi, 1), yes(rhi), fw.ci(wm, wlo, whi), yes(whi), f"{dt:+.1f}"])
        gate.table(
            "| Dropped | Frame AUC | Contributes | R-precision, points | Contributes | Within-burst wAUC | Contributes | Top-1, points |", rows
        )

    print("## Each group alone\n")
    print(
        f"A model of only that group's columns (with its missing-value indicators), held out as above. Frame AUC and R-precision: "
        f"the mean over the {len(qual)} qualifying folders (per-folder SD in brackets); R-precision chance "
        f"{float(np.mean([100 * picks[fi] / sizes[fi] for fi in qual])):.1f}%. Within-burst: weighted pairwise AUC and Top-1 "
        "over the scorable bursts. On `F0+ctx`, sharpness is the relative and the absolute `ln sharpness`, burst position "
        "is position and first frame (no `ln burst size`).\n"
    )
    rows = []
    for label, _ in ALONE:
        r = [label]
        for b in BASES:
            k = (b, "alone " + label)
            fm, fs, _ = mean(k, "frame")
            rm, rs, _ = mean(k, "rp")
            r += [fw.pm(fm, fs), fw.pm(rm, rs, digits=1), gate.f3(summ[k]["wauc"]), gate.pc(summ[k]["top1"])]
        rows.append(r)
    for b in BASES:
        r = [f"All (`{b}` unablated)"]
        for bb in BASES:
            if bb != b:
                r += ["", "", "", ""]
                continue
            k = (b, "ref")
            fm, fs, _ = mean(k, "frame")
            rm, rs, _ = mean(k, "rp")
            r += [fw.pm(fm, fs), fw.pm(rm, rs, digits=1), gate.f3(summ[k]["wauc"]), gate.pc(summ[k]["top1"])]
        rows.append(r)
    gate.table(
        "| Alone | `F0+ctx` frame AUC | R-precision % | Within-burst wAUC | Top-1 | `W0` frame AUC | R-precision % | Within-burst wAUC | Top-1 |",
        rows,
    )

    print("## Fitted weights (all 27 folders)\n")
    print(
        "Each base fit on every folder (`W0` on every scorable burst) with `C` chosen by the same inner grouped CV, on "
        "standardized columns, so the weights compare across columns. Grouped as `rank.rs`'s `REASONS`; Look is the L2 norm "
        "of its 32 PCA weights.\n"
    )
    fc, fwt, fb = fitted[F0_CTX]
    wc, wwt, _ = fitted[W0]
    rows = [["", "`C`", f"{wc:g}", f"{fc:g}"], ["", "Intercept", "-", f"{fb:+.3f}"]]
    rows.append(["Look", "L2 norm of 32 PCA weights", f"{np.linalg.norm(wwt[W_PCA]):.3f}", f"{np.linalg.norm(fwt[F_PCA]):.3f}"])
    names = {W0: gate.TECH + gate.POSITION, F0_CTX: folder_gate.TECH + folder_gate.CONTEXT}
    at = {b: {name: i for i, name in enumerate(names[b])} for b in BASES}
    for g in GROUPS:
        if g == "Look":
            continue
        cols = [names[F0_CTX][i] for i in GROUPS[g][F0_CTX]]
        cols += [names[W0][j] for j in GROUPS[g][W0] if names[W0][j] not in cols]
        for col in cols:
            cell = [f"{wt[at[b][col]]:+.3f}" if at[b].get(col) in GROUPS[g][b] else "-" for b, wt in ((W0, wwt), (F0_CTX, fwt))]
            rows.append([g, col] + cell)
    rows.append(["(context)", "ln burst size", "-", f"{fwt[F_LN_SIZE]:+.3f}"])
    gate.table("| Reason | Column | `W0` | `F0+ctx` |", rows)

    print("`C` chosen per fold (count of folds):\n")
    gate.table(
        "| Model | " + " | ".join(str(c) for c in gate.CS) + " |",
        [[f"`{b}` {label}"] + [sum(1 for v in cs[(b, label)].values() if v == c) for c in gate.CS] for b, label in cs],
    )

    print("## Wall time\n")
    print(f"{len(work)} held-out fits {fold_time:.0f} s, the all-folder fits {weight_time:.0f} s, total {time.time() - start:.0f} s on {jobs} jobs.")


if __name__ == "__main__":
    if len(sys.argv) not in (4, 5):
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]) if len(sys.argv) > 4 else 8)
