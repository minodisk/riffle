"""Step 2: the held-out scores of the 69 candidates of the exploration on every
dump frame, fit as pre-registered in modelsearch.md, with the label-free
readings only.

Usage: python -I -B searchfit.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <discard-labels-dir> <strict-labels-dir> <retarget-dir> <out-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv` (not read here; the argument keeps the command line Step 3's);
<reject-filter-eval-dir> `scores_reject.tsv` (the stored `G0`);
<discard-labels-dir> `labels.tsv`, read for the training of the discard
target and `stack` only; <strict-labels-dir> the strict sample's
`manifest.tsv`, read for the population only (its `labels.tsv` is never
opened); <retarget-dir> `scores_retarget.tsv` (the stored `D0`); <out-dir>
holds Step 1's `lockbox.tsv`. Writes `scores_pick.tsv` (the pick fit set: the
20 exploration folders, lockbox rows `nan`) and `scores_discard.tsv` (the
discard fit set: all 27 folders), each `folder`, `file` and one column per
candidate (held out, higher keeps), into <out-dir>, prints the label-free
readings (Markdown) and saves them as `searchfit.out.md` there. Every fitted
candidate's held-out scores are cached in `<out-dir>/searchfit-cache/`, so a
rerun fits only what is missing. No score is read against the labels or the
picks beyond the pre-registered label-free `PL`.
"""

import os
import re
import sys
import time

import numpy as np
from joblib import Parallel, delayed
from sklearn.decomposition import PCA
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression
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

rf, gate, folder_gate = bf.rf, bf.gate, bf.folder_gate

FEATURES = {
    "T+C": (True, 0),
    "All@4": (True, 4),
    "All@8": (True, 8),
    "All@16": (True, 16),
    "All@32": (True, 32),
    "Emb@8": (False, 8),
    "Emb@32": (False, 32),
}
HGB_REG = dict(max_iter=100, learning_rate=0.05, max_leaf_nodes=7, l2_regularization=10.0, min_samples_leaf=50, early_stopping=False, random_state=0)
HGB_FORMS = {"HGB": rf.HGB, "HGB-reg": HGB_REG, "HGB-mono": rf.HGB}
FORMS = ["HGB", "HGB-reg", "HGB-mono", "LR-L2", "LR-L1"]
TARGETS = ["pick", "discard"]
TECH_SIGNS = [1, 1, 1, -1, -1, -1, 1, 0, 0, 0, 1]
N_CONTEXT = 3
CAP_FACTOR = 5
COMBOS = ["G0 ⊕ ef", "G0 | ef-rule", "stack"]
G0_RECIPE = "All@32 / HGB / pick"
TOL = 1e-5
MONO_ROWS = 200
MONO_GRID = 9


def candidates():
    out = []
    for feat in FEATURES:
        for form in FORMS:
            if form == "HGB-mono" and not FEATURES[feat][0]:
                continue
            for target in TARGETS:
                out.append((f"{feat} / {form} / {target}", feat, form, target))
    return out


def design(d, train_mask, tech, dims):
    """`folder_gate.design`'s recipe for any PCA size: the technical NaN mean-filled on the training folders, `dims` whitened PCA dims, every column standardized on them."""
    cols = []
    if tech:
        t = d["tech"].copy()
        mean = np.nanmean(t[train_mask], axis=0)
        idx = np.where(np.isnan(t))
        t[idx] = np.take(mean, idx[1])
        cols += [t, d["context"]]
    if dims:
        pca = PCA(n_components=dims, whiten=True, random_state=0).fit(d["emb"][train_mask])
        cols.append(pca.transform(d["emb"]))
    x = np.hstack(cols)
    mu, sd = x[train_mask].mean(axis=0), x[train_mask].std(axis=0)
    sd[sd == 0] = 1
    return (x - mu) / sd


def mono_cst(dims, target):
    sign = 1 if target == "pick" else -1
    return [sign * v for v in TECH_SIGNS] + [0] * (N_CONTEXT + dims)


def lr(form, c):
    if form == "LR-L1":
        return LogisticRegression(C=c, l1_ratio=1.0, solver="liblinear", max_iter=2000)
    return LogisticRegression(C=c, max_iter=2000)


def mean1(w):
    return w * len(w) / w.sum()


def choose_c(d, x, idx, y, w, target, form):
    """The inner `GroupKFold` choice of `C` over `gate.CS`: pick, the mean per-folder pick AUC (`folder_gate.choose_c`'s); discard, the mean weighted discard AUC per inner fold."""
    groups = d["folder_of"][idx]
    k = min(5, len(set(groups.tolist())))
    splits = list(GroupKFold(n_splits=k).split(idx, groups=groups))
    best, best_c = -1, None
    for c in gate.CS:
        vals = []
        for tr, va in splits:
            m = lr(form, c).fit(x[idx[tr]], y[tr], sample_weight=mean1(w[tr]))
            s = m.decision_function(x[idx[va]])
            if target == "pick":
                for fi in sorted(set(groups[va].tolist())):
                    fm = groups[va] == fi
                    a = folder_gate.auc(s[fm], y[va][fm].astype(bool))
                    if a is not None:
                        vals.append(a)
            elif 0 < y[va].sum() < len(va):
                vals.append(roc_auc_score(y[va], s, sample_weight=w[va]))
        v = float(np.mean(vals))
        if v > best:
            best, best_c = v, c
    return best_c


def training(d, s, train_folders, target, cap):
    tm = np.isin(d["folder_of"], train_folders)
    if target == "pick":
        idx = np.where(tm)[0]
        return tm, idx, d["pick"][idx].astype(int), d["weight"][idx]
    ltr = np.isin(s["folder"], train_folders)
    return tm, s["idx"][ltr], s["discard"][ltr].astype(int), np.minimum(s["w"][ltr], cap)


def fold(d, s, held, train_folders, feat, form, target, cap, c_fixed=None, filled=False):
    """One held-out folder of one candidate: fit on `train_folders`, score the held-out folder (higher keeps); also the lockbox rows the fit saw."""
    tech, dims = FEATURES[feat]
    tm, idx, y, w = training(d, s, train_folders, target, cap)
    x = design(d, tm, tech, dims)
    c = None
    if form in HGB_FORMS:
        if tech and not (form == "HGB-mono" and filled):
            x[:, : d["tech"].shape[1]][np.isnan(d["tech"])] = np.nan
        params = dict(HGB_FORMS[form])
        if form == "HGB-mono":
            params["monotonic_cst"] = mono_cst(dims, target)
        m = HistGradientBoostingClassifier(**params).fit(x[idx], y, sample_weight=mean1(w))
    else:
        c = c_fixed if c_fixed is not None else choose_c(d, x, idx, y, w, target, form)
        m = lr(form, c).fit(x[idx], y, sample_weight=mean1(w))
    sc = m.decision_function(x[d["folder_of"] == held])
    return held, c, sc if target == "pick" else -sc, int(d["box"][tm].sum())


def stack_fold(d, s, held, train_folders, g0, cap):
    """`stack`: a logistic on the fit set's held-out `G0`, `eye_focus` logit and `ln sharpness`, target `discard`, clipped weights; higher keeps."""
    tm, idx, y, w = training(d, s, train_folders, "discard", cap)
    x = np.column_stack([g0, d["tech"][:, 1], d["tech"][:, -1]])
    mean = np.nanmean(x[tm], axis=0)
    nan = np.where(np.isnan(x[:, 1:]))
    x[:, 1:][nan] = np.take(mean[1:], nan[1])
    mu, sd = x[tm].mean(axis=0), x[tm].std(axis=0)
    sd[sd == 0] = 1
    x = (x - mu) / sd
    m = LogisticRegression(C=1.0, max_iter=2000).fit(x[idx], y, sample_weight=mean1(w))
    return held, None, -m.decision_function(x[d["folder_of"] == held]), int(d["box"][tm].sum())


def collect(d, res):
    score = np.full(len(d["rows"]), np.nan)
    cs, box = {}, 0
    for held, c, sc, b in res:
        score[d["folder_of"] == held] = sc
        cs[held] = c
        box += b
    return score, cs, box


def norm_rank(v):
    """Average ranks of ties, scaled to 0-1."""
    if len(v) == 1:
        return np.array([0.5])
    _, inv, counts = np.unique(v, return_inverse=True, return_counts=True)
    ends = np.cumsum(counts)
    return ((ends - (counts - 1) / 2)[inv] - 1) / (len(v) - 1)


def combine(d, g0, ef):
    """`G0 ⊕ ef` and `G0 | ef-rule` from a fit set's `G0`; `nan` where `G0` is."""
    plus = np.full(len(g0), np.nan)
    rule = np.full(len(g0), np.nan)
    for fi in range(len(d["folders"])):
        m = np.where((d["folder_of"] == fi) & ~np.isnan(g0))[0]
        if not len(m):
            continue
        rg = norm_rank(g0[m])
        has = ~np.isnan(ef[m])
        plus[m] = rg
        if has.any():
            plus[m[has]] = (rg[has] + norm_rank(ef[m][has])) / 2
        upper = ~has | (ef[m] >= gate.GOOD_EYE_FOCUS)
        rule[m] = rg + upper
    return plus, rule


def mono_check(d, s, held, target, cap):
    """`HGB-mono` on `All@32`, one fold: whether the fit raises, and the largest step against the constraint over a grid of each constrained column's non-missing values, other columns as they are."""
    tm, idx, y, w = training(d, s, [f for f in range(len(d["folders"])) if f != held], target, cap)
    x = design(d, tm, True, 32)
    nt = d["tech"].shape[1]
    x[:, :nt][np.isnan(d["tech"])] = np.nan
    cst = mono_cst(32, target)
    try:
        m = HistGradientBoostingClassifier(**rf.HGB, monotonic_cst=cst).fit(x[idx], y, sample_weight=mean1(w))
    except Exception as e:  # noqa: BLE001
        return f"raised {type(e).__name__}: {e}", None
    test = np.where(d["folder_of"] == held)[0]
    rng = np.random.default_rng(0)
    worst = 0.0
    for j in range(nt):
        if cst[j] == 0:
            continue
        rows = test[~np.isnan(x[test, j])]
        rows = rng.choice(rows, min(MONO_ROWS, len(rows)), replace=False)
        grid = np.unique(np.quantile(x[idx, j][~np.isnan(x[idx, j])], np.linspace(0, 1, MONO_GRID)))
        vals = []
        for g in grid:
            xg = x[rows].copy()
            xg[:, j] = g
            vals.append(m.decision_function(xg))
        steps = np.diff(np.array(vals), axis=0) * cst[j]
        worst = max(worst, float(-steps.min()) if steps.size else 0.0)
    return "fit", worst


def slug(name):
    return re.sub(r"[^A-Za-z0-9@+.=-]+", "_", name)


def cell(name):
    return "`" + name.replace("|", "\\|") + "`"


def main(dump, eval_dir, reject_dir, labels_dir, strict_dir, retarget_dir, out_dir, jobs):
    start = time.time()
    strict_root = os.path.normcase(os.path.abspath(strict_dir))
    strict_opened = set()

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            if p.startswith(strict_root + os.sep):
                strict_opened.add(os.path.relpath(p, strict_root))

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
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    g0 = stored_rej["G0"]
    stored_ret, _ = rf.load_scores(os.path.join(retarget_dir, "scores_retarget.tsv"), d)
    d0 = stored_ret["D0"]
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])

    box_names = {r["folder"] for r in bf.read_tsv(os.path.join(out_dir, "lockbox.tsv")) if r["lockbox"] == "1"}
    box_f = sorted(folders.index(name) for name in box_names)
    expl_f = [fi for fi in range(nf) if fi not in box_f]
    d["box"] = np.isin(folder_of, box_f)

    labels = bf.read_tsv(os.path.join(labels_dir, "labels.tsv"))
    s = {
        "idx": np.array([index[(r["folder"], r["file"])] for r in labels]),
        "discard": np.array([r["label"] == "discard" for r in labels]),
        "pi": np.array([float(r["inclusion_prob"]) for r in labels]),
    }
    label_set = {r["label"] for r in labels}
    s["w"] = 1 / s["pi"]
    s["folder"] = folder_of[s["idx"]]
    cap = CAP_FACTOR * float(np.median(s["w"]))
    seen = np.zeros(n, dtype=bool)
    for r in bf.read_tsv(os.path.join(strict_dir, "manifest.tsv")):
        seen[index[(r["folder"], r["file"])]] = True
    pop = ~seen
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    sets = {"pick": expl_f, "discard": list(range(nf))}
    cache = os.path.join(out_dir, "searchfit-cache")
    os.makedirs(cache, exist_ok=True)
    cands = candidates()
    spot = max(range(nf), key=lambda fi: int((folder_of == fi).sum()))
    sm = folder_of == spot

    t = time.time()
    mono = {target: mono_check(d, s, spot, target, cap) for target in TARGETS}
    mono_time = time.time() - t
    filled = any(status != "fit" or worst > 1e-9 for status, worst in mono.values())
    print(f"HGB-mono check: {mono}, filled fallback {filled}", file=sys.stderr, flush=True)

    scores = {k: {} for k in sets}
    info = {k: {} for k in sets}

    with Parallel(n_jobs=jobs) as par:

        def run(fs, name, task):
            path = os.path.join(cache, f"{fs}__{slug(name)}.npz")
            if os.path.exists(path):
                z = np.load(path)
                r = {"score": z["score"], "cs": z["cs"], "box": int(z["box"]), "time": float(z["time"]), "cached": True}
            else:
                t = time.time()
                score, cs, box = collect(d, par(task(h, [f for f in sets[fs] if f != h]) for h in sets[fs]))
                r = {"score": score, "cs": np.array([np.nan if cs[h] is None else cs[h] for h in sets[fs]]), "box": box, "time": time.time() - t, "cached": False}
                np.savez(path, **{k: r[k] for k in ("score", "cs", "box", "time")})
            print(f"{fs} {name}: {r['time']:.0f} s{' (cached)' if r['cached'] else ''}", file=sys.stderr, flush=True)
            return r

        for fs in sets:
            for name, feat, form, target in cands:
                r = run(fs, name, lambda h, tr: delayed(fold)(d, s, h, tr, feat, form, target, cap, None, filled))
                if form.startswith("LR") and target == "discard":
                    vals, cnt = np.unique(r["cs"], return_counts=True)
                    r["chosen"] = r["cs"]
                    if cnt.max() <= len(sets[fs]) / 2:
                        fb = run(fs, name + " C=1", lambda h, tr: delayed(fold)(d, s, h, tr, feat, form, target, cap, 1.0, filled))
                        fb["chosen"], fb["fallback"] = r["cs"], True
                        fb["time"] += r["time"]
                        r = fb
                info[fs][name] = r
                scores[fs][name] = r["score"]
            g = g0 if fs == "discard" else scores[fs][G0_RECIPE]
            t = time.time()
            plus, rule = combine(d, g, ef)
            for name, sc in zip(COMBOS[:2], (plus, rule)):
                scores[fs][name] = sc
                info[fs][name] = {"score": sc, "box": 0, "time": (time.time() - t) / 2, "cached": False}
            info[fs]["stack"] = run(fs, "stack", lambda h, tr: delayed(stack_fold)(d, s, h, tr, g, cap))
            scores[fs]["stack"] = info[fs]["stack"]["score"]

    t = time.time()
    _, _, d0_spot, _ = fold(d, s, spot, [f for f in range(nf) if f != spot], "All@32", "HGB", "discard", np.inf)
    others = [f for f in range(nf) if f != spot]
    tm = np.isin(folder_of, others)
    x_full, x_full_ref = design(d, tm, True, 32), folder_gate.design(d, tm, "full")
    x_noemb, x_noemb_ref = design(d, tm, True, 0), folder_gate.design(d, tm, "noemb")
    check_time = time.time() - t

    names = [c[0] for c in cands] + COMBOS
    for fs, fname in (("pick", "scores_pick.tsv"), ("discard", "scores_discard.tsv")):
        with open(os.path.join(out_dir, fname), "w", encoding="utf-8", newline="") as out:
            out.write("folder\tfile\t" + "\t".join(names) + "\n")
            for i, f in enumerate(rows):
                out.write(f"{f['folder']}\t{f['file']}\t" + "\t".join(repr(float(scores[fs][k][i])) for k in names) + "\n")

    print("# Model-search fits: held-out scores and label-free readings\n")
    print(
        "No candidate score of this step is read against `labels.tsv` or against the picks: the discard labels and the picks enter training "
        "only, and no `DP`, `RR`, `P@q` or AUC is computed. The one reading that touches the picks is the pre-registered label-free `PL` of the "
        "discard task's cuts (discard fit set).\n"
    )

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{nf} / {n} / {missing}"),
            ("Dump picks / strict-sample frames / population", f"{int(pick.sum())} / {int(seen.sum())} / {int(pop.sum())}"),
            ("Lockbox folders / frames / picks", f"{len(box_f)} / {int(d['box'].sum())} / {int((pick & d['box']).sum())}"),
            ("Exploration folders / frames / picks", f"{len(expl_f)} / {int((~d['box']).sum())} / {int((pick & ~d['box']).sum())}"),
            ("Labeled frames / discard / keep / in the lockbox folders", f"{len(labels)} / {int(s['discard'].sum())} / {int((~s['discard']).sum())} / {int(d['box'][s['idx']].sum())}"),
            ("Median sampled `1/π` / weight cap (5 x the median) / labeled frames clipped", f"{np.median(s['w']):.3f} / {cap:.3f} / {int((s['w'] > cap).sum())}"),
            ("Candidates: fitted (designs x targets) / combinations / total", f"{len(cands)} / {len(COMBOS)} / {len(cands) + len(COMBOS)}"),
            ("`HGB-mono` technical columns", "mean-filled (fallback)" if filled else "NaN restored, as `HGB`"),
        ],
    )

    print("## Per-fold training\n")
    print(
        "Per fit set and held-out folder: the pick target's training frames and picks (every frame of the other folders of the set), and the "
        "discard target's labeled training frames, `discard` among them and the Kish effective sample size `(Σw)² / Σw²` of `1/π` and of the "
        "clipped weights. Every discard-target candidate and `stack` train on these frames and weights within a fit set. Lockbox rows: the "
        "lockbox frames inside the fold's training folders (PCA, fill, standardization and fit).\n"
    )
    ess = {}
    for fs in sets:
        frows, e_raw, e_clip = [], [], []
        for h in sets[fs]:
            tr = [f for f in sets[fs] if f != h]
            tm = np.isin(folder_of, tr)
            ltr = np.isin(s["folder"], tr)
            a, b = retargetfit.kish(s["w"][ltr]), retargetfit.kish(np.minimum(s["w"][ltr], cap))
            e_raw.append(a)
            e_clip.append(b)
            frows.append([f"`{folders[h]}`", int((folder_of == h).sum()), int(tm.sum()), int((pick & tm).sum()), int(ltr.sum()), int((s["discard"] & ltr).sum()),
                          f"{a:.1f}", f"{b:.1f}", int(d["box"][tm].sum())])
        ess[fs] = (e_raw, e_clip, frows)
        print(f"### {fs.capitalize()} fit set ({len(sets[fs])} folds)\n")
        gate.table("| Held-out folder | Frames | Pick training frames | Picks | Labeled training frames | Discard | ESS `1/π` | ESS clipped | Lockbox rows |", frows)
    gate.table(
        "| Reading | Pick fit set min / max | Discard fit set min / max |",
        [
            ("Labeled training frames", *(f"{min(r[4] for r in ess[fs][2])} / {max(r[4] for r in ess[fs][2])}" for fs in sets)),
            ("Kish ESS of `1/π` (no clip)", *(f"{min(ess[fs][0]):.1f} / {max(ess[fs][0]):.1f}" for fs in sets)),
            ("Kish ESS of the clipped weights", *(f"{min(ess[fs][1]):.1f} / {max(ess[fs][1]):.1f}" for fs in sets)),
        ],
    )

    print("## `C` chosen per fold (logistic forms)\n")
    print(
        f"Count of folds choosing each `C` of `gate.CS` by the inner `GroupKFold`. Fallback (discard target only): no `C` chosen by more than half "
        f"the folds of the set, so the candidate is refit with `C` = 1.0 in every fold; its chosen counts are those of the first run.\n"
    )
    crows = []
    for fs in sets:
        for name, feat, form, target in cands:
            if not form.startswith("LR"):
                continue
            r = info[fs][name]
            ch = r.get("chosen", r["cs"])
            crows.append([fs, cell(name)] + [int((ch == c).sum()) for c in gate.CS] + ["yes, `C` = 1.0" if r.get("fallback") else "no"])
    gate.table("| Fit set | Candidate | " + " | ".join(str(c) for c in gate.CS) + " | Fallback |", crows)

    print("## Label-free cuts: removed share and pick loss (discard fit set)\n")
    print(
        f"Per folder on the full dump, the bottom `floor(q * n / 100)` frames by the discard fit set's score, ties by the seed-0 draw "
        f"(`bf.graded_cut`). Removed: the share of the population's frames inside the cut (full dump's after the slash). `PL`: the share of the "
        f"dump's picks inside the cut (full dump; population after the slash). The pre-registered label-free conditions: `PL` (full dump) <= "
        f"{df.PL_BAR:.0f}% and removed (population) >= {df.REMOVED_BAR}%. `G0` and `D0` (stored) are references, not candidates.\n"
    )
    lrows, verdict = [], {}
    for k, sc in [("G0", g0), ("D0", d0)] + [(k, scores["discard"][k]) for k in names]:
        for q in df.QS:
            removed, out = bf.graded_cut(sc, d, tb, q)
            cover = ~out
            pl = 100 * (removed & pick & cover).sum() / (pick & cover).sum()
            pl_pop = 100 * (removed & pick & cover & pop).sum() / (pick & cover & pop).sum()
            rem_pop = 100 * (removed & cover & pop).sum() / (cover & pop).sum()
            rem_dump = 100 * (removed & cover).sum() / cover.sum()
            ok_pl, ok_rem = pl <= df.PL_BAR, rem_pop >= df.REMOVED_BAR
            verdict[(k, q)] = (ok_pl, ok_rem)
            lrows.append([cell(k), f"q{q}", int(out.sum()), f"{rem_pop:.2f} / {rem_dump:.2f}", f"{pl:.2f} / {pl_pop:.2f}", "yes" if ok_pl else "no", "yes" if ok_rem else "no"])
    gate.table("| Score | Cut | Frames out | Removed % | `PL` % | `PL` <= 5% | Removed >= 4.5% |", lrows)
    print("Candidates failing a label-free condition, per cut:\n")
    gate.table(
        "| Cut | Fail `PL` <= 5% | Fail removed >= 4.5% | Pass both |",
        [[f"q{q}", sum(not verdict[(k, q)][0] for k in names), sum(not verdict[(k, q)][1] for k in names), sum(all(verdict[(k, q)]) for k in names)] for q in df.QS],
    )

    print("## Checks\n")
    tol = TOL * np.maximum(1, np.abs(g0[sm]))
    d_g0 = np.abs(scores["discard"][G0_RECIPE][sm] - g0[sm])
    d_g0_all = np.abs(scores["discard"][G0_RECIPE] - g0)
    d_d0 = np.abs(d0_spot - d0[sm])
    tol_d0 = TOL * np.maximum(1, np.abs(d0[sm]))
    box_seen = sum(info["pick"][k]["box"] for k in names)
    strict_train = int(seen[s["idx"]].sum())
    checks = [
        (f"Local `design` at `All@32` vs `folder_gate.design(..., \"full\")`, fold `{folders[spot]}`: max abs difference", 0, f"{np.abs(x_full - x_full_ref).max():.2e}", bool(np.abs(x_full - x_full_ref).max() <= 1e-12)),
        ("Local `design` at `T+C` vs `folder_gate.design(..., \"noemb\")`, same fold: max abs difference", 0, f"{np.abs(x_noemb - x_noemb_ref).max():.2e}", bool(np.abs(x_noemb - x_noemb_ref).max() <= 1e-12)),
        (f"`{G0_RECIPE}` (discard fit set, all 27 folders) vs stored `G0`, spot fold `{folders[spot]}` ({int(sm.sum())} frames): max abs difference",
         f"<= {TOL:g} x max(1, abs)", f"{d_g0.max():.2e}", bool((d_g0 <= tol).all())),
        (f"`{G0_RECIPE}` (discard fit set) vs stored `G0`, every frame (reading): max abs difference", "-", f"{d_g0_all.max():.2e}", True),
        ("`All@32 / HGB / discard` with the clip disabled vs stored `D0`, same spot fold: max abs difference", f"<= {TOL:g} x max(1, abs)", f"{d_d0.max():.2e}", bool((d_d0 <= tol_d0).all())),
    ]
    for target in TARGETS:
        status, worst = mono[target]
        ok = status == "fit" and worst <= 1e-9
        checks.append((f"`HGB-mono` (`All@32`, {target}) on the spot fold: fit, and the largest step against a constraint over the non-missing grid", "fit, 0",
                       (status if worst is None else f"{status}, {worst:.2e}") + ("" if ok else "; mean-filled fallback applied"), ok or filled))
    pick_ok = all(np.isfinite(scores["pick"][k][~d["box"]]).all() and np.isnan(scores["pick"][k][d["box"]]).all() for k in names)
    disc_ok = all(np.isfinite(scores["discard"][k]).all() for k in names)
    checks += [
        (f"Pick fit set: every candidate finite on the {int((~d['box']).sum())} exploration frames and `nan` on the {int(d['box'].sum())} lockbox frames", "yes", "yes" if pick_ok else "no", pick_ok),
        (f"Discard fit set: every candidate finite on every dump frame ({n})", "yes", "yes" if disc_ok else "no", disc_ok),
        ("Lockbox frames inside the training folders of any pick-fit-set fold, summed over the candidates and folds", 0, box_seen, box_seen == 0),
        ("Labeled training frames that are in the strict sample", 0, strict_train, strict_train == 0),
        ("Files opened under the strict-labels directory", "`manifest.tsv` only", ", ".join(f"`{p}`" for p in sorted(strict_opened)) or "none", strict_opened <= {"manifest.tsv"}),
        ("`labels.tsv` labels within {keep, discard}", "yes", "yes" if label_set <= {"keep", "discard"} else "no", label_set <= {"keep", "discard"}),
    ]
    gate.table("| Check | Expected | This run | Result |", [(k, e, v, "pass" if ok else "FAIL") for k, e, v, ok in checks])

    print("## Wall time\n")
    print("Seconds per candidate and fit set (a logistic discard-target candidate under the `C` fallback: both runs). A cached run shows the time of the run that wrote the cache.\n")
    wrows = []
    for k in names:
        p, q = info["pick"][k], info["discard"][k]
        wrows.append([cell(k), f"{p['time']:.0f}" + (" (cached)" if p["cached"] else ""), f"{q['time']:.0f}" + (" (cached)" if q["cached"] else "")])
    gate.table("| Candidate | Pick fit set | Discard fit set |", wrows)
    fit_total = sum(info[fs][k]["time"] for fs in sets for k in names)
    gate.table(
        "| Stage | Seconds |",
        [
            ("Load (dump, stored `G0` / `D0`, lockbox, labels, strict manifest)", f"{load_time:.0f}"),
            ("`HGB-mono` spot checks (2 fits)", f"{mono_time:.0f}"),
            ("Fits, every candidate and fit set (sum, cached runs at their recorded time)", f"{fit_total:.0f}"),
            ("Spot-fold checks (1 fit, 4 designs)", f"{check_time:.0f}"),
        ],
    )
    print(f"This run: {time.time() - start:.0f} s on {jobs} jobs.\n")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "searchfit.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))
    if not all(ok for *_, ok in checks):
        raise SystemExit("a check failed")


if __name__ == "__main__":
    if len(sys.argv) not in (8, 9):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:8], int(sys.argv[8]) if len(sys.argv) > 8 else 8)
