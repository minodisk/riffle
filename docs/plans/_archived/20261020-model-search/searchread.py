"""Step 3: the exploration read of the 69 candidates of modelsearch.md on both
tasks, the reading aids, and the winners chosen by the pre-registered
selection rules.

Usage: python -I -B searchread.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <discard-labels-dir> <strict-labels-dir> <retarget-dir> <out-dir> [jobs] [--check-all]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv` (the stored `W0`, `F0+ctx`); <reject-filter-eval-dir>
`scores_reject.tsv` (the stored `G0`); <discard-labels-dir> `labels.tsv` and
`strata.tsv`; <strict-labels-dir> the strict sample's `labels.tsv` and
`manifest.tsv`; <retarget-dir> `scores_retarget.tsv` (the stored `D0`);
<out-dir> Step 1's `lockbox.tsv` and Step 2's `scores_pick.tsv` and
`scores_discard.tsv`. Prints the tables (Markdown) and saves them as
`searchread.out.md` in <out-dir>.

The pick task reads the pick fit set on the qualifying exploration folders
only; no lockbox folder's picks are read against a candidate. `--check-all`
runs only the pick task's code path on all 27 folders for the references
(`W0`, `F0+ctx`, `G0`; no candidate is loaded) and saves
`searchread.check-all.out.md`; the main run prints the same check.
"""

import math
import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261018-discard-labels"))
import discardfilter as df  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261019-discard-retarget"))
import retargetread  # noqa: E402

sys.path.insert(0, HERE)
import searchfit as sf  # noqa: E402

rf, gate, folder_gate, fw = bf.rf, bf.gate, bf.folder_gate, bf.fw

QS = df.QS
PICK_QS = (5, 10, 20)
SELECT_Q = 10
REFS = ["G0", "W0", "F0+ctx", "D0", fw.EYE, fw.RANDOM]
BANDS = retargetread.BANDS
DIMS = ["T+C", "All@4", "All@8", "All@16", "All@32"]
EMB = ["Emb@8", "Emb@32"]
CHECK_ALL = [
    ("`W0` mean frame AUC (SD) / pooled", "W0", "frame", "0.666 (0.095) / 0.718"),
    ("`W0` mean R-precision % (SD) / pooled", "W0", "rp", "28.6 (10.8) / 34.4"),
    ("`W0` mean lift at the top tenth (SD)", "W0", "lift", "2.32 (0.82)"),
    ("`F0+ctx` mean frame AUC (SD) / pooled", "F0+ctx", "frame", "0.666 (0.102) / 0.727"),
    ("`F0+ctx` mean R-precision % (SD) / pooled", "F0+ctx", "rp", "27.0 (11.5) / 33.5"),
    ("`F0+ctx` mean lift at the top tenth (SD)", "F0+ctx", "lift", "2.08 (0.98)"),
    ("`G0` mean frame AUC", "G0", "frame1", "0.670"),
]


def nm(k):
    return sf.cell(k) if "`" not in k else k.replace("|", "\\|")


def pick_folder(sc, pk, t):
    """One folder's pick-task metrics: `P@q` / `R@q` on the top `floor(q * n / 100)` (ties by the seed-0 draw), R-precision, frame AUC, lift at the top tenth."""
    n = len(sc)
    order = np.lexsort((-t, -sc))
    out = {}
    for q in PICK_QS:
        top = order[: q * n // 100]
        out[f"P@{q}"] = 100 * float(pk[top].mean())
        out[f"R@{q}"] = 100 * float(pk[top].sum() / pk.sum())
    out["rp"] = 100 * fw.r_precision(sc, pk, t)
    out["frame"] = folder_gate.auc(sc, pk)
    out["lift"] = float(pk[order[: math.ceil(n / 10)]].mean() / pk.mean())
    return out


def pick_task(d, tb, scores, read):
    """Per score, the per-folder metrics on the folders `read` (no other folder's picks are touched)."""
    out = {}
    for k, sc in scores.items():
        pf = {}
        for fi in read:
            m = d["folder_of"] == fi
            if np.isnan(sc[m]).any():
                raise SystemExit(f"{k}: NaN in a read folder")
            pf[fi] = pick_folder(sc[m], d["pick"][m], tb[m])
        out[k] = pf
    return out


def summ(pf, key, sizes):
    v = {fi: r[key] for fi, r in pf.items()}
    m, s, _ = folder_gate.mean_sd(v)
    return m, s, fw.pooled(pf, key, list(pf), sizes)


def pdiff(pa, pb, key):
    diff = [pa[fi][key] - pb[fi][key] for fi in sorted(pa)]
    mean, lo, hi = fw.boot_folders(diff)
    _, p, _ = folder_gate.signed_rank(diff)
    return mean, lo, hi, p, sum(1 for x in diff if x >= 0), len(diff)


def check_all(d, tb, stored, stored_rej):
    """The pick task's code path on every qualifying folder of the 27, references only."""
    folder_of, pick = d["folder_of"], d["pick"]
    nf = len(d["folders"])
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    qual = [fi for fi in range(nf) if sizes[fi] >= fw.MIN_FRAMES and int((pick & (folder_of == fi)).sum()) >= fw.MIN_PICKS]
    pt = pick_task(d, tb, {"W0": stored["W0"], "F0+ctx": stored["F0+ctx"], "G0": stored_rej["G0"]}, qual)
    rows = [("Qualifying folders of the 27 (>= 50 frames, >= 5 picks)", "25", str(len(qual)))]
    for lab, k, key, exp in CHECK_ALL:
        m, s, p = summ(pt[k], "frame" if key == "frame1" else key, sizes)
        if key == "frame1":
            v = f"{m:.3f}"
        elif key == "frame":
            v = f"{m:.3f} ({s:.3f}) / {p:.3f}"
        elif key == "rp":
            v = f"{m:.1f} ({s:.1f}) / {p:.1f}"
        else:
            v = f"{m:.2f} ({s:.2f})"
        rows.append((lab, exp, v))
    return [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in rows]


def auc_draws(dis, sc, w, cs, cc):
    return (
        retargetread.auc_of(dis, sc, w),
        retargetread.auc_of(dis, sc, np.ones(len(sc))),
        np.array([retargetread.auc_of(dis, sc, w * cs[b]) for b in range(bf.BOOT)]),
        np.array([retargetread.auc_of(dis, sc, w * cc[b]) for b in range(bf.BOOT)]),
    )


def main(dump, eval_dir, reject_dir, labels_dir, strict_dir, retarget_dir, out_dir, jobs, only_check):
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
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)

    if only_check:
        print("# Model-search read: the `--check-all` reference check\n")
        print("The pick task's code path run on every qualifying folder of the 27 for the stored references only; no candidate score is loaded.\n")
        ca = check_all(d, tb, stored, stored_rej)
        gate.table("| Check | Expected | This run | Result |", ca)
        print(f"Total {time.time() - start:.0f} s.\n")
        sys.stdout = tee.stream
        with open(os.path.join(out_dir, "searchread.check-all.out.md"), "w", encoding="utf-8", newline="") as out:
            out.write("".join(tee.parts))
        if not all(r[3] == "pass" for r in ca):
            raise SystemExit("a reference check failed")
        return

    stored_ret, _ = rf.load_scores(os.path.join(retarget_dir, "scores_retarget.tsv"), d)
    sp, _ = rf.load_scores(os.path.join(out_dir, "scores_pick.tsv"), d)
    sd, _ = rf.load_scores(os.path.join(out_dir, "scores_discard.tsv"), d)
    names = [c[0] for c in sf.candidates()] + sf.COMBOS
    assert list(sp) == names and list(sd) == names
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    ef_last = np.where(np.isnan(ef), -np.inf, ef)
    refs = {"G0": stored_rej["G0"], "W0": stored["W0"], "F0+ctx": stored["F0+ctx"], "D0": stored_ret["D0"], fw.EYE: ef_last, fw.RANDOM: tb}

    box_names = {r["folder"] for r in bf.read_tsv(os.path.join(out_dir, "lockbox.tsv")) if r["lockbox"] == "1"}
    box_f = sorted(folders.index(x) for x in box_names)
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    npicks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(nf)}
    expl_q = [fi for fi in range(nf) if fi not in box_f and sizes[fi] >= fw.MIN_FRAMES and npicks[fi] >= fw.MIN_PICKS]

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
    g0 = refs["G0"]
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

    graded = {**refs, **sd}
    res, removed_of = {}, {}
    for k, sc in graded.items():
        for q in QS:
            removed, out = bf.graded_cut(sc, d, tb, q)
            removed_of[(k, q)] = removed
            rem, ok = removed[idx], ~out[idx]
            cover = ~out
            boot, clus = df.metrics(s, rem, ok, cs), df.metrics(s, rem, ok, cc)
            res[(k, q)] = {
                "rem": rem, "ok": ok, "point": df.metrics(s, rem, ok), "unw": df.unweighted(s, rem, ok), "boot": boot, "cboot": clus,
                "ci": {m: bf.quant(v) for m, v in boot.items()}, "cci": {m: bf.quant(v) for m, v in clus.items()},
                "pl": 100 * (removed & pick & cover).sum() / (pick & cover).sum(),
                "pl_pop": 100 * (removed & pick & cover & pop).sum() / (pick & cover & pop).sum(),
                "removed_pop": 100 * (removed & cover & pop).sum() / (cover & pop).sum(),
                "removed_dump": 100 * (removed & cover).sum() / cover.sum(),
                "n_rem": int((rem & ok).sum()), "n_dis": int((rem & ok & dis).sum()), "out": int((~ok).sum()), "frames_out": int(out.sum()),
            }
    cut_time = time.time() - t0

    t1 = time.time()
    auc_keys = list(graded)
    oks = {k: res[(k, 5)]["ok"] for k in auc_keys}
    draws = Parallel(n_jobs=jobs)(
        delayed(auc_draws)(dis[oks[k]], bf.finite_by_folder(graded[k], d)[idx][oks[k]], w[oks[k]], cs[:, oks[k]], cc[:, oks[k]]) for k in auc_keys
    )
    auc = {k: dict(zip(("point", "unw", "boot", "cboot"), v)) for k, v in zip(auc_keys, draws)}
    auc_time = time.time() - t1
    print(f"AUCs: {auc_time:.0f} s", file=sys.stderr, flush=True)

    t2 = time.time()
    pk_scores = {**refs, **sp}
    pt = pick_task(d, tb, pk_scores, expl_q)
    read_box = sorted(set(expl_q) & set(box_f))
    ca = check_all(d, tb, stored, stored_rej)
    pick_time = time.time() - t2

    def dp_vs(k, base, q, m="DP"):
        a, b = res[(k, q)], res[(base, q)]
        return a["point"][m] - b["point"][m], bf.quant(a["boot"][m] - b["boot"][m]), bf.quant(a["cboot"][m] - b["cboot"][m])

    def auc_vs(k, base):
        a, b = auc[k], auc[base]
        return a["point"] - b["point"], bf.quant(a["boot"] - b["boot"]), bf.quant(a["cboot"] - b["cboot"])

    def eff(p, lohi, clohi, digits=1):
        return f"{p:+.{digits}f} [{lohi[0]:+.{digits}f}, {lohi[1]:+.{digits}f}] ([{clohi[0]:+.{digits}f}, {clohi[1]:+.{digits}f}])"

    def pk_vs(k, base, key="P@10", digits=1):
        m, lo, hi, p, above, nn = pdiff(pt[k], pt[base], key)
        return f"{m:+.{digits}f} [{lo:+.{digits}f}, {hi:+.{digits}f}]", f"{p:.4f}", f"{above} of {nn}"

    def ivs(k, q, m):
        r = res[(k, q)]
        return bf.iv(r["point"][m], *r["ci"][m])

    def aucs_iv(k):
        return bf.iv(auc[k]["point"], *bf.quant(auc[k]["boot"]), 3)

    print("# Model-search read: the exploration of both tasks and the winners\n")
    print(
        "Every candidate score is read from Step 2's files: the discard task on `scores_discard.tsv` (all 27 folders, the 600 labels), the pick "
        f"task on `scores_pick.tsv` on the {len(expl_q)} qualifying exploration folders only. No lockbox folder's picks are read against a candidate. "
        "The references are the stored scores (`G0`, `W0`, `F0+ctx`, `D0`), `eye_focus` (missing last) and random (seed 0). This read states facts; "
        "it applies no bar.\n"
    )

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{nf} / {n} / {missing}"),
            ("Strict-sample frames excluded / population", f"{int(seen.sum())} / {int(pop.sum())}"),
            ("Sampled frames (`labels.tsv`) / keep / discard", f"{ns} / {int((~dis).sum())} / {int(dis.sum())}"),
            ("Strict sample (`labels.tsv` there) / strict bad", f"{len(t['idx'])} / {int(t['bad'].sum())}"),
            ("Lockbox folders / exploration folders / qualifying exploration folders (the pick task)", f"{len(box_f)} / {nf - len(box_f)} / {len(expl_q)}"),
            ("Pick task frames / picks / chance (mean pick rate of the read folders) %",
             f"{sum(sizes[fi] for fi in expl_q)} / {sum(npicks[fi] for fi in expl_q)} / {np.mean([100 * npicks[fi] / sizes[fi] for fi in expl_q]):.1f}"),
            ("Candidates / references", f"{len(names)} / {len(REFS)}"),
        ],
    )

    total_w = w.sum()
    base = 100 * (w * dis).sum() / total_w
    base_ci = bf.quant(bf.ratio(cs, w * dis, w))
    base_cci = bf.quant(bf.ratio(cc, w * dis, w))
    print("## Discardable base rate\n")
    gate.table(
        "| Reading | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |",
        [("Discardable base rate", f"{base:.2f}", f"[{base_ci[0]:.2f}, {base_ci[1]:.2f}]", f"[{base_cci[0]:.2f}, {base_cci[1]:.2f}]", f"{100 * dis.mean():.2f}")],
    )

    print("## Reference checks\n")
    print(
        "The discard task's code path on the stored `G0`, `eye_focus` and random must reproduce discard.md's Results (as retarget.md did); the pick "
        "task's code path run on all 27 folders for the references only (the `--check-all` path, which reads no candidate) must reproduce "
        "folderwide.out.md's `W0` / `F0+ctx` and rejectfilter.out.md's `G0`. The guard rows check that the pick task read no lockbox folder.\n"
    )
    g0r = (res[("G0", 5)]["point"]["DP"] - res[(fw.RANDOM, 5)]["point"]["DP"], res[("G0", 5)]["boot"]["DP"] - res[(fw.RANDOM, 5)]["boot"]["DP"])
    pick_finite = all(np.isfinite(sp[k][~np.isin(folder_of, box_f)]).all() and np.isnan(sp[k][np.isin(folder_of, box_f)]).all() for k in names)
    frames_out = sum(res[(k, q)]["frames_out"] for k in names for q in QS)
    checks = [
        ("`strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump", f"0 of {len(strata_file)}", f"{strata_diff} of {len(strata_file)}"),
        ("Sampled frames whose recomputed `G0` band / `π` differ from `labels.tsv`", "0 / 0", f"{band_diff} / {pi_diff}"),
        ("Sampled frames in the strict sample", "0", str(in_strict)),
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
        ("`D0` AUC (retarget.md)", "0.761 [0.697, 0.827]", aucs_iv("D0")),
        ("Frames left out of a cut, over every candidate and `q`", "0", str(frames_out)),
        ("Pick fit set: every candidate finite on the exploration frames and `nan` on the lockbox frames", "yes", "yes" if pick_finite else "no"),
        ("Lockbox folders among the folders the pick task reads", "0", str(len(read_box))),
    ]
    checks = [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in checks] + [(f"`--check-all`: {c}", e, v, r) for c, e, v, r in ca]
    gate.table("| Check | Expected | This run | Result |", checks)

    print("## Pick task: the top of the qualifying exploration folders\n")
    print(
        f"Per folder, the top `floor(q * n / 100)` frames by score (ties by the seed-0 draw); `P@q` the pick share among them, `R@q` the share of "
        f"the folder's picks among them, R-precision and the frame AUC. The mean over the {len(expl_q)} qualifying exploration folders with the "
        "per-folder SD in brackets; pooled: the frame-weighted mean of the per-folder values (`folderwide.pooled`). Candidates from the pick fit set; "
        "references stored (trained leave one folder out on all 27). This task optimizes the top.\n"
    )
    prow = []
    for k in REFS + names:
        r = [nm(k)]
        for key in ("P@5", "P@10", "P@20", "R@10", "rp"):
            m, sdv, _ = summ(pt[k], key, sizes)
            r.append(fw.pm(m, sdv, digits=1))
        m, sdv, _ = summ(pt[k], "frame", sizes)
        r.append(fw.pm(m, sdv))
        r += [f"{summ(pt[k], 'P@10', sizes)[2]:.1f}", f"{summ(pt[k], 'R@10', sizes)[2]:.1f}", f"{summ(pt[k], 'frame', sizes)[2]:.3f}"]
        prow.append(r)
    gate.table("| Score | `P@5` % | `P@10` % | `P@20` % | `R@10` % | R-precision % | Frame AUC | Pooled `P@10` | Pooled `R@10` | Pooled frame AUC |", prow)

    print("### Paired per-folder differences against the references\n")
    print(
        f"Over the {len(expl_q)} folders: the mean paired difference with the 1,000-draw folder bootstrap 95% interval (seed 0), the one-sided "
        "Wilcoxon signed-rank p (zeros dropped), and the folders at or above the reference (`P@10`).\n"
    )
    drow = []
    for k in names:
        a, p, above = pk_vs(k, "W0")
        b, pb, _ = pk_vs(k, "G0")
        c, pc, _ = pk_vs(k, "W0", "frame", 3)
        drow.append([nm(k), a, p, above, b, pb, c, pc])
    gate.table("| Candidate | `P@10` - `W0`, points | p | Folders at or above `W0` | `P@10` - `G0`, points | p | Frame AUC - `W0` | p |", drow)

    def pick_key(k):
        return summ(pt[k], "P@10", sizes)[0], summ(pt[k], "frame", sizes)[0]

    pick_rank = sorted(names, key=lambda k: (-pick_key(k)[0], -pick_key(k)[1], names.index(k)))
    pick_win, pick_run = pick_rank[0], pick_rank[1]

    print("## Discard task: the bottom on the 600 labels\n")
    print(
        "Weighted (`1/π`) with the stratified bootstrap 95% interval (1,000 draws, seed 0, within each band and each `control` folder cell; one count "
        "matrix for every score) and the folder-cluster interval (1,000 draws, seed 0) beside it; unweighted in brackets. Sampled: sampled frames the "
        "cut removes / of them `discard`. `PL`: the dump's picks inside the cut (full dump; population after the slash); removed: the population's "
        "share (full dump's after the slash). Candidates from the discard fit set. Out: sampled frames in a folder the score leaves out (a NaN).\n"
    )
    mrow = []
    for k in REFS + names:
        for q in QS:
            r = res[(k, q)]
            mrow.append([
                nm(k), f"q{q}", r["out"], f"{r['n_rem']} / {r['n_dis']}",
                f"{ivs(k, q, 'DP')} ({bf.f1(r['unw']['DP'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["DP"]),
                f"{ivs(k, q, 'RR')} ({bf.f1(r['unw']['RR'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["RR"]),
                f"{r['pl']:.2f} / {r['pl_pop']:.2f}", ivs(k, q, "DR"), ivs(k, q, "DO"), f"{r['removed_pop']:.2f} / {r['removed_dump']:.2f}",
            ])
    gate.table("| Score | Cut | Out | Sampled removed / discard | `DP` % | Cluster | `RR` % | Cluster | `PL` % | `DR` % | `DO` % | Removed % |", mrow)

    print("### Threshold-free: weighted AUC of `discard` vs `keep`\n")
    print(
        "`roc_auc_score(discard, -score, sample_weight=1/π)` on the sampled frames the score covers, a missing value at the folder's lowest finite "
        "score minus 1; stratified 95% interval, cluster interval, unweighted; the paired differences against `G0` and `eye_focus` on the same "
        "draws (stratified, cluster in parentheses).\n"
    )
    arow = []
    for k in REFS + names:
        r = [nm(k), aucs_iv(k), "[{:.3f}, {:.3f}]".format(*bf.quant(auc[k]["cboot"])), f"{auc[k]['unw']:.3f}"]
        for b in ("G0", fw.EYE):
            r.append("-" if k == b or k == "W0" else eff(*auc_vs(k, b), 3))
        arow.append(r)
    gate.table("| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `G0` | AUC - `eye_focus` |", arow)

    def lf_pass(k, q):
        r = res[(k, q)]
        return r["pl"] <= df.PL_BAR and r["removed_pop"] >= df.REMOVED_BAR

    def disc_rank(q):
        ok = [k for k in names if lf_pass(k, q)]
        return sorted(ok, key=lambda k: (-res[(k, q)]["point"]["DP"], -auc[k]["point"], names.index(k)))

    rank10, rank5 = disc_rank(SELECT_Q), disc_rank(5)
    sel_q = SELECT_Q if rank10 else 5
    sel = rank10 if rank10 else rank5
    disc_win = sel[0] if sel else None
    disc_run = sel[1] if len(sel) > 1 else None

    print("## Selection rules applied\n")
    print(
        f"**Discard**: among the candidates whose q{SELECT_Q} cut passes `PL` <= {df.PL_BAR:.0f}% (full dump) and removed >= {df.REMOVED_BAR}% of the "
        f"population ({len(rank10)} of {len(names)}), the highest weighted `DP@{SELECT_Q}`, ties by the weighted AUC; the q5 rank (among the {len(rank5)} "
        "q5 passers, by `DP@5`) beside it as a reading.\n"
    )
    srow = []
    for i, k in enumerate(rank10):
        r10, r5 = res[(k, 10)], res[(k, 5)]
        srow.append([i + 1, nm(k), ivs(k, 10, "DP"), ivs(k, 10, "RR"), f"{r10['pl']:.2f}", f"{r10['removed_pop']:.2f}", aucs_iv(k),
                     ivs(k, 5, "DP"), f"{r5['pl']:.2f}", rank5.index(k) + 1 if k in rank5 else "-"])
    gate.table("| Rank | Candidate | `DP@10` % | `RR@10` % | `PL@10` % | Removed q10 % | AUC | `DP@5` % | `PL@5` % | q5 rank |", srow)
    print("The q5 ranking (a reading; it does not change the winner), the first 10:\n")
    gate.table(
        "| q5 rank | Candidate | `DP@5` % | `RR@5` % | `PL@5` % | q10 rank |",
        [[i + 1, nm(k), ivs(k, 5, "DP"), ivs(k, 5, "RR"), f"{res[(k, 5)]['pl']:.2f}", rank10.index(k) + 1 if k in rank10 else "fails q10"] for i, k in enumerate(rank5[:10])],
    )
    print(f"Discard winner (q{sel_q}): {'none' if disc_win is None else nm(disc_win)}; runner-up: {'none' if disc_run is None else nm(disc_run)}.\n")

    print(f"**Pick**: among all {len(names)} candidates, the highest mean `P@10` over the {len(expl_q)} qualifying exploration folders, ties by the mean frame AUC. The first 10:\n")
    gate.table(
        "| Rank | Candidate | `P@10` % | `R@10` % | Frame AUC | `P@10` - `W0`, points | p | Folders at or above `W0` |",
        [[i + 1, nm(k), fw.pm(*summ(pt[k], "P@10", sizes)[:2], digits=1), fw.pm(*summ(pt[k], "R@10", sizes)[:2], digits=1), fw.pm(*summ(pt[k], "frame", sizes)[:2]), *pk_vs(k, "W0")]
         for i, k in enumerate(pick_rank[:10])],
    )
    print(f"Pick winner: {nm(pick_win)}; runner-up: {nm(pick_run)}.\n")

    print("## The winners beside the references\n")
    print("Discard task (discard fit set; stratified intervals):\n")
    wrow = []
    for k in [x for x in (disc_win, disc_run) if x] + ["G0", "W0", fw.EYE, fw.RANDOM, "D0"]:
        wrow.append([nm(k)] + [f"{ivs(k, q, 'DP')} / {res[(k, q)]['point']['RR']:.1f} / {res[(k, q)]['pl']:.2f}" for q in (5, 10)] + [aucs_iv(k)])
    gate.table("| Score | q5 `DP` / `RR` / `PL` % | q10 `DP` / `RR` / `PL` % | AUC |", wrow)
    if disc_win:
        gate.table(
            "| Paired difference (stratified, cluster in parentheses) | q5 | q10 | q20 |",
            [[f"{nm(disc_win)} - `G0`, `DP` points"] + [eff(*dp_vs(disc_win, "G0", q)) for q in QS],
             [f"{nm(disc_win)} - `eye_focus`, `DP` points"] + [eff(*dp_vs(disc_win, fw.EYE, q)) for q in QS],
             [f"{nm(disc_win)} - random, `DP` points"] + [eff(*dp_vs(disc_win, fw.RANDOM, q)) for q in QS],
             [f"{nm(disc_win)} - `G0`, AUC", eff(*auc_vs(disc_win, "G0"), 3), "", ""],
             [f"{nm(disc_win)} - `eye_focus`, AUC", eff(*auc_vs(disc_win, fw.EYE), 3), "", ""]],
        )
    print(f"Pick task ({len(expl_q)} qualifying exploration folders; pick fit set for the candidates):\n")
    gate.table(
        "| Score | `P@10` % | `R@10` % | Frame AUC | R-precision % | `P@10` - `W0`, points | p | Folders at or above `W0` |",
        [[nm(k), fw.pm(*summ(pt[k], "P@10", sizes)[:2], digits=1), fw.pm(*summ(pt[k], "R@10", sizes)[:2], digits=1), fw.pm(*summ(pt[k], "frame", sizes)[:2]),
          fw.pm(*summ(pt[k], "rp", sizes)[:2], digits=1), *(("-", "-", "-") if k == "W0" else pk_vs(k, "W0"))]
         for k in [pick_win, pick_run, "W0", "G0", "F0+ctx", fw.EYE, fw.RANDOM]],
    )

    def aid_row(k):
        p10 = pk_vs(k, "G0")
        return [nm(k), eff(*dp_vs(k, "G0", 10)), eff(*dp_vs(k, "G0", 5)), eff(*auc_vs(k, "G0"), 3), f"{p10[0]} (p {p10[1]})"]

    aid_head = "| Candidate | `DP@10` - `G0`, points | `DP@5` - `G0`, points | AUC - `G0` | `P@10` - `G0`, points (pick task) |"
    print("## Reading aids\n")
    print(
        "Paired differences against the stored `G0`: discard side (discard fit set) with the stratified interval and the cluster interval in "
        "parentheses; pick side (pick fit set, the qualifying exploration folders) with the folder bootstrap interval and the one-sided Wilcoxon p. "
        "Readings, not candidates' verdicts.\n"
    )
    print("### PCA-dimension curve per form and target\n")
    for form in sf.FORMS:
        for target in sf.TARGETS:
            feats = DIMS + ([] if form == "HGB-mono" else EMB)
            print(f"`{form}` / `{target}`:\n")
            gate.table(aid_head, [aid_row(f"{f} / {form} / {target}") for f in feats])
    print("### Form comparison at `All@32`\n")
    gate.table(aid_head, [aid_row(f"All@32 / {form} / {target}") for target in sf.TARGETS for form in sf.FORMS])
    print("### Target comparison per design\n")
    print("The discard target minus the pick target of the same design, paired as above (discard side stratified, cluster in parentheses; pick side folder bootstrap).\n")
    trow = []
    for feat in sf.FEATURES:
        for form in sf.FORMS:
            if form == "HGB-mono" and not sf.FEATURES[feat][0]:
                continue
            a, b = f"{feat} / {form} / discard", f"{feat} / {form} / pick"
            p10 = pk_vs(a, b)
            trow.append([f"`{feat} / {form}`", eff(*dp_vs(a, b, 10)), eff(*dp_vs(a, b, 5)), eff(*auc_vs(a, b), 3), f"{p10[0]} (p {p10[1]})"])
    gate.table("| Design | `DP@10`, points | `DP@5`, points | AUC | `P@10`, points (pick task) |", trow)
    print("### The combinations and the clip\n")
    print("The three combinations against `G0`, and the clipped-weight `All@32 / HGB / discard` against the stored unclipped `D0` (discard side only; `D0` is a reference fit on all 27 folders).\n")
    gate.table(aid_head, [aid_row(k) for k in sf.COMBOS])
    gate.table(
        "| Comparison | `DP@10`, points | `DP@5`, points | AUC |",
        [["`All@32 / HGB / discard` - `D0`", eff(*dp_vs("All@32 / HGB / discard", "D0", 10)), eff(*dp_vs("All@32 / HGB / discard", "D0", 5)), eff(*auc_vs("All@32 / HGB / discard", "D0"), 3)]],
    )

    print("### The two label-free combinations on the pick winner (a reading only)\n")
    if pick_win in sf.COMBOS:
        print(f"The pick winner {nm(pick_win)} is itself a combination; not applied.\n")
    else:
        pp, pr = sf.combine(d, sp[pick_win], ef)
        dp_, dr_ = sf.combine(d, sd[pick_win], ef)
        extra_p = {f"{pick_win} ⊕ ef": pp, f"{pick_win} | ef-rule": pr}
        extra_d = {f"{pick_win} ⊕ ef": dp_, f"{pick_win} | ef-rule": dr_}
        ept = pick_task(d, tb, extra_p, expl_q)
        crow = []
        for k in [pick_win] + list(extra_p):
            pf = pt[k] if k == pick_win else ept[k]
            if k == pick_win:
                rq = {q: res[(k, q)] for q in (5, 10)}
                a = auc[k]["point"]
            else:
                rq = {}
                for q in (5, 10):
                    removed, out = bf.graded_cut(extra_d[k], d, tb, q)
                    rem, ok = removed[idx], ~out[idx]
                    rq[q] = {"point": df.metrics(s, rem, ok), "pl": 100 * (removed & pick).sum() / pick.sum()}
                a = retargetread.auc_of(dis, bf.finite_by_folder(extra_d[k], d)[idx], w)
            crow.append([nm(k), fw.pm(*summ(pf, "P@10", sizes)[:2], digits=1), fw.pm(*summ(pf, "R@10", sizes)[:2], digits=1), fw.pm(*summ(pf, "frame", sizes)[:2])]
                        + [f"{rq[q]['point']['DP']:.1f} / {rq[q]['point']['RR']:.1f} / {rq[q]['pl']:.2f}" for q in (5, 10)] + [f"{a:.3f}"])
        gate.table("| Score | `P@10` % | `R@10` % | Frame AUC | q5 `DP` / `RR` / `PL` % | q10 `DP` / `RR` / `PL` % | AUC |", crow)

    print("## Coherence check: the discard winner's own bands\n")
    print(
        "The score's own per-folder bands on the full dump; the weighted `discard` rate on the 600 labels (stratified 95%) beside the weighted "
        "strict-bad rate on the strict sample with its own `π` and strata. Expected: strict-bad below `discard` in each band (a check, not a bar). "
        "`G0`'s rows for comparison.\n"
    )
    hrows = []
    for k in [x for x in (disc_win,) if x] + ["G0"]:
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
                nm(k), lab, f"{int(m.sum())} / {int((dis & m).sum())}", bf.iv(rate, *bf.quant(bf.ratio(cs, w * dis * m, w * m))),
                f"{int(tm.sum())} / {int((t['bad'] & tm).sum())}", bf.iv(srate, *bf.quant(bf.ratio(ct, t["w"] * t["bad"] * tm, t["w"] * tm))),
                "yes" if srate < rate else "no",
            ])
    gate.table("| Score | Band | Sampled / discard | Weighted `discard` rate % | Strict sample frames / strict bad | Weighted strict-bad rate % | Strict below discard |", hrows)

    print("## Wall time\n")
    gate.table(
        "| Stage | Seconds |",
        [("Load (dump, stored and Step 2 scores, labels, strata, strict labels)", f"{load_time:.0f}"), ("Cuts, metrics and bootstraps", f"{cut_time:.0f}"),
         (f"AUC bootstraps ({len(auc_keys)} scores, {jobs} jobs)", f"{auc_time:.0f}"), ("Pick task and the `--check-all` path", f"{pick_time:.0f}")],
    )
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "searchread.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    if not all(r[3] == "pass" for r in checks):
        raise SystemExit("a reference check failed")


if __name__ == "__main__":
    argv = [a for a in sys.argv[1:] if a != "--check-all"]
    if len(argv) not in (7, 8):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*argv[:7], int(argv[7]) if len(argv) > 7 else 8, "--check-all" in sys.argv[1:])
