"""Step 2: the held-out scores of the five discard candidates of discardmodel.md's
"Pre-registration of the exploration" on every dump frame, with the label-free
readings only.

Usage: python -I -B discardfit.py <app-dump-dir> <face-choice-dir> <reject-filter-eval-dir> <strict-labels-dir> <discard-labels-dir> <model-search-dir> <out-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend` (`badfilter.load` is not
called); <face-choice-dir> holds the per-face dump `faces\\<folder>.tsv`;
<reject-filter-eval-dir> is only audited (only `scores_reject.tsv` may be
opened there; this step opens nothing); <strict-labels-dir> the strict
sample's `manifest.tsv` (the rounds' populations; its `labels.tsv` is never
opened); <discard-labels-dir> round 1 and <model-search-dir> round 2, each
`manifest.tsv` (the `1/π`) and `labels.tsv` (the training target) only. Writes
`scores_model.tsv` (`folder`, `file`, one column per candidate, higher keeps,
every learned score out of fold; `ef-fill` `nan` where it stays missing) into
<out-dir>, prints the label-free readings (Markdown) and saves them as
`discardfit.out.md` there. No score is read against the labels; the one
reading that touches the picks is the pre-registered label-free `PL`.
"""

import os
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.metrics import roc_auc_score
from sklearn.model_selection import GroupKFold

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261018-discard-labels"))
import discardfilter as df  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261019-discard-retarget"))
import retargetfit  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261020-model-search"))
import searchfit  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261023-face-choice"))
import facechoiceread as fcr  # noqa: E402

rf, gate, folder_gate = bf.rf, bf.gate, bf.folder_gate

QS = df.QS
CANDS = ["ef-fill", "LR/S", "LR/T", "HGB-mono/T", "HGB/T"]
LEARNED = CANDS[1:]
REFS = ["ef", "random"]
T_NAMES = [
    "`eye_focus` logit, filled",
    "no `eye_focus` today",
    "ln rel sharpness",
    "no `eye_focus` after the fill",
    "EAR",
    "\\|yaw\\|",
    "\\|pitch\\|",
    "\\|roll\\|",
    "no pose",
    "whole-set faces >= 0.8, capped at 3",
]
N_S = 3
# On the discard probability: keep rises with the logit, the sharpness and the EAR, falls with |yaw|, |pitch|, |roll|.
MONO_CST = [-1, 0, -1, 0, -1, 1, 1, 1, 0, 0]
ROUNDS = {1: {"labels": (211, 389), "population": 31259}, 2: {"labels": (248, 352), "population": 30659}}
MISSING_TODAY = "12.4"
MISSING_AF_WHOLE = "4.5"


def per_round_mean1(w, rnd):
    """Rescale to mean 1 within each round's rows, no further rescale of the pool."""
    out = np.empty(len(w))
    for r in ROUNDS:
        m = rnd == r
        if m.any():
            out[m] = w[m] * m.sum() / w[m].sum()
    return out


def lr_design(t, tm):
    """`T` (or `S`) NaN mean-filled on the training folders' frames, every column standardized on them."""
    x = t.copy()
    mean = np.nanmean(x[tm], axis=0)
    idx = np.where(np.isnan(x))
    x[idx] = np.take(mean, idx[1])
    mu, sd = x[tm].mean(axis=0), x[tm].std(axis=0)
    sd[sd == 0] = 1
    return (x - mu) / sd


def choose_c(x, idx, y, w, rnd, groups):
    """`searchfit.choose_c`'s discard branch: inner `GroupKFold` over the training folders, the mean weighted discard AUC per inner fold, the inner training weights rescaled per round."""
    k = min(5, len(set(groups.tolist())))
    splits = list(GroupKFold(n_splits=k).split(idx, groups=groups))
    best, best_c = -1, None
    for c in gate.CS:
        vals = []
        for tr, va in splits:
            m = searchfit.lr("LR-L2", c).fit(x[idx[tr]], y[tr], sample_weight=per_round_mean1(w[tr], rnd[tr]))
            s = m.decision_function(x[idx[va]])
            if 0 < y[va].sum() < len(va):
                vals.append(roc_auc_score(y[va], s, sample_weight=w[va]))
        v = float(np.mean(vals))
        if v > best:
            best, best_c = v, c
    return best_c


def training(fo, L, held):
    tm = fo != held
    ltr = L["folder"] != held
    return tm, L["idx"][ltr], L["discard"][ltr].astype(int), per_round_mean1(L["w_clip"][ltr], L["round"][ltr]), L["round"][ltr]


def fold(fo, t, L, held, form, c_fixed=None, filled=False):
    """One held-out folder of one learned candidate: fit on the other 26 folders' labeled frames, score the held-out folder (higher keeps)."""
    tm, idx, y, w, rnd = training(fo, L, held)
    cols = t[:, :N_S] if form == "LR/S" else t
    c = None
    if form.startswith("LR"):
        x = lr_design(cols, tm)
        c = c_fixed if c_fixed is not None else choose_c(x, idx, y, w, rnd, fo[idx])
        m = searchfit.lr("LR-L2", c).fit(x[idx], y, sample_weight=w)
    else:
        x = lr_design(cols, tm) if form == "HGB-mono/T" and filled else cols
        params = dict(rf.HGB)
        if form == "HGB-mono/T":
            params["monotonic_cst"] = MONO_CST
        m = HistGradientBoostingClassifier(**params).fit(x[idx], y, sample_weight=w)
    return held, c, -m.decision_function(x[fo == held])


def mono_check(fo, t, L, held):
    """`HGB-mono/T` on one fold with the NaN kept: whether the fit raises, and the largest step against a constraint over a grid of each constrained column's non-missing values."""
    _, idx, y, w, _ = training(fo, L, held)
    try:
        m = HistGradientBoostingClassifier(**rf.HGB, monotonic_cst=MONO_CST).fit(t[idx], y, sample_weight=w)
    except Exception as e:  # noqa: BLE001
        return f"raised {type(e).__name__}: {e}", None
    test = np.where(fo == held)[0]
    rng = np.random.default_rng(0)
    worst = 0.0
    for j, cst in enumerate(MONO_CST):
        if cst == 0:
            continue
        rows = test[~np.isnan(t[test, j])]
        rows = rng.choice(rows, min(searchfit.MONO_ROWS, len(rows)), replace=False)
        grid = np.unique(np.quantile(t[idx, j][~np.isnan(t[idx, j])], np.linspace(0, 1, searchfit.MONO_GRID)))
        vals = []
        for g in grid:
            xg = t[rows].copy()
            xg[:, j] = g
            vals.append(m.decision_function(xg))
        steps = np.diff(np.array(vals), axis=0) * cst
        worst = max(worst, float(-steps.min()) if steps.size else 0.0)
    return "fit", worst


def pct(num, den):
    return f"{100 * num / den:.1f}" if den else "-"


def yn(ok):
    return "yes" if ok else "no"


def main(dump, face_dir, reject_dir, strict_dir, r1_dir, r2_dir, out_dir, jobs):
    start = time.time()
    dirs = {1: r1_dir, 2: r2_dir}
    roots = {k: os.path.normcase(os.path.abspath(p)) for k, p in (("reject-filter-eval", reject_dir), ("strict-labels", strict_dir), ("round-1", r1_dir), ("round-2", r2_dir))}
    opened = {k: [] for k in roots}

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            for k, r in roots.items():
                if p.startswith(r + os.sep):
                    opened[k].append(os.path.relpath(p, r))

    sys.addaudithook(audit)
    os.makedirs(out_dir, exist_ok=True)
    tee = bf.Tee(sys.stdout)
    sys.stdout = tee

    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n, nf = len(rows), len(folders)
    tb = np.random.default_rng(0).random(n)
    pick = d["pick"]
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])

    faces_of, extra = fcr.load_faces(os.path.join(face_dir, "faces"), index, n)
    no_faces = sum(fr is None for fr in faces_of)
    if no_faces:
        raise SystemExit(f"{no_faces} dump frames have no row in the face dump")
    af, af_whole, nconf = np.full(n, np.nan), np.full(n, np.nan), np.zeros(n, dtype=int)
    for i, fr in enumerate(faces_of):
        _, vals, nconf[i], _ = fcr.choose(fr)
        af[i], af_whole[i] = vals["af"], vals["af-whole"]
    has_af = np.array([fr["af"] for fr in faces_of])
    no_ef = np.isnan(ef)
    ef_fill = np.where(no_ef, af_whole, ef)
    no_fill = np.isnan(ef_fill)
    touched = no_ef & ~no_fill

    ear_raw = rf.load_ear(dump, d)
    ear = np.where(touched, np.nan, ear_raw)
    tech = d["tech"]
    t = np.column_stack([
        np.array([np.nan if np.isnan(v) else gate.logit(v) for v in ef_fill]),
        no_ef.astype(float),
        tech[:, 0],
        no_fill.astype(float),
        ear,
        tech[:, 3],
        tech[:, 4],
        tech[:, 5],
        tech[:, 9],
        np.minimum(nconf, 3).astype(float),
    ])
    pose_on_touched = int((touched & (tech[:, 9] == 0)).sum())
    ear_on_touched = int((touched & ~np.isnan(ear_raw)).sum())

    strict_seen = np.zeros(n, dtype=bool)
    for r in bf.read_tsv(os.path.join(strict_dir, "manifest.tsv")):
        strict_seen[index[(r["folder"], r["file"])]] = True
    man = {rd: bf.read_tsv(os.path.join(dirs[rd], "manifest.tsv")) for rd in ROUNDS}
    lab = {rd: bf.read_tsv(os.path.join(dirs[rd], "labels.tsv")) for rd in ROUNDS}
    seen = {1: strict_seen.copy(), 2: strict_seen.copy()}
    for r in man[1]:
        seen[2][index[(r["folder"], r["file"])]] = True
    pop = {rd: ~seen[rd] for rd in ROUNDS}

    parts, rinfo = {k: [] for k in ("idx", "discard", "round", "w_raw", "w_clip")}, {}
    for rd in ROUNDS:
        by_name = {r["name"]: r for r in man[rd]}
        w_man = np.array([1 / float(r["inclusion_prob"]) for r in man[rd]])
        cap = searchfit.CAP_FACTOR * float(np.median(w_man))
        idx = np.array([index[(r["folder"], r["file"])] for r in lab[rd]])
        w = np.array([1 / float(by_name[r["name"]]["inclusion_prob"]) for r in lab[rd]])
        man_diff = sum(r["name"] not in by_name or r["band"] != by_name[r["name"]]["band"] or r["inclusion_prob"] != by_name[r["name"]]["inclusion_prob"]
                       or (r["folder"], r["file"]) != (by_name[r["name"]]["folder"], by_name[r["name"]]["file"]) for r in lab[rd])
        dis = np.array([r["label"] == "discard" for r in lab[rd]])
        rinfo[rd] = {"cap": cap, "median": float(np.median(w_man)), "clipped": int((w > cap).sum()), "sum_w": float(w_man.sum()), "man_diff": man_diff,
                     "labels": {r["label"] for r in lab[rd]}, "n": len(idx), "keep": int((~dis).sum()), "discard": int(dis.sum()),
                     "in_earlier": int(seen[rd][idx].sum()), "n_man": len(man[rd])}
        parts["idx"].append(idx)
        parts["discard"].append(dis)
        parts["round"].append(np.full(len(idx), rd))
        parts["w_raw"].append(w)
        parts["w_clip"].append(np.minimum(w, cap))
    L = {k: np.concatenate(v) for k, v in parts.items()}
    L["folder"] = folder_of[L["idx"]]
    dup = len(L["idx"]) - len(set(L["idx"].tolist()))
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    spot = max(range(nf), key=lambda fi: int((folder_of == fi).sum()))
    t0 = time.time()
    mono = mono_check(folder_of, t, L, spot)
    mono_time = time.time() - t0
    filled = mono[0] != "fit" or mono[1] > 1e-9
    print(f"HGB-mono/T check: {mono}, filled fallback {filled}", file=sys.stderr, flush=True)

    scores = {"ef-fill": ef_fill}
    info = {}
    with Parallel(n_jobs=jobs) as par:

        def run(form, c_fixed=None):
            t1 = time.time()
            res = par(delayed(fold)(folder_of, t, L, h, form, c_fixed, filled) for h in range(nf))
            sc, cs = np.full(n, np.nan), np.full(nf, np.nan)
            for h, c, s in res:
                sc[folder_of == h] = s
                cs[h] = np.nan if c is None else c
            return sc, cs, time.time() - t1

        for form in LEARNED:
            sc, cs, secs = run(form)
            r = {"chosen": cs, "fallback": False, "time": secs}
            if form.startswith("LR"):
                _, cnt = np.unique(cs, return_counts=True)
                if cnt.max() <= nf / 2:
                    sc, _, secs2 = run(form, 1.0)
                    r["fallback"], r["time"] = True, secs + secs2
            scores[form], info[form] = sc, r
            print(f"{form}: {r['time']:.0f} s", file=sys.stderr, flush=True)

    with open(os.path.join(out_dir, "scores_model.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("folder\tfile\t" + "\t".join(CANDS) + "\n")
        for i, f in enumerate(rows):
            out.write(f"{f['folder']}\t{f['file']}\t" + "\t".join(repr(float(scores[k][i])) for k in CANDS) + "\n")

    t2 = time.time()
    cut_scores = {k: np.where(np.isnan(scores[k]), -np.inf, scores[k]) for k in CANDS}
    cut_scores["ef"] = np.where(no_ef, -np.inf, ef)
    cut_scores["random"] = tb
    lrows, verdict, frames_out = [], {}, 0
    for k in REFS + CANDS:
        for q in QS:
            removed, outm = bf.graded_cut(cut_scores[k], d, tb, q)
            frames_out += int(outm.sum())
            pl = 100 * (removed & pick).sum() / pick.sum()
            rem = 100 * removed.sum() / n
            rem_r = {rd: 100 * (removed & pop[rd]).sum() / pop[rd].sum() for rd in ROUNDS}
            ok_pl, ok_rem = pl <= df.PL_BAR, rem >= df.REMOVED_BAR
            verdict[(k, q)] = (ok_pl, ok_rem)
            lrows.append([f"`{k}`" + (" (reference)" if k in REFS else ""), f"q{q}", int(removed.sum()), f"{rem:.2f}", f"{rem_r[2]:.2f} / {rem_r[1]:.2f}", f"{pl:.2f}",
                          yn(ok_pl), yn(ok_rem)])
    cut_time = time.time() - t2

    print("# Discard model: held-out scores and label-free readings\n")
    print(
        "The five candidates of discardmodel.md's \"Pre-registration of the exploration\", fit leave-one-folder-out over the 27 dump folders on "
        "the 1,200 blind \"OK to discard?\" labels of both rounds (pooled, each frame's own round's `1/π` clipped at 5 x that round's median "
        "sampled `1/π` and rescaled to mean 1 within the round over the fold's training rows). No score of this step is read against "
        "`labels.tsv`: the labels enter training only, and no `DP`, `RR` or AUC is computed. The one reading that touches the picks is the "
        "pre-registered label-free `PL`.\n"
    )

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{nf} / {n} / {missing}"),
            ("Dump picks / strict-sample frames", f"{int(pick.sum())} / {int(strict_seen.sum())}"),
            ("Face-dump rows of files outside the dump", str(extra)),
            ("Dump frames with a trusted AF point", f"{int(has_af.sum())} ({pct(has_af.sum(), n)}%)"),
            *[(f"Round {rd}: labels / keep / discard", f"{rinfo[rd]['n']} / {rinfo[rd]['keep']} / {rinfo[rd]['discard']}") for rd in ROUNDS],
            *[(f"Round {rd}: median sampled `1/π` / cap (5 x the median) / labeled frames clipped", f"{rinfo[rd]['median']:.3f} / {rinfo[rd]['cap']:.3f} / {rinfo[rd]['clipped']}") for rd in ROUNDS],
            ("`HGB-mono/T` columns", "mean-filled (fallback)" if filled else "NaN kept, as `HGB/T`"),
        ],
    )

    print("## The columns\n")
    print(
        "`S` is the first three columns, `T` all ten. The fill: today's `eye_focus`, or, where it is missing, the `af-whole` face's (the whole "
        "set's nearest face to the AF point, `facechoiceread.choose`). Share of the dump frames missing per column (NaN), label-free:\n"
    )
    gate.table("| # | Column | Missing % | `HGB-mono/T` constraint (on keep) |",
               [[j + 1, name, pct(np.isnan(t[:, j]).sum(), n), {-1: "+1", 1: "-1", 0: "0"}[MONO_CST[j]]] for j, name in enumerate(T_NAMES)])

    print("## Missing shares\n")
    gate.table(
        "| Score | Frames missing | Dump % | Expected |",
        [
            ("Today's `eye_focus` (`ef`)", int(no_ef.sum()), pct(no_ef.sum(), n), f"{MISSING_TODAY}"),
            ("`af-whole` (face-choice rule)", int(np.isnan(af_whole).sum()), pct(np.isnan(af_whole).sum(), n), f"{MISSING_AF_WHOLE}"),
            ("`ef-fill` (after the fill)", int(no_fill.sum()), pct(no_fill.sum(), n), f"<= `af-whole`'s"),
            ("Frames the fill touches (missing today, filled)", int(touched.sum()), pct(touched.sum(), n), "-"),
        ],
    )

    print("## Per-fold training\n")
    print(
        "Per held-out folder: its dump frames, the labeled training frames (the other 26 folders, both rounds), `discard` among them, and the "
        "Kish effective sample size `(Σw)² / Σw²` of the pooled training weights, each rescaled to mean 1 within its round, without and with "
        "the clip.\n"
    )
    frows, e_raw, e_clip, n_tr = [], [], [], []
    for h in range(nf):
        ltr = L["folder"] != h
        a = retargetfit.kish(per_round_mean1(L["w_raw"][ltr], L["round"][ltr]))
        b = retargetfit.kish(per_round_mean1(L["w_clip"][ltr], L["round"][ltr]))
        e_raw.append(a)
        e_clip.append(b)
        n_tr.append(int(ltr.sum()))
        frows.append([f"`{folders[h]}`", int((folder_of == h).sum()), int(((L["round"] == 1) & ltr).sum()), int(((L["round"] == 2) & ltr).sum()), int(ltr.sum()),
                      int((L["discard"] & ltr).sum()), f"{a:.1f}", f"{b:.1f}"])
    gate.table("| Held-out folder | Frames | Round 1 training labels | Round 2 training labels | Training labels | Discard | ESS no clip | ESS clipped |", frows)
    gate.table(
        "| Reading | Min / max over the folds |",
        [
            ("Labeled training frames", f"{min(n_tr)} / {max(n_tr)}"),
            ("Kish ESS, no clip", f"{min(e_raw):.1f} / {max(e_raw):.1f}"),
            ("Kish ESS, clipped", f"{min(e_clip):.1f} / {max(e_clip):.1f}"),
        ],
    )

    print("## `C` chosen per fold (logistic forms)\n")
    print("Folds choosing each `C` of `gate.CS` by the inner `GroupKFold` (weighted discard AUC). Fallback: no `C` chosen by more than half the 27 folds, so the candidate is refit with `C` = 1.0 in every fold.\n")
    gate.table("| Candidate | " + " | ".join(str(c) for c in gate.CS) + " | Fallback |",
               [[f"`{k}`"] + [int((info[k]["chosen"] == c).sum()) for c in gate.CS] + ["yes, `C` = 1.0" if info[k]["fallback"] else "no"] for k in LEARNED if k.startswith("LR")])
    print("Per fold:\n")
    gate.table("| Held-out folder | " + " | ".join(f"`{k}`" for k in LEARNED if k.startswith("LR")) + " |",
               [[f"`{folders[h]}`"] + [f"{info[k]['chosen'][h]:g}" for k in LEARNED if k.startswith("LR")] for h in range(nf)])

    print("## Label-free cuts: removed share and pick loss\n")
    print(
        f"Per folder on the full dump, the bottom `floor(q * n / 100)` frames by the score, a missing score last, ties by the seed-0 draw "
        f"(`bf.graded_cut`). Removed: the share of the full dump's frames inside the cut (each round's population after it, a reading). `PL`: the "
        f"share of the dump's picks inside the cut (full dump). The label-free conditions of the selection rule, at q5 or q10 (amended 2026-10-24; pre-registered at q10): `PL` <= {df.PL_BAR:.0f}% "
        f"and removed >= {df.REMOVED_BAR}%. `ef` and random (seed 0) are references, not candidates.\n"
    )
    gate.table("| Score | Cut | Frames removed | Removed % (dump) | Removed % (round 2 / round 1 population) | `PL` % | `PL` <= 5% | Removed >= 4.5% |", lrows)
    gate.table("| Candidate | q5 passes | q10 passes | Eligible (q5 or q10) |", [[f"`{k}`", yn(all(verdict[(k, 5)])), yn(all(verdict[(k, 10)])), yn(all(verdict[(k, 5)]) or all(verdict[(k, 10)]))] for k in CANDS])

    print("## Checks\n")
    dump_close = (np.isnan(ef) == np.isnan(af)) & (np.isnan(ef) | (np.abs(ef - af) <= 1e-9))
    learned_ok = all(np.isfinite(scores[k]).all() for k in LEARNED)
    allowed = {"reject-filter-eval": {"scores_reject.tsv"}, "strict-labels": {"manifest.tsv"},
               "round-1": {"labels.tsv", "manifest.tsv", "strata.tsv"}, "round-2": {"labels.tsv", "manifest.tsv", "strata.tsv"}}
    checks = [
        ("Dump frames (with a vector)", "32459", str(n), n == 32459),
        ("Dump frames without a row in the face dump", "0", str(no_faces), no_faces == 0),
        ("`af` (face dump) vs the dump's stored `eye_focus`: frames differing (> 1e-9, or missing on one side only)", "0", str(int((~dump_close).sum())), bool(dump_close.all())),
        ("Today's `eye_focus` missing % of the dump", MISSING_TODAY, pct(no_ef.sum(), n), pct(no_ef.sum(), n) == MISSING_TODAY),
        ("`af-whole` missing % of the dump", MISSING_AF_WHOLE, pct(np.isnan(af_whole).sum(), n), pct(np.isnan(af_whole).sum(), n) == MISSING_AF_WHOLE),
        ("`ef-fill` missing frames at or below `af-whole`'s", "yes", yn(no_fill.sum() <= np.isnan(af_whole).sum()), bool(no_fill.sum() <= np.isnan(af_whole).sum())),
        ("Frames the fill touches with a frame-level EAR (set to NaN) / with a frame-level pose", "reading / 0", f"{ear_on_touched} / {pose_on_touched}", pose_on_touched == 0),
        ("Frames left out of a cut (a folder with a NaN), over the scores and `q`", "0", str(frames_out), frames_out == 0),
        ("Every learned candidate finite on every dump frame", "yes", yn(learned_ok), learned_ok),
        ("`HGB-mono/T` on the spot fold " + f"`{folders[spot]}`" + ": fit, and the largest step against a constraint over the non-missing grid", "fit, 0",
         (mono[0] if mono[1] is None else f"{mono[0]}, {mono[1]:.2e}") + ("; mean-filled fallback applied" if filled else ""), True),
        ("Labeled frames in more than one round (duplicates)", "0", str(dup), dup == 0),
    ]
    for rd in ROUNDS:
        ri = rinfo[rd]
        checks += [
            (f"Round {rd}: `labels.tsv` labels within {{keep, discard}}", "yes", yn(ri["labels"] <= {"keep", "discard"}), ri["labels"] <= {"keep", "discard"}),
            (f"Round {rd}: labels / keep / discard", f"600 / {ROUNDS[rd]['labels'][0]} / {ROUNDS[rd]['labels'][1]}", f"{ri['n']} / {ri['keep']} / {ri['discard']}",
             (ri["n"], ri["keep"], ri["discard"]) == (600, *ROUNDS[rd]["labels"])),
            (f"Round {rd}: `labels.tsv` rows whose name, file, band or `π` differ from `manifest.tsv` / manifest rows", f"0 / {ri['n']}", f"{ri['man_diff']} / {ri['n_man']}",
             ri["man_diff"] == 0 and ri["n_man"] == ri["n"]),
            (f"Round {rd}: population / sum of `1/π` over `manifest.tsv`", f"{ROUNDS[rd]['population']} / {ROUNDS[rd]['population']}.0", f"{int(pop[rd].sum())} / {ri['sum_w']:.1f}",
             int(pop[rd].sum()) == ROUNDS[rd]["population"] and abs(ri["sum_w"] - ROUNDS[rd]["population"]) < 0.05),
            (f"Round {rd}: labeled frames in an earlier sample", "0", str(ri["in_earlier"]), ri["in_earlier"] == 0),
        ]
    for k, ps in opened.items():
        got = sorted(set(ps))
        checks.append((f"Files opened under the {k} directory", "within " + ", ".join(f"`{p}`" for p in sorted(allowed[k])), ", ".join(f"`{p}`" for p in got) or "none",
                       set(got) <= allowed[k]))
    gate.table("| Check | Expected | This run | Result |", [(c, e, v, "pass" if ok else "FAIL") for c, e, v, ok in checks])

    print("## Wall time\n")
    gate.table(
        "| Stage | Seconds |",
        [
            ("Load (dump, face dump, EAR, manifests, labels)", f"{load_time:.0f}"),
            ("`HGB-mono/T` spot check (1 fit)", f"{mono_time:.0f}"),
            *[(f"`{k}`, 27 folds" + (" (both runs, `C` fallback)" if info[k]["fallback"] else ""), f"{info[k]['time']:.0f}") for k in LEARNED],
            ("Label-free cuts", f"{cut_time:.0f}"),
        ],
    )
    print(f"This run: {time.time() - start:.0f} s on {jobs} jobs.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "discardfit.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    if not all(ok for *_, ok in checks):
        raise SystemExit("a check failed")


if __name__ == "__main__":
    if len(sys.argv) not in (8, 9):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:8], int(sys.argv[8]) if len(sys.argv) > 8 else 8)
