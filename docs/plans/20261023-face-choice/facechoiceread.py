"""Step 3: the single pre-registered read of the face-choice rules of
facechoice.md's "Pre-registration" against the two rounds of blind
"OK to discard?" labels.

Usage: python -I -B facechoiceread.py <app-dump-dir> <reject-filter-eval-dir> <strict-labels-dir> <discard-labels-dir> <model-search-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend` (what `badfilter.load` runs);
<reject-filter-eval-dir> holds `scores_reject.tsv` (the stored `G0`, round 1's
and round 2's bands); <strict-labels-dir> the strict sample's `manifest.tsv`
(outside both rounds' populations; no `labels.tsv` is opened there);
<discard-labels-dir> round 1 (`manifest.tsv`, `strata.tsv`, `labels.tsv`);
<model-search-dir> round 2 (`manifest.tsv`, `strata.tsv`, `labels.tsv`,
`scores_discard.tsv` for its `G0 ⊕ ef` bands); <out-dir> Step 2's data folder
with `faces\\<folder>.tsv`. Prints the tables (Markdown) and saves them as
`facechoiceread.out.md` in <out-dir>; refuses to run when that file exists,
so the read happens once. The label-free checks run before any `labels.tsv`
is opened, and a failure stops the read there; nothing is written on a
failure.
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

QS = df.QS
FLOOR = np.float32(0.8)
IOU = 0.5
RR_BAR = 5.0
RULES = ["af", "af-whole", "largest", "sharpest", "score", "mean"]
OTHERS = RULES[1:]
STORED = "stored"
ROUNDS = {
    2: {"strata": ["b05", "b10", "b20", "g0only", "rest"], "population": 30659, "earlier": 1800},
    1: {"strata": ["b05", "b10", "b20", "rest"], "population": 31259, "earlier": 1200},
}
EXPECTED = {
    2: {
        "base": ("42.31 [35.85, 48.91]", "[33.44, 51.61]"),
        "auc": ("0.785 [0.723, 0.844]", "[0.693, 0.881]", "0.764"),
        "rr": ("5.5 [1.1, 14.5]", "12.9 [7.2, 21.8]", "12.8 [8.9, 17.9]"),
        "pl": ("4.44", "9.14", "15.59"),
        "removed": ("4.80", "9.70", "19.64"),
    },
    1: {
        "base": ("48.45 [42.03, 55.11]", "[40.74, 53.87]"),
        "auc": ("0.777 [0.716, 0.841]", None, "0.803"),
        "rr": ("16.8 [1.2, 43.3]", "14.3 [2.9, 29.9]", "28.3 [15.7, 40.2]"),
        "pl": ("4.44", "9.14", "15.59"),
        "removed": ("4.83", "9.78", "19.87"),
    },
}


def nm(k):
    return "stored `eye_focus`" if k == STORED else f"`{k}`"


def load_faces(path, index, n):
    """Step 2's per-face dump as one record per dump frame: the AF flag and both face sets in detection order."""
    frames = [None] * n
    extra = 0
    for name in sorted(os.listdir(path)):
        if not name.endswith(".tsv"):
            continue
        for r in bf.read_tsv(os.path.join(path, name)):
            i = index.get((r["folder"], r["file"]))
            if i is None:
                extra += 1
                continue
            if frames[i] is None:
                frames[i] = {"af": r["af"] == "1", "crop": [], "whole": []}
            if r["face"] == "-":
                continue
            w, h = float(r["w"]), float(r["h"])
            frames[i][r["set"]].append({
                "box": (float(r["x"]), float(r["y"]), w, h),
                "area": float(np.float32(w) * np.float32(h)),
                "score": float(r["score"]),
                "conf": bool(np.float32(r["score"]) >= FLOOR),
                "nearest": r["nearest"] == "1",
                "ef": float(r["eye_focus"]),
            })
    return frames, extra


def last_max(faces, key):
    """`Iterator::max_by`: the maximum, ties to the later face."""
    best = None
    for f in faces:
        if best is None or key(f) >= key(best):
            best = f
    return best


def choose(fr):
    """Each rule's face (None when it picks none) and `eye_focus` (NaN when missing) on one frame."""
    crop, whole = fr["crop"], fr["whole"]
    conf = [f for f in whole if f["conf"]]
    largest = last_max(conf, lambda f: f["area"])
    top = last_max(whole, lambda f: f["score"])
    faces = {
        "af": next((f for f in crop if f["nearest"]), None) if fr["af"] else largest,
        "af-whole": next((f for f in whole if f["nearest"]), None) if fr["af"] else largest,
        "largest": largest,
        "sharpest": last_max(conf, lambda f: f["ef"]),
        "score": top if top is not None and top["conf"] else None,
        "mean": None,
    }
    vals = {k: np.nan if f is None else f["ef"] for k, f in faces.items()}
    vals["mean"] = float(np.mean([f["ef"] for f in conf])) if conf else np.nan
    return faces, vals, len(conf), len(crop)


def iou(a, b):
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    iw = max(0.0, min(ax + aw, bx + bw) - max(ax, bx))
    ih = max(0.0, min(ay + ah, by + bh) - max(ay, by))
    inter = iw * ih
    union = aw * ah + bw * bh - inter
    return inter / union if union > 0 else 0.0


def sample_of(rows, index, folder_of, labeled):
    s = {
        "name": [r["name"] for r in rows],
        "idx": np.array([index[(r["folder"], r["file"])] for r in rows]),
        "pi": np.array([float(r["inclusion_prob"]) for r in rows]),
        "band": np.array([r["band"] for r in rows], dtype=object),
    }
    if labeled:
        s["bad"] = np.array([r["label"] == "discard" for r in rows])
    s["w"] = 1 / s["pi"]
    s["folder"] = folder_of[s["idx"]]
    s["stratum"] = np.where(s["band"] == "control", "rest", s["band"])
    return s


def strata_check(d, pop, band, strata_rows, m):
    """The round's harness check: `strata.tsv` against the recomputed bands, and each sampled frame's band and `π`."""
    folders, folder_of = d["folders"], d["folder_of"]
    diff, pi_pop = 0, np.zeros(len(folder_of))
    for r in strata_rows:
        msk = pop & (band == r["band"])
        sm = m["band"] == r["band"]
        if r["band"] == "control":
            fi = folders.index(r["folder"])
            msk = msk & (folder_of == fi)
            sm = sm & (m["folder"] == fi)
        size, drawn = int(msk.sum()), int(sm.sum())
        pi_pop[msk] = drawn / size
        diff += size != int(r["population"]) or drawn != int(r["drawn"]) or abs(drawn / size - float(r["inclusion_prob"])) > 1e-5 * float(r["inclusion_prob"])
    band_diff = int((band[m["idx"]] != m["band"]).sum())
    pi_diff = int((np.abs(pi_pop[m["idx"]] - m["pi"]) > 1e-5 * m["pi"]).sum())
    return diff, band_diff, pi_diff


def auc_of(dis, sc, w):
    keep = w > 0
    y = dis[keep]
    if y.all() or not y.any():
        return np.nan
    return float(roc_auc_score(y, -sc[keep], sample_weight=w[keep]))


def auc_draws(s, sc, m, cs, cc):
    """The weighted AUC on the sampled frames `m`, unweighted, and per stratified and cluster draw."""
    dis, w, x = s["bad"][m], s["w"][m], sc[m]
    return {
        "point": auc_of(dis, x, w),
        "unw": auc_of(dis, x, np.ones(len(x))),
        "boot": np.array([auc_of(dis, x, w * cs[b][m]) for b in range(bf.BOOT)]),
        "cboot": np.array([auc_of(dis, x, w * cc[b][m]) for b in range(bf.BOOT)]),
    }


def cut_metrics(s, rem, m, cs, cc):
    with np.errstate(invalid="ignore", divide="ignore"):
        boot, clus = df.metrics(s, rem, m, cs), df.metrics(s, rem, m, cc)
        return {
            "point": df.metrics(s, rem, m), "unw": df.unweighted(s, rem, m), "boot": boot, "cboot": clus,
            "ci": {k: bf.quant(v) for k, v in boot.items()}, "cci": {k: bf.quant(v) for k, v in clus.items()},
            "n_rem": int((rem & m).sum()), "n_dis": int((rem & m & s["bad"]).sum()),
        }


def diff(a, b, key=None):
    """`a` minus `b`: the point, the stratified and the cluster 95% quantiles of the paired draws."""
    pa, pb = (a["point"], b["point"]) if key is None else (a["point"][key], b["point"][key])
    ba, bb = (a["boot"], b["boot"]) if key is None else (a["boot"][key], b["boot"][key])
    ca, cb = (a["cboot"], b["cboot"]) if key is None else (a["cboot"][key], b["cboot"][key])
    return pa - pb, bf.quant(ba - bb), bf.quant(ca - cb)


def eff(p, lohi, clohi, digits=1):
    return f"{p:+.{digits}f} [{lohi[0]:+.{digits}f}, {lohi[1]:+.{digits}f}] ([{clohi[0]:+.{digits}f}, {clohi[1]:+.{digits}f}])"


def auc_s(a):
    return bf.iv(a["point"], *bf.quant(a["boot"]), 3)


def cauc_s(a):
    return "[{:.3f}, {:.3f}]".format(*bf.quant(a["cboot"]))


def rr_s(r):
    return bf.iv(r["point"]["RR"], *r["ci"]["RR"])


def pct(num, den):
    return f"{100 * num / den:.1f}" if den else "-"


def yn(ok):
    return "yes" if ok else "no"


def main(dump, reject_dir, strict_dir, r1_dir, r2_dir, out_dir):
    out_path = os.path.join(out_dir, "facechoiceread.out.md")
    if os.path.exists(out_path):
        raise SystemExit(f"{out_path} exists: the face-choice rules are read once")
    start = time.time()
    dirs = {1: r1_dir, 2: r2_dir}
    roots = {k: os.path.normcase(os.path.abspath(p)) for k, p in (("strict-labels", strict_dir), ("round-1", r1_dir), ("round-2", r2_dir))}
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
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    sd, _ = rf.load_scores(os.path.join(r2_dir, "scores_discard.tsv"), d)
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])

    faces_of, extra = load_faces(os.path.join(out_dir, "faces"), index, n)
    no_faces = sum(fr is None for fr in faces_of)
    if no_faces:
        raise SystemExit(f"{no_faces} dump frames have no row in the face dump")
    face = {k: [None] * n for k in RULES}
    val = {k: np.full(n, np.nan) for k in RULES}
    nconf, ncrop = np.zeros(n, dtype=int), np.zeros(n, dtype=int)
    for i, fr in enumerate(faces_of):
        fc, vc, nconf[i], ncrop[i] = choose(fr)
        for k in RULES:
            face[k][i], val[k][i] = fc[k], vc[k]
    has_af = np.array([fr["af"] for fr in faces_of])
    multi = nconf >= 2
    scores = {k: np.where(np.isnan(val[k]), -np.inf, val[k]) for k in RULES}
    scores[STORED] = np.where(np.isnan(ef), -np.inf, ef)
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
    ms = {rd: sample_of(man[rd], index, folder_of, False) for rd in ROUNDS}
    seen = {1: strict_seen.copy()}
    seen[2] = strict_seen.copy()
    seen[2][ms[1]["idx"]] = True
    pop = {rd: ~seen[rd] for rd in ROUNDS}

    g0 = stored_rej["G0"]
    cg = {q: bf.graded_cut(g0, d, tb, q)[0] for q in QS}
    cw = {q: bf.graded_cut(sd["G0 ⊕ ef"], d, tb, q)[0] for q in QS}
    band = {1: np.full(n, "control", dtype=object), 2: np.full(n, "control", dtype=object)}
    band[1][cg[20]] = "b20"
    band[1][cg[10]] = "b10"
    band[1][cg[5]] = "b05"
    band[2][cg[20] & ~cw[20]] = "g0only"
    band[2][cw[20]] = "b20"
    band[2][cw[10]] = "b10"
    band[2][cw[5]] = "b05"

    res_pl = {(k, q): 100 * (cuts[(k, q)][0] & pick).sum() / pick.sum() for k in scores for q in QS}
    res_removed = {(rd, k, q): 100 * (cuts[(k, q)][0] & pop[rd]).sum() / pop[rd].sum() for rd in ROUNDS for k in scores for q in QS}
    dump_close = np.isnan(ef) == np.isnan(val["af"])
    dump_close &= np.isnan(ef) | (np.abs(ef - val["af"]) <= 1e-9)

    checks_a = [
        ("Dump frames (with a vector)", "32459", str(n)),
        ("Dump frames without a row in the face dump", "0", str(no_faces)),
        ("`af` vs the dump's stored `eye_focus` on every dump frame: frames differing (> 1e-9, or missing on one side only)", f"0 of {n}", f"{int((~dump_close).sum())} of {n}"),
        ("Frames left out of a cut, over the seven scores and `q`", "0", str(sum(int(cuts[c][1].sum()) for c in cuts))),
    ]
    for rd in ROUNDS:
        m = ms[rd]
        mv = np.array([float(r["eye_focus"]) for r in man[rd]])
        av = val["af"][m["idx"]]
        bad_rep = ~((np.isnan(mv) == np.isnan(av)) & (np.isnan(mv) | np.isnan(av) | (np.abs(mv - av) <= 1e-6)))
        sdiff, bdiff, pdiff = strata_check(d, pop[rd], band[rd], strata_rows[rd], m)
        checks_a += [
            (f"Round {rd}: `af` vs `manifest.tsv`'s `eye_focus`: frames differing (> 1e-6, or missing on one side only)", f"0 of {len(m['idx'])}", f"{int(bad_rep.sum())} of {len(m['idx'])}"),
            (f"Round {rd}: earlier-labeled frames excluded / population", f"{ROUNDS[rd]['earlier']} / {ROUNDS[rd]['population']}", f"{int(seen[rd].sum())} / {int(pop[rd].sum())}"),
            (f"Round {rd}: sampled frames in an earlier sample", "0", str(int(seen[rd][m["idx"]].sum()))),
            (f"Round {rd}: `strata.tsv` rows whose population, drawn count or `π` differ when recomputed from the dump", f"0 of {len(strata_rows[rd])}", f"{sdiff} of {len(strata_rows[rd])}"),
            (f"Round {rd}: `manifest.tsv` frames whose recomputed band / `π` differ", "0 / 0", f"{bdiff} / {pdiff}"),
            (f"Round {rd}: sum of `1/π` over the sampled frames", f"{ROUNDS[rd]['population']}.0", f"{m['w'].sum():.1f}"),
        ]
        for k in (STORED, "af"):
            checks_a.append((f"Round {rd}: {nm(k)} `PL` % (full dump) q5 / q10 / q20", " / ".join(EXPECTED[rd]["pl"]), " / ".join(f"{res_pl[(k, q)]:.2f}" for q in QS)))
            checks_a.append((f"Round {rd}: {nm(k)} removed % of the round's population q5 / q10 / q20", " / ".join(EXPECTED[rd]["removed"]), " / ".join(f"{res_removed[(rd, k, q)]:.2f}" for q in QS)))
    checks_a = [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in checks_a]
    early = {k: sorted({p for p in v if os.path.basename(p) == "labels.tsv"}) for k, v in opened.items()}
    checks_a.append(("`labels.tsv` files opened before these checks", "none", ", ".join(f"{k}: `{p}`" for k, ps in early.items() for p in ps) or "none",
                     "pass" if not any(early.values()) else "FAIL"))
    head_a = "| Check (label-free, before any `labels.tsv` is opened) | Expected | This run | Result |"
    if not all(r[3] == "pass" for r in checks_a):
        print("# Face choice: the single read\n")
        fail([(head_a, checks_a)])

    # The labels: each round's own sample, strata and bootstraps.
    lab = {rd: bf.read_tsv(os.path.join(dirs[rd], "labels.tsv")) for rd in ROUNDS}
    S = {}
    for rd in ROUNDS:
        s = sample_of(lab[rd], index, folder_of, True)
        s["labels"] = {r["label"] for r in lab[rd]}
        by_name = {r["name"]: r for r in man[rd]}
        s["man_diff"] = sum(r["name"] not in by_name or r["band"] != by_name[r["name"]]["band"] or r["inclusion_prob"] != by_name[r["name"]]["inclusion_prob"] for r in lab[rd])
        s["cs"] = df.counts(s, folders, ROUNDS[rd]["strata"])
        s["cc"] = bf.cluster_counts(s, folders)
        S[rd] = s

    res, auc, sub = {}, {}, {}
    for rd, s in S.items():
        idx, cs, cc = s["idx"], s["cs"], s["cc"]
        allm = np.ones(len(idx), dtype=bool)
        mm = multi[idx]
        for k in scores:
            for q in QS:
                rem = cuts[(k, q)][0][idx]
                res[(rd, k, q, "all")] = cut_metrics(s, rem, allm, cs, cc)
                if k != STORED:
                    res[(rd, k, q, "multi")] = cut_metrics(s, rem, mm, cs, cc)
            auc[(rd, k, "all")] = auc_draws(s, finite[k][idx], allm, cs, cc)
            if k != STORED:
                auc[(rd, k, "multi")] = auc_draws(s, finite[k][idx], mm, cs, cc)
        for other in ("largest", "sharpest"):
            fa = [face["af"][i] for i in idx]
            fo = [face[other][i] for i in idx]
            both = np.array([a is not None and b is not None for a, b in zip(fa, fo)])
            one = np.array([(a is None) != (b is None) for a, b in zip(fa, fo)])
            dm = np.array([a is not None and b is not None and iou(a["box"], b["box"]) < IOU for a, b in zip(fa, fo)])
            smaller = np.array([dm[j] and fa[j]["area"] < fo[j]["area"] for j in range(len(idx))])
            lower = np.array(["-" if not dm[j] else "af" if fa[j]["ef"] < fo[j]["ef"] else other if fo[j]["ef"] < fa[j]["ef"] else "equal" for j in range(len(idx))], dtype=object)
            sa = {k: auc_draws(s, finite[k][idx], dm, cs, cc) for k in RULES}
            sub[(rd, other)] = {"both": both, "one": one, "dm": dm, "smaller": smaller, "lower": lower, "auc": sa}
        print(f"round {rd}: {time.time() - t0:.0f} s", file=sys.stderr, flush=True)

    dump_dis = {}
    for other in ("largest", "sharpest"):
        both = np.array([face["af"][i] is not None and face[other][i] is not None for i in range(n)])
        one = np.array([(face["af"][i] is None) != (face[other][i] is None) for i in range(n)])
        dm = np.array([both[i] and iou(face["af"][i]["box"], face[other][i]["box"]) < IOU for i in range(n)])
        smaller = np.array([dm[i] and face["af"][i]["area"] < face[other][i]["area"] for i in range(n)])
        dump_dis[other] = (both, one, dm, smaller)

    checks_b = []
    base = {}
    for rd, s in S.items():
        w, dis, cs, cc = s["w"], s["bad"], s["cs"], s["cc"]
        b = 100 * (w * dis).sum() / w.sum()
        bci, bcci = bf.quant(bf.ratio(cs, w * dis, w)), bf.quant(bf.ratio(cc, w * dis, w))
        base[rd] = (b, bci, bcci, 100 * dis.mean())
        e = EXPECTED[rd]
        checks_b += [
            (f"Round {rd}: `labels.tsv` labels within {{keep, discard}}", "yes", yn(s["labels"] <= {"keep", "discard"})),
            (f"Round {rd}: `labels.tsv` rows whose name, band or `π` differ from `manifest.tsv`", "0", str(s["man_diff"])),
            (f"Round {rd}: discardable base rate % [stratified 95%] (cluster)", f"{e['base'][0]} ({e['base'][1]})",
             f"{b:.2f} [{bci[0]:.2f}, {bci[1]:.2f}] ([{bcci[0]:.2f}, {bcci[1]:.2f}])"),
        ]
        for k in (STORED, "af"):
            a = auc[(rd, k, "all")]
            exp = e["auc"][0] + (f" ({e['auc'][1]})" if e["auc"][1] else "") + f", unweighted {e['auc'][2]}"
            got = auc_s(a) + (f" ({cauc_s(a)})" if e["auc"][1] else "") + f", unweighted {a['unw']:.3f}"
            checks_b.append((f"Round {rd}: {nm(k)} AUC [stratified 95%]" + (" (cluster)" if e["auc"][1] else ""), exp, got))
            checks_b.append((f"Round {rd}: {nm(k)} `RR` % q5 / q10 / q20 [stratified 95%]", " / ".join(e["rr"]), " / ".join(rr_s(res[(rd, k, q, "all")]) for q in QS)))
    checks_b = [(c, e, v, "pass" if e == v else "FAIL") for c, e, v in checks_b]
    allowed = {"strict-labels": {"manifest.tsv"}, "round-1": {"manifest.tsv", "strata.tsv", "labels.tsv"},
               "round-2": {"manifest.tsv", "strata.tsv", "labels.tsv", "scores_discard.tsv"}}
    for k, ps in opened.items():
        got = sorted(set(ps))
        checks_b.append((f"Files opened under the {k} directory", ", ".join(f"`{p}`" for p in sorted(allowed[k])), ", ".join(f"`{p}`" for p in got) or "none",
                         "pass" if set(got) <= allowed[k] else "FAIL"))
    head_b = "| Check (labels) | Expected | This run | Result |"
    if not all(r[3] == "pass" for r in checks_b):
        print("# Face choice: the single read\n")
        fail([(head_a, checks_a), (head_b, checks_b)])

    verdict = {}
    for k in OTHERS:
        da = diff(auc[(2, k, "all")], auc[(2, "af", "all")])
        dr = diff(res[(2, k, 10, "all")], res[(2, "af", 10, "all")], "RR")
        c1, c2 = da[1][0] > 0, dr[1][1] <= RR_BAR
        verdict[k] = {"auc": auc[(2, k, "all")]["point"], "da": da, "dr": dr, "c1": c1, "c2": c2, "beats": c1 and c2, "fragile": c1 and c2 and da[2][0] <= 0}
    beating = [k for k in OTHERS if verdict[k]["beats"]]
    winner = max(beating, key=lambda k: (verdict[k]["auc"], -OTHERS.index(k))) if beating else None
    compute_time = time.time() - t0

    # ---------------------------------------------------------------- print
    print("# Face choice: the single read\n")
    print(
        "The six face-choice rules of facechoice.md's \"Pre-registration\" on the per-face dump of Step 2 (`faces\\<folder>.tsv`, every face of the "
        "crop and the whole set with its `eye_focus`), each rule's `eye_focus` per frame with a missing value last, read against the two rounds of "
        "blind \"OK to discard?\" labels: round 2 (the model-search confirmation) the primary read, round 1 (discard-labels) the replication. Each "
        "round keeps its own population, `π` and strata; one count matrix per round (stratified, 1,000 draws, seed 0) and one folder-cluster matrix "
        "(1,000 draws, seed 0) are shared by every rule, so the differences against `af` are paired. The whole-set floor is 0.8; the IoU threshold "
        "of the disagreement test 0.5.\n"
    )
    print("## Checks\n")
    gate.table(head_a, checks_a)
    gate.table(head_b, checks_b)

    print("## Data\n")
    drow = [("Dump frames / folders / dump picks / frames without a vector (dropped)", f"{n} / {len(folders)} / {int(pick.sum())} / {missing}"),
            ("Face-dump rows of files outside the dump (copies or frames without a vector)", str(extra)),
            ("Dump frames with a trusted AF point", f"{int(has_af.sum())} ({pct(has_af.sum(), n)}%)")]
    for rd, s in S.items():
        mm = multi[s["idx"]]
        drow.append((f"Round {rd}: labels / keep / discard", f"{len(s['idx'])} / {int((~s['bad']).sum())} / {int(s['bad'].sum())}"))
        drow.append((f"Round {rd}: multi-face labels (2+ whole faces at or above 0.8) / keep / discard", f"{int(mm.sum())} / {int((mm & ~s['bad']).sum())} / {int((mm & s['bad']).sum())}"))
    gate.table("| Count | Value |", drow)
    gate.table(
        "| Round | Discardable base rate, weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |",
        [[f"{rd}", f"{base[rd][0]:.2f}", "[{:.2f}, {:.2f}]".format(*base[rd][1]), "[{:.2f}, {:.2f}]".format(*base[rd][2]), f"{base[rd][3]:.2f}"] for rd in S],
    )

    print("## 1. Face counts\n")
    print(
        "The share of frames with 0, 1 or 2+ faces in the whole set at or above 0.8, and, among the frames with a trusted AF point, in the crop set "
        "(its detector's 0.6 floor). Labeled frames weighted by the round's `1/π` (the population estimate) and unweighted; the whole dump "
        "label-free.\n"
    )
    frow = []

    def count_row(label, fm, w):
        wa = w * has_af[fm]
        cells = [label, str(int(len(fm)))]
        for k in (0, 1):
            cells.append(pct((w * (nconf[fm] == k)).sum(), w.sum()))
        cells.append(pct((w * (nconf[fm] >= 2)).sum(), w.sum()))
        cells.append(pct(wa.sum(), w.sum()))
        for k in (0, 1):
            cells.append(pct((wa * (ncrop[fm] == k)).sum(), wa.sum()))
        cells.append(pct((wa * (ncrop[fm] >= 2)).sum(), wa.sum()))
        return cells

    for rd, s in S.items():
        frow.append(count_row(f"Round {rd}, weighted", s["idx"], s["w"]))
        frow.append(count_row(f"Round {rd}, unweighted", s["idx"], np.ones(len(s["idx"]))))
    frow.append(count_row("Whole dump", np.arange(n), np.ones(n)))
    gate.table("| Frames | n | Whole 0 % | Whole 1 % | Whole 2+ % | AF frames % | Crop 0 % (of AF) | Crop 1 % (of AF) | Crop 2+ % (of AF) |", frow)
    print("The share of frames each rule leaves missing (it picks no face):\n")
    mrow = []
    for k in RULES:
        miss = np.isnan(val[k])
        r = [nm(k)]
        for rd, s in S.items():
            r += [pct((s["w"] * miss[s["idx"]]).sum(), s["w"].sum()), pct(miss[s["idx"]].sum(), len(s["idx"]))]
        r.append(pct(miss.sum(), n))
        mrow.append(r)
    gate.table("| Rule | Round 2 weighted % | Round 2 unweighted % | Round 1 weighted % | Round 1 unweighted % | Whole dump % |", mrow)

    print("## 2. The rules\n")
    print(
        "Per folder on the full dump, the bottom `floor(q * n / 100)` by the rule's `eye_focus` (`bf.graded_cut`, missing last, ties by the seed-0 "
        "draw). Weighted (`1/π`) with the stratified 95% interval, the folder-cluster interval and the unweighted value; `PL` the share of the dump's "
        "picks inside the cut (full dump, label-free), removed the round population's share. AUC: `roc_auc_score(discard, -score, "
        "sample_weight=1/π)`, a missing value at the folder's lowest finite score minus 1. Differences against `af` on the same draws, stratified "
        "95% with the cluster interval in parentheses. On the multi-face frames (2+ whole faces at or above 0.8) the cut is still the full-dump cut; "
        "only the labeled frames read are restricted.\n"
    )
    for rd in (2, 1):
        for part, title in (("all", "all labeled frames"), ("multi", "multi-face frames")):
            print(f"### Round {rd}{' (primary)' if rd == 2 else ' (replication)'}, {title}\n")
            arow = []
            for k in RULES:
                a = auc[(rd, k, part)]
                arow.append([nm(k), auc_s(a), cauc_s(a), f"{a['unw']:.3f}", "" if k == "af" else eff(*diff(a, auc[(rd, "af", part)]), 3)])
            gate.table("| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |", arow)
            crow = []
            for k in RULES:
                for q in QS:
                    r = res[(rd, k, q, part)]
                    cells = [nm(k), f"q{q}", f"{r['n_rem']} / {r['n_dis']}",
                             f"{bf.iv(r['point']['DP'], *r['ci']['DP'])} ({bf.f1(r['unw']['DP'])})",
                             f"{rr_s(r)} ({bf.f1(r['unw']['RR'])})", "[{:.1f}, {:.1f}]".format(*r["cci"]["RR"])]
                    if part == "all":
                        cells += [f"{res_pl[(k, q)]:.2f}", f"{res_removed[(rd, k, q)]:.2f}"]
                    cells.append("" if k == "af" else eff(*diff(r, res[(rd, "af", q, part)], "RR")))
                    crow.append(cells)
            if part == "all":
                gate.table("| Rule | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `PL` % | Removed % | `RR` - `af`, points |", crow)
            else:
                gate.table("| Rule | Cut | Sampled removed / discard | `DP` % | `RR` % | `RR` cluster | `RR` - `af`, points |", crow)

    print("## 3. The disagreement test\n")
    print(
        f"The labeled frames where both `af` and the other rule pick a face and the two boxes (stored coordinates) have IoU < {IOU}; frames where "
        "exactly one of the two picks a face are counted and left out. \"`af` smaller\": the `af` face's box area is below the other's. The "
        "whole dump's counts beside them, label-free.\n"
    )
    for other in ("largest", "sharpest"):
        both_d, one_d, dm_d, smaller_d = dump_dis[other]
        print(f"### `af` vs `{other}`\n")
        srow = []
        for rd in (2, 1):
            s, x = S[rd], sub[(rd, other)]
            w = s["w"]
            srow.append([f"Round {rd}", int(x["both"].sum()), int(x["one"].sum()), f"{int(x['dm'].sum())} ({pct((w * x['dm']).sum(), w.sum())}% weighted)",
                         f"{int((x['dm'] & s['bad']).sum())} / {int((x['dm'] & ~s['bad']).sum())}",
                         f"{int(x['smaller'].sum())} ({pct(x['smaller'].sum(), x['dm'].sum())}%)"])
        srow.append(["Whole dump", int(both_d.sum()), int(one_d.sum()), f"{int(dm_d.sum())} ({pct(dm_d.sum(), n)}%)", "-",
                     f"{int(smaller_d.sum())} ({pct(smaller_d.sum(), dm_d.sum())}%)"])
        gate.table(f"| Frames | Both pick a face | Exactly one picks | IoU < {IOU} (the subset) | Subset discard / keep | `af` smaller |", srow)
        for rd in (2, 1):
            x = sub[(rd, other)]
            print(f"Round {rd}, the subset's AUC:\n")
            gate.table(
                "| Rule | AUC [stratified 95%] | Cluster | Unweighted | AUC - `af` |",
                [[nm(k), auc_s(x["auc"][k]), cauc_s(x["auc"][k]), f"{x['auc'][k]['unw']:.3f}", "" if k == "af" else eff(*diff(x["auc"][k], x["auc"]["af"]), 3)] for k in RULES],
            )
            s = S[rd]
            w, dis, cs = s["w"], s["bad"], s["cs"]
            print(f"Round {rd}, the label by which face has the lower `eye_focus` (weighted discard share with its stratified 95% interval):\n")
            xrow = []
            for lv, label in (("af", "`af`'s"), (other, f"`{other}`'s"), ("equal", "equal")):
                lm = x["lower"] == lv
                rate = pct((w * dis * lm).sum(), (w * lm).sum())
                ci = bf.quant(bf.ratio(cs, w * dis * lm, w * lm)) if lm.any() else (np.nan, np.nan)
                xrow.append([label, int(lm.sum()), int((lm & ~dis).sum()), int((lm & dis).sum()), rate if not lm.any() else f"{rate} [{ci[0]:.1f}, {ci[1]:.1f}]"])
            gate.table("| Lower `eye_focus` | Frames | Keep | Discard | Weighted discard % |", xrow)

    print("## 4. The reading rule applied\n")
    print(
        f"On round 2, all labeled frames: a rule beats today when (1) the paired AUC difference against `af` has its stratified 2.5% quantile above "
        f"zero and (2) the paired stratified 97.5% quantile of its `RR` minus `af`'s `RR` at q10 is at most +{RR_BAR:.0f} points. The winner is the "
        "beating rule with the highest round-2 AUC; a beat whose folder-cluster interval of the AUC difference reaches zero is fragile.\n"
    )
    vrow = []
    for k in OTHERS:
        v = verdict[k]
        vrow.append([nm(k), f"{v['auc']:.3f}", f"{v['da'][0]:+.3f}, 2.5% {v['da'][1][0]:+.3f} (cluster {v['da'][2][0]:+.3f}): {yn(v['c1'])}",
                     f"{v['dr'][0]:+.1f}, 97.5% {v['dr'][1][1]:+.1f} (cluster {v['dr'][2][1]:+.1f}): {yn(v['c2'])}",
                     ("beats" + (", fragile" if v["fragile"] else "")) if v["beats"] else "does not beat"])
    gate.table("| Rule | AUC | (1) AUC - `af` | (2) `RR` q10 - `af`, points | Verdict |", vrow)
    print(f"**Winner**: {nm(winner) + (' (fragile)' if verdict[winner]['fragile'] else '') if winner else 'no winner'}.\n")

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored scores, face dump)", f"{load_time:.0f}"), ("Cuts, checks, metrics and bootstraps", f"{compute_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.\n")
    sys.stdout = tee.stream
    with open(out_path, "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))


if __name__ == "__main__":
    if len(sys.argv) != 7:
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:7])
