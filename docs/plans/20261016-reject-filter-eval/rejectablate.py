"""Step 3: the pre-registered column and group ablations of `F0+ctx` and `W0`
(the archived `ablate.specs`, fit by `ablate.task` unchanged) and the drop-one
ablations of `G0`, held out by folder, read as a bottom (reject) filter on
Step 2's metrics against the unablated held-out scores. The design and the bars
are pre-registered in reject.md.

Usage: python -I -B rejectablate.py <app-dump-dir> <ball-labels-dir> <folder-wide-eval-dir> <out-dir> [jobs]

Arguments as for rejectfilter.py; <out-dir> also holds Step 2's
`scores_reject.tsv` (the unablated `G0`). Writes `scores_ablate.tsv` (folder,
file, one held-out score column per ablated model) into <out-dir> and prints
the ablation tables (Markdown) on stdout.
"""

import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.ensemble import HistGradientBoostingClassifier

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rejectfilter as rf  # noqa: E402
import ablate  # noqa: E402

gate, folder_gate, ballgate, fw = rf.gate, rf.folder_gate, rf.ballgate, rf.fw
W0, F0_CTX, G0 = rf.W0, rf.F0_CTX, rf.G0
BASES = [F0_CTX, W0, G0]
FULL = ablate.FULL


def g0_specs():
    return [(label, kind, cols) for label, kind, cols in ablate.specs(F0_CTX) if label == FULL or label.startswith("drop ")]


def g0_task(d, label, cols, held):
    """`rejectfilter.g0_fold` (weights rescaled to mean 1 per training fold) on the named columns of its design."""
    train_mask = d["folder_of"] != held
    x = folder_gate.design(d, train_mask, "full")
    x[:, : d["tech"].shape[1]][np.isnan(d["tech"])] = np.nan
    x = x[:, cols]
    idx = np.where(train_mask)[0]
    w = d["weight"][idx]
    m = HistGradientBoostingClassifier(**rf.HGB).fit(x[idx], d["pick"][idx].astype(int), sample_weight=w * len(idx) / w.sum())
    return G0, label, held, None, m.decision_function(x[d["folder_of"] == held])


def yes(hi):
    return "yes" if hi < 0 else "no"


def main(dump, labels_dir, eval_dir, out_dir, jobs):
    start = time.time()
    frames, _ = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    dw = dict(d)
    dw["tech"] = d["tech"].copy()
    folder_gate.extend(d, dump)
    sc = gate.scorable(d)
    n, nf = len(d["rows"]), len(d["folders"])
    tb = np.random.default_rng(0).random(n)
    pick, folder_of = d["pick"], d["folder_of"]
    rej = d["reject"] & ~pick
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    picks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(nf)}
    qual = [fi for fi in range(nf) if sizes[fi] >= fw.MIN_FRAMES and picks[fi] >= fw.MIN_PICKS]
    _, c1 = ballgate.load_extras([os.path.join(labels_dir, "extras_c1.tsv")], d)
    _, c2 = ballgate.load_extras([os.path.join(labels_dir, "extras_c2.tsv")], d)
    blocks = {"c1": c1, "c2": c2, "genres": d["genres"]}
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_g0, _ = rf.load_scores(os.path.join(out_dir, "scores_reject.tsv"), d)
    ref = {F0_CTX: stored[F0_CTX], W0: stored[W0], G0: stored_g0[G0]}
    load_time = time.time() - start

    folds = {W0: sorted({d["bursts"][b][0] for b in sc}), F0_CTX: list(range(nf))}
    specs = {F0_CTX: ablate.specs(F0_CTX), W0: ablate.specs(W0), G0: g0_specs()}
    work = [delayed(g0_task)(d, label, cols, h) for label, _, cols in specs[G0] for h in range(nf)]
    work += [delayed(ablate.task)(d, dw, sc, blocks, tb, base, label, kind, payload, h) for base in (F0_CTX, W0) for label, kind, payload in specs[base] for h in folds[base]]
    print(f"{len(work)} fits on {jobs} jobs", file=sys.stderr, flush=True)
    t = time.time()
    res = Parallel(n_jobs=jobs, verbose=5)(work)
    fold_time = time.time() - t
    models = {}
    for base, label, held, _, s in res:
        models.setdefault((base, label), np.full(n, np.nan))[folder_of == held] = s

    keys = [(b, label) for b in BASES for label, _, _ in specs[b]]
    with open(os.path.join(out_dir, "scores_ablate.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("folder\tfile\t" + "\t".join(f"{b} {label}".replace("\\", "") for b, label in keys) + "\n")
        for i, f in enumerate(d["rows"]):
            out.write(f"{f['folder']}\t{f['file']}\t" + "\t".join(f"{models[k][i]:.6g}" for k in keys) + "\n")

    t = time.time()
    pf = {k: rf.evaluate(k, models[k], d, tb, qual, rej) for k in keys}
    for b in BASES:
        pf[(b, "ref")] = rf.evaluate(b, ref[b], d, tb, qual, rej)
    metric_time = time.time() - t

    def vals(k, key):
        return {fi: pf[k][fi][key] for fi in qual}

    def mean(k, key):
        return folder_gate.mean_sd(vals(k, key))

    def lf(k):
        return [pf[k][fi]["q95"] for fi in qual]

    print("## Check: the unablated rerun against the stored scores\n")
    rows = []
    for b in BASES:
        s, r = models[(b, FULL)], ref[b]
        ok = ~np.isnan(r)
        src = "`scores_reject.tsv`" if b == G0 else "`scores.tsv`"
        rows.append([f"`{b}`", src, f"{np.abs(s[ok] - r[ok]).max():.2e}", rf.f1(mean((b, "ref"), "rn95")[0]), rf.f1(mean((b, FULL), "rn95")[0]),
                     gate.f3(mean((b, "ref"), "pauc")[0]), gate.f3(mean((b, FULL), "pauc")[0])])
    gate.table("| Base | Stored in | Max abs score difference | `RN@95` %, stored | `RN@95` %, rerun | `pAUC95`, stored | `pAUC95`, rerun |", rows)

    print("## Drop one (ablated minus unablated)\n")
    print(
        "Each row is a full held-out rerun (leave one folder out; for `F0+ctx` and `W0` the same `C` grid and inner CV as the "
        "archived ablations, for `G0` its fixed setting) without the named columns, against the unablated model's stored "
        f"held-out score. The ablated mean `RN@95`, then the mean paired per-folder difference over the {len(qual)} qualifying "
        "folders with the 1,000-draw folder bootstrap 95% interval (seed 0, fresh per comparison and metric), in points for "
        "`RN`. *Contributes* when the interval lies entirely below zero. `G0` runs the drop-one rows only (Step 2 named it the "
        "best filter).\n"
    )
    for b in BASES:
        print(f"### `{b}`\n")
        rows = []
        for label, _, _ in specs[b]:
            if not label.startswith("drop "):
                continue
            k = (b, label)
            r = [label[5:], rf.f1(mean(k, "rn95")[0])]
            for key, digits in (("rn95", 1), ("rn99", 1), ("pauc", 3)):
                m, lo, hi = fw.boot_folders(folder_gate.paired(vals(k, key), vals((b, "ref"), key)))
                r += [fw.ci(m, lo, hi, digits), yes(hi)]
            rows.append(r)
        gate.table(
            "| Dropped | `RN@95` % | `RN@95`, points | Contributes | `RN@99`, points | Contributes | `pAUC95` | Contributes |", rows
        )

    print("## Each group alone\n")
    print(
        f"A model of only that group's columns (with its missing-value indicators), held out as above; the mean over the {len(qual)} "
        "qualifying folders (per-folder SD in brackets) and the label-free `quantile@95` pair (mean realized pick recall % / "
        "mean `RN` %, gap to 95 in brackets). On `F0+ctx`, sharpness is the relative and the absolute `ln sharpness`, burst "
        "position is position and first frame (no `ln burst size`). Random (seed 0) reads `RN@95` 5.8%, `pAUC95` 0.504.\n"
    )
    for b in (F0_CTX, W0):
        print(f"### `{b}`\n")
        rows = []
        for label, _ in ablate.ALONE + [(f"All (`{b}` unablated)", None)]:
            k = (b, "ref") if label.startswith("All") else (b, "alone " + label)
            r = [label]
            for key, digits in (("rn95", 1), ("rn99", 1), ("pauc", 3)):
                m, s, _ = mean(k, key)
                r.append(fw.pm(m, s, digits=digits))
            r += [rf.pair(lf(k), 95), gate.f3(mean(k, "frame")[0])]
            rows.append(r)
        gate.table("| Alone | `RN@95` % | `RN@99` % | `pAUC95` | `quantile@95` recall / `RN` (gap) | Frame AUC |", rows)

    print("## Wall time\n")
    print(f"Load {load_time:.0f} s, {len(work)} held-out fits {fold_time:.0f} s, metrics {metric_time:.0f} s, total {time.time() - start:.0f} s on {jobs} jobs.")


if __name__ == "__main__":
    if len(sys.argv) not in (5, 6):
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]) if len(sys.argv) > 5 else 8)
