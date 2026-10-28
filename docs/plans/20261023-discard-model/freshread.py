"""Step 6: the single confirmatory read of the frozen winner `LR/T` on the
fresh blind "OK to discard?" labels, with the bars and the follow-up rule of
discardmodel.md's "Pre-registration of the confirmation".

Usage: python -I -B freshread.py <fresh-dump-dir> <out-dir>

<fresh-dump-dir> is Step 4's dump of the six fresh folders (`features\\`,
`embed.tsv`, `genre_text.tsv`), read by the archived gate.py's `load` /
`build`; <out-dir> the data folder with Step 5's `scores_fresh.tsv`,
`manifest.tsv` and `strata.tsv` and `freshlabels.py`'s `labels.tsv`. Prints
the tables (Markdown) and saves them as `freshread.out.md` in <out-dir>;
refuses to run when that file exists, so the read happens once. Every value is
computed before anything is printed; the label-free checks run before
`labels.tsv` is opened, and a failure stops the read there. Nothing is written
on a failed check. No sidecar under `sample\\` is opened.
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

sys.path.insert(0, os.path.join(ARCHIVED, "20261018-discard-labels"))
import discardfilter as df  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261023-face-choice"))
import facechoiceread as fcr  # noqa: E402

rf, gate = bf.rf, bf.gate

QS = df.QS
ELIGIBLE = (5, 10)
WINNER = "LR/T"
REFS = ["ef", "ef-fill", "random"]
SCORES = [WINNER] + REFS
BANDS = ["b05", "b10", "b20", "efonly", "control"]
STRATA = ["b05", "b10", "b20", "efonly", "rest"]
RR_BAR, RR_HI_BAR = 25.0, 35.0
PL_BAR, REMOVED_BAR = df.PL_BAR, df.REMOVED_BAR
POPULATION, PICKS, REJECTS, SAMPLE = 14525, 1795, 68, 600
EF_GAP = 5.1e-5
# Step 5's label-free readings (freshsample.out.md): `PL` % at q5 / q10 / q20 on the fresh folders' picks, and the removed share.
STEP5_PL = {
    WINNER: ("2.56", "5.91", "14.48"),
    "ef": ("4.07", "7.30", "14.99"),
    "ef-fill": ("2.84", "5.96", "13.09"),
    "random": ("4.46", "10.25", "20.33"),
}
STEP5_REMOVED = ("4.98", "9.98", "20.00")
# Step 3's exploration numbers of the winner (discardmodel.md, "Results of the exploration").
EXPLORATION = {
    2: {"auc": "0.812 [0.757, 0.863]", "ef": "0.785 [0.723, 0.844]", "diff": "+0.027 [-0.020, +0.078] ([-0.031, +0.069])", "rr": ("12.8 [6.3, 24.7]", "15.3 [9.1, 24.6]"),
        "ef_rr": ("5.5", "12.9"), "base": "42.31"},
    1: {"auc": "0.795 [0.737, 0.851]", "ef": "0.777 [0.716, 0.841]", "diff": "+0.018 [-0.018, +0.053] ([-0.014, +0.054])", "rr": ("9.5 [3.0, 19.6]", "21.7 [5.3, 39.1]"),
        "ef_rr": ("16.8", "14.3"), "base": "48.45"},
}
FOLLOW = {1: "(1) pass: an in-app suggestion plan", 2: "(2) close: more labels (another fresh round)", 3: "(3) otherwise: nothing, `eye_focus` stays"}
PARTS = {"all": "all labeled frames", "miss": "the missing-face subset (no `eye_focus`)", "has": "its complement (`eye_focus` present)"}


def nm(k):
    return f"`{k}`" + (" (reference)" if k in REFS else " (winner)")


def yn(ok):
    return "yes" if ok else "no"


def pct(num, den):
    return f"{100 * num / den:.1f}" if den else "-"


def main(fresh, out_dir):
    out_path = os.path.join(out_dir, "freshread.out.md")
    if os.path.exists(out_path):
        raise SystemExit(f"{out_path} exists: the confirmation is read once")
    start = time.time()
    out_root = os.path.normcase(os.path.abspath(out_dir))
    skip_roots = [os.path.normcase(os.path.abspath(fresh)), os.path.join(out_root, "sample")]
    sample_root = os.path.join(out_root, "sample")
    opened, sample_opened = [], []

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            if p.startswith(sample_root + os.sep):
                sample_opened.append(os.path.relpath(p, sample_root))
            elif p.startswith(out_root + os.sep) and not any(p.startswith(r + os.sep) for r in skip_roots):
                opened.append(os.path.relpath(p, out_root))

    sys.addaudithook(audit)
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee

    def fail(tables):
        print("# Discard model: the confirmation read\n")
        for head, rows in tables:
            gate.table(head, rows)
        sys.stdout = tee.stream
        raise SystemExit("a check failed; nothing written")

    frames, missing = gate.load(fresh)
    e = gate.build(frames, gate.load_text(os.path.join(fresh, "genre_text.tsv")))
    rows, folders, folder_of = e["rows"], e["folders"], e["folder_of"]
    n = len(rows)
    tb = np.random.default_rng(0).random(n)
    pick, reject = e["pick"], e["reject"]
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    sf, sf_rows = rf.load_scores(os.path.join(out_dir, "scores_fresh.tsv"), e)
    ef, ef_fill, win = sf["ef"], sf["ef-fill"], sf[WINNER]
    no_ef = np.isnan(ef)
    feat_ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    both = ~no_ef & ~np.isnan(feat_ef)
    ef_gap = float(np.max(np.abs(ef[both] - feat_ef[both]))) if both.any() else 0.0
    fill_ok = bool(((ef_fill == ef) | no_ef).all())
    scores = {WINNER: win, "ef": np.where(no_ef, -np.inf, ef), "ef-fill": np.where(np.isnan(ef_fill), -np.inf, ef_fill), "random": tb}
    finite = {k: bf.finite_by_folder(sc, e) for k, sc in scores.items()}
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    # Label-free: cuts, strata, the design and Step 5's readings.
    t0 = time.time()
    cuts = {(k, q): bf.graded_cut(sc, e, tb, q) for k, sc in scores.items() for q in QS}
    res_pl = {(k, q): 100 * (cuts[(k, q)][0] & pick).sum() / pick.sum() for k in SCORES for q in QS}
    res_rem = {(k, q): 100 * cuts[(k, q)][0].sum() / n for k in SCORES for q in QS}
    cut_miss = {(k, q): (int((cuts[(k, q)][0] & no_ef).sum()), int(cuts[(k, q)][0].sum()), int((cuts[(k, q)][0] & no_ef & pick).sum()),
                         int((cuts[(k, q)][0] & pick).sum())) for k in SCORES for q in QS}
    cw = {q: cuts[(WINNER, q)][0] for q in QS}
    ce = {q: cuts[("ef", q)][0] for q in QS}
    band = np.full(n, "control", dtype=object)
    band[ce[20] & ~cw[20]] = "efonly"
    band[cw[20]] = "b20"
    band[cw[10]] = "b10"
    band[cw[5]] = "b05"
    man = bf.read_tsv(os.path.join(out_dir, "manifest.tsv"))
    strata_rows = bf.read_tsv(os.path.join(out_dir, "strata.tsv"))
    ms = fcr.sample_of(man, index, folder_of, False)
    sdiff, bdiff, pdiff = fcr.strata_check(e, np.ones(n, dtype=bool), band, strata_rows, ms)

    checks_a = [
        ("Fresh frames (with a vector) / without a vector", f"{POPULATION} / 0", f"{n} / {missing}"),
        ("Fresh folders", "6", str(len(folders))),
        ("Fresh picks / rejects (`gate.load`)", f"{PICKS} / {REJECTS}", f"{int(pick.sum())} / {int(reject.sum())}"),
        ("`scores_fresh.tsv` rows / fresh frames", f"{n} / {n}", f"{sf_rows} / {n}"),
        ("The winner finite on every fresh frame", "yes", yn(np.isfinite(win).all())),
        ("`ef` of `scores_fresh.tsv` vs the features TSV's `eye_focus`: frames missing on one side only", "0", str(int((no_ef != np.isnan(feat_ef)).sum()))),
        ("`ef` of `scores_fresh.tsv` vs the features TSV's `eye_focus`: largest difference within 5.1e-05", "yes", yn(ef_gap <= EF_GAP)),
        ("`ef-fill` equals `ef` wherever `ef` is present", "yes", yn(fill_ok)),
        ("`ef` missing % / `ef-fill` missing % (Step 4)", "9.00 / 1.18", f"{100 * no_ef.mean():.2f} / {100 * np.isnan(ef_fill).mean():.2f}"),
        ("Frames left out of a cut, over the four scores and `q`", "0", str(sum(int(cuts[c][1].sum()) for c in cuts))),
    ]
    for k in SCORES:
        checks_a.append((f"{nm(k)} `PL` % / removed % (fresh folders) q5 / q10 / q20, as Step 5", " / ".join(STEP5_PL[k]) + " ; " + " / ".join(STEP5_REMOVED),
                         " / ".join(f"{res_pl[(k, q)]:.2f}" for q in QS) + " ; " + " / ".join(f"{res_rem[(k, q)]:.2f}" for q in QS)))
    checks_a += [
        ("`manifest.tsv` rows / distinct frames", f"{SAMPLE} / {SAMPLE}", f"{len(man)} / {len(set(ms['idx'].tolist()))}"),
        ("`strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the cuts", f"0 of {len(strata_rows)}", f"{sdiff} of {len(strata_rows)}"),
        ("`manifest.tsv` frames whose recomputed stratum / `π` differ", "0 / 0", f"{bdiff} / {pdiff}"),
        ("Sum of `1/π` over the sampled frames", f"{POPULATION}.0", f"{ms['w'].sum():.1f}"),
    ]
    checks_a = [(c, x, v, "pass" if x == v else "FAIL") for c, x, v in checks_a]
    early = sorted({p for p in opened if os.path.basename(p) == "labels.tsv"})
    checks_a.append(("`labels.tsv` opened before these checks", "no", ", ".join(f"`{p}`" for p in early) or "no", "pass" if not early else "FAIL"))
    head_a = "| Check (label-free, before `labels.tsv` is opened) | Expected | This run | Result |"
    if not all(r[3] == "pass" for r in checks_a):
        fail([(head_a, checks_a)])

    # The labels.
    lab = bf.read_tsv(os.path.join(out_dir, "labels.tsv"))
    s = fcr.sample_of(lab, index, folder_of, True)
    label_set = {r["label"] for r in lab}
    by_name = {r["name"]: r for r in man}
    man_diff = sum(r["name"] not in by_name or r["band"] != by_name[r["name"]]["band"] or r["inclusion_prob"] != by_name[r["name"]]["inclusion_prob"]
                   or (r["folder"], r["file"]) != (by_name[r["name"]]["folder"], by_name[r["name"]]["file"]) for r in lab)
    cs = df.counts(s, folders, STRATA)
    cc = bf.cluster_counts(s, folders)
    idx, w, dis = s["idx"], s["w"], s["bad"]
    masks = {"all": np.ones(len(idx), dtype=bool), "miss": no_ef[idx], "has": ~no_ef[idx]}

    res, auc = {}, {}
    for k in SCORES:
        for q in QS:
            res[(k, q)] = fcr.cut_metrics(s, cuts[(k, q)][0][idx], masks["all"], cs, cc)
        for part, msk in masks.items():
            auc[(k, part)] = fcr.auc_draws(s, finite[k][idx], msk, cs, cc)
    da = {(k, part): fcr.diff(auc[(k, part)], auc[("ef", part)]) for k in SCORES if k != "ef" for part in PARTS}
    d_fill = fcr.diff(auc[(WINNER, "all")], auc[("ef-fill", "all")])
    drr = {(k, q): fcr.diff(res[(k, q)], res[("ef", q)], "RR") for k in SCORES if k != "ef" for q in QS}
    base = 100 * (w * dis).sum() / w.sum()
    base_ci, base_cci = bf.quant(bf.ratio(cs, w * dis, w)), bf.quant(bf.ratio(cc, w * dis, w))
    base_part = {}
    for part in ("miss", "has"):
        mk = masks[part]
        base_part[part] = (pct((w * dis * mk).sum(), (w * mk).sum()), bf.quant(bf.ratio(cs, w * dis * mk, w * mk)), bf.quant(bf.ratio(cc, w * dis * mk, w * mk)),
                           pct((dis & mk).sum(), mk.sum()), int(mk.sum()), int((mk & ~dis).sum()), int((mk & dis).sum()), pct((w * mk).sum(), w.sum()))

    checks_b = [
        ("`labels.tsv` labels within {keep, discard}", "yes", yn(label_set <= {"keep", "discard"})),
        ("`labels.tsv` rows / distinct frames", f"{SAMPLE} / {SAMPLE}", f"{len(lab)} / {len(set(idx.tolist()))}"),
        ("`labels.tsv` rows whose name, file, stratum or `π` differ from `manifest.tsv`", "0", str(man_diff)),
        ("Sum of `1/π` over the labeled frames", f"{POPULATION}.0", f"{w.sum():.1f}"),
    ]
    checks_b = [(c, x, v, "pass" if x == v else "FAIL") for c, x, v in checks_b]
    allowed = {"scores_fresh.tsv", "manifest.tsv", "strata.tsv", "labels.tsv"}
    got = sorted(set(opened))
    checks_b.append(("Files opened in the data folder (outside `dump\\` and `sample\\`)", "within " + ", ".join(f"`{p}`" for p in sorted(allowed)),
                     ", ".join(f"`{p}`" for p in got) or "none", "pass" if set(got) <= allowed else "FAIL"))
    checks_b.append(("Files opened under `sample\\`", "none", ", ".join(f"`{p}`" for p in sorted(set(sample_opened))) or "none", "pass" if not sample_opened else "FAIL"))
    head_b = "| Check (labels) | Expected | This run | Result |"
    if not all(r[3] == "pass" for r in checks_b):
        fail([(head_a, checks_a), (head_b, checks_b)])

    # The bars and the follow-up rule.
    ae = da[(WINNER, "all")]
    verdict = {}
    for q in QS:
        r = res[(WINNER, q)]
        rr, rr_hi, rr_chi = r["point"]["RR"], r["ci"]["RR"][1], r["cci"]["RR"][1]
        dp_lo, dp_clo = r["ci"]["DP"][0], r["cci"]["DP"][0]
        c = [rr <= RR_BAR and rr_hi <= RR_HI_BAR, res_pl[(WINNER, q)] <= PL_BAR, res_rem[(WINNER, q)] >= REMOVED_BAR, dp_lo > base, ae[1][0] > 0]
        cl = [rr <= RR_BAR and rr_chi <= RR_HI_BAR, c[1], c[2], dp_clo > base, ae[2][0] > 0]
        usable = all(c)
        verdict[q] = {"c": c, "cl": cl, "usable": usable, "fragile": usable and not all(cl), "rr": rr, "rr_hi": rr_hi, "rr_chi": rr_chi, "dp": r["point"]["DP"],
                      "dp_lo": dp_lo, "dp_clo": dp_clo}
    passing = [q for q in ELIGIBLE if verdict[q]["usable"]]
    best = max(passing) if passing else None
    close_rr = {q: verdict[q]["rr"] <= RR_BAR and verdict[q]["c"][1] and verdict[q]["c"][2] for q in ELIGIBLE}
    close_auc = ae[0] > 0 and ae[1][0] <= 0 <= ae[1][1]
    follow = 1 if best is not None else 2 if any(close_rr.values()) or close_auc else 3
    compute_time = time.time() - t0

    # ---------------------------------------------------------------- print
    print("# Discard model: the confirmation read\n")
    print(
        "The frozen winner `LR/T` (refit once on the 27 dump folders, Step 5's `scores_fresh.tsv`) and the references `ef` (the `af` value, missing "
        "last), `ef-fill` and random (seed 0), read once against the 600 fresh blind \"OK to discard?\" labels of the six fresh folders, weighted by "
        "the fresh `1/π`. The bars and the follow-up rule are discardmodel.md's \"Pre-registration of the confirmation\". One stratified count matrix "
        "(1,000 draws, seed 0, within `b05`, `b10`, `b20`, `efonly` and each `control` folder cell) and one folder-cluster matrix (the six folders, "
        "1,000 draws, seed 0) are shared by every score and quantity, so the differences are paired. A missing score sorts last: `-inf` in the cut, "
        "the folder's lowest finite value minus 1 in the AUC. No label of rounds 1 and 2 is read.\n"
    )
    print("## Checks\n")
    gate.table(head_a, checks_a)
    gate.table(head_b, checks_b)

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Fresh folders / frames / picks / explicit rejects", f"{len(folders)} / {n} / {int(pick.sum())} / {int(reject.sum())}"),
            ("Fresh frames without `eye_focus` / after the fill (`ef-fill`)", f"{int(no_ef.sum())} ({pct(no_ef.sum(), n)}%) / {int(np.isnan(ef_fill).sum())} ({pct(np.isnan(ef_fill).sum(), n)}%)"),
            ("Labels / keep / discard", f"{len(idx)} / {int((~dis).sum())} / {int(dis.sum())}"),
            ("Missing-face labels / keep / discard (weighted share of the population)",
             f"{base_part['miss'][4]} / {base_part['miss'][5]} / {base_part['miss'][6]} ({base_part['miss'][7]}%)"),
        ],
    )
    gate.table(
        "| Stratum | Population | Sampled | Keep | Discard | `π` |",
        [[f"`{b}`", sum(int(r["population"]) for r in strata_rows if r["band"] == b), int((s["band"] == b).sum()), int(((s["band"] == b) & ~dis).sum()),
          int(((s["band"] == b) & dis).sum()), f"{s['pi'][s['band'] == b][0]:.6g}" if b != "control" else "per folder"] for b in BANDS],
    )
    print("### Discardable base rate (fresh labels)\n")
    brow = [("all", f"{base:.2f}", "[{:.2f}, {:.2f}]".format(*base_ci), "[{:.2f}, {:.2f}]".format(*base_cci), f"{100 * dis.mean():.2f}")]
    for part, title in (("miss", "missing-face"), ("has", "complement")):
        bp = base_part[part]
        brow.append((title, bp[0], "[{:.1f}, {:.1f}]".format(*bp[1]), "[{:.1f}, {:.1f}]".format(*bp[2]), bp[3]))
    gate.table("| Frames | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |", brow)

    print("## Where the cuts sit (label-free)\n")
    print("Per score and cut on the fresh folders: the removed frames without `eye_focus` and the fresh picks among the removed.\n")
    wrow = []
    for k in SCORES:
        for q in QS:
            a, b, c, x = cut_miss[(k, q)]
            wrow.append([nm(k), f"q{q}", b, f"{a} ({pct(a, b)}%)", x, f"{c} ({pct(c, x)}%)"])
    gate.table("| Score | Cut | Removed | Removed without `eye_focus` | Picks removed | Picks removed without `eye_focus` |", wrow)

    print("## Weighted discard AUC\n")
    for part, title in PARTS.items():
        print(f"### {title[0].upper() + title[1:]}\n")
        if part != "all":
            print("A reading beside the pre-registered read, as Step 3's subsets; it decides nothing.\n")
        arow = []
        for k in SCORES:
            a = auc[(k, part)]
            arow.append([nm(k), fcr.auc_s(a), fcr.cauc_s(a), f"{a['unw']:.3f}", "" if k == "ef" else fcr.eff(*da[(k, part)], 3)])
        gate.table("| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |", arow)
    print(f"`{WINNER}` - `ef-fill`, AUC: {fcr.eff(*d_fill, 3)}.\n")

    print("## The cuts on the fresh labels\n")
    print(
        "Per folder on the fresh folders, the bottom `floor(q * n / 100)` by score, ties by the seed-0 draw (`bf.graded_cut`). Weighted (`1/π`) "
        "with the stratified 95% interval and the unweighted value in parentheses; the cluster interval beside. `PL`: the fresh folders' picks "
        "inside the cut; removed: the share of the fresh population.\n"
    )
    crow = []
    for k in SCORES:
        for q in QS:
            r = res[(k, q)]
            crow.append([nm(k), f"q{q}", f"{r['n_rem']} / {r['n_dis']}", f"{bf.iv(r['point']['DP'], *r['ci']['DP'])} ({bf.f1(r['unw']['DP'])})",
                         "[{:.1f}, {:.1f}]".format(*r["cci"]["DP"]), f"{fcr.rr_s(r)} ({bf.f1(r['unw']['RR'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["RR"]),
                         f"{res_pl[(k, q)]:.2f}", f"{res_rem[(k, q)]:.2f}", "" if k == "ef" else fcr.eff(*drr[(k, q)])])
    gate.table("| Score | Cut | Sampled removed / discard | `DP` % | `DP` cluster | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `ef`, points |", crow)

    print("## The bars\n")
    print(
        f"A cut of `{WINNER}` at q5 or q10 is usable when all five hold (the stratified intervals decide; a pass whose cluster interval crosses its "
        f"bar is fragile): (1) `RR` <= {RR_BAR:.0f}% and its stratified 97.5% quantile <= {RR_HI_BAR:.0f}%; (2) `PL` <= {PL_BAR:.0f}% (the fresh "
        f"folders' picks); (3) removed >= {REMOVED_BAR}% of the fresh population; (4) `DP`'s stratified 2.5% quantile above the fresh base rate "
        f"{base:.2f}% (point); (5) AUC - `ef`, paired, stratified 2.5% quantile above zero (one value for every cut). q20 is read, not eligible.\n"
    )
    vrow = []
    for q in QS:
        v = verdict[q]
        c, cl = v["c"], v["cl"]
        if q in ELIGIBLE:
            word = ("usable" + (", fragile" if v["fragile"] else "")) if v["usable"] else "not usable"
        else:
            word = "not eligible" + (" (would pass)" if v["usable"] else "")
        vrow.append([
            f"q{q}",
            f"{v['rr']:.1f}, 97.5% {v['rr_hi']:.1f} (cluster {v['rr_chi']:.1f}): {yn(c[0])}" + ("" if c[0] == cl[0] else f", cluster {yn(cl[0])}"),
            f"{res_pl[(WINNER, q)]:.2f}: {yn(c[1])}",
            f"{res_rem[(WINNER, q)]:.2f}: {yn(c[2])}",
            f"{v['dp']:.1f}, 2.5% {v['dp_lo']:.1f} (cluster {v['dp_clo']:.1f}) vs {base:.2f}: {yn(c[3])}" + ("" if c[3] == cl[3] else f", cluster {yn(cl[3])}"),
            f"{ae[0]:+.3f}, 2.5% {ae[1][0]:+.3f} (cluster {ae[2][0]:+.3f}): {yn(c[4])}" + ("" if c[4] == cl[4] else f", cluster {yn(cl[4])}"),
            word,
        ])
    gate.table("| Cut | (1) `RR` % | (2) `PL` % | (3) Removed % | (4) `DP` % vs base | (5) AUC - `ef` | Verdict |", vrow)
    print(f"**Verdict**: `{WINNER}` " + (f"passes, best cut q{best}" + (" (fragile on the cluster bootstrap)" if verdict[best]["fragile"] else "") if best is not None
                                         else "does not pass (no usable cut at q5 or q10)") + ".\n")
    print("The \"close\" conditions: " + "; ".join(
        f"q{q}: `RR` point {verdict[q]['rr']:.1f} <= {RR_BAR:.0f}% with (2) and (3) holding: {yn(close_rr[q])}" for q in ELIGIBLE)
        + f"; AUC - `ef` point {ae[0]:+.3f} above zero with the stratified interval [{ae[1][0]:+.3f}, {ae[1][1]:+.3f}] spanning zero: {yn(close_auc)}.\n")
    print(f"**Follow-up rule**: {FOLLOW[follow]}.\n")

    print("## Exploration and confirmation\n")
    print("The winner's numbers on the two exploration rounds (Step 3, its out-of-fold scores, each round's own `1/π`) beside the fresh read.\n")
    fr = {q: res[(WINNER, q)] for q in ELIGIBLE}
    gate.table(
        "| Read | `LR/T` AUC | `ef` AUC | AUC - `ef` | `LR/T` `RR` % q5 | `LR/T` `RR` % q10 | `ef` `RR` % q5 / q10 | Base rate % |",
        [[f"Round {rd}" + (" (exploration, primary)" if rd == 2 else " (exploration, replication)"), x["auc"], x["ef"], x["diff"], x["rr"][0], x["rr"][1],
          " / ".join(x["ef_rr"]), x["base"]] for rd, x in EXPLORATION.items()]
        + [["Fresh (confirmation)", fcr.auc_s(auc[(WINNER, "all")]), fcr.auc_s(auc[("ef", "all")]), fcr.eff(*ae, 3), fcr.rr_s(fr[5]), fcr.rr_s(fr[10]),
            " / ".join(f"{res[('ef', q)]['point']['RR']:.1f}" for q in ELIGIBLE), f"{base:.2f}"]],
    )
    print(f"AUC - `ef` from round 2 / round 1 to fresh: +0.027 / +0.018 to {ae[0]:+.3f} (a change of {ae[0] - 0.027:+.3f} / {ae[0] - 0.018:+.3f}).\n")

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (fresh dump, `scores_fresh.tsv`)", f"{load_time:.0f}"), ("Cuts, checks, metrics and bootstraps", f"{compute_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(out_path, "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    print("freshread: done", file=sys.stderr, flush=True)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:3])
