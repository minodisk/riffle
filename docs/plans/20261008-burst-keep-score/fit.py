"""The Step 3 fits: thresholds and combinations from the picks, held out by
folder.

Usage: python -I fit.py <dump-dir> [<frozen-fail-check.json>]

Prints the tables of fit.md (Markdown) on stdout; with a second argument,
also writes the chosen variant (CHOSEN below), fitted on every ARW folder,
to that file. Reads the Step 1 dumps through metrics.py (same folder), so
the frames, labels, bursts and scorable bursts are Step 2's.

Every variant is fitted on the scorable bursts of the sidecar-labeled ARW
folders with leave-one-folder-out: for each folder, the thresholds or
coefficients come from the other folders and the metrics are taken on the
held-out one, then the held-out counts are pooled. The "training" columns
fit and evaluate on every folder. The DNG blocks have too few picks to fit
on (68 and 32); the threshold variants fitted on every ARW folder are
applied to them as a transfer check.

Variants:
- (i) Per-feature thresholds from the picks' distribution: for each feature
  of a set, the p-th percentile of the training picks having it (the
  (1 - p)-th for the pose axes, as -|angle|); a frame fails if any feature
  crosses its threshold, and a frame lacking a feature is not failed by it.
  p = 1, 2, 5%, and a "matched" form whose common p is the largest that
  keeps the union's training pick false-fail at or under 1 / 2 / 5%.
- (ii) Positive-unlabeled logistic: picks 1, non-picks 0, a plain
  maximum-likelihood logistic (Newton) on the standardized features below,
  missing values imputed with the training mean and flagged by an
  indicator. A frame fails if its score is under the score that keeps 99 /
  98 / 95% of the training picks. Elkan and Noto's c = E[g | pick] gives
  the implied share of technically-OK frames among the non-picks; the
  ranking and the pick-kept threshold do not depend on it.
- (iii) Pairwise within-burst logistic: pick minus non-pick feature
  differences of one burst, label 1, no intercept.
- (iv) The pointwise logistic as a ranking, which is (ii)'s fit.
"""

import json
import math
import operator
import os
import random
import statistics
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import metrics  # noqa: E402

GAP = 1000
KEEPS = (0.99, 0.98, 0.95)
TARGETS = (0.01, 0.02, 0.05)
PS = (0.01, 0.02, 0.05)
MIN_FOLDER_PICKS = 20

THRESH_FEATS = {
    "rel": lambda f: f["rel"],
    "abs": lambda f: f["sharp"],
    "ef": lambda f: f["ef"],
    "eo": lambda f: f["eo"],
    "yaw": lambda f: None if f["yaw"] is None else -abs(f["yaw"]),
    "pitch": lambda f: None if f["pitch"] is None else -abs(f["pitch"]),
    "roll": lambda f: None if f["roll"] is None else -abs(f["roll"]),
}
SETS = [
    ("sharpness (rel)", ("rel",)),
    ("sharpness (abs)", ("abs",)),
    ("rel + abs floor", ("rel", "abs")),
    ("`eye_focus`", ("ef",)),
    ("eyes open", ("eo",)),
    ("`eye_focus` + eyes open", ("ef", "eo")),
    ("rel + `eye_focus` + eyes open", ("rel", "ef", "eo")),
    ("rel + abs + `eye_focus` + eyes open", ("rel", "abs", "ef", "eo")),
    ("all six (rel, ef, eo, yaw, pitch, roll)", ("rel", "ef", "eo", "yaw", "pitch", "roll")),
]


def clip(v, hi):
    return min(abs(v), hi)


def logit(p):
    p = min(max(p, 1e-3), 1 - 1e-3)
    return math.log(p / (1 - p))


# (name, group, value or None)
LOGIT_FEATS = [
    ("ln rel", "sharp", lambda f: math.log(max(f["rel"], 1e-3))),
    ("`eye_focus` logit", "ef", lambda f: None if f["ef"] is None else logit(f["ef"])),
    ("eyes open", "eo", lambda f: f["eo"]),
    ("\\|yaw\\| (<= 90)", "pose", lambda f: None if f["yaw"] is None else clip(f["yaw"], 90)),
    ("\\|pitch\\| (<= 60)", "pose", lambda f: None if f["pitch"] is None else clip(f["pitch"], 60)),
    ("\\|roll\\| (<= 60)", "pose", lambda f: None if f["roll"] is None else clip(f["roll"], 60)),
    ("\\|yaw - median\\| (<= 60)", "posediff", lambda f: None if f["dyaw"] is None else clip(f["dyaw"], 60)),
    ("\\|pitch - median\\| (<= 45)", "posediff", lambda f: None if f["dpitch"] is None else clip(f["dpitch"], 45)),
    ("\\|roll - median\\| (<= 45)", "posediff", lambda f: None if f["droll"] is None else clip(f["droll"], 45)),
]
INDICATORS = [
    ("no cue face", ("ef",), lambda f: f["ef"] is None),
    ("no eyes judgment", ("eo",), lambda f: f["eo"] is None),
    ("no pose", ("pose", "posediff"), lambda f: f["yaw"] is None),
]
GROUPS = ("sharp", "ef", "eo", "pose", "posediff")

CHOSEN = {"set": ("rel", "ef", "eo"), "target": 0.01}


# ---------------------------------------------------------------- data


def arw_bursts(folders, gap):
    rng = random.Random(0)
    by = {}
    for name, kind, frames in folders:
        if kind != "arw":
            continue
        bs = metrics.scorable_of(metrics.bursts_of(frames, gap), rng)
        if bs:
            by[name] = bs
    return by


def kind_bursts(folders, kind, gap):
    rng = random.Random(0)
    out = []
    for name, k, frames in folders:
        if k == kind:
            out.extend(metrics.scorable_of(metrics.bursts_of(frames, gap), rng))
    return out


def flat(bursts):
    return [f for b in bursts for f in b]


# ---------------------------------------------------------------- counting


def count(bursts, fails):
    """(picks failed, picks, non-picks flagged, non-picks, sum of kept shares, bursts)."""
    c = [0, 0, 0, 0, 0.0, 0]
    for b in bursts:
        x = [fails(f) for f in b]
        c[4] += 1 - sum(x) / len(b)
        c[5] += 1
        for f, y in zip(b, x):
            if f["pick"]:
                c[0] += y
                c[1] += 1
            else:
                c[2] += y
                c[3] += 1
    return c


def add(a, b):
    return [x + y for x, y in zip(a, b)]


def rate(n, d):
    return n / d if d else None


def p(x, digits=1):
    return "-" if x is None else f"{100 * x:.{digits}f}%"


def summary(c):
    return rate(c[0], c[1]), rate(c[2], c[3]), rate(c[4], c[5])


def spread(per):
    """Per held-out folder with enough picks: the non-pick flag rates' and
    pick false-fail rates' (median, min, max, stdev)."""
    flags = [rate(c[2], c[3]) for c in per.values() if c[1] >= MIN_FOLDER_PICKS and c[3]]
    ffs = [rate(c[0], c[1]) for c in per.values() if c[1] >= MIN_FOLDER_PICKS]
    return flags, ffs


def lofo(by, fit, apply):
    """Held-out counts pooled and per folder, and the training counts."""
    pooled, per = [0, 0, 0, 0, 0.0, 0], {}
    for held in by:
        train = [b for n, bs in by.items() if n != held for b in bs]
        model = fit(train)
        c = count(by[held], apply(model))
        per[held] = c
        pooled = add(pooled, c)
    allb = [b for bs in by.values() for b in bs]
    model = fit(allb)
    return pooled, per, count(allb, apply(model)), model


# ---------------------------------------------------------------- (i) thresholds


def thresholds(train, feats, q):
    out = {}
    for k in feats:
        t = metrics.threshold(train, THRESH_FEATS[k], 1 - q)
        if t is not None:
            out[k] = t
    return out


def fails_thresholds(model):
    items = [(THRESH_FEATS[k], t) for k, t in strip(model).items()]

    def fails(f):
        for score, t in items:
            v = score(f)
            if v is not None and v < t:
                return True
        return False

    return fails


def fit_matched(feats, target):
    def fit(train):
        picks = [f for b in train for f in b if f["pick"]]
        lo, hi = 0.0, 0.2
        best = thresholds(train, feats, 0.0)
        for _ in range(24):
            mid = (lo + hi) / 2
            m = thresholds(train, feats, mid)
            fails = fails_thresholds(m)
            ff = sum(fails(f) for f in picks) / len(picks)
            if ff <= target:
                lo, best = mid, m
            else:
                hi = mid
        best["_q"] = lo
        return best

    return fit


def strip(model):
    return {k: v for k, v in model.items() if not k.startswith("_")}


# ---------------------------------------------------------------- logistic


def solve(a, b):
    n = len(b)
    m = [row[:] + [b[i]] for i, row in enumerate(a)]
    for c in range(n):
        piv = max(range(c, n), key=lambda r: abs(m[r][c]))
        if abs(m[piv][c]) < 1e-12:
            raise ArithmeticError("singular")
        m[c], m[piv] = m[piv], m[c]
        for r in range(n):
            if r != c:
                k = m[r][c] / m[c][c]
                if k:
                    row, crow = m[r], m[c]
                    for j in range(c, n + 1):
                        row[j] -= k * crow[j]
    return [m[i][n] / m[i][i] for i in range(n)]


def sigmoid(z):
    if z >= 0:
        return 1 / (1 + math.exp(-z))
    e = math.exp(z)
    return e / (1 + e)


def newton(cols, y, ridge=0.0):
    """Maximum-likelihood logistic on column-major X (the intercept, if any,
    is a column of ones). Returns (weights, iterations, converged)."""
    d, n = len(cols), len(y)
    w = [0.0] * d
    mul = operator.mul
    for it in range(1, 51):
        z = [0.0] * n
        for j in range(d):
            if w[j]:
                wj = w[j]
                z = [a + wj * b for a, b in zip(z, cols[j])]
        pr = [sigmoid(v) for v in z]
        r = [a - b for a, b in zip(y, pr)]
        wt = [q * (1 - q) for q in pr]
        g = [sum(map(mul, r, cols[j])) - ridge * w[j] for j in range(d)]
        wc = [[a * b for a, b in zip(wt, cols[j])] for j in range(d)]
        h = [[0.0] * d for _ in range(d)]
        for j in range(d):
            for k in range(j, d):
                v = sum(map(mul, wc[j], cols[k]))
                h[j][k] = h[k][j] = v + (ridge if j == k else 0.0)
        step = solve(h, g)
        w = [a + b for a, b in zip(w, step)]
        if max(abs(s) for s in step) < 1e-7:
            return w, it, True
    return w, 50, False


def design(groups):
    feats = [(n, fn) for n, g, fn in LOGIT_FEATS if g in groups]
    inds = [(n, fn) for n, gs, fn in INDICATORS if any(g in groups for g in gs)]
    return feats, inds


def standardizer(frames, feats):
    st = []
    for _, fn in feats:
        vals = [v for f in frames for v in [fn(f)] if v is not None]
        mu = statistics.fmean(vals)
        sd = statistics.pstdev(vals) or 1.0
        st.append((mu, sd))
    return st


def row(f, feats, inds, st):
    x = []
    for (_, fn), (mu, sd) in zip(feats, st):
        v = fn(f)
        x.append(0.0 if v is None else (v - mu) / sd)
    for _, fn in inds:
        x.append(1.0 if fn(f) else 0.0)
    return x


def fit_pointwise(groups):
    feats, inds = design(groups)

    def fit(train):
        frames = flat(train)
        st = standardizer(frames, feats)
        rows = [row(f, feats, inds, st) for f in frames]
        cols = [[1.0] * len(rows)] + [list(c) for c in zip(*rows)]
        y = [1.0 if f["pick"] else 0.0 for f in frames]
        ridge = 0.0
        try:
            w, it, ok = newton(cols, y)
        except ArithmeticError:
            ok = False
        if not ok:
            ridge = 1e-3
            w, it, ok = newton(cols, y, ridge)
        return {"feats": feats, "inds": inds, "st": st, "w": w, "it": it, "ok": ok, "ridge": ridge}

    return fit


def fit_pairwise(groups):
    feats, inds = design(groups)

    def fit(train):
        frames = flat(train)
        st = standardizer(frames, feats)
        diffs = []
        for b in train:
            rows = [(f["pick"], row(f, feats, inds, st)) for f in b]
            pos = [x for k, x in rows if k]
            neg = [x for k, x in rows if not k]
            for a in pos:
                for c in neg:
                    diffs.append([u - v for u, v in zip(a, c)])
        cols = [list(c) for c in zip(*diffs)]
        y = [1.0] * len(diffs)
        ridge = 0.0
        try:
            w, it, ok = newton(cols, y)
        except ArithmeticError:
            ok = False
        if not ok:
            ridge = 1e-3
            w, it, ok = newton(cols, y, ridge)
        return {"feats": feats, "inds": inds, "st": st, "w": [0.0] + w, "it": it, "ok": ok, "ridge": ridge, "pairs": len(diffs)}

    return fit


def linear(model):
    feats, inds, st, w = model["feats"], model["inds"], model["st"], model["w"]
    return lambda f: w[0] + sum(a * b for a, b in zip(w[1:], row(f, feats, inds, st)))


def with_keep(fit, keep):
    def fit2(train):
        m = fit(train)
        s = linear(m)
        m["t"] = metrics.threshold(train, s, keep)
        return m

    return fit2


def fails_linear(model):
    s, t = linear(model), model["t"]
    return lambda f: s(f) < t


def heldout_scores(by, fit):
    out, models = [], []
    for held in by:
        train = [b for n, bs in by.items() if n != held for b in bs]
        m = fit(train)
        models.append(m)
        s = linear(m)
        for b in by[held]:
            for f in b:
                f["s"] = s(f)
            out.append(b)
    return out, models


# ---------------------------------------------------------------- report


def table(head, rows):
    metrics.table(head, rows)


FLAG_HEAD = (
    "| Variant | Held-out pick false-fail | Held-out non-pick flagged | Mean share kept | "
    "Per-folder flagged median (min-max) | Per-folder false-fail max | Training false-fail / flagged |"
)


def flag_row(name, pooled, per, train):
    ff, fl, kept = summary(pooled)
    flags, ffs = spread(per)
    tff, tfl, _ = summary(train)
    return [
        name,
        f"{p(ff)} ({pooled[0]})",
        f"{p(fl)} ({pooled[2]})",
        p(kept),
        f"{p(statistics.median(flags))} ({p(min(flags))}-{p(max(flags))})" if flags else "-",
        p(max(ffs)) if ffs else "-",
        f"{p(tff)} / {p(tfl)}",
    ]


def show_thresholds(model):
    parts = []
    for k, t in strip(model).items():
        if k in ("yaw", "pitch", "roll"):
            parts.append(f"\\|{k}\\| <= {-t:.1f}")
        elif k == "abs":
            parts.append(f"abs >= {t:.1f}")
        else:
            parts.append(f"{k} >= {t:.3f}")
    return ", ".join(parts)


def compare(by, base_per, var_per):
    """Per held-out folder: variant minus baseline non-pick flag rate."""
    d = []
    for n in by:
        b, v = base_per[n], var_per[n]
        if b[1] >= MIN_FOLDER_PICKS and b[3]:
            d.append(rate(v[2], v[3]) - rate(b[2], b[3]))
    return d


def report(folders, frozen_path):
    by = arw_bursts(folders, GAP)
    allb = [b for bs in by.values() for b in bs]
    fr = flat(allb)
    print(f"Sidecar-labeled ARW block, gap {GAP} ms: {len(by)} folders with a scorable burst, "
          f"{len(allb)} scorable bursts, {sum(f['pick'] for f in fr)} picks, "
          f"{sum(not f['pick'] for f in fr)} non-picks. Per-folder spreads are over the "
          f"{sum(1 for bs in by.values() if sum(f['pick'] for b in bs for f in b) >= MIN_FOLDER_PICKS)} "
          f"held-out folders with at least {MIN_FOLDER_PICKS} picks.\n")

    print("## (i) Per-feature thresholds from the picks\n")
    print("### Fixed percentile\n")
    print("Each feature of the set at the p-th percentile of the training picks; a frame fails if any feature crosses.\n")
    rows = []
    for name, feats in SETS:
        for q in PS:
            pooled, per, train, model = lofo(by, lambda tr, f=feats, q=q: thresholds(tr, f, q), fails_thresholds)
            rows.append(flag_row(f"{name}, p {q:.0%}", pooled, per, train) + [show_thresholds(model)])
    table(FLAG_HEAD + " Thresholds (all folders) |", rows)

    print("### Matched to a pick false-fail\n")
    print(
        "One common p per set, the largest whose union fails at most the target share of the "
        "training picks. \"Gain\" is the held-out non-pick flag rate over the sharpness-alone "
        "(rel) row of the same target, pooled; the per-folder gain is the mean (sd) of the "
        "per-folder differences and the share of folders where it is positive.\n"
    )
    matched = {}
    rows = []
    for target in TARGETS:
        base = None
        for name, feats in SETS:
            pooled, per, train, model = lofo(by, fit_matched(feats, target), fails_thresholds)
            matched[(feats, target)] = (pooled, per, train, model)
            if feats == ("rel",):
                base = (pooled, per)
            gain = summary(pooled)[1] - summary(base[0])[1]
            d = compare(by, base[1], per)
            dtxt = f"{100 * statistics.fmean(d):+.1f} ({100 * statistics.pstdev(d):.1f}), {sum(x > 0 for x in d)}/{len(d)}"
            rows.append(
                flag_row(f"{name}, target {target:.0%}", pooled, per, train)
                + [f"{100 * gain:+.1f} pt", dtxt, f"{model['_q']:.4f}", show_thresholds(model)]
            )
    table(FLAG_HEAD + " Gain over rel | Per-folder gain mean (sd), folders > 0 | p (all folders) | Thresholds (all folders) |", rows)

    six = SETS[-1][1]
    for title, full_set in (("the chosen set", CHOSEN["set"]), ("all six", six)):
        print(f"### Drop one feature ({title}, matched)\n")
        rows = []
        for target in TARGETS:
            for drop in (None,) + full_set:
                feats = tuple(k for k in full_set if k != drop)
                pooled, per, train, model = lofo(by, fit_matched(feats, target), fails_thresholds)
                rows.append(flag_row(f"target {target:.0%}, " + ("none dropped" if drop is None else f"without {drop}"), pooled, per, train))
        table(FLAG_HEAD, rows)

    print("### Transfer to the DNG blocks\n")
    print("Thresholds fitted on every ARW folder (matched), applied to the DNG blocks (no AF point, so no `eye_focus`).\n")
    rows = []
    for kind in ("dng-sidecar", "dng-output"):
        kb = kind_bursts(folders, kind, GAP)
        for target in TARGETS:
            for feats in (("rel",), ("rel", "ef", "eo"), six):
                model = matched[(feats, target)][3]
                c = count(kb, fails_thresholds(strip(model)))
                ff, fl, kept = summary(c)
                rows.append([kind, f"target {target:.0%}", ", ".join(feats), f"{p(ff)} ({c[0]}/{c[1]})", f"{p(fl)} ({c[2]}/{c[3]})", p(kept)])
    table("| Block | Fit | Set | Pick false-fail | Non-pick flagged | Mean share kept |", rows)

    print("## (ii) Positive-unlabeled logistic\n")
    print("Thresholded at the score that keeps 99 / 98 / 95% of the training picks.\n")
    rows = []
    variants = [("all features", GROUPS), ("no pose", ("sharp", "ef", "eo"))]
    for name, groups in variants:
        for keep in KEEPS:
            pooled, per, train, model = lofo(by, with_keep(fit_pointwise(groups), keep), fails_linear)
            rows.append(flag_row(f"{name}, keep {keep:.0%}", pooled, per, train))
    table(FLAG_HEAD, rows)

    print("### Drop one group (all features, keep 98%)\n")
    rows = []
    for drop in (None,) + GROUPS:
        groups = tuple(g for g in GROUPS if g != drop)
        pooled, per, train, model = lofo(by, with_keep(fit_pointwise(groups), 0.98), fails_linear)
        rows.append(flag_row("none dropped" if drop is None else f"without {drop}", pooled, per, train))
    table(FLAG_HEAD, rows)

    full = fit_pointwise(GROUPS)(allb)
    print("### Coefficients (all folders, standardized features)\n")
    names = ["intercept"] + [n for n, _ in full["feats"]] + [n for n, _ in full["inds"]]
    print(f"Newton iterations {full['it']}, converged {full['ok']}, ridge {full['ridge']}.\n")
    table("| Term | Coefficient |", [[n, f"{w:+.4f}"] for n, w in zip(names, full["w"])])

    s = linear(full)
    g = [sigmoid(s(f)) for f in fr]
    picks = [x for x, f in zip(g, fr) if f["pick"]]
    unl = [x for x, f in zip(g, fr) if not f["pick"]]
    c = statistics.fmean(picks)
    ok = sum((1 - c) / c * x / (1 - x) for x in unl) / len(unl)
    t98 = metrics.threshold(allb, s, 0.98)
    g98 = sigmoid(t98)
    print("### Class prior sensitivity\n")
    print(
        f"Elkan-Noto c = E[g | pick] = {c:.3f}, implying {p(ok)} of the non-picks are technically OK "
        "under the selected-completely-at-random assumption (which the user's framing breaks: a pick "
        "is chosen by composition and moment among the OK frames, so the true share is likely higher). "
        f"The 98% pick-kept threshold sits at g = {g98:.3f}; P(OK) = g / c there for an assumed OK share "
        "of the non-picks:\n"
    )
    P, U = len(picks), len(unl)
    rows = []
    for share in (0.3, 0.5, 0.7, 0.9):
        cc = P / (P + share * U)
        rows.append([p(share, 0), f"{cc:.3f}", f"{min(1.0, g98 / cc):.3f}"])
    table("| Assumed OK share of non-picks | c | P(OK) at the 98% threshold |", rows)

    print("## Ranking: (iii) pairwise and (iv) pointwise logistic, held out\n")
    print("Position metrics as in results.md over the held-out scores; baselines need no fit.\n")
    rows = []
    for name, fit in (
        ("(iii) pairwise, all features", fit_pairwise(GROUPS)),
        ("(iii) pairwise, no pose", fit_pairwise(("sharp", "ef", "eo"))),
        ("(iv) pointwise, all features", fit_pointwise(GROUPS)),
        ("(iv) pointwise, no pose", fit_pointwise(("sharp", "ef", "eo"))),
    ):
        bursts, models = heldout_scores(by, fit)
        m = metrics.position_metrics(bursts, lambda f: f["s"], False)
        rows.append([name, m["top_half"], m["top_third"], m["worst_median"], m["worst_p90"], m["worst_near1"], m["pair_auc"], m["abs_auc"], m["top1"], m["mean_rank"]])
    for name, score in (("Sharpness / burst max", lambda f: f["rel"]), ("First frame", lambda f: -f["pos"]), ("Random (seed 0)", lambda f: f["rand"])):
        m = metrics.position_metrics(allb, score, False)
        rows.append([name, m["top_half"], m["top_third"], m["worst_median"], m["worst_p90"], m["worst_near1"], m["pair_auc"], m["abs_auc"], m["top1"], m["mean_rank"]])
    table(
        "| Score | Picks in top half | Top third | Worst pick median | Worst pick p90 | Bursts with worst pick >= 0.9 | Pairwise AUC | Absolute AUC | Top-1 hit | Mean pick q |",
        rows,
    )
    pw = fit_pairwise(GROUPS)(allb)
    print(f"Pairwise fit on all folders: {pw['pairs']} pairs, Newton iterations {pw['it']}, converged {pw['ok']}, ridge {pw['ridge']}.\n")
    table("| Term | Coefficient |", [[n, f"{w:+.4f}"] for n, w in zip(names[1:], pw["w"][1:])])

    print("## Chosen variant\n")
    feats, target = CHOSEN["set"], CHOSEN["target"]
    pooled, per, train, model = matched[(feats, target)]
    base_pooled, base_per, _, _ = matched[(("rel",), target)]
    print(f"Set {', '.join(feats)}, matched to a {target:.0%} training pick false-fail.\n")
    print("### Per held-out folder\n")
    rows = []
    for n in by:
        c, b = per[n], base_per[n]
        rows.append([f"`{n}`", c[1], c[3], f"{p(rate(c[0], c[1]))} / {p(rate(c[2], c[3]))}", f"{p(rate(b[0], b[1]))} / {p(rate(b[2], b[3]))}"])
    table("| Folder | Picks | Non-picks | Chosen: false-fail / flagged | rel alone: false-fail / flagged |", rows)

    print("### Other gaps\n")
    rows = []
    for gap in (1000, 2000, 5000):
        b2 = arw_bursts(folders, gap)
        for fs in (("rel",), feats):
            pooled2, per2, train2, _ = lofo(b2, fit_matched(fs, target), fails_thresholds)
            ff, fl, kept = summary(pooled2)
            rows.append([gap, ", ".join(fs), sum(len(v) for v in b2.values()), p(ff), p(fl), p(kept)])
    table("| Gap ms | Set | Scorable bursts | Held-out false-fail | Held-out flagged | Mean share kept |", rows)

    if frozen_path:
        t = strip(model)
        frozen = {
            "variant": "(i) per-feature thresholds from the picks, matched",
            "fitted_on": "every sidecar-labeled ARW folder of the data set, scorable bursts",
            "gap_ms": GAP,
            "target_pick_false_fail": target,
            "common_percentile": model["_q"],
            "features": {
                "rel": {"definition": "sharpness / max sharpness of the burst (1.0 when the max is 0)", "fails_below": t["rel"]},
                "eye_focus": {"definition": "the cue's eye_focus (in-focus probability of the AF eye)", "fails_below": t["ef"]},
                "eyes_open": {"definition": "1 - Eyes.probability of the judged face (>= EYES_MIN_FACE)", "fails_below": t["eo"]},
            },
            "missing_feature": "a frame lacking a feature is not failed by it (a face-free frame is judged on rel alone)",
            "single_frame": "never flagged",
            "heldout": {
                "pick_false_fail": rate(pooled[0], pooled[1]),
                "non_pick_flagged": rate(pooled[2], pooled[3]),
                "mean_share_kept": rate(pooled[4], pooled[5]),
            },
        }
        with open(frozen_path, "w", encoding="utf-8", newline="\n") as out:
            json.dump(frozen, out, indent=2)
            out.write("\n")


if __name__ == "__main__":
    report(metrics.load(sys.argv[1]), sys.argv[2] if len(sys.argv) > 2 else None)
