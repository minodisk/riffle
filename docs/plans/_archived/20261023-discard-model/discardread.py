"""Step 3: the exploration read of the five discard candidates of discardmodel.md's
"Pre-registration of the exploration" on both rounds of blind "OK to discard?"
labels, the winner by the pre-registered selection rule (as amended
2026-10-24: q5 or q10) and the stop rule's verdict.

Usage: python -I -B discardread.py <app-dump-dir> <face-choice-dir> <reject-filter-eval-dir> <strict-labels-dir> <discard-labels-dir> <model-search-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend` (`badfilter.load` is not
called); <face-choice-dir> holds the per-face dump `faces\\<folder>.tsv` (the
`ef-fill` and `af` reproduction checks); <reject-filter-eval-dir>
`scores_reject.tsv` (`G0`, round 1's bands and round 2's `g0only`);
<strict-labels-dir> the strict sample's `manifest.tsv` (its `labels.tsv` is
never opened); <discard-labels-dir> round 1 and <model-search-dir> round 2
(`manifest.tsv`, `strata.tsv`, `labels.tsv`, and round 2's
`scores_discard.tsv` for its `G0 ⊕ ef` bands); <out-dir> Step 2's data folder
with `scores_model.tsv`. Prints the tables (Markdown) and saves them as
`discardread.out.md` in <out-dir>; refuses to run when that file exists, so
the read happens once. Every value is computed before anything is printed;
the label-free checks run before any `labels.tsv` is opened, and a failure
stops the read there. Nothing is written on a failed check.
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

rf, gate, folder_gate = bf.rf, bf.gate, bf.folder_gate

QS = df.QS
CANDS = ["ef-fill", "LR/S", "LR/T", "HGB-mono/T", "HGB/T"]
REFS = ["ef", "random"]
SCORES = REFS + CANDS
ROUNDS = fcr.ROUNDS
EXPECTED = fcr.EXPECTED
LABELS = {1: (211, 389), 2: (248, 352)}
# Step 2's label-free readings (discardfit.out.md): `PL` % at q5 / q10 / q20 on the full dump.
STEP2_PL = {
    "ef": ("4.44", "9.14", "15.59"),
    "random": ("4.71", "9.48", "19.26"),
    "ef-fill": ("3.91", "6.99", "12.83"),
    "LR/S": ("2.99", "6.72", "14.03"),
    "LR/T": ("2.94", "6.54", "13.78"),
    "HGB-mono/T": ("2.47", "6.00", "13.42"),
    "HGB/T": ("3.17", "6.79", "14.19"),
}
STEP2_REMOVED = ("4.96", "9.96", "19.97")
PARTS = {"all": "all labeled frames", "miss": "the missing-face subset (no `eye_focus` today)", "has": "its complement (`eye_focus` today)"}


def nm(k):
    return f"`{k}`" + (" (reference)" if k in REFS else "")


def yn(ok):
    return "yes" if ok else "no"


def pct(num, den):
    return f"{100 * num / den:.1f}" if den else "-"


def main(dump, face_dir, reject_dir, strict_dir, r1_dir, r2_dir, out_dir):
    out_path = os.path.join(out_dir, "discardread.out.md")
    if os.path.exists(out_path):
        raise SystemExit(f"{out_path} exists: the exploration is read once")
    start = time.time()
    dirs = {1: r1_dir, 2: r2_dir}
    roots = {k: os.path.normcase(os.path.abspath(p)) for k, p in (("reject-filter-eval", reject_dir), ("strict-labels", strict_dir), ("round-1", r1_dir),
                                                                  ("round-2", r2_dir), ("out", out_dir))}
    opened = {k: [] for k in roots}

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            for k, r in roots.items():
                if p.startswith(r + os.sep):
                    opened[k].append(os.path.relpath(p, r))

    sys.addaudithook(audit)
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee

    def fail(tables):
        print("# Discard model: the exploration read\n")
        for head, rows in tables:
            gate.table(head, rows)
        sys.stdout = tee.stream
        raise SystemExit("a check failed; nothing written")

    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n = len(rows)
    tb = np.random.default_rng(0).random(n)
    pick = d["pick"]
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    no_ef = np.isnan(ef)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    sd, _ = rf.load_scores(os.path.join(r2_dir, "scores_discard.tsv"), d)
    sm, sm_rows = rf.load_scores(os.path.join(out_dir, "scores_model.tsv"), d)

    faces_of, extra = fcr.load_faces(os.path.join(face_dir, "faces"), index, n)
    no_faces = sum(fr is None for fr in faces_of)
    if no_faces:
        raise SystemExit(f"{no_faces} dump frames have no row in the face dump")
    af, af_whole = np.full(n, np.nan), np.full(n, np.nan)
    for i, fr in enumerate(faces_of):
        _, vals, _, _ = fcr.choose(fr)
        af[i], af_whole[i] = vals["af"], vals["af-whole"]
    ef_fill = np.where(no_ef, af_whole, ef)

    scores = {"ef": np.where(no_ef, -np.inf, ef), "random": tb}
    for k in CANDS:
        scores[k] = np.where(np.isnan(sm[k]), -np.inf, sm[k])
    finite = {k: bf.finite_by_folder(sc, d) for k, sc in scores.items()}
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    # Label-free: cuts, populations, bands and the reproduction checks.
    t0 = time.time()
    cuts = {(k, q): bf.graded_cut(sc, d, tb, q) for k, sc in scores.items() for q in QS}
    strict_seen = np.zeros(n, dtype=bool)
    for r in bf.read_tsv(os.path.join(strict_dir, "manifest.tsv")):
        strict_seen[index[(r["folder"], r["file"])]] = True
    man = {rd: bf.read_tsv(os.path.join(dirs[rd], "manifest.tsv")) for rd in ROUNDS}
    strata_rows = {rd: bf.read_tsv(os.path.join(dirs[rd], "strata.tsv")) for rd in ROUNDS}
    ms = {rd: fcr.sample_of(man[rd], index, folder_of, False) for rd in ROUNDS}
    seen = {1: strict_seen.copy(), 2: strict_seen.copy()}
    seen[2][ms[1]["idx"]] = True
    pop = {rd: ~seen[rd] for rd in ROUNDS}

    cg = {q: bf.graded_cut(stored_rej["G0"], d, tb, q)[0] for q in QS}
    cw = {q: bf.graded_cut(sd["G0 ⊕ ef"], d, tb, q)[0] for q in QS}
    band = {1: np.full(n, "control", dtype=object), 2: np.full(n, "control", dtype=object)}
    band[1][cg[20]] = "b20"
    band[1][cg[10]] = "b10"
    band[1][cg[5]] = "b05"
    band[2][cg[20] & ~cw[20]] = "g0only"
    band[2][cw[20]] = "b20"
    band[2][cw[10]] = "b10"
    band[2][cw[5]] = "b05"

    res_pl = {(k, q): 100 * (cuts[(k, q)][0] & pick).sum() / pick.sum() for k in SCORES for q in QS}
    res_rem_dump = {(k, q): 100 * cuts[(k, q)][0].sum() / n for k in SCORES for q in QS}
    res_removed = {(rd, k, q): 100 * (cuts[(k, q)][0] & pop[rd]).sum() / pop[rd].sum() for rd in ROUNDS for k in SCORES for q in QS}
    lf_pass = {(k, q): res_pl[(k, q)] <= df.PL_BAR and res_rem_dump[(k, q)] >= df.REMOVED_BAR for k in SCORES for q in QS}
    eligible = {k: lf_pass[(k, 5)] or lf_pass[(k, 10)] for k in CANDS}
    # Where each cut sits, label-free: the share of its removed frames without `eye_focus` today, and the share of its picks among them.
    cut_miss = {(k, q): (int((cuts[(k, q)][0] & no_ef).sum()), int(cuts[(k, q)][0].sum()), int((cuts[(k, q)][0] & no_ef & pick).sum()),
                         int((cuts[(k, q)][0] & pick).sum())) for k in SCORES for q in QS}

    dump_close = (np.isnan(ef) == np.isnan(af)) & (np.isnan(ef) | (np.abs(ef - af) <= 1e-9))
    fill_same = (np.isnan(ef_fill) == np.isnan(sm["ef-fill"])) & (np.isnan(ef_fill) | (ef_fill == sm["ef-fill"]))
    learned_ok = all(np.isfinite(sm[k]).all() for k in CANDS[1:])
    checks_a = [
        ("Dump frames (with a vector)", "32459", str(n)),
        ("Dump frames without a row in the face dump", "0", str(no_faces)),
        ("`scores_model.tsv` rows / dump frames", f"{n} / {n}", f"{sm_rows} / {n}"),
        ("`af` (face dump) vs the dump's stored `eye_focus`: frames differing (> 1e-9, or missing on one side only)", f"0 of {n}", f"{int((~dump_close).sum())} of {n}"),
        ("`ef-fill` of `scores_model.tsv` vs its recomputation from the face dump: frames differing", f"0 of {n}", f"{int((~fill_same).sum())} of {n}"),
        ("Every learned candidate finite on every dump frame", "yes", yn(learned_ok)),
        ("Frames left out of a cut, over the seven scores and `q`", "0", str(sum(int(cuts[c][1].sum()) for c in cuts))),
    ]
    for k in SCORES:
        checks_a.append((f"{nm(k)} `PL` % / removed % (full dump) q5 / q10 / q20, as Step 2", " / ".join(STEP2_PL[k]) + " ; " + " / ".join(STEP2_REMOVED),
                         " / ".join(f"{res_pl[(k, q)]:.2f}" for q in QS) + " ; " + " / ".join(f"{res_rem_dump[(k, q)]:.2f}" for q in QS)))
    for rd in ROUNDS:
        m = ms[rd]
        mv = np.array([float(r["eye_focus"]) for r in man[rd]])
        ev = ef[m["idx"]]
        bad_rep = ~((np.isnan(mv) == np.isnan(ev)) & (np.isnan(mv) | np.isnan(ev) | (np.abs(mv - ev) <= 1e-6)))
        sdiff, bdiff, pdiff = fcr.strata_check(d, pop[rd], band[rd], strata_rows[rd], m)
        checks_a += [
            (f"Round {rd}: stored `eye_focus` vs `manifest.tsv`'s: frames differing (> 1e-6, or missing on one side only)", f"0 of {len(m['idx'])}", f"{int(bad_rep.sum())} of {len(m['idx'])}"),
            (f"Round {rd}: earlier-labeled frames excluded / population", f"{ROUNDS[rd]['earlier']} / {ROUNDS[rd]['population']}", f"{int(seen[rd].sum())} / {int(pop[rd].sum())}"),
            (f"Round {rd}: sampled frames in an earlier sample", "0", str(int(seen[rd][m["idx"]].sum()))),
            (f"Round {rd}: `strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump", f"0 of {len(strata_rows[rd])}", f"{sdiff} of {len(strata_rows[rd])}"),
            (f"Round {rd}: `manifest.tsv` frames whose recomputed band / `π` differ", "0 / 0", f"{bdiff} / {pdiff}"),
            (f"Round {rd}: sum of `1/π` over the sampled frames", f"{ROUNDS[rd]['population']}.0", f"{m['w'].sum():.1f}"),
            (f"Round {rd}: `ef` removed % of the round's population q5 / q10 / q20", " / ".join(EXPECTED[rd]["removed"]), " / ".join(f"{res_removed[(rd, 'ef', q)]:.2f}" for q in QS)),
        ]
    checks_a = [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in checks_a]
    early = {k: sorted({p for p in v if os.path.basename(p) == "labels.tsv"}) for k, v in opened.items()}
    checks_a.append(("`labels.tsv` files opened before these checks", "none", ", ".join(f"{k}: `{p}`" for k, ps in early.items() for p in ps) or "none",
                     "pass" if not any(early.values()) else "FAIL"))
    head_a = "| Check (label-free, before any `labels.tsv` is opened) | Expected | This run | Result |"
    if not all(r[3] == "pass" for r in checks_a):
        fail([(head_a, checks_a)])

    # The labels: each round's own sample, strata and bootstraps.
    lab = {rd: bf.read_tsv(os.path.join(dirs[rd], "labels.tsv")) for rd in ROUNDS}
    S = {}
    for rd in ROUNDS:
        s = fcr.sample_of(lab[rd], index, folder_of, True)
        s["labels"] = {r["label"] for r in lab[rd]}
        by_name = {r["name"]: r for r in man[rd]}
        s["man_diff"] = sum(r["name"] not in by_name or r["band"] != by_name[r["name"]]["band"] or r["inclusion_prob"] != by_name[r["name"]]["inclusion_prob"]
                            or (r["folder"], r["file"]) != (by_name[r["name"]]["folder"], by_name[r["name"]]["file"]) for r in lab[rd])
        s["cs"] = df.counts(s, folders, ROUNDS[rd]["strata"])
        s["cc"] = bf.cluster_counts(s, folders)
        S[rd] = s

    res, auc = {}, {}
    for rd, s in S.items():
        idx, cs, cc = s["idx"], s["cs"], s["cc"]
        masks = {"all": np.ones(len(idx), dtype=bool), "miss": no_ef[idx], "has": ~no_ef[idx]}
        s["masks"] = masks
        for k in SCORES:
            for part, msk in masks.items():
                for q in QS:
                    res[(rd, k, q, part)] = fcr.cut_metrics(s, cuts[(k, q)][0][idx], msk, cs, cc)
                auc[(rd, k, part)] = fcr.auc_draws(s, finite[k][idx], msk, cs, cc)
        print(f"round {rd}: {time.time() - t0:.0f} s", file=sys.stderr, flush=True)

    checks_b, base = [], {}
    for rd, s in S.items():
        w, dis, cs, cc = s["w"], s["bad"], s["cs"], s["cc"]
        b = 100 * (w * dis).sum() / w.sum()
        bci, bcci = bf.quant(bf.ratio(cs, w * dis, w)), bf.quant(bf.ratio(cc, w * dis, w))
        base[rd] = (b, bci, bcci, 100 * dis.mean())
        e = EXPECTED[rd]
        a = auc[(rd, "ef", "all")]
        exp = e["auc"][0] + (f" ({e['auc'][1]})" if e["auc"][1] else "") + f", unweighted {e['auc'][2]}"
        got = fcr.auc_s(a) + (f" ({fcr.cauc_s(a)})" if e["auc"][1] else "") + f", unweighted {a['unw']:.3f}"
        checks_b += [
            (f"Round {rd}: `labels.tsv` labels within {{keep, discard}}", "yes", yn(s["labels"] <= {"keep", "discard"})),
            (f"Round {rd}: labels / keep / discard", f"600 / {LABELS[rd][0]} / {LABELS[rd][1]}", f"{len(s['idx'])} / {int((~s['bad']).sum())} / {int(s['bad'].sum())}"),
            (f"Round {rd}: `labels.tsv` rows whose name, file, band or `π` differ from `manifest.tsv`", "0", str(s["man_diff"])),
            (f"Round {rd}: discardable base rate % [stratified 95%] (cluster)", f"{e['base'][0]} ({e['base'][1]})",
             f"{b:.2f} [{bci[0]:.2f}, {bci[1]:.2f}] ([{bcci[0]:.2f}, {bcci[1]:.2f}])"),
            (f"Round {rd}: `ef` AUC [stratified 95%]" + (" (cluster)" if e["auc"][1] else ""), exp, got),
            (f"Round {rd}: `ef` `RR` % q5 / q10 / q20 [stratified 95%]", " / ".join(e["rr"]), " / ".join(fcr.rr_s(res[(rd, "ef", q, "all")]) for q in QS)),
        ]
    checks_b = [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in checks_b]
    allowed = {"reject-filter-eval": {"scores_reject.tsv"}, "strict-labels": {"manifest.tsv"}, "round-1": {"labels.tsv", "manifest.tsv", "strata.tsv"},
               "round-2": {"labels.tsv", "manifest.tsv", "strata.tsv", "scores_discard.tsv"}, "out": {"scores_model.tsv"}}
    for k, ps in opened.items():
        got = sorted(set(ps))
        checks_b.append((f"Files opened under the {k} directory", "within " + ", ".join(f"`{p}`" for p in sorted(allowed[k])), ", ".join(f"`{p}`" for p in got) or "none",
                         "pass" if set(got) <= allowed[k] else "FAIL"))
    head_b = "| Check (labels) | Expected | This run | Result |"
    if not all(r[3] == "pass" for r in checks_b):
        fail([(head_a, checks_a), (head_b, checks_b)])

    # The selection rule and the stop rule.
    da = {(rd, k, part): fcr.diff(auc[(rd, k, part)], auc[(rd, "ef", part)]) for rd in ROUNDS for k in SCORES if k != "ef" for part in PARTS}
    pool = [k for k in CANDS if eligible[k]]
    winner = max(pool, key=lambda k: (auc[(2, k, "all")]["point"], auc[(1, k, "all")]["point"], -CANDS.index(k))) if pool else None
    tie = winner is not None and sum(auc[(2, k, "all")]["point"] == auc[(2, winner, "all")]["point"] for k in pool) > 1
    fresh = winner is not None and all(da[(rd, winner, "all")][0] > 0 for rd in ROUNDS)
    compute_time = time.time() - t0

    # ---------------------------------------------------------------- print
    print("# Discard model: the exploration read\n")
    print(
        "The five candidates of discardmodel.md's \"Pre-registration of the exploration\" (Step 2's held-out scores, `scores_model.tsv`, every "
        "learned score out of fold) and the references `ef` (today's `eye_focus`, missing last) and random (seed 0), read against the two rounds "
        "of blind \"OK to discard?\" labels: round 2 (the model-search confirmation) the primary read, round 1 (discard-labels) the replication. "
        "Each round keeps its own population, `π` and strata; one stratified count matrix per round (1,000 draws, seed 0) and one folder-cluster "
        "matrix (1,000 draws, seed 0) are shared by every score, so the differences against `ef` are paired. A missing score sorts last: `-inf` "
        "in the cut, the folder's lowest finite value minus 1 in the AUC.\n"
    )
    print("## Checks\n")
    gate.table(head_a, checks_a)
    gate.table(head_b, checks_b)

    print("## Data\n")
    drow = [("Dump frames / folders / dump picks / frames without a vector (dropped)", f"{n} / {len(folders)} / {int(pick.sum())} / {missing}"),
            ("Face-dump rows of files outside the dump", str(extra)),
            ("Dump frames without `eye_focus` today / after the fill (`ef-fill`)", f"{int(no_ef.sum())} ({pct(no_ef.sum(), n)}%) / {int(np.isnan(ef_fill).sum())} ({pct(np.isnan(ef_fill).sum(), n)}%)")]
    for rd, s in S.items():
        mm = s["masks"]["miss"]
        w = s["w"]
        drow.append((f"Round {rd}: labels / keep / discard", f"{len(s['idx'])} / {int((~s['bad']).sum())} / {int(s['bad'].sum())}"))
        drow.append((f"Round {rd}: missing-face labels / keep / discard (weighted share of the population)",
                     f"{int(mm.sum())} / {int((mm & ~s['bad']).sum())} / {int((mm & s['bad']).sum())} ({pct((w * mm).sum(), w.sum())}%)"))
    gate.table("| Count | Value |", drow)
    brow = [[f"{rd}", "all", f"{base[rd][0]:.2f}", "[{:.2f}, {:.2f}]".format(*base[rd][1]), "[{:.2f}, {:.2f}]".format(*base[rd][2]), f"{base[rd][3]:.2f}"] for rd in (2, 1)]
    for rd in (2, 1):
        s = S[rd]
        w, dis, cs, cc = s["w"], s["bad"], s["cs"], s["cc"]
        for part in ("miss", "has"):
            mk = s["masks"][part]
            brow.append([f"{rd}", {"miss": "missing-face", "has": "complement"}[part], pct((w * dis * mk).sum(), (w * mk).sum()), "[{:.1f}, {:.1f}]".format(*bf.quant(bf.ratio(cs, w * dis * mk, w * mk))),
                         "[{:.1f}, {:.1f}]".format(*bf.quant(bf.ratio(cc, w * dis * mk, w * mk))), pct((dis & mk).sum(), mk.sum())])
    gate.table("| Round | Frames | Discardable base rate, weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |", brow)

    print("## Where the cuts sit (label-free)\n")
    print(
        "Per score and cut on the full dump: the removed frames without `eye_focus` today (the missing-face frames) and the dump's picks among "
        "the removed, all and on the missing-face frames.\n"
    )
    wrow = []
    for k in SCORES:
        for q in QS:
            a, b, c, e = cut_miss[(k, q)]
            wrow.append([nm(k), f"q{q}", b, f"{a} ({pct(a, b)}%)", e, f"{c} ({pct(c, e)}%)"])
    gate.table("| Score | Cut | Removed | Removed without `eye_focus` today | Picks removed | Picks removed without `eye_focus` today |", wrow)

    for rd in (2, 1):
        print(f"## Round {rd}{' (primary)' if rd == 2 else ' (replication)'}\n")
        for part, title in PARTS.items():
            print(f"### Round {rd}, {title}\n")
            if part == "miss":
                print("The cut is still the full-dump cut; only the labeled frames read are restricted. Here `ef` is the folder's lowest finite "
                      "value minus 1 on every frame, so its AUC orders folders only and its cut is the seed-0 draw within each folder.\n")
            elif part == "has":
                print("The cut is still the full-dump cut; only the labeled frames read are restricted.\n")
            arow = []
            for k in SCORES:
                a = auc[(rd, k, part)]
                arow.append([nm(k), fcr.auc_s(a), fcr.cauc_s(a), f"{a['unw']:.3f}", "" if k == "ef" else fcr.eff(*da[(rd, k, part)], 3)])
            gate.table("| Score | AUC [stratified 95%] | Cluster | Unweighted | AUC - `ef` |", arow)
            crow = []
            for k in SCORES:
                for q in QS:
                    r = res[(rd, k, q, part)]
                    cells = [nm(k), f"q{q}", f"{r['n_rem']} / {r['n_dis']}", f"{bf.iv(r['point']['DP'], *r['ci']['DP'])} ({bf.f1(r['unw']['DP'])})",
                             f"{fcr.rr_s(r)} ({bf.f1(r['unw']['RR'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["RR"])]
                    if part == "all":
                        cells += [f"{res_pl[(k, q)]:.2f}", f"{res_removed[(rd, k, q)]:.2f}"]
                    cells.append("" if k == "ef" else fcr.eff(*fcr.diff(r, res[(rd, "ef", q, part)], "RR")))
                    crow.append(cells)
            if part == "all":
                gate.table("| Score | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `ef`, points |", crow)
            else:
                gate.table("| Score | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `RR` - `ef`, points |", crow)

    print("## The selection rule and the stop rule applied\n")
    print(
        f"Selection rule (as amended 2026-10-24): among the candidates whose q5 or q10 cut passes the label-free conditions on the full dump (`PL` "
        f"<= {df.PL_BAR:.0f}%, removed >= {df.REMOVED_BAR}%), the one with the highest weighted AUC on round 2; ties by round 1's AUC. The "
        "references are not eligible. Stop rule: the winner goes to the fresh round only if its AUC - `ef` point is above zero on both rounds.\n"
    )
    vrow = []
    for k in CANDS:
        vrow.append([nm(k), yn(lf_pass[(k, 5)]), yn(lf_pass[(k, 10)]), yn(eligible[k]), f"{auc[(2, k, 'all')]['point']:.3f}", f"{auc[(1, k, 'all')]['point']:.3f}",
                     fcr.eff(*da[(2, k, "all")], 3), fcr.eff(*da[(1, k, "all")], 3)])
    gate.table("| Candidate | q5 passes | q10 passes | Eligible | Round 2 AUC | Round 1 AUC | Round 2 AUC - `ef` | Round 1 AUC - `ef` |", vrow)
    if winner is None:
        print("**Winner**: none (no candidate passes the label-free conditions at q5 or q10).\n")
    else:
        print(f"**Winner**: {nm(winner)}, round 2 AUC {auc[(2, winner, 'all')]['point']:.3f}" + (" (a tie on round 2, broken by round 1)" if tie else "") + ".\n")
    if winner is None:
        print("**Stop rule**: no winner, so no fresh round; Steps 4-6 are skipped.\n")
    else:
        pts = ", ".join(f"round {rd} {da[(rd, winner, 'all')][0]:+.3f}" for rd in (2, 1))
        print(f"**Stop rule**: the winner's AUC - `ef` points are {pts}; " + ("both above zero, so the winner goes to the fresh round (Steps 4-6).\n" if fresh else
                                                                              "not above zero on both rounds, so no fresh round; Steps 4-6 are skipped.\n"))

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored scores, `scores_model.tsv`, face dump)", f"{load_time:.0f}"), ("Cuts, checks, metrics and bootstraps", f"{compute_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(out_path, "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))


if __name__ == "__main__":
    if len(sys.argv) != 8:
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:8])
