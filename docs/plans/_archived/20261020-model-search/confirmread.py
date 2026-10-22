"""Step 5: the single confirmatory read of the two frozen winners of
modelsearch.md, with the bars and the follow-up rule of its "Pre-registration
of the confirmation".

Usage: python -I -B confirmread.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <discard-labels-dir> <strict-labels-dir> <out-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv` (the stored `W0`), <reject-filter-eval-dir> `scores_reject.tsv`
(the stored `G0`), <discard-labels-dir> and <strict-labels-dir> the two
earlier samples' `manifest.tsv`, whose frames are outside the fresh
population (no `labels.tsv` is opened there), <out-dir> the data folder with
Step 1's `lockbox.tsv`, Step 2's `scores_pick.tsv` and `scores_discard.tsv`,
Step 4's `strata.tsv` and `confirmlabels.py`'s `labels.tsv`. Prints the tables
(Markdown) and saves them as `confirmread.out.md` in <out-dir>; refuses to run
when that file exists, so the read happens once.

Discard: the discard winner, `G0`, `eye_focus` and random on the fresh labels.
Pick: the pick winner fit once on the 20 exploration folders and read against
`W0` on the 7 lockbox folders' picks.
"""

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

sys.path.insert(0, HERE)
import searchread as sr  # noqa: E402

sf, df = sr.sf, sr.df
rf, gate, folder_gate, fw = bf.rf, bf.gate, bf.folder_gate, bf.fw

DISCARD_WINNER = "G0 ⊕ ef"
PICK_WINNER = "All@32 / HGB-mono / pick"
FEAT, TARGET = "All@32", "pick"
QS = df.QS
ELIGIBLE = (5, 10)
STRATA = ["b05", "b10", "b20", "g0only", "rest"]
RR_BAR, RR_HI_BAR = 25.0, 35.0
PICK_GAIN = 2.0
PICK_FOLDERS = 5
EARLIER = 1800
POPULATION = 30659
EXPECTED = {
    DISCARD_WINNER: (("0.77", "2.10", "5.73"), ("4.52", "9.34", "19.28")),
    "G0": (("0.68", "2.10", "5.18"), ("4.53", "9.09", "18.72")),
}
FOLLOW = {1: "(1) an in-app suggestion plan", 2: "(2) more labels or more folders", 3: "(3) nothing"}


def pick_fit(d, train_folders, score_mask):
    """The pick winner's recipe (`searchfit.fold` for `All@32 / HGB-mono / pick`) fit once on `train_folders`, scored on `score_mask`."""
    tech, dims = sf.FEATURES[FEAT]
    tm = np.isin(d["folder_of"], train_folders)
    idx = np.where(tm)[0]
    x = sf.design(d, tm, tech, dims)
    x[:, : d["tech"].shape[1]][np.isnan(d["tech"])] = np.nan
    params = dict(rf.HGB)
    params["monotonic_cst"] = sf.mono_cst(dims, TARGET)
    m = HistGradientBoostingClassifier(**params).fit(x[idx], d["pick"][idx].astype(int), sample_weight=sf.mean1(d["weight"][idx]))
    out = np.full(len(d["rows"]), np.nan)
    out[score_mask] = m.decision_function(x[score_mask])
    return out


def yn(ok):
    return "yes" if ok else "no"


def main(dump, eval_dir, reject_dir, labels_dir, strict_dir, out_dir, jobs):
    out_path = os.path.join(out_dir, "confirmread.out.md")
    if os.path.exists(out_path):
        raise SystemExit(f"{out_path} exists: the confirmation is read once")
    start = time.time()
    roots = {k: os.path.normcase(os.path.abspath(p)) for k, p in (("discard-labels", labels_dir), ("strict-labels", strict_dir))}
    opened = {k: set() for k in roots}

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            for k, r in roots.items():
                if p.startswith(r + os.sep):
                    opened[k].add(os.path.relpath(p, r))

    sys.addaudithook(audit)
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n, nf = len(rows), len(folders)
    tb = np.random.default_rng(0).random(n)
    pick = d["pick"]
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    sp, _ = rf.load_scores(os.path.join(out_dir, "scores_pick.tsv"), d)
    sd, _ = rf.load_scores(os.path.join(out_dir, "scores_discard.tsv"), d)
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    ef_last = np.where(np.isnan(ef), -np.inf, ef)

    seen = np.zeros(n, dtype=bool)
    for path in (strict_dir, labels_dir):
        for r in bf.read_tsv(os.path.join(path, "manifest.tsv")):
            seen[index[(r["folder"], r["file"])]] = True
    pop = ~seen

    labels = bf.read_tsv(os.path.join(out_dir, "labels.tsv"))
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
    strata_file = bf.read_tsv(os.path.join(out_dir, "strata.tsv"))
    box_names = {r["folder"] for r in bf.read_tsv(os.path.join(out_dir, "lockbox.tsv")) if r["lockbox"] == "1"}
    box_f = sorted(folders.index(x) for x in box_names)
    expl_f = [fi for fi in range(nf) if fi not in box_f]
    box = np.isin(folder_of, box_f)
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    t0 = time.time()
    idx, dis, w = s["idx"], s["bad"], s["w"]
    win, g0 = sd[DISCARD_WINNER], stored_rej["G0"]
    cw = {q: bf.graded_cut(win, d, tb, q)[0] for q in QS}
    cg = {q: bf.graded_cut(g0, d, tb, q)[0] for q in QS}
    band = np.full(n, "control", dtype=object)
    band[cg[20] & ~cw[20]] = "g0only"
    band[cw[20]] = "b20"
    band[cw[10]] = "b10"
    band[cw[5]] = "b05"
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
    in_earlier = int(seen[idx].sum())

    cs = df.counts(s, folders, STRATA)
    cc = bf.cluster_counts(s, folders)
    scores = {DISCARD_WINNER: win, "G0": g0, fw.EYE: ef_last, fw.RANDOM: tb}
    res = {}
    for k, sc in scores.items():
        for q in QS:
            removed, out = bf.graded_cut(sc, d, tb, q)
            rem, ok = removed[idx], ~out[idx]
            cover = ~out
            boot, clus = df.metrics(s, rem, ok, cs), df.metrics(s, rem, ok, cc)
            res[(k, q)] = {
                "ok": ok, "point": df.metrics(s, rem, ok), "unw": df.unweighted(s, rem, ok), "boot": boot, "cboot": clus,
                "ci": {m: bf.quant(v) for m, v in boot.items()}, "cci": {m: bf.quant(v) for m, v in clus.items()},
                "pl": 100 * (removed & pick & cover).sum() / (pick & cover).sum(),
                "pl_pop": 100 * (removed & pick & cover & pop).sum() / (pick & cover & pop).sum(),
                "removed_pop": 100 * (removed & cover & pop).sum() / (cover & pop).sum(),
                "removed_dump": 100 * (removed & cover).sum() / cover.sum(),
                "n_rem": int((rem & ok).sum()), "n_dis": int((rem & ok & dis).sum()), "frames_out": int(out.sum()),
            }
    cut_time = time.time() - t0

    t1 = time.time()
    keys = list(scores)
    draws = Parallel(n_jobs=jobs)(
        delayed(sr.auc_draws)(dis[res[(k, 5)]["ok"]], bf.finite_by_folder(scores[k], d)[idx][res[(k, 5)]["ok"]], w[res[(k, 5)]["ok"]], cs[:, res[(k, 5)]["ok"]], cc[:, res[(k, 5)]["ok"]])
        for k in keys
    )
    auc = {k: dict(zip(("point", "unw", "boot", "cboot"), v)) for k, v in zip(keys, draws)}
    auc_time = time.time() - t1
    print(f"AUCs: {auc_time:.0f} s", file=sys.stderr, flush=True)

    t2 = time.time()
    spot = max(expl_f, key=lambda fi: int((folder_of == fi).sum()))
    spot_m = folder_of == spot
    spot_sc = pick_fit(d, [f for f in expl_f if f != spot], spot_m)
    spot_diff = float(np.abs(spot_sc[spot_m] - sp[PICK_WINNER][spot_m]).max())
    spot_tol = bool((np.abs(spot_sc[spot_m] - sp[PICK_WINNER][spot_m]) <= sf.TOL * np.maximum(1, np.abs(sp[PICK_WINNER][spot_m]))).all())
    pick_sc = pick_fit(d, expl_f, box)
    fit_time = time.time() - t2
    print(f"pick fits: {fit_time:.0f} s", file=sys.stderr, flush=True)
    w0 = stored["W0"]
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    npicks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(nf)}
    box_q = [fi for fi in box_f if sizes[fi] >= fw.MIN_FRAMES and npicks[fi] >= fw.MIN_PICKS]
    pt = sr.pick_task(d, tb, {PICK_WINNER: pick_sc, "W0": w0}, box_q)

    def dp_vs(k, base_k, q, m="DP"):
        a, b = res[(k, q)], res[(base_k, q)]
        return a["point"][m] - b["point"][m], bf.quant(a["boot"][m] - b["boot"][m]), bf.quant(a["cboot"][m] - b["cboot"][m])

    def auc_vs(k, base_k):
        a, b = auc[k], auc[base_k]
        return a["point"] - b["point"], bf.quant(a["boot"] - b["boot"]), bf.quant(a["cboot"] - b["cboot"])

    def eff(p, lohi, clohi, digits=1):
        return f"{p:+.{digits}f} [{lohi[0]:+.{digits}f}, {lohi[1]:+.{digits}f}] ([{clohi[0]:+.{digits}f}, {clohi[1]:+.{digits}f}])"

    def ivs(k, q, m):
        r = res[(k, q)]
        return bf.iv(r["point"][m], *r["ci"][m])

    def nm(k):
        return sr.nm(k)

    print("# Model-search confirmation: the single read of the two frozen winners\n")
    print(
        f"Discard: the frozen winner {nm(DISCARD_WINNER)} (its stored `scores_discard.tsv` column), `G0` (stored), `eye_focus` (missing last) and "
        "random (seed 0) on the fresh blind labels, weighted by the fresh `1/π`. Pick: the frozen winner "
        f"{nm(PICK_WINNER)} fit once on the {len(expl_f)} exploration folders and scored on the {len(box_f)} lockbox folders, against the stored "
        "`W0`. The bars and the follow-up rule are modelsearch.md's \"Pre-registration of the confirmation\"; the exploration's 600 labels are not "
        "read.\n"
    )

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{nf} / {n} / {missing}"),
            ("Earlier samples' frames excluded / fresh population", f"{int(seen.sum())} / {int(pop.sum())}"),
            ("Fresh labels (`labels.tsv`) / keep / discard", f"{len(idx)} / {int((~dis).sum())} / {int(dis.sum())}"),
            ("Lockbox folders / qualifying / frames / picks", f"{len(box_f)} / {len(box_q)} / {int(box.sum())} / {int((pick & box).sum())}"),
        ],
    )
    gate.table(
        "| Stratum | Sampled | Keep | Discard | `π` |",
        [[f"`{b}`", int((s["band"] == b).sum()), int(((s["band"] == b) & ~dis).sum()), int(((s["band"] == b) & dis).sum()),
          f"{s['pi'][s['band'] == b][0]:.6g}" if b != "control" else "per folder"] for b in ("b05", "b10", "b20", "g0only", "control")],
    )

    total_w = w.sum()
    base = 100 * (w * dis).sum() / total_w
    base_ci = bf.quant(bf.ratio(cs, w * dis, w))
    base_cci = bf.quant(bf.ratio(cc, w * dis, w))
    print("## Discardable base rate (fresh labels)\n")
    gate.table(
        "| Reading | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |",
        [("Discardable base rate", f"{base:.2f}", f"[{base_ci[0]:.2f}, {base_ci[1]:.2f}]", f"[{base_cci[0]:.2f}, {base_cci[1]:.2f}]", f"{100 * dis.mean():.2f}")],
    )

    print("## Checks\n")
    checks = [
        ("`labels.tsv` labels within {keep, discard}", "yes", yn(label_set <= {"keep", "discard"})),
        ("Earlier samples' frames (strict + discard)", str(EARLIER), str(int(seen.sum()))),
        ("Fresh population", str(POPULATION), str(int(pop.sum()))),
        ("Fresh labels in an earlier sample", "0", str(in_earlier)),
        ("`strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump", f"0 of {len(strata_file)}", f"{strata_diff} of {len(strata_file)}"),
        ("Fresh labels whose recomputed stratum / `π` differ from `labels.tsv`", "0 / 0", f"{band_diff} / {pi_diff}"),
        ("Sum of `1/π` over the fresh labels", f"{POPULATION}.0", f"{total_w:.1f}"),
        ("Frames left out of a cut, over the four scores and `q`", "0", str(sum(res[(k, q)]["frames_out"] for k in scores for q in QS))),
    ]
    for k, (pl_exp, rem_exp) in EXPECTED.items():
        checks.append((f"{nm(k)} `PL` % (full dump) q5 / q10 / q20 (Step 3, Step 4)", " / ".join(pl_exp), " / ".join(f"{res[(k, q)]['pl']:.2f}" for q in QS)))
        checks.append((f"{nm(k)} removed % of the fresh population q5 / q10 / q20 (Step 4)", " / ".join(rem_exp), " / ".join(f"{res[(k, q)]['removed_pop']:.2f}" for q in QS)))
    checks += [
        (f"Pick recipe refit on the 19 other exploration folders vs `scores_pick.tsv`, spot fold `{folders[spot]}`: max abs difference",
         f"<= {sf.TOL:g} x max(1, abs)", f"{spot_diff:.2e}" if spot_tol else f"{spot_diff:.2e} (over)"),
        ("Lockbox folders among the pick fit's training folders", "0", str(len(set(expl_f) & set(box_f)))),
        ("Pick winner finite on every lockbox frame / `W0` finite on every lockbox frame", "yes / yes", f"{yn(np.isfinite(pick_sc[box]).all())} / {yn(np.isfinite(w0[box]).all())}"),
        ("Qualifying lockbox folders", "7", str(len(box_q))),
    ]
    checks = [(c, e, v, "pass" if e == v or (c.startswith("Pick recipe refit") and spot_tol) else "FAIL") for c, e, v in checks]
    for k in roots:
        ok = opened[k] <= {"manifest.tsv"}
        checks.append((f"Files opened under the {k} directory", "`manifest.tsv` only", ", ".join(f"`{p}`" for p in sorted(opened[k])) or "none", "pass" if ok else "FAIL"))
    gate.table("| Check | Expected | This run | Result |", checks)
    if not all(r[3] == "pass" for r in checks):
        sys.stdout = tee.stream
        raise SystemExit("a check failed; nothing written")

    print("## Discard: the cuts on the fresh labels\n")
    print(
        "Per folder on the full dump, the bottom `floor(q * n / 100)` by score, ties by the seed-0 draw (`bf.graded_cut`). Weighted (`1/π`) with "
        "the stratified bootstrap 95% interval (1,000 draws, seed 0, within `b05`, `b10`, `b20`, `g0only` and each `control` folder cell; one count "
        "matrix for every score) and the folder-cluster interval (1,000 draws, seed 0) beside it; unweighted in parentheses. `PL`: the dump's picks "
        "inside the cut (full dump; fresh population after the slash); removed: the fresh population's share (full dump's after the slash).\n"
    )
    mrow = []
    for k in scores:
        for q in QS:
            r = res[(k, q)]
            mrow.append([
                nm(k), f"q{q}", f"{r['n_rem']} / {r['n_dis']}",
                f"{ivs(k, q, 'DP')} ({bf.f1(r['unw']['DP'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["DP"]),
                f"{ivs(k, q, 'RR')} ({bf.f1(r['unw']['RR'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["RR"]),
                f"{r['pl']:.2f} / {r['pl_pop']:.2f}", ivs(k, q, "DR"), ivs(k, q, "MR"), ivs(k, q, "DO"), f"{r['removed_pop']:.2f} / {r['removed_dump']:.2f}",
            ])
    gate.table("| Score | Cut | Sampled removed / discard | `DP` % | Cluster | `RR` % | Cluster | `PL` % | `DR` % | `MR` % | `DO` % | Removed % |", mrow)

    print("### Weighted discard AUC\n")
    print("`roc_auc_score(discard, -score, sample_weight=1/π)`, a missing value at the folder's lowest finite score minus 1; stratified 95%, cluster, unweighted.\n")
    gate.table(
        "| Score | AUC [stratified 95%] | Cluster | Unweighted |",
        [[nm(k), bf.iv(auc[k]["point"], *bf.quant(auc[k]["boot"]), 3), "[{:.3f}, {:.3f}]".format(*bf.quant(auc[k]["cboot"])), f"{auc[k]['unw']:.3f}"] for k in scores],
    )

    print("### Paired differences of the discard winner\n")
    print("On the same draws; stratified 95%, cluster in parentheses.\n")
    prow = []
    for base_k in ("G0", fw.EYE, fw.RANDOM):
        prow.append([f"{nm(DISCARD_WINNER)} - {nm(base_k)}, `DP` points"] + [eff(*dp_vs(DISCARD_WINNER, base_k, q)) for q in QS])
    for base_k in ("G0", fw.EYE, fw.RANDOM):
        prow.append([f"{nm(DISCARD_WINNER)} - {nm(base_k)}, AUC", eff(*auc_vs(DISCARD_WINNER, base_k), 3), "", ""])
    prow.append(["`G0` - random, `DP` points"] + [eff(*dp_vs("G0", fw.RANDOM, q)) for q in QS])
    prow.append(["`G0` - `eye_focus`, AUC", eff(*auc_vs("G0", fw.EYE), 3), "", ""])
    gate.table("| Difference | q5 | q10 | q20 |", prow)

    print("## Discard: the bars\n")
    print(
        f"A cut of {nm(DISCARD_WINNER)} is usable when all six hold (stratified intervals decide; the cluster interval beside it, a pass whose "
        f"cluster interval crosses its bar is fragile): (1) `RR` <= {RR_BAR:.0f}% and its 97.5% quantile <= {RR_HI_BAR:.0f}%; (2) `PL` <= "
        f"{df.PL_BAR:.0f}% (full dump); (3) removed >= {df.REMOVED_BAR}% of the fresh population; (4) `DP`'s 2.5% quantile above the base rate "
        f"{base:.2f}%; (5) `DP` - `G0` paired 2.5% quantile above zero; (6) AUC - `eye_focus` paired 2.5% quantile above zero. q20 is read but "
        "a pass needs q5 or q10.\n"
    )
    ae = auc_vs(DISCARD_WINNER, fw.EYE)
    verdict, brow = {}, []
    for q in QS:
        r = res[(DISCARD_WINNER, q)]
        rr, rr_hi, rr_chi = r["point"]["RR"], r["ci"]["RR"][1], r["cci"]["RR"][1]
        dp_lo, dp_clo = r["ci"]["DP"][0], r["cci"]["DP"][0]
        dg = dp_vs(DISCARD_WINNER, "G0", q)
        c = [rr <= RR_BAR and rr_hi <= RR_HI_BAR, r["pl"] <= df.PL_BAR, r["removed_pop"] >= df.REMOVED_BAR, dp_lo > base, dg[1][0] > 0, ae[1][0] > 0]
        cl = [rr <= RR_BAR and rr_chi <= RR_HI_BAR, c[1], c[2], dp_clo > base, dg[2][0] > 0, ae[2][0] > 0]
        usable = all(c)
        verdict[q] = {"c": c, "usable": usable, "fragile": usable and not all(cl), "rr": rr, "dg": dg}
        brow.append([
            f"q{q}",
            f"{rr:.1f}, 97.5% {rr_hi:.1f} (cluster {rr_chi:.1f}): {yn(c[0])}",
            f"{r['pl']:.2f}: {yn(c[1])}",
            f"{r['removed_pop']:.2f}: {yn(c[2])}",
            f"2.5% {dp_lo:.1f} (cluster {dp_clo:.1f}) vs {base:.2f}: {yn(c[3])}",
            f"{dg[0]:+.1f}, 2.5% {dg[1][0]:+.1f} (cluster {dg[2][0]:+.1f}): {yn(c[4])}",
            f"{ae[0]:+.3f}, 2.5% {ae[1][0]:+.3f} (cluster {ae[2][0]:+.3f}): {yn(c[5])}",
            ("usable" + (", fragile" if verdict[q]["fragile"] else "")) if usable and q in ELIGIBLE else ("passes, not eligible" if usable else "not usable"),
        ])
    gate.table("| Cut | (1) `RR` % | (2) `PL` % | (3) Removed % | (4) `DP` vs base | (5) `DP` - `G0` | (6) AUC - `eye_focus` | Verdict |", brow)
    passing = [q for q in ELIGIBLE if verdict[q]["usable"]]
    best = max(passing) if passing else None
    if best is not None:
        d_follow = 1
    elif any(verdict[q]["rr"] <= RR_BAR and verdict[q]["c"][1] and verdict[q]["c"][2] for q in ELIGIBLE) or any(
        verdict[q]["dg"][0] > 0 and verdict[q]["dg"][1][0] <= 0 <= verdict[q]["dg"][1][1] for q in ELIGIBLE
    ):
        d_follow = 2
    else:
        d_follow = 3
    close = [
        f"q{q}: `RR` point <= {RR_BAR:.0f}% with (2) and (3) holding {yn(verdict[q]['rr'] <= RR_BAR and verdict[q]['c'][1] and verdict[q]['c'][2])}; "
        f"`DP` - `G0` point positive with the interval spanning zero {yn(verdict[q]['dg'][0] > 0 and verdict[q]['dg'][1][0] <= 0 <= verdict[q]['dg'][1][1])}"
        for q in ELIGIBLE
    ]
    print(f"**Discard verdict**: {'passes, best cut q' + str(best) if best is not None else 'does not pass (no usable cut at q5 or q10)'}. "
          f"Follow-up rule: {FOLLOW[d_follow]}. The \"close\" conditions: {'; '.join(close)}.\n")

    print("## Pick: the lockbox folders\n")
    print(
        f"The pick winner fit once on the {len(expl_f)} exploration folders (PCA, mean fill, standardization on them; the pick target on their "
        f"frames, their per-folder class weights rescaled to mean 1), scored on the {len(box_f)} lockbox folders; `W0` stored (fit leave one folder "
        "out on all 27, so it saw the other lockbox folders). Per folder the top `floor(q * n / 100)` by score, ties by the seed-0 draw. Mean over "
        "the qualifying lockbox folders (per-folder SD); pooled: the frame-weighted mean.\n"
    )
    srow = []
    for k in (PICK_WINNER, "W0"):
        r = [nm(k)]
        for key in ("P@5", "P@10", "P@20", "R@10", "rp"):
            r.append(fw.pm(*sr.summ(pt[k], key, sizes)[:2], digits=1))
        r.append(fw.pm(*sr.summ(pt[k], "frame", sizes)[:2]))
        r += [f"{sr.summ(pt[k], 'P@10', sizes)[2]:.1f}", f"{sr.summ(pt[k], 'R@10', sizes)[2]:.1f}", f"{sr.summ(pt[k], 'frame', sizes)[2]:.3f}"]
        srow.append(r)
    gate.table("| Score | `P@5` % | `P@10` % | `P@20` % | `R@10` % | R-precision % | Frame AUC | Pooled `P@10` | Pooled `R@10` | Pooled frame AUC |", srow)
    print("Per folder:\n")
    gate.table(
        "| Folder | Frames | Picks | Winner `P@10` % | `W0` `P@10` % | Winner `R@10` % | `W0` `R@10` % | Winner frame AUC | `W0` frame AUC |",
        [[f"`{folders[fi]}`", sizes[fi], npicks[fi], f"{pt[PICK_WINNER][fi]['P@10']:.1f}", f"{pt['W0'][fi]['P@10']:.1f}", f"{pt[PICK_WINNER][fi]['R@10']:.1f}",
          f"{pt['W0'][fi]['R@10']:.1f}", f"{pt[PICK_WINNER][fi]['frame']:.3f}", f"{pt['W0'][fi]['frame']:.3f}"] for fi in box_q],
    )
    print("Paired per-folder differences against `W0` (1,000-draw folder bootstrap 95%, seed 0; one-sided Wilcoxon signed-rank p, zeros dropped; folders at or above `W0`):\n")
    drow, diffs = [], {}
    for key, digits in (("P@10", 1), ("R@10", 1), ("P@5", 1), ("P@20", 1), ("rp", 1), ("frame", 3)):
        mean, lo, hi, p, above, nn = sr.pdiff(pt[PICK_WINNER], pt["W0"], key)
        diffs[key] = (mean, lo, hi, p, above, nn)
        drow.append([f"`{key}`" if key != "rp" else "R-precision", f"{mean:+.{digits}f} [{lo:+.{digits}f}, {hi:+.{digits}f}]", f"{p:.4f}", f"{above} of {nn}"])
    gate.table("| Metric | Winner - `W0` | p | Folders at or above `W0` |", drow)
    gain, lo, hi, _, above, nn = diffs["P@10"]
    pc = [gain >= PICK_GAIN, lo > 0, above >= PICK_FOLDERS]
    print("## Pick: the bars\n")
    gate.table(
        "| Condition | Value | Holds |",
        [
            (f"(1) mean `P@10` >= `W0`'s + {PICK_GAIN:.1f} points", f"{gain:+.1f} points", yn(pc[0])),
            ("(2) folder-bootstrap 2.5% quantile of the mean paired `P@10` difference above zero", f"{lo:+.1f}", yn(pc[1])),
            (f"(3) at least {PICK_FOLDERS} of the {nn} folders at or above `W0`", f"{above} of {nn}", yn(pc[2])),
        ],
    )
    p_pass = all(pc)
    p_follow = 1 if p_pass else 2 if gain > 0 and lo <= 0 <= hi else 3
    print(f"**Pick verdict**: {'passes' if p_pass else 'does not pass'}. Follow-up rule: {FOLLOW[p_follow]} (the `P@10` gain point "
          f"{gain:+.1f} [{lo:+.1f}, {hi:+.1f}]: positive with the interval spanning zero {yn(gain > 0 and lo <= 0 <= hi)}).\n")

    print("## Wall time\n")
    gate.table(
        "| Stage | Seconds |",
        [("Load (dump, stored and Step 2 scores, fresh labels, strata, lockbox)", f"{load_time:.0f}"), ("Cuts, metrics and bootstraps", f"{cut_time:.0f}"),
         (f"AUC bootstraps ({len(keys)} scores, {jobs} jobs)", f"{auc_time:.0f}"), ("Pick fits (the spot-fold check and the lockbox fit)", f"{fit_time:.0f}")],
    )
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(out_path, "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))


if __name__ == "__main__":
    if len(sys.argv) not in (7, 8):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:7], int(sys.argv[7]) if len(sys.argv) > 7 else 8)
