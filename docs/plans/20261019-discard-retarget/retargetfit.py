"""Step 3: the held-out scores of `G0+B`, `D0` and `D0+B` on every dump frame,
fit as pre-registered in retarget.md, with the label-free readings only.

Usage: python -I -B retargetfit.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <discard-labels-dir> <strict-labels-dir> <out-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv` (not read here; the argument keeps the command line Step 4's);
<reject-filter-eval-dir> `scores_reject.tsv` (the stored `G0`);
<discard-labels-dir> `labels.tsv`, read for the training of `D0` / `D0+B`
only; <strict-labels-dir> the strict sample's `manifest.tsv`, read for the
population only; <out-dir> holds Step 1's `burst_features.tsv`. Writes
`scores_retarget.tsv` (`folder`, `file`, `G0+B`, `D0`, `D0+B`; held out,
higher keeps) into <out-dir>, prints the label-free readings (Markdown) and
saves them as `retargetfit.out.md` there. No score is read against the labels.
"""

import csv
import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.ensemble import HistGradientBoostingClassifier

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261018-discard-labels"))
import discardfilter as df  # noqa: E402

sys.path.insert(0, HERE)
import burstfeat  # noqa: E402

rf, gate, folder_gate = bf.rf, bf.gate, bf.folder_gate

CELLS = ["G0+B", "D0", "D0+B"]
TOL = 1e-5


def base_design(d, held):
    """`G0`'s 46 columns for the fold holding out `held`, the NaN of the technical columns restored."""
    x = folder_gate.design(d, d["folder_of"] != held, "full")
    x[:, : d["tech"].shape[1]][np.isnan(d["tech"])] = np.nan
    return x


def g0b_fold(d, held, b):
    """`rejectfilter.g0_fold` with the columns `b` appended: target `pick`, every frame of the other folders."""
    x = np.hstack([base_design(d, held), b])
    idx = np.where(d["folder_of"] != held)[0]
    w = d["weight"][idx]
    m = HistGradientBoostingClassifier(**rf.HGB).fit(x[idx], d["pick"][idx].astype(int), sample_weight=w * len(idx) / w.sum())
    return held, None, m.decision_function(x[d["folder_of"] == held])


def d0_fold(d, held, s, b):
    """The `discard` target on the other folders' labeled frames, `1/π` rescaled to mean 1; higher keeps."""
    x = base_design(d, held)
    if b is not None:
        x = np.hstack([x, b])
    tr = s["folder"] != held
    w = s["w"][tr]
    m = HistGradientBoostingClassifier(**rf.HGB).fit(x[s["idx"][tr]], s["discard"][tr].astype(int), sample_weight=w * tr.sum() / w.sum())
    return held, None, -m.decision_function(x[d["folder_of"] == held])


def kish(w):
    return float(w.sum() ** 2 / (w**2).sum())


def load_b(path, d):
    with open(path, encoding="utf-8", newline="") as f:
        by = {(r["folder"], r["file"]): [float(r[c]) for c in burstfeat.COLUMNS] for r in csv.DictReader(f, delimiter="\t")}
    return np.array([by[(r["folder"], r["file"])] for r in d["rows"]]), len(by)


def main(dump, eval_dir, reject_dir, labels_dir, strict_dir, out_dir, jobs):
    start = time.time()
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n, nf = len(rows), len(folders)
    tb = np.random.default_rng(0).random(n)
    pick = d["pick"]
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    g0 = stored_rej["G0"]
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}

    b, b_rows = load_b(os.path.join(out_dir, "burst_features.tsv"), d)
    b_re = burstfeat.features(d, rf.load_ear(dump, d))
    b_same = bool(np.array_equal(np.isnan(b), np.isnan(b_re)) and np.array_equal(b[~np.isnan(b)], b_re[~np.isnan(b_re)]))

    labels = bf.read_tsv(os.path.join(labels_dir, "labels.tsv"))
    s = {
        "idx": np.array([index[(r["folder"], r["file"])] for r in labels]),
        "discard": np.array([r["label"] == "discard" for r in labels]),
        "pi": np.array([float(r["inclusion_prob"]) for r in labels]),
    }
    label_set = {r["label"] for r in labels}
    s["w"] = 1 / s["pi"]
    s["folder"] = folder_of[s["idx"]]
    seen = np.zeros(n, dtype=bool)
    for r in bf.read_tsv(os.path.join(strict_dir, "manifest.tsv")):
        seen[index[(r["folder"], r["file"])]] = True
    pop = ~seen
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    scores, times = {}, {}

    def run(name, f):
        t = time.time()
        scores[name], _ = bf.fw.collect(d, Parallel(n_jobs=jobs)(f(h) for h in range(nf)))
        times[name] = time.time() - t
        print(f"{name}: {times[name]:.0f} s", file=sys.stderr, flush=True)

    run("G0+B", lambda h: delayed(g0b_fold)(d, h, b))
    run("D0", lambda h: delayed(d0_fold)(d, h, s, None))
    run("D0+B", lambda h: delayed(d0_fold)(d, h, s, b))

    t = time.time()
    spot = max(range(nf), key=lambda fi: int((folder_of == fi).sum()))
    sm = folder_of == spot
    _, _, zeroed = g0b_fold(d, spot, np.zeros_like(b))
    _, _, refit = rf.g0_fold(d, spot)
    times["Spot-fold checks (2 fits)"] = time.time() - t

    with open(os.path.join(out_dir, "scores_retarget.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("folder\tfile\t" + "\t".join(CELLS) + "\n")
        for i, f in enumerate(rows):
            out.write(f"{f['folder']}\t{f['file']}\t" + "\t".join(repr(float(scores[k][i])) for k in CELLS) + "\n")

    print("# Retarget fits: held-out scores and label-free readings\n")
    print("No score of this step is read against `labels.tsv`: the labels enter the training of `D0` / `D0+B` only, and no `DP`, `RR` or AUC is computed.\n")

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{nf} / {n} / {missing}"),
            ("Dump picks / strict-sample frames / population", f"{int(pick.sum())} / {int(seen.sum())} / {int(pop.sum())}"),
            ("`burst_features.tsv` rows / columns", f"{b_rows} / {b.shape[1]}"),
            ("Labeled frames (training of `D0` / `D0+B`) / discard / keep", f"{len(labels)} / {int(s['discard'].sum())} / {int((~s['discard']).sum())}"),
            ("Design columns: base / with B", f"{d['tech'].shape[1] + d['context'].shape[1] + gate.PCA_DIMS} / {d['tech'].shape[1] + d['context'].shape[1] + gate.PCA_DIMS + b.shape[1]}"),
        ],
    )

    print("## Per-fold training\n")
    print(
        "Per held-out folder: its frames, and each cell's training frames, positives (`G0+B`: the other folders' picks; `D0` / `D0+B`: the other "
        "folders' labeled `discard` frames) and the Kish effective sample size `(Σw)² / Σw²` of the training weights (`G0+B`: the dump's per-folder "
        "class weights; `D0` / `D0+B`: `1/π`). `D0+B` trains on the same frames and weights as `D0`.\n"
    )
    frows, ess_g, ess_d = [], [], []
    for fi in range(nf):
        tr = folder_of != fi
        ltr = s["folder"] != fi
        eg, ed = kish(d["weight"][tr]), kish(s["w"][ltr])
        ess_g.append(eg)
        ess_d.append(ed)
        frows.append([f"`{folders[fi]}`", int((~tr).sum()), int(tr.sum()), int((pick & tr).sum()), f"{eg:.0f}", int(ltr.sum()), int((s["discard"] & ltr).sum()), f"{ed:.1f}"])
    gate.table("| Held-out folder | Frames | `G0+B` training frames | Picks | ESS | `D0` training frames | Discard | ESS |", frows)
    gate.table(
        "| Reading | Min | Max |",
        [
            ("`G0+B` training frames", min(r[2] for r in frows), max(r[2] for r in frows)),
            ("`G0+B` Kish ESS", f"{min(ess_g):.0f}", f"{max(ess_g):.0f}"),
            ("`D0` / `D0+B` training frames", min(r[5] for r in frows), max(r[5] for r in frows)),
            ("`D0` / `D0+B` discard among them", min(r[6] for r in frows), max(r[6] for r in frows)),
            ("`D0` / `D0+B` Kish ESS", f"{min(ess_d):.1f}", f"{max(ess_d):.1f}"),
        ],
    )

    print("## Label-free cuts: removed share and pick loss\n")
    print(
        f"Per folder on the full dump, the bottom `floor(q * n / 100)` frames by the score, ties by the seed-0 draw (`bf.graded_cut`). Removed: "
        f"the share of the population's frames inside the cut (full dump's after the slash). `PL`: the share of the dump's picks inside the cut "
        f"(full dump; population after the slash). Bars (label-free, decided here): `PL` (full dump) <= {df.PL_BAR:.0f}% and removed (population) "
        f">= {df.REMOVED_BAR}%. `G0` (stored) is the reference, not a candidate.\n"
    )
    crows = []
    for k, sc in [("G0", g0)] + [(k, scores[k]) for k in CELLS]:
        for q in df.QS:
            removed, out = bf.graded_cut(sc, d, tb, q)
            cover = ~out
            pl = 100 * (removed & pick & cover).sum() / (pick & cover).sum()
            pl_pop = 100 * (removed & pick & cover & pop).sum() / (pick & cover & pop).sum()
            rem_pop = 100 * (removed & cover & pop).sum() / (cover & pop).sum()
            rem_dump = 100 * (removed & cover).sum() / cover.sum()
            ok_pl, ok_rem = pl <= df.PL_BAR, rem_pop >= df.REMOVED_BAR
            crows.append([f"`{k}`", f"q{q}", int(out.sum()), f"{rem_pop:.2f} / {rem_dump:.2f}", f"{pl:.2f} / {pl_pop:.2f}",
                          "yes" if ok_pl else "no", "yes" if ok_rem else "no"])
    gate.table("| Score | Cut | Frames out | Removed % | `PL` % | `PL` <= 5% | Removed >= 4.5% |", crows)

    print("## Checks\n")
    tol = TOL * np.maximum(1, np.abs(g0[sm]))
    d_zero, d_refit = np.abs(zeroed - g0[sm]), np.abs(refit - g0[sm])
    strict_train = int(seen[s["idx"]].sum())
    checks = [
        ("`burst_features.tsv` matches Step 1's `features` recomputed (NaN pattern and values)", "equal", "equal" if b_same else "differ", b_same),
        (f"`G0+B` with the B columns zeroed vs stored `G0`, spot fold `{folders[spot]}` ({int(sm.sum())} frames): max abs difference",
         f"<= {TOL:g} x max(1, abs)", f"{d_zero.max():.2e}", bool((d_zero <= tol).all())),
        ("`rejectfilter.g0_fold` refit vs stored `G0`, same fold: max abs difference", f"<= {TOL:g} x max(1, abs)", f"{d_refit.max():.2e}", bool((d_refit <= tol).all())),
    ]
    checks += [(f"`{k}` finite on every dump frame", n, int(np.isfinite(scores[k]).sum()), bool(np.isfinite(scores[k]).all())) for k in CELLS]
    checks += [
        ("Labeled training frames that are in the strict sample (the strict labels do not enter training)", 0, strict_train, strict_train == 0),
        ("`labels.tsv` labels within {keep, discard}", "yes", "yes" if label_set <= {"keep", "discard"} else "no", label_set <= {"keep", "discard"}),
    ]
    gate.table("| Check | Expected | This run | Result |", [(k, e, v, "pass" if ok else "FAIL") for k, e, v, ok in checks])

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored `G0`, B file and recomputation, labels)", f"{load_time:.0f}")] + [(f"`{k}`" if k in CELLS else k, f"{v:.0f}") for k, v in times.items()])
    print(f"Total {time.time() - start:.0f} s on {jobs} jobs.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "retargetfit.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    if not all(ok for *_, ok in checks):
        raise SystemExit("a check failed")


if __name__ == "__main__":
    if len(sys.argv) not in (7, 8):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:7], int(sys.argv[7]) if len(sys.argv) > 7 else 8)
