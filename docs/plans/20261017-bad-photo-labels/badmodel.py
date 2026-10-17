"""Step 5: the dedicated bad-photo models `B0` (gradient boosting) and `L0`
(logistic regression) on `G0`'s 46 columns, trained on the user's bad /
not-bad labels with `1/π` weights, held out by folder, read on the
within-sample cuts beside the best existing score. The design and the bars
are pre-registered in badfilter.md.

Usage: python -I -B badmodel.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <out-dir> [jobs]

Arguments as for badfilter.py. Writes `scores_bad.tsv` (`name`, `B0`, `L0`;
held out, higher keeps) into <out-dir>, prints the result tables (Markdown)
and saves them as `badmodel.out.md` in <out-dir>.
"""

import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import badfilter as bf  # noqa: E402
import ablate  # noqa: E402

rf, gate, folder_gate, fw = bf.rf, bf.gate, bf.folder_gate, bf.fw

MODELS = ("B0", "L0")
GROUPS = {g: cols[ablate.F0_CTX] for g, cols in ablate.GROUPS.items()}
REPEATS = 5


def fold(d, held, s):
    """One held-out folder: `B0` and `L0` fit on the other folders' sampled frames, scored (higher keeps) on its own, and `B0`'s permuted scores per group."""
    train_mask = d["folder_of"] != held
    x = folder_gate.design(d, train_mask, "full")
    xh = x.copy()
    xh[:, : d["tech"].shape[1]][np.isnan(d["tech"])] = np.nan
    tr, te = s["folder"] != held, s["folder"] == held
    itr, ite = s["idx"][tr], s["idx"][te]
    y = s["bad"][tr].astype(int)
    w = s["w"][tr] * tr.sum() / s["w"][tr].sum()
    b0 = HistGradientBoostingClassifier(**rf.HGB).fit(xh[itr], y, sample_weight=w)
    mu, sd = x[itr].mean(axis=0), x[itr].std(axis=0)
    sd[sd == 0] = 1
    z = (x - mu) / sd
    wl = s["w"][tr].copy()
    for c in (0, 1):
        wl[y == c] *= (len(y) / 2) / wl[y == c].sum()
    l0 = LogisticRegression(C=1.0, max_iter=2000).fit(z[itr], y, sample_weight=wl)
    xt = xh[ite]
    rng = np.random.default_rng(0)
    perm = {}
    for g, cols in GROUPS.items():
        out = []
        for _ in range(REPEATS):
            xp = xt.copy()
            xp[:, cols] = xt[rng.permutation(len(xt))][:, cols]
            out.append(-b0.decision_function(xp))
        perm[g] = np.array(out)
    return held, -b0.decision_function(xt), -l0.decision_function(z[ite]), perm


def sample_cut(score, s, tbs, q):
    """Within each folder's sampled frames: remove the longest bottom prefix (ties by the seed-0 draw) whose `1/π` sum is <= q% of the folder's; a folder with a NaN is left out."""
    removed = np.zeros(len(score), dtype=bool)
    out = np.zeros(len(score), dtype=bool)
    for fi in np.unique(s["folder"]):
        m = np.flatnonzero(s["folder"] == fi)
        if np.isnan(score[m]).any():
            out[m] = True
            continue
        o = m[rf.bottom(score[m], tbs[m])]
        k = int((np.cumsum(s["w"][o]) <= q / 100 * s["w"][m].sum() * (1 + 1e-12)).sum())
        removed[o[:k]] = True
    return removed, out


def name_of(k):
    return k if "`" in k else f"`{k}`"


def main(dump, eval_dir, reject_dir, out_dir, jobs=4):
    start = time.time()
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee
    d, tb, stored, stored_rej, ear, masks, count, s, missing = bf.load(dump, eval_dir, reject_dir, out_dir)
    rows, folders = d["rows"], d["folders"]
    idx, ns = s["idx"], len(s["idx"])
    tbs = tb[idx]
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)
    assert d["tech"].shape[1] + d["context"].shape[1] + gate.PCA_DIMS == 46

    t = time.time()
    results = Parallel(n_jobs=int(jobs))(delayed(fold)(d, fi, s) for fi in range(len(folders)))
    fit_time = time.time() - t
    model = {k: np.full(ns, np.nan) for k in MODELS}
    perm = {g: np.full((REPEATS, ns), np.nan) for g in GROUPS}
    for held, b0, l0, pm in results:
        te = s["folder"] == held
        model["B0"][te], model["L0"][te] = b0, l0
        for g in GROUPS:
            perm[g][:, te] = pm[g]
    with open(os.path.join(out_dir, "scores_bad.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("name\tB0\tL0\n")
        for i, nm in enumerate(s["name"]):
            f.write(f"{nm}\t{model['B0'][i]:.6f}\t{model['L0'][i]:.6f}\n")

    t = time.time()
    cs = bf.strat_counts(s, folders)
    cc = bf.cluster_counts(s, folders)
    w0 = stored["W0"]
    centered = w0.copy()
    for _, a, e in d["bursts"]:
        centered[a:e] -= w0[a:e].mean()
    sharp = np.array([np.nan if f["sharp"] is None else f["sharp"] for f in rows])
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    graded = {k: stored[k] for k in bf.STORED}
    graded.update({k: stored_rej[k] for k in bf.STORED_REJ})
    graded[fw.CENTERED] = centered
    graded[fw.RANDOM] = tb
    graded[fw.SHARP] = np.where(np.isnan(sharp), -np.inf, sharp)
    graded[fw.EYE] = np.where(np.isnan(ef), -np.inf, ef)
    graded[fw.TIER] = d["tech"][:, 6]

    def row(rem, ok):
        point = bf.rates(s, rem, ok)
        r = {"rem": rem, "ok": ok, "point": point, "unw": bf.unweighted(s, rem, ok), "ci": [bf.quant(b) for b in bf.rates(s, rem, ok, cs)],
             "cci": [bf.quant(b) for b in bf.rates(s, rem, ok, cc)], "out": int((~ok).sum())}
        br, nl, p = point
        r["pass"] = nl <= bf.LOSS_CEIL and br >= bf.RECALL_BAR and r["ci"][0][0] > bf.RECALL_LO and p >= bf.PRECISION_BAR
        r["cpass"] = r["pass"] and r["cci"][0][0] > bf.RECALL_LO
        return r

    within, full = {}, {}
    for k, sc in list(model.items()) + [(k, v[idx]) for k, v in graded.items()]:
        for q in bf.QS:
            rem, out = sample_cut(sc, s, tbs, q)
            within[(k, f"q{q}")] = row(rem, ~out)
    for k, sc in graded.items():
        for q in bf.QS:
            rem, out = bf.graded_cut(sc, d, tb, q)
            full[(k, f"q{q}")] = row(rem[idx], ~out[idx])
    g0_removed, _, _ = bf.g0_q95_cut(stored_rej["G0"], d, tb)
    binary = {bf.G0Q95: ("q@95", g0_removed)}
    binary.update({label: ("rule", masks[key]) for label, key in bf.RULES})
    for k, (c, rem) in binary.items():
        within[(k, c)] = full[(k, c)] = row(rem[idx], np.ones(ns, dtype=bool))

    def best_of(table, keys):
        """Per score: its cut with the highest weighted `BR` among those with `NL` <= 5% (None when none)."""
        out = {}
        for k in keys:
            live = [c for (kk, c) in table if kk == k and table[(kk, c)]["point"][1] <= bf.LOSS_CEIL]
            out[k] = max(live, key=lambda c: table[(k, c)]["point"][0]) if live else None
        return out

    existing = list(graded) + list(binary)
    best_w = best_of(within, list(MODELS) + existing)
    best_f = best_of(full, existing)
    top_w = max((k for k in existing if best_w[k]), key=lambda k: within[(k, best_w[k])]["point"][0])
    top_f = max((k for k in existing if best_f[k]), key=lambda k: full[(k, best_f[k])]["point"][0])

    auc = {k: bf.aucs(s, model[k], np.ones(ns, dtype=bool), cs) for k in MODELS}
    for k in ("R0", "G0", top_w):
        if k in graded and k not in auc:
            auc[k] = bf.aucs(s, bf.finite_by_folder(graded[k], d)[idx], full[(k, "q5")]["ok"], cs)
    metric_time = time.time() - t

    print("## Dedicated model: data and fits\n")
    gate.table(
        "| Reading | Value |",
        [
            ("Sampled frames / bad / not bad / folds (leave one folder out)", f"{ns} / {int(s['bad'].sum())} / {int((~s['bad']).sum())} / {len(folders)}"),
            ("Design columns (`folder_gate.design(d, mask, \"full\")`)", d["tech"].shape[1] + d["context"].shape[1] + gate.PCA_DIMS),
            ("Training frames per fold: min / max; bad among them: min / max",
             f"{min(int((s['folder'] != fi).sum()) for fi in range(len(folders)))} / {max(int((s['folder'] != fi).sum()) for fi in range(len(folders)))}; "
             f"{min(int((s['bad'] & (s['folder'] != fi)).sum()) for fi in range(len(folders)))} / {max(int((s['bad'] & (s['folder'] != fi)).sum()) for fi in range(len(folders)))}"),
            ("Held-out folders with no bad sampled frame", sum(int(not (s["bad"] & (s["folder"] == fi)).any()) for fi in range(len(folders)))),
        ],
    )

    print("## Dedicated model: within-sample cuts\n")
    print(
        "Every score cut **within each folder's sampled frames**: remove the lowest-scored sampled frames (ties by the seed-0 draw) while their "
        "`1/π` sum stays <= q% of the folder's `1/π` sum (the Pre-registration's Dedicated model cut; coarse on about 45 frames per folder). "
        "Weighted `BR` / `NL` / `P` with the stratified 95% interval and the unweighted value in brackets; Bar: the worth-a-filter bar. "
        "The binary rules and `G0` `quantile@95` have one cut, the same as in Results. `B0` and `L0` first, then every existing score.\n"
    )
    wrows = []
    for k in list(MODELS) + existing:
        for (kk, c), r in within.items():
            if kk != k:
                continue
            cells = [f"{bf.iv(r['point'][j], *r['ci'][j])} ({bf.f1(r['unw'][j])})" for j in range(3)]
            wrows.append([name_of(k), c + (" (best)" if best_w[k] == c else ""), r["out"]] + cells + ["pass" + ("" if r["cpass"] else ", fragile") if r["pass"] else "-"])
    gate.table("| Score | Cut | Out | `BR` % | `NL` % | `P` % | Bar |", wrows)

    print("## Dedicated model: the best existing score, within-sample beside full-dump\n")
    print(
        f"The best existing score by the bar's rule (highest weighted `BR` among the cuts with `NL` <= 5%): on the full-dump cuts (Results) "
        f"{name_of(top_f)} at `{best_f[top_f]}`; on the within-sample cuts {name_of(top_w)} at `{best_w[top_w]}`. Each row: its best cut on "
        "each reading.\n"
    )
    crows = []
    for k in dict.fromkeys([top_f, top_w, "G0"]):
        for label, table, b in (("full-dump", full, best_f), ("within-sample", within, best_w)):
            c = b[k]
            if c is None:
                crows.append([name_of(k), label, "-", "-", "-", "-"])
                continue
            r = table[(k, c)]
            crows.append([name_of(k), label, c] + [bf.iv(r["point"][j], *r["ci"][j]) for j in range(3)])
    for k in MODELS:
        c = best_w[k]
        if c is None:
            crows.append([f"`{k}`", "within-sample", "-", "-", "-", "-"])
            continue
        r = within[(k, c)]
        crows.append([f"`{k}`", "within-sample", c] + [bf.iv(r["point"][j], *r["ci"][j]) for j in range(3)])
    gate.table("| Score | Cuts | Best cut (`NL` <= 5%) | `BR` % | `NL` % | `P` % |", crows)

    print("## Dedicated model: paired differences\n")
    print(
        "Model minus comparison, in points, each at its best within-sample cut with `NL` <= 5% (the dedicated-model bar's reading), and at the "
        "same `q` for q5 / q10 (omitted where it repeats the best-cut row); stratified bootstrap 95% interval (the same draws for both) and the folder-cluster interval. Frames outside "
        "either score's coverage are left out of both. Bar: `BR` difference >= +10 points with its stratified 2.5% quantile above zero.\n"
    )

    def diff(ka, ca, kb, cb):
        ra, rb = within[(ka, ca)], within[(kb, cb)]
        ok = ra["ok"] & rb["ok"]
        a, b = bf.rates(s, ra["rem"], ok), bf.rates(s, rb["rem"], ok)
        ab, bb = bf.rates(s, ra["rem"], ok, cs), bf.rates(s, rb["rem"], ok, cs)
        ac, bc = bf.rates(s, ra["rem"], ok, cc), bf.rates(s, rb["rem"], ok, cc)
        return [(a[j] - b[j], bf.quant(ab[j] - bb[j]), bf.quant(ac[j] - bc[j])) for j in range(3)]

    drows, bar = [], {}
    comps = list(dict.fromkeys([top_w, top_f, "G0"]))
    for m in MODELS:
        for kb in comps + ([] if m == "L0" else ["L0"]):
            pairs = [(best_w[m], best_w[kb], "best")] + [(f"q{q}", f"q{q}", "same q") for q in (5, 10) if (kb, f"q{q}") in within]
            pairs = [p for i, p in enumerate(pairs) if p[:2] not in [o[:2] for o in pairs[:i]]]
            for ca, cb, how in pairs:
                if ca is None or cb is None:
                    continue
                dv = diff(m, ca, kb, cb)
                if how == "best" and kb == top_w:
                    bar[m] = (dv[0][0] >= 10 and dv[0][1][0] > 0, dv[0])
                drows.append([f"`{m}` ({ca}) - {name_of(kb)} ({cb})", how] + [f"{bf.civ(v, *ci)} (cluster [{cl[0]:+.1f}, {cl[1]:+.1f}])" for v, ci, cl in dv])
    gate.table("| Comparison | Cuts | `BR`, points | `NL`, points | `P`, points |", drows)

    print("## Dedicated model: threshold-free\n")
    arows = []
    for k in auc:
        (pa, pp), (ua, up), lo_a, lo_p = auc[k]
        arows.append([name_of(k), f"{bf.iv(pa, *lo_a, 3)} ({ua:.3f})", f"{bf.iv(pp, *lo_p, 3)} ({up:.3f})"])
    gate.table("| Score | AUC | Partial AUC (`NL` <= 10%) |", arows)

    print("## `B0` feature importance (permutation, held out)\n")
    print(
        f"Each group's columns (`rank.rs`'s `REASONS` groups on `F0+ctx`'s layout, the missing-value indicators with their group, as the archived "
        f"ablations) permuted jointly among each held-out folder's sampled frames, {REPEATS} repeats (`default_rng(0)` per fold), the fold's fitted "
        "`B0` rescoring them; the drop of the pooled weighted AUC / partial AUC from the unpermuted held-out score, the mean over the repeats with "
        "the smallest and largest drop. Positive: the group carries `B0`'s held-out ranking.\n"
    )
    base_a, base_p = bf.auc_pair(s["bad"], model["B0"], s["w"])
    irows = []
    for g in GROUPS:
        vals = np.array([bf.auc_pair(s["bad"], perm[g][r], s["w"]) for r in range(REPEATS)])
        da, dp = base_a - vals[:, 0], base_p - vals[:, 1]
        irows.append([g, len(GROUPS[g]), f"{da.mean():+.3f} [{da.min():+.3f}, {da.max():+.3f}]", f"{dp.mean():+.3f} [{dp.min():+.3f}, {dp.max():+.3f}]"])
    irows.sort(key=lambda r: -float(r[2].split()[0]))
    gate.table("| Group | Columns | AUC drop | Partial AUC drop |", irows)

    print("## Dedicated-model bar\n")
    any_existing = any(r["pass"] for r in full.values()) or any(r["pass"] for (k, _), r in within.items() if k not in MODELS)
    brows = []
    for m in MODELS:
        passes = [c for (k, c), r in within.items() if k == m and r["pass"]]
        gain, (v, ci, cl) = bar.get(m, (False, (np.nan, (np.nan, np.nan), (np.nan, np.nan))))
        second = not any_existing and bool(passes)
        brows.append([f"`{m}`", best_w[m] or "-", bf.civ(v, *ci) if not np.isnan(v) else "-", "yes" if gain else "no", ", ".join(passes) if passes else "no",
                      "yes" if second else "no", "**met**" if gain or second else "not met"])
    gate.table(
        f"| Model | Best cut | `BR` gain over {name_of(top_w)} (points) | +10 with interval above zero | Passes the worth-a-filter bar at | "
        "No existing passes and the model does | Dedicated-model bar |",
        brows,
    )
    print(f"Existing scores or rules passing the worth-a-filter bar on any full-dump or within-sample cut: {'yes' if any_existing else 'none'}.\n")

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored scores, rules, labels)", f"{load_time:.0f}"), (f"27 folds, `B0` + `L0` + permutations ({jobs} jobs)", f"{fit_time:.0f}"),
                                      ("Cuts, metrics and bootstraps", f"{metric_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "badmodel.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))


if __name__ == "__main__":
    if len(sys.argv) not in (5, 6):
        raise SystemExit(__doc__)
    main(*sys.argv[1:6])
