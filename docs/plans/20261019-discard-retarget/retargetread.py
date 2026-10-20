"""Step 4: every quantity of retarget.md's Pre-registration against the blind
keep / discard labels: the per-folder cuts of `G0`, `G0+B`, `D0`, `D0+B`,
`eye_focus` and random, their metrics and AUCs weighted by `1/π`, the paired
effects A and B with stratified and folder-cluster bootstrap intervals, the
coherence check on the strict labels, and the bars under the chosen and the
rejected option.

Usage: python -I -B retargetread.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <discard-labels-dir> <strict-labels-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv` (not read here; the argument keeps the command line Step 3's);
<reject-filter-eval-dir> `scores_reject.tsv` (the stored `G0`);
<discard-labels-dir> `labels.tsv` and `strata.tsv`; <strict-labels-dir> the
strict sample's `labels.tsv` and `manifest.tsv`; <out-dir> holds Step 3's
`scores_retarget.tsv`. Prints the result tables (Markdown) and saves them as
`retargetread.out.md` in <out-dir>.
"""

import os
import sys
import time

import numpy as np
from sklearn.metrics import roc_auc_score

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261018-discard-labels"))
import discardfilter as df  # noqa: E402

rf, gate, folder_gate, fw = bf.rf, bf.gate, bf.folder_gate, bf.fw

CELLS = ["G0+B", "D0", "D0+B"]
SCORES = ["G0"] + CELLS + [fw.EYE, fw.RANDOM]
QS = df.QS
BARS = [("chosen", 25.0, 35.0), ("rejected (a reading only)", df.RR_BAR, df.RR_HI_BAR)]
CONDS = ("RR", "RR97.5", "PL", "removed", "random", "G0")
EFFECTS = [
    ("A: `D0` - `G0`", {"D0": 1, "G0": -1}),
    ("A: `D0+B` - `G0+B`", {"D0+B": 1, "G0+B": -1}),
    ("B: `G0+B` - `G0`", {"G0+B": 1, "G0": -1}),
    ("B: `D0+B` - `D0`", {"D0+B": 1, "D0": -1}),
    ("Interaction: (`D0+B` - `D0`) - (`G0+B` - `G0`)", {"D0+B": 1, "D0": -1, "G0+B": -1, "G0": 1}),
    ("A+B: `D0+B` - `G0`", {"D0+B": 1, "G0": -1}),
] + [(f"`{k}` - {r}", {k: 1, r: -1}) for k in CELLS for r in (fw.EYE, fw.RANDOM)]
BANDS = [("bottom 5%", None, 5), ("5-10%", 5, 10), ("10-20%", 10, 20), ("above 20%", 20, None)]


def name(k):
    return df.name(k)


def auc_of(dis, sc, w):
    keep = w > 0
    return float(roc_auc_score(dis[keep], -sc[keep], sample_weight=w[keep]))


def combine(vals, coefs):
    return sum(c * vals[k] for k, c in coefs.items())


def cl(lo_hi):
    return f"[{lo_hi[0]:+.1f}, {lo_hi[1]:+.1f}]"


def main(dump, eval_dir, reject_dir, labels_dir, strict_dir, out_dir):
    start = time.time()
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n = len(rows)
    tb = np.random.default_rng(0).random(n)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    retarget, _ = rf.load_scores(os.path.join(out_dir, "scores_retarget.tsv"), d)
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    pick_d = d["pick"]

    labels = bf.read_tsv(os.path.join(labels_dir, "labels.tsv"))
    s = {
        "idx": np.array([index[(r["folder"], r["file"])] for r in labels]),
        "bad": np.array([r["label"] == "discard" for r in labels]),
        "pi": np.array([float(r["inclusion_prob"]) for r in labels]),
        "band": np.array([r["band"] for r in labels], dtype=object),
    }
    label_set = {r["label"] for r in labels}
    s["w"] = 1 / s["pi"]
    s["folder"] = folder_of[s["idx"]]
    s["stratum"] = np.where(s["band"] == "control", "rest", s["band"])
    strict_labels = bf.read_tsv(os.path.join(strict_dir, "labels.tsv"))
    t = {
        "idx": np.array([index[(r["folder"], r["file"])] for r in strict_labels]),
        "bad": np.array([r["label"] == "bad" for r in strict_labels]),
        "pi": np.array([float(r["inclusion_prob"]) for r in strict_labels]),
        "stratum": np.array([r["stratum"] for r in strict_labels], dtype=object),
    }
    t["w"] = 1 / t["pi"]
    t["folder"] = folder_of[t["idx"]]
    seen = np.zeros(n, dtype=bool)
    for r in bf.read_tsv(os.path.join(strict_dir, "manifest.tsv")):
        seen[index[(r["folder"], r["file"])]] = True
    pop = ~seen
    strata_file = bf.read_tsv(os.path.join(labels_dir, "strata.tsv"))
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    t0 = time.time()
    idx, dis, w = s["idx"], s["bad"], s["w"]
    ns = len(idx)
    g0 = stored_rej["G0"]
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    graded = {"G0": g0, **{k: retarget[k] for k in CELLS}, fw.EYE: np.where(np.isnan(ef), -np.inf, ef), fw.RANDOM: tb}

    cut = {q: bf.graded_cut(g0, d, tb, q)[0] for q in QS}
    band = np.full(n, "control", dtype=object)
    band[cut[20]] = "b20"
    band[cut[10]] = "b10"
    band[cut[5]] = "b05"
    strata_diff = 0
    pi_pop = np.zeros(n)
    for r in strata_file:
        m = pop & (band == r["band"])
        sm = s["band"] == r["band"]
        if r["band"] == "control":
            fi = folders.index(r["folder"])
            m = m & (folder_of == fi)
            sm = sm & (s["folder"] == fi)
        size, drawn = int(m.sum()), int(sm.sum())
        pi_pop[m] = drawn / size
        strata_diff += size != int(r["population"]) or drawn != int(r["drawn"]) or abs(drawn / size - float(r["inclusion_prob"])) > 1e-5 * float(r["inclusion_prob"])
    band_diff = int((band[idx] != s["band"]).sum())
    pi_diff = int((np.abs(pi_pop[idx] - s["pi"]) > 1e-5 * s["pi"]).sum())
    in_strict = int(seen[idx].sum())

    cs = df.counts(s, folders, ["b05", "b10", "b20", "rest"])
    cc = bf.cluster_counts(s, folders)
    ct = bf.strat_counts(t, folders)

    res, removed_of, frames_out = {}, {}, 0
    for k, sc in graded.items():
        for q in QS:
            removed, out = bf.graded_cut(sc, d, tb, q)
            removed_of[(k, q)] = removed
            frames_out += int(out.sum())
            rem, ok = removed[idx], ~out[idx]
            cover = ~out
            boot, clus = df.metrics(s, rem, ok, cs), df.metrics(s, rem, ok, cc)
            res[(k, q)] = {
                "rem": rem, "point": df.metrics(s, rem, ok), "unw": df.unweighted(s, rem, ok), "boot": boot, "cboot": clus,
                "ci": {m: bf.quant(v) for m, v in boot.items()}, "cci": {m: bf.quant(v) for m, v in clus.items()},
                "pl": 100 * (removed & pick_d & cover).sum() / (pick_d & cover).sum(),
                "pl_pop": 100 * (removed & pick_d & cover & pop).sum() / (pick_d & cover & pop).sum(),
                "removed_pop": 100 * (removed & cover & pop).sum() / (cover & pop).sum(),
                "removed_dump": 100 * (removed & cover).sum() / cover.sum(),
                "n_rem": int(rem.sum()), "n_dis": int((rem & dis).sum()),
                "picks_rem": int((rem & pick_d[idx]).sum()), "picks_rem_dis": int((rem & pick_d[idx] & dis).sum()),
            }

    auc = {}
    for k in SCORES:
        sc = bf.finite_by_folder(graded[k], d)[idx]
        auc[k] = {
            "point": auc_of(dis, sc, w), "unw": auc_of(dis, sc, np.ones(ns)),
            "boot": np.array([auc_of(dis, sc, w * cs[b]) for b in range(bf.BOOT)]),
            "cboot": np.array([auc_of(dis, sc, w * cc[b]) for b in range(bf.BOOT)]),
        }
    print(f"metrics and AUCs: {time.time() - t0:.0f} s", file=sys.stderr, flush=True)

    def effect(coefs, q, m):
        """A linear combination of the scores' metric `m` at `q` (or AUC when `q` is None): point, stratified and cluster draws."""
        if q is None:
            return tuple(combine({k: auc[k][f] for k in coefs}, coefs) for f in ("point", "boot", "cboot"))
        return (combine({k: res[(k, q)]["point"][m] for k in coefs}, coefs), combine({k: res[(k, q)]["boot"][m] for k in coefs}, coefs),
                combine({k: res[(k, q)]["cboot"][m] for k in coefs}, coefs))

    total_w = w.sum()
    base = 100 * (w * dis).sum() / total_w
    base_ci = bf.quant(bf.ratio(cs, w * dis, w))
    base_cci = bf.quant(bf.ratio(cc, w * dis, w))

    verdicts = {}
    for bar, rr_bar, rr_hi in BARS:
        for k in CELLS:
            for q in QS:
                r = res[(k, q)]
                p, b, c = effect({k: 1, "G0": -1}, q, "DP")
                conds = {
                    "RR": r["point"]["RR"] <= rr_bar,
                    "RR97.5": r["ci"]["RR"][1] <= rr_hi,
                    "PL": r["pl"] <= df.PL_BAR,
                    "removed": r["removed_pop"] >= df.REMOVED_BAR,
                    "random": r["ci"]["DP"][0] > base,
                    "G0": p > 0 and bf.quant(b)[0] > 0,
                }
                usable = all(conds.values())
                fragile = usable and (r["cci"]["RR"][1] > rr_hi or r["cci"]["DP"][0] <= base or bf.quant(c)[0] <= 0)
                verdicts[(bar, k, q)] = {"conds": conds, "usable": usable, "fragile": fragile, "g0": (p, bf.quant(b), bf.quant(c))}

    def best_of(bar):
        passing = [(k, q) for k in CELLS for q in QS if verdicts[(bar, k, q)]["usable"]]
        if not passing:
            return None
        deepest = max(q for _, q in passing)
        k = max((k for k, q in passing if q == deepest), key=lambda k: verdicts[(bar, k, deepest)]["g0"][0])
        return k, deepest

    follow = {lab: effect(coefs, 5, "DP") for lab, coefs in [("A: `D0` - `G0`", {"D0": 1, "G0": -1}), ("A+B: `D0+B` - `G0`", {"D0+B": 1, "G0": -1})]}
    best = best_of(BARS[0][0])
    if best is not None:
        branch = "1 (a usable cut exists: an in-app discard candidates sorted-last view, starting with a fresh blind confirmatory round)"
    elif any(bf.quant(b)[0] > 0 for _, b, _ in follow.values()):
        branch = "2 (no usable cut, effect A or A+B positive at q5: more labels)"
    else:
        branch = "3 (otherwise: nothing)"
    metric_time = time.time() - t0

    print("# Retarget read: the cells against the blind discard labels\n")

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{len(folders)} / {n} / {missing}"),
            ("Strict-sample frames excluded / population", f"{int(seen.sum())} / {int(pop.sum())}"),
            ("Sampled frames (`labels.tsv`) / keep / discard", f"{ns} / {int((~dis).sum())} / {int(dis.sum())}"),
            ("Strict sample (`labels.tsv` there) / strict bad", f"{len(t['idx'])} / {int(t['bad'].sum())}"),
        ],
    )

    print("## Discardable base rate\n")
    gate.table(
        "| Reading | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |",
        [("Discardable base rate", f"{base:.2f}", f"[{base_ci[0]:.2f}, {base_ci[1]:.2f}]", f"[{base_cci[0]:.2f}, {base_cci[1]:.2f}]", f"{100 * dis.mean():.2f}")],
    )

    print("## Reference checks\n")
    print("The stored `G0`, `eye_focus` and random must reproduce discard.md's Results on the same code path; the strata and `π` must match `strata.tsv`.\n")

    def ivs(k, q, m):
        r = res[(k, q)]
        return bf.iv(r["point"][m], *r["ci"][m])

    def aucs_iv(k):
        return bf.iv(auc[k]["point"], *bf.quant(auc[k]["boot"]), 3)

    g0r = effect({"G0": 1, fw.RANDOM: -1}, 5, "DP")
    checks = [
        ("`strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump", f"0 of {len(strata_file)}", f"{strata_diff} of {len(strata_file)}"),
        ("Sampled frames whose recomputed `G0` band / `π` differ from `labels.tsv`", "0 / 0", f"{band_diff} / {pi_diff}"),
        ("Sampled frames in the strict sample", "0", str(in_strict)),
        ("Frames left out of a cut (a NaN in a folder), over every score and `q`", "0", str(frames_out)),
        ("`labels.tsv` labels within {keep, discard}", "yes", "yes" if label_set <= {"keep", "discard"} else "no"),
        ("Sum of `1/π` over the sampled frames", f"{int(pop.sum())}.0", f"{total_w:.1f}"),
        ("Discardable base rate % [stratified 95%]", "48.45 [42.03, 55.11]", bf.iv(base, *base_ci, 2)),
        ("`G0` `DP` % q5 / q10 / q20", "82.7 [76.0, 88.7] / 76.3 [71.3, 81.3] / 72.1 [67.5, 76.7]", " / ".join(ivs("G0", q, "DP") for q in QS)),
        ("`G0` `RR` % q5 / q10 / q20", "17.3 [11.3, 24.0] / 23.7 [18.7, 28.7] / 27.9 [23.3, 32.5]", " / ".join(ivs("G0", q, "RR") for q in QS)),
        ("`G0` `PL` % (full dump) q5 / q10 / q20", "0.68 / 2.10 / 5.18", " / ".join(f"{res[('G0', q)]['pl']:.2f}" for q in QS)),
        ("`G0` folder-cluster `RR` 97.5% quantile q5 / q10 / q20", "25.7 / 32.8 / 36.3", " / ".join(f"{res[('G0', q)]['cci']['RR'][1]:.1f}" for q in QS)),
        ("`G0` AUC", "0.710 [0.646, 0.771]", aucs_iv("G0")),
        ("`eye_focus` (missing last) `RR` % q5 / q10 / q20", "16.8 [1.2, 43.3] / 14.3 [2.9, 29.9] / 28.3 [15.7, 40.2]", " / ".join(ivs(fw.EYE, q, "RR") for q in QS)),
        ("`eye_focus` (missing last) `PL` % q5 / q10 / q20", "4.44 / 9.14 / 15.59", " / ".join(f"{res[(fw.EYE, q)]['pl']:.2f}" for q in QS)),
        ("`eye_focus` (missing last) AUC", "0.777 [0.716, 0.841]", aucs_iv(fw.EYE)),
        ("Random (seed 0) `DP` % q5 / q10 / q20", "52.0 [19.8, 85.5] / 61.5 [37.9, 84.8] / 56.0 [40.5, 71.5]", " / ".join(ivs(fw.RANDOM, q, "DP") for q in QS)),
        ("Random (seed 0) AUC", "0.528 [0.455, 0.604]", aucs_iv(fw.RANDOM)),
        ("`G0` q5 - random q5, `DP` points", "+30.7 [-3.8, +63.9]", bf.civ(g0r[0], *bf.quant(g0r[1]))),
    ]
    gate.table("| Check | Expected | This run | Result |", [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in checks])

    print("## Label-free cuts\n")
    print(f"`PL`: the share of the dump's picks inside the cut (full dump; population after the slash). Removed: the population's share (full dump's after the slash). Bars `PL` <= {df.PL_BAR:.0f}%, removed >= {df.REMOVED_BAR}%.\n")
    gate.table(
        "| Score | Cut | Removed % | `PL` % | `PL` <= 5% | Removed >= 4.5% |",
        [[name(k), f"q{q}", f"{res[(k, q)]['removed_pop']:.2f} / {res[(k, q)]['removed_dump']:.2f}", f"{res[(k, q)]['pl']:.2f} / {res[(k, q)]['pl_pop']:.2f}",
          "yes" if res[(k, q)]["pl"] <= df.PL_BAR else "no", "yes" if res[(k, q)]["removed_pop"] >= df.REMOVED_BAR else "no"] for k in SCORES for q in QS],
    )

    print("## Main table\n")
    print(
        "Weighted (`1/π`) with the stratified bootstrap 95% interval (1,000 draws, seed 0, within each band and each `control` folder cell) and the "
        "unweighted value in brackets. Sampled: the sampled frames the cut removes / of them `discard`; picks: sampled dump picks removed / of them "
        "`discard`. `PL` full dump / population; removed population / full dump. No cut leaves a sampled frame out.\n"
    )
    mrows = []
    for k in SCORES:
        for q in QS:
            r = res[(k, q)]
            cells = [f"{ivs(k, q, m)} ({bf.f1(r['unw'][m])})" for m in ("DP", "RR", "DR", "MR", "DO")]
            mrows.append([name(k), f"q{q}", f"{r['n_rem']} / {r['n_dis']}", f"{r['picks_rem']} / {r['picks_rem_dis']}"] + cells[:2]
                         + [f"{r['pl']:.2f} / {r['pl_pop']:.2f}"] + cells[2:] + [f"{r['removed_pop']:.2f} / {r['removed_dump']:.2f}"])
    gate.table("| Score | Cut | Sampled removed / discard | Picks removed / discard | `DP` % | `RR` % | `PL` % | `DR` % | `MR` % | `DO` % | Removed % |", mrows)

    print("## Folder-cluster intervals of the main quantities\n")
    gate.table(
        "| Score | Cut | `DP` % cluster | `RR` % cluster | `DR` % cluster | `DO` % cluster |",
        [[name(k), f"q{q}"] + [f"[{res[(k, q)]['cci'][m][0]:.1f}, {res[(k, q)]['cci'][m][1]:.1f}]" for m in ("DP", "RR", "DR", "DO")] for k in SCORES for q in QS],
    )

    print("## Threshold-free: weighted AUC of `discard` vs `keep`\n")
    print("`roc_auc_score(discard, -score, sample_weight=1/π)` on the sampled frames, a missing value at the folder's lowest finite score minus 1; stratified 95% interval, folder-cluster interval, unweighted.\n")
    gate.table(
        "| Score | AUC [stratified 95%] | Folder-cluster 95% | Unweighted |",
        [[name(k), aucs_iv(k), "[{:.3f}, {:.3f}]".format(*bf.quant(auc[k]["cboot"])), f"{auc[k]['unw']:.3f}"] for k in SCORES],
    )

    print("## Effects\n")
    print(
        "Paired on the same bootstrap draws (the labeled frames resampled with the models and the cuts fixed), in points of weighted `DP` and `DR` at "
        "each `q` and in AUC: the point, the stratified 95% interval and the folder-cluster 95% interval.\n"
    )
    erows = []
    for lab, coefs in EFFECTS:
        row = [lab]
        for q in QS:
            for m in ("DP", "DR"):
                p, b, c = effect(coefs, q, m)
                row += [bf.civ(p, *bf.quant(b)), cl(bf.quant(c))]
        p, b, c = effect(coefs, None, None)
        row += ["{:+.3f} [{:+.3f}, {:+.3f}]".format(p, *bf.quant(b)), "[{:+.3f}, {:+.3f}]".format(*bf.quant(c))]
        erows.append(row)
    gate.table(
        "| Effect | `DP` q5 | Cluster | `DR` q5 | Cluster | `DP` q10 | Cluster | `DR` q10 | Cluster | `DP` q20 | Cluster | `DR` q20 | Cluster | AUC | Cluster |",
        erows,
    )

    print("## Coherence check: each cell's own bands, `discard` rate beside the strict-bad rate\n")
    print("The cell's own per-folder bands on the full dump; the weighted `discard` rate on the 600 labels (stratified 95%) and the weighted strict-bad rate on the strict sample with its own `π` and strata. Expected: strict-bad below `discard` in each band (a check, not a bar).\n")
    hrows = []
    for k in ["G0"] + CELLS:
        for lab, lo, hi in BANDS:
            inband = np.ones(n, dtype=bool)
            if lo is not None:
                inband &= ~removed_of[(k, lo)]
            if hi is not None:
                inband &= removed_of[(k, hi)]
            m, tm = inband[idx], inband[t["idx"]]
            rate = 100 * (w * dis * m).sum() / (w * m).sum()
            srate = 100 * (t["w"] * t["bad"] * tm).sum() / (t["w"] * tm).sum()
            hrows.append([
                name(k), lab, f"{int(m.sum())} / {int((dis & m).sum())}", bf.iv(rate, *bf.quant(bf.ratio(cs, w * dis * m, w * m))),
                f"{int(tm.sum())} / {int((t['bad'] & tm).sum())}", bf.iv(srate, *bf.quant(bf.ratio(ct, t["w"] * t["bad"] * tm, t["w"] * tm))),
                "yes" if srate < rate else "no",
            ])
    gate.table("| Score | Band | Sampled / discard | Weighted `discard` rate % | Strict sample frames / strict bad | Weighted strict-bad rate % | Strict below discard |", hrows)

    for bar, rr_bar, rr_hi in BARS:
        print(f"## Bars per new cell and cut: the {bar} bar\n")
        print(
            f"Usable: weighted `RR` <= {rr_bar:.0f}% with its stratified 97.5% quantile <= {rr_hi:.0f}%, `PL` (full dump) <= {df.PL_BAR:.0f}%, removed "
            f">= {df.REMOVED_BAR}% of the population, `DP`'s stratified 2.5% quantile above the base rate's point estimate ({base:.2f}%), and `DP` - "
            "`G0`'s at the same `q` above zero with the paired stratified 2.5% quantile above zero. Cluster values in parentheses; fragile: a pass whose "
            "cluster `RR` 97.5% quantile, `DP` 2.5% quantile or paired `DP` - `G0` 2.5% quantile crosses its bar.\n"
        )
        vrows = []
        for k in CELLS:
            for q in QS:
                r, v = res[(k, q)], verdicts[(bar, k, q)]
                p, lohi, clohi = v["g0"]
                vrows.append([
                    name(k), f"q{q}", f"{r['point']['RR']:.1f} / {r['ci']['RR'][1]:.1f} ({r['cci']['RR'][1]:.1f})", f"{r['pl']:.2f}", f"{r['removed_pop']:.2f}",
                    f"{r['ci']['DP'][0]:.1f} ({r['cci']['DP'][0]:.1f})", f"{p:+.1f} / {lohi[0]:+.1f} ({clohi[0]:+.1f})",
                    " / ".join("yes" if v["conds"][x] else "no" for x in CONDS),
                    ("usable" + (", fragile" if v["fragile"] else "")) if v["usable"] else "not usable",
                ])
        gate.table(
            "| Cell | Cut | `RR` % / 97.5% quantile (cluster) | `PL` % | Removed % | `DP` 2.5% quantile (cluster) | `DP` - `G0`, points / 2.5% quantile (cluster) | `RR` / quantile / `PL` / removed / random / `G0` | Verdict |",
            vrows,
        )
        b = best_of(bar)
        print(f"Best cut under the {bar} bar: {'none' if b is None else f'{name(b[0])} at `q{b[1]}`'}.\n")

    print("## Follow-up rule inputs\n")
    gate.table(
        "| Input | Value |",
        [("A usable cut under the chosen bar", "yes" if best is not None else "no")]
        + [(f"{lab} at q5, `DP` points [stratified 95%] (cluster)", f"{bf.civ(p, *bf.quant(b))} {cl(bf.quant(c))}") for lab, (p, b, c) in follow.items()]
        + [("The rule's branch these inputs select", branch)],
    )

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored and Step 3 scores, labels, strata, strict labels)", f"{load_time:.0f}"), ("Cuts, metrics, AUCs and bootstraps", f"{metric_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "retargetread.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    if not all(e == v for _, e, v in checks):
        raise SystemExit("a reference check failed")


if __name__ == "__main__":
    if len(sys.argv) != 7:
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:7])
