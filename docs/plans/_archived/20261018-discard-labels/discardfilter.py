"""Step 4: `G0`'s per-folder discard cuts and the comparison scores against the
user's blind keep / discard labels, weighted back to the population by the
inverse inclusion probability, with stratified and folder-cluster bootstrap
intervals. The labels, cuts, metrics, comparisons and bars are pre-registered
in discard.md.

Usage: python -I -B discardfilter.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <strict-labels-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv`, <reject-filter-eval-dir> `scores_reject.tsv`,
<strict-labels-dir> the bad-photo-labels sample's `labels.tsv` and
`manifest.tsv`; <out-dir> is Step 1's data folder with `strata.tsv` and Step
3's `labels.tsv`. Prints the result tables (Markdown) and saves them as
`discardfilter.out.md` in <out-dir>.
"""

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

rf, gate, folder_gate, fw = bf.rf, bf.gate, bf.folder_gate, bf.fw

QS = (5, 10, 20)
BANDS = ["b05", "b10", "b20", "control"]
RR_BAR = 15.0
RR_HI_BAR = 25.0
PL_BAR = 5.0
REMOVED_BAR = 4.5
RR_FOLLOW = 2 * RR_BAR
DO_FOLLOW = 20.0
HYPOTHESIS = 5.0
G0Q95 = "G0 `quantile@95`"
SCORES = ["G0", "W0", "R0", fw.SHARP, fw.EYE, fw.RANDOM]


def counts(s, folders, strata):
    """`badfilter.strat_counts` over this design's cells (the bands, `control` per folder as its `rest`)."""
    saved = bf.STRATA
    bf.STRATA = strata
    try:
        return bf.strat_counts(s, folders)
    finally:
        bf.STRATA = saved


def name(k):
    return k if "`" in k else f"`{k}`"


def share(c, num, den):
    """A weighted share in percent, or per bootstrap draw with the count matrix `c`."""
    if c is None:
        return 100 * num.sum() / den.sum() if den.sum() else np.nan
    return bf.ratio(c, num, den)


def metrics(s, rem, ok, c=None):
    """`DP`, `RR`, `DR`, `MR`, `DO` in percent on the sampled frames `ok`, weighted (or per bootstrap draw)."""
    w, dis = s["w"] * ok, s["bad"]
    dp = share(c, w * (dis & rem), w * rem)
    dr = share(c, w * (dis & rem), w * dis)
    do = share(c, w * (dis & ~rem), w * ~rem)
    return {"DP": dp, "RR": 100 - dp, "DR": dr, "MR": 100 - dr, "DO": do}


def unweighted(s, rem, ok):
    dis = s["bad"] & ok
    r, k = rem & ok, ~rem & ok
    dp = 100 * (dis & r).sum() / r.sum() if r.sum() else np.nan
    dr = 100 * (dis & r).sum() / dis.sum()
    return {"DP": dp, "RR": 100 - dp, "DR": dr, "MR": 100 - dr, "DO": 100 * (dis & k).sum() / k.sum()}


def main(dump, eval_dir, reject_dir, strict_dir, out_dir):
    start = time.time()
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n = len(rows)
    tb = np.random.default_rng(0).random(n)
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    pick_d, reject_d = d["pick"], d["reject"]

    labels = bf.read_tsv(os.path.join(out_dir, "labels.tsv"))
    s = {
        "idx": np.array([index[(r["folder"], r["file"])] for r in labels]),
        "bad": np.array([r["label"] == "discard" for r in labels]),
        "pi": np.array([float(r["inclusion_prob"]) for r in labels]),
        "band": np.array([r["band"] for r in labels], dtype=object),
    }
    assert set(r["label"] for r in labels) <= {"keep", "discard"}
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
    strict_manifest = bf.read_tsv(os.path.join(strict_dir, "manifest.tsv"))
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    t0 = time.time()
    idx, dis, w = s["idx"], s["bad"], s["w"]
    ns = len(idx)
    seen = np.zeros(n, dtype=bool)
    for r in strict_manifest:
        seen[index[(r["folder"], r["file"])]] = True
    pop = ~seen

    g0 = stored_rej["G0"]
    cut = {q: bf.graded_cut(g0, d, tb, q)[0] for q in QS}
    band = np.full(n, "control", dtype=object)
    band[cut[20]] = "b20"
    band[cut[10]] = "b10"
    band[cut[5]] = "b05"

    strata_file = bf.read_tsv(os.path.join(out_dir, "strata.tsv"))
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

    cs = counts(s, folders, ["b05", "b10", "b20", "rest"])
    cc = bf.cluster_counts(s, folders)
    ct = bf.strat_counts(t, folders)

    sharp = np.array([np.nan if f["sharp"] is None else f["sharp"] for f in rows])
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    graded = {
        "G0": g0,
        "W0": stored["W0"],
        "R0": stored_rej["R0"],
        fw.SHARP: np.where(np.isnan(sharp), -np.inf, sharp),
        fw.EYE: np.where(np.isnan(ef), -np.inf, ef),
        fw.RANDOM: tb,
    }

    cuts = {}
    for k, sc in graded.items():
        for q in QS:
            cuts[(k, f"q{q}")] = bf.graded_cut(sc, d, tb, q)
    q95, qual, _ = bf.g0_q95_cut(g0, d, tb)
    cuts[(G0Q95, "q@95")] = (q95, np.zeros(n, dtype=bool))
    names = SCORES[:1] + [G0Q95] + SCORES[1:]

    def cuts_of(k):
        return [c for (kk, c) in cuts if kk == k]

    res = {}
    for (k, c), (removed, out) in cuts.items():
        rem, ok = removed[idx], ~out[idx]
        cover = ~out
        point = metrics(s, rem, ok)
        boot = metrics(s, rem, ok, cs)
        clus = metrics(s, rem, ok, cc)
        r = {
            "rem": rem, "ok": ok, "point": point, "unw": unweighted(s, rem, ok), "boot": boot, "cboot": clus,
            "ci": {m: bf.quant(v) for m, v in boot.items()}, "cci": {m: bf.quant(v) for m, v in clus.items()},
            "pl": 100 * (removed & pick_d & cover).sum() / (pick_d & cover).sum(),
            "pl_pop": 100 * (removed & pick_d & cover & pop).sum() / (pick_d & cover & pop).sum(),
            "removed_pop": 100 * (removed & cover & pop).sum() / (cover & pop).sum(),
            "removed_dump": 100 * (removed & cover).sum() / cover.sum(),
            "n_rem": int((rem & ok).sum()), "n_dis": int((rem & ok & dis).sum()), "out": int((~ok).sum()),
            "picks_rem": int((rem & ok & pick_d[idx]).sum()), "picks_rem_dis": int((rem & ok & pick_d[idx] & dis).sum()),
        }
        res[(k, c)] = r

    auc = {}
    for k in SCORES:
        sc = bf.finite_by_folder(graded[k], d)[idx]
        ok = res[(k, "q5")]["ok"]
        auc[k] = bf.aucs(s, sc, ok, cs)

    def vs(k, ck, base_k, cb, matrix):
        """`k` at `ck` minus `base_k` at `cb` on the frames both cover, point and per draw."""
        ok = res[(k, ck)]["ok"] & res[(base_k, cb)]["ok"]
        a, b = metrics(s, res[(k, ck)]["rem"], ok), metrics(s, res[(base_k, cb)]["rem"], ok)
        ab, bb = metrics(s, res[(k, ck)]["rem"], ok, matrix), metrics(s, res[(base_k, cb)]["rem"], ok, matrix)
        return {m: (a[m] - b[m], ab[m] - bb[m]) for m in ("DP", "DR")}

    g0_cuts = [("G0", f"q{q}") for q in QS] + [(G0Q95, "q@95")]

    def random_of(c):
        return "q20" if c == "q@95" else c

    verdict = {}
    for k, c in g0_cuts:
        r = res[(k, c)]
        diff = vs(k, c, fw.RANDOM, random_of(c), cs)
        cdiff = vs(k, c, fw.RANDOM, random_of(c), cc)
        conds = {
            "RR": r["point"]["RR"] <= RR_BAR,
            "RR97.5": r["ci"]["RR"][1] <= RR_HI_BAR,
            "PL": r["pl"] <= PL_BAR,
            "removed": r["removed_pop"] >= REMOVED_BAR,
            "random": diff["DP"][0] > 0 and bf.quant(diff["DP"][1])[0] > 0,
        }
        usable = all(conds.values())
        fragile = usable and (r["cci"]["RR"][1] > RR_HI_BAR or bf.quant(cdiff["DP"][1])[0] <= 0)
        verdict[(k, c)] = {"conds": conds, "usable": usable, "fragile": fragile, "diff": diff, "cdiff": cdiff}
    metric_time = time.time() - t0

    total_w = w.sum()
    base = 100 * (w * dis).sum() / total_w
    base_ci = bf.quant(bf.ratio(cs, w * dis, w))
    base_cci = bf.quant(bf.ratio(cc, w * dis, w))
    strict_base = 100 * (t["w"] * t["bad"]).sum() / t["w"].sum()
    strict_base_ci = bf.quant(bf.ratio(ct, t["w"] * t["bad"], t["w"]))

    print("## Data\n")
    pick, reject = pick_d[idx], reject_d[idx]
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{len(folders)} / {n} / {missing}"),
            ("Strict-sample frames excluded / population", f"{int(seen.sum())} / {int(pop.sum())}"),
            ("Sampled frames (`labels.tsv`) / keep / discard", f"{ns} / {int((~dis).sum())} / {int(dis.sum())}"),
            ("Sampled picks / explicit rejects / singles", f"{int(pick.sum())} / {int(reject.sum())} / {int(d['single'][idx].sum())}"),
            ("Sampled frames without `W0` (folder `2026-07-23`)", int(np.isnan(stored["W0"][idx]).sum())),
            ("Strict sample (`labels.tsv` there) / strict bad", f"{len(t['idx'])} / {int(t['bad'].sum())}"),
        ],
    )

    print("## Discardable base rate\n")
    print("The weighted (Horvitz-Thompson, `1/π`) share of the population's frames labeled `discard`, the stratified bootstrap 95% interval, the folder-cluster interval, and the unweighted sample share.\n")
    gate.table(
        "| Reading | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |",
        [("Discardable base rate", f"{base:.2f}", f"[{base_ci[0]:.2f}, {base_ci[1]:.2f}]", f"[{base_cci[0]:.2f}, {base_cci[1]:.2f}]", f"{100 * dis.mean():.2f}")],
    )

    print("## Reference checks\n")
    rnd = [res[(fw.RANDOM, f"q{q}")]["point"]["DR"] for q in QS]
    lf = rf.label_free(g0, d, tb, qual, reject_d & ~pick_d)
    lf_rec = float(np.mean([lf[fi]["q95"]["recall"] for fi in qual]))
    lf_rn = float(np.mean([lf[fi]["q95"]["rn"] for fi in qual]))
    my = [rf.point(q95[folder_of == fi], pick_d[folder_of == fi], (reject_d & ~pick_d)[folder_of == fi]) for fi in qual]
    my_rec, my_rn = float(np.mean([p["recall"] for p in my])), float(np.mean([p["rn"] for p in my]))
    gate.table(
        "| Check | Expected | This run |",
        [
            ("`strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump", "0", f"{strata_diff} of {len(strata_file)}"),
            ("Sampled frames whose recomputed `G0` band / `π` differ from `labels.tsv`", "0 / 0", f"{band_diff} / {pi_diff}"),
            ("Sampled frames that are in the strict sample", "0", in_strict),
            ("Sum of `1/π` over the sampled frames (the population)", f"{int(pop.sum())}", f"{total_w:.1f}"),
            ("Random (seed 0) weighted `DR` % at q = " + " / ".join(str(q) for q in QS), "near q", " / ".join(bf.f1(v) for v in rnd)),
            ("Random (seed 0) weighted AUC", "near 0.5", f"{auc[fw.RANDOM][0][0]:.3f}"),
            ("`G0` `quantile@95` over the qualifying folders: mean pick recall / non-pick removal %", "94.7 / 16.8 (`rf.label_free` " + f"{lf_rec:.1f} / {lf_rn:.1f})", f"{my_rec:.1f} / {my_rn:.1f}"),
            ("Strict base rate from the strict sample, weighted % [stratified 95%]", "5.52 [4.15, 7.04]", bf.iv(strict_base, *strict_base_ci, 2)),
        ],
    )

    print("## Per band\n")
    print("The weighted `discard` rate of each `G0` band (stratified 95% interval), the band's pick share in the population (label-free), and the strict-bad rate of the same band read on the strict sample with its own `π` and strata.\n")
    brows = []
    tband = band[t["idx"]]
    for b in BANDS:
        m = s["band"] == b
        pm = pop & (band == b)
        tm = tband == b
        rate = 100 * (w * dis * m).sum() / (w * m).sum()
        srate = 100 * (t["w"] * t["bad"] * tm).sum() / (t["w"] * tm).sum()
        brows.append([
            f"`{b}`", int(pm.sum()), int(m.sum()), int((dis & m).sum()), bf.iv(rate, *bf.quant(bf.ratio(cs, w * dis * m, w * m))),
            bf.f1(100 * (dis & m).sum() / m.sum()), f"{100 * (pm & pick_d).sum() / pm.sum():.2f}",
            f"{int(tm.sum())} / {int((t['bad'] & tm).sum())}", bf.iv(srate, *bf.quant(bf.ratio(ct, t["w"] * t["bad"] * tm, t["w"] * tm))),
            "yes" if srate < rate else "no",
        ])
    gate.table(
        "| Band | Population | Sampled | Discard (sampled) | Weighted `discard` rate % | Unweighted % | Pick share % | Strict sample frames / strict bad | Weighted strict-bad rate % | Strict below discard |",
        brows,
    )

    print("## Dump-label consistency\n")
    nonpick = ~pick
    gate.table(
        "| Reading | Value |",
        [
            ("Sampled dump picks labeled `discard`: weighted share % [stratified 95%] / sampled count", f"{bf.iv(100 * (w * dis * pick).sum() / (w * pick).sum(), *bf.quant(bf.ratio(cs, w * dis * pick, w * pick)))} / {int((dis & pick).sum())} of {int(pick.sum())}"),
            ("Sampled explicit dump rejects labeled `keep`", f"{int((~dis & reject).sum())} of {int(reject.sum())}"),
        ],
    )

    print("## Main table: discard precision, rescue rate, pick loss, recall and the discardable share outside the cut\n")
    print(
        "Weighted (`1/π`), with the stratified bootstrap 95% interval (1,000 draws, seed 0, within each band and each `control` folder cell) and the "
        "unweighted sample value in brackets. `qX`: the folder's bottom `floor(X * n / 100)` frames by the score, cut on the full dump; `q@95`: `G0`'s "
        "label-free `quantile@95` share. `PL`: the share of the dump's picks inside the cut (full dump, label-free; population value after the slash). "
        "Removed: the share of the population's frames inside the cut (full dump's after the slash), on the folders the score covers. Sampled: the "
        "sampled frames the cut removes / of them `discard`; picks: sampled dump picks removed / of them `discard`. Out: sampled frames in folders the "
        "score does not cover.\n"
    )
    mrows = []
    for k in names:
        for c in cuts_of(k):
            r = res[(k, c)]
            cells = [f"{bf.iv(r['point'][m], *r['ci'][m])} ({bf.f1(r['unw'][m])})" for m in ("DP", "RR", "DR", "MR", "DO")]
            mrows.append([name(k), c, r["out"], f"{r['n_rem']} / {r['n_dis']}", f"{r['picks_rem']} / {r['picks_rem_dis']}"] + cells[:2]
                         + [f"{r['pl']:.2f} / {r['pl_pop']:.2f}"] + cells[2:] + [f"{r['removed_pop']:.2f} / {r['removed_dump']:.2f}"])
    gate.table("| Score | Cut | Out | Sampled removed / discard | Picks removed / discard | `DP` % | `RR` % | `PL` % | `DR` % | `MR` % | `DO` % | Removed % |", mrows)

    print("## Threshold-free: weighted AUC of `discard` vs `keep`\n")
    print("`roc_auc_score(discard, -score, sample_weight=1/π)` on the sampled frames, pooled over folders, missing values at the folder's lowest finite score minus 1; stratified 95% interval; unweighted in brackets.\n")
    gate.table("| Score | AUC |", [[name(k), f"{bf.iv(auc[k][0][0], *auc[k][2], 3)} ({auc[k][1][0]:.3f})"] for k in SCORES])

    print("## Paired differences\n")
    print(
        "`G0` minus each comparison score at the same `q` (`G0` `quantile@95` against random at q20), in points, on the sampled frames both cover, "
        "with the stratified bootstrap 95% interval (the same draws for both) and the folder-cluster interval.\n"
    )
    prows = []
    for q in QS:
        for k in SCORES[1:]:
            a, b = vs("G0", f"q{q}", k, f"q{q}", cs), vs("G0", f"q{q}", k, f"q{q}", cc)
            prows.append([f"`G0` (q{q}) - {name(k)} (q{q})"] + sum([[bf.civ(a[m][0], *bf.quant(a[m][1])), f"[{bf.quant(b[m][1])[0]:+.1f}, {bf.quant(b[m][1])[1]:+.1f}]"] for m in ("DP", "DR")], []))
    a, b = vs(G0Q95, "q@95", fw.RANDOM, "q20", cs), vs(G0Q95, "q@95", fw.RANDOM, "q20", cc)
    prows.append([f"{G0Q95} (q@95) - {name(fw.RANDOM)} (q20)"] + sum([[bf.civ(a[m][0], *bf.quant(a[m][1])), f"[{bf.quant(b[m][1])[0]:+.1f}, {bf.quant(b[m][1])[1]:+.1f}]"] for m in ("DP", "DR")], []))
    gate.table("| Comparison | `DP`, points | Cluster | `DR`, points | Cluster |", prows)

    print("## The non-pick framing\n")
    rq = res[(G0Q95, "q@95")]["rem"]
    framing = [
        ("`discard` rate among the dump's non-picks", w * dis * nonpick, w * nonpick, dis & nonpick, nonpick),
        ("`discard` rate among the dump's picks", w * dis * pick, w * pick, dis & pick, pick),
        ("`discard` share of the non-picks `G0` `quantile@95` removes (Decision 2's `RN`)", w * dis * nonpick * rq, w * nonpick * rq, dis & nonpick & rq, nonpick & rq),
    ]
    frows = []
    for label, a, b, ua, ub in framing:
        frows.append([label, f"{int(ub.sum())} / {int(ua.sum())}", bf.iv(100 * a.sum() / b.sum(), *bf.quant(bf.ratio(cs, a, b))),
                      f"[{bf.quant(bf.ratio(cc, a, b))[0]:.1f}, {bf.quant(bf.ratio(cc, a, b))[1]:.1f}]", bf.f1(100 * ua.sum() / ub.sum())])
    gate.table("| Reading | Sampled / discard | Weighted % [stratified 95%] | Folder-cluster 95% | Unweighted % |", frows)

    print("## Bars per `G0` cut\n")
    print(
        f"Usable discard cut: weighted `RR` <= {RR_BAR:.0f}% with its stratified 97.5% quantile <= {RR_HI_BAR:.0f}%, `PL` (full dump) <= {PL_BAR:.0f}%, "
        f"removed >= {REMOVED_BAR}% of the population's frames, and weighted `DP` above random's at the same `q` (`quantile@95`: random at q20) with the "
        "paired stratified 2.5% quantile above zero. Fragile: a pass whose folder-cluster `RR` 97.5% quantile or paired `DP` 2.5% quantile crosses its bar.\n"
    )
    vrows = []
    for k, c in g0_cuts:
        r, v = res[(k, c)], verdict[(k, c)]
        dp_diff = v["diff"]["DP"]
        vrows.append([
            name(k), c, f"{r['point']['RR']:.1f} / {r['ci']['RR'][1]:.1f} ({r['cci']['RR'][1]:.1f})", f"{r['pl']:.2f}", f"{r['removed_pop']:.2f}",
            f"{dp_diff[0]:+.1f} / {bf.quant(dp_diff[1])[0]:+.1f} ({bf.quant(v['cdiff']['DP'][1])[0]:+.1f})",
            " / ".join("yes" if v["conds"][x] else "no" for x in ("RR", "RR97.5", "PL", "removed", "random")),
            ("usable" + (", fragile" if v["fragile"] else "")) if v["usable"] else "not usable",
        ])
    gate.table(
        "| Score | Cut | `RR` % / 97.5% quantile (cluster) | `PL` % | Removed % | `DP` - random, points / 2.5% quantile (cluster) | `RR` / quantile / `PL` / removed / random | Verdict |",
        vrows,
    )
    usable = [(k, c) for k, c in g0_cuts if verdict[(k, c)]["usable"]]
    best = max(usable, key=lambda kc: res[kc]["removed_pop"]) if usable else None
    r5 = res[("G0", "q5")]
    print(f"Best cut (the deepest usable one, by removed share): {'none' if best is None else f'{name(best[0])} at `{best[1]}`'}.\n")
    print(
        f"Follow-up rule inputs: a usable cut {'exists' if usable else 'does not exist'}; weighted `RR@5` {r5['point']['RR']:.1f}% "
        f"(<= {RR_FOLLOW:.0f}%: {'yes' if r5['point']['RR'] <= RR_FOLLOW else 'no'}); `DO@5` {r5['point']['DO']:.1f}% "
        f"(>= {DO_FOLLOW:.0f}%: {'yes' if r5['point']['DO'] >= DO_FOLLOW else 'no'}).\n"
    )
    print(
        f"Hypothesis `RR@5` <= {HYPOTHESIS:.0f}% (the unblinded 70/71 reading): weighted `RR@5` {bf.iv(r5['point']['RR'], *r5['ci']['RR'])}% "
        f"(unweighted {bf.f1(r5['unw']['RR'])}%, {r5['n_rem'] - r5['n_dis']} of {r5['n_rem']} sampled frames kept): "
        f"{'confirmed' if r5['point']['RR'] <= HYPOTHESIS else 'not confirmed'}.\n"
    )

    print("## Folder-cluster sensitivity\n")
    print("The 1,000-draw folder-cluster bootstrap (resample the 27 folders, seed 0) beside the stratified interval, for the base rate and each `G0` cut's labeled quantities.\n")
    krows = [["Discardable base rate", "-", f"{base:.1f} [{base_ci[0]:.1f}, {base_ci[1]:.1f}]", f"[{base_cci[0]:.1f}, {base_cci[1]:.1f}]", "", "", "", "", ""]]
    for k, c in g0_cuts:
        r = res[(k, c)]
        krows.append([name(k), c] + sum([[bf.iv(r["point"][m], *r["ci"][m]), f"[{r['cci'][m][0]:.1f}, {r['cci'][m][1]:.1f}]"] for m in ("RR", "DR", "DO")], [])
                     + [("usable" + (", fragile" if verdict[(k, c)]["fragile"] else "")) if verdict[(k, c)]["usable"] else "not usable"])
    gate.table("| Reading | Cut | `RR` % (stratified) | Cluster | `DR` % (stratified) | Cluster | `DO` % (stratified) | Cluster | Bar |", krows)

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored scores, labels)", f"{load_time:.0f}"), ("Cuts, metrics and bootstraps", f"{metric_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "discardfilter.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))


if __name__ == "__main__":
    if len(sys.argv) != 6:
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:6])
