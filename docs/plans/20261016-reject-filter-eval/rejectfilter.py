"""Step 2: every held-out score of the folder-wide evaluation, its baselines,
the rule filters on the app's technical cues and the reject-trained variants
`R0` / `G0`, read as a bottom (reject) filter under a pick recall constraint,
held out by folder. The design, the metrics and the bars are pre-registered in
reject.md.

Usage: python -I -B rejectfilter.py <app-dump-dir> <ball-labels-dir> <folder-wide-eval-dir> <out-dir> [jobs]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <ball-labels-dir> holds
`extras_c1.tsv` and `extras_c2.tsv`; <folder-wide-eval-dir> holds the
folder-wide evaluation's `scores.tsv`. Writes `scores_reject.tsv` (folder,
file, one held-out score column per new score) into <out-dir> and prints the
result tables (Markdown) on stdout.
"""

import csv
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
for sub in (
    "20261012-image-embeddings",
    "20261014-folder-wide-pick-model",
    "20261014-ball-game-ranking-features",
    "20261014-genre-ranking-offsets",
    "20261015-folder-wide-ranking-eval",
):
    sys.path.insert(0, os.path.join(ARCHIVED, sub))
import gate  # noqa: E402
import folder_gate  # noqa: E402
import ballgate  # noqa: E402
import genre_offsets  # noqa: E402,F401
import folderwide as fw  # noqa: E402

GENRES = os.path.normpath(os.path.join(ARCHIVED, "..", "..", "..", "crates", "core", "genres", "siglip2.json"))
fw.GENRES = folder_gate.GENRES = GENRES

TARGETS = (95, 99)
LOST_AT = (10, 20, 30, 50)
GRID = 2000
PICK_WEIGHT = 4
MIN_SUB_PICKS = 20
BAR_GAIN = 3.0
BAR_FOLDERS = 16
ALPHA = 0.05
FLOOR_RECALL = 93.0
FLOOR_RN = 25.0
GOOD_EYE_FOCUS = gate.GOOD_EYE_FOCUS
EYES_CLOSED_EAR = 0.137
GOOD_MAX_YAW = gate.GOOD_MAX_YAW
GOOD_MAX_PITCH = gate.GOOD_MAX_PITCH
HGB = dict(max_iter=200, learning_rate=0.05, max_leaf_nodes=15, l2_regularization=1.0, early_stopping=False, random_state=0)

W0, F0_CTX = fw.W0, fw.F0_CTX
R0, G0 = "R0", "G0"
NOT_GOOD = "Rule: not good"
CLOSED = "Rule: eyes closed"
FOCUS_LOW = "Rule: eye focus low"
AWAY = "Rule: face away"
SHARP10 = "Rule: sharpness bottom 10%"
SHARP20 = "Rule: sharpness bottom 20%"
ANY = "Rule: any of"
COUNT = "rule count"
BINARY = [NOT_GOOD, CLOSED, FOCUS_LOW, AWAY, SHARP10, SHARP20, ANY]


# ---------------------------------------------------------------- metrics


def need(r, picks):
    """The picks a folder must keep at pick recall r percent: ceil(r * picks / 100), in integers."""
    return -(-r * picks // 100)


def bottom(score, tb):
    return np.lexsort((tb, score))


def cut_at(pk_sorted, r):
    """The oracle cut: the length of the longest bottom prefix holding at most picks - need(r) picks."""
    p = int(pk_sorted.sum())
    return int(np.flatnonzero(pk_sorted)[p - need(r, p)]), p - need(r, p)


def oracle_rn(score, pick, tb, r):
    pk = pick[bottom(score, tb)]
    k, held = cut_at(pk, r)
    return 100 * (k - held) / (~pick).sum()


def finite(score):
    s = score.astype(np.float64).copy()
    bad = ~np.isfinite(s)
    if bad.any():
        s[bad] = s[~bad].min() - 1 if (~bad).any() else 0.0
    return s


def point(removed, pick, rej):
    p, n = int(pick.sum()), int((~pick).sum())
    kept = p - int((removed & pick).sum())
    return {
        "recall": 100 * kept / p,
        "rn": 100 * int((removed & ~pick).sum()) / n,
        "rf": 100 * float(removed.mean()),
        "rej": int((removed & rej).sum()),
        "kept": kept,
    }


def graded(score, pick, tb, rej):
    o = bottom(score, tb)
    pk = pick[o]
    n, p = len(pk), int(pk.sum())
    cum = np.concatenate([[0], np.cumsum(pk)])
    out = {"frame": folder_gate.auc(score, pick)}
    for r in TARGETS:
        k, held = cut_at(pk, r)
        out[f"rn{r}"] = 100 * (k - held) / (n - p)
        out[f"rf{r}"] = 100 * k / n
        out[f"rej{r}"] = int(rej[o][:k].sum())
    out["pauc"] = float(roc_auc_score(~pick, -finite(score), max_fpr=0.05))
    for x in LOST_AT:
        out[f"lost{x}"] = 100 * cum[x * n // 100] / p
    return out


def binary(removed, pick, rej):
    pt = point(removed, pick, rej)
    out = {"frame": folder_gate.auc((~removed).astype(float), pick), "pauc": None}
    for r in TARGETS:
        ok = pt["kept"] >= need(r, int(pick.sum()))
        out[f"reach{r}"] = ok
        out[f"rn{r}"] = pt["rn"] if ok else 0.0
        out[f"rf{r}"] = pt["rf"] if ok else 0.0
        out[f"rej{r}"] = pt["rej"] if ok else 0
        out[f"q{r}"] = pt
    for x in LOST_AT:
        out[f"lost{x}"] = None
    return out


def label_free(score, d, tb, qual, rej):
    """`quantile@r` and `absolute@r` per qualifying folder, each from the other qualifying folders' held-out scores."""
    pick, folder_of = d["pick"], d["folder_of"]
    info = {}
    for g in qual:
        m = np.where(folder_of == g)[0]
        o = m[bottom(score[m], tb[m])]
        cum = np.concatenate([[0], np.cumsum(pick[o])])
        ks = (np.arange(GRID + 1) * len(m)) // GRID
        info[g] = (m, o, 1 - cum[ks] / cum[-1], np.sort(score[m][pick[m]]))
    total = sum(v[2] for v in info.values())
    out = {}
    for h in qual:
        m, o, rec, _ = info[h]
        others = (total - rec) / (len(qual) - 1)
        pooled = np.sort(np.concatenate([info[g][3] for g in qual if g != h]))
        res = {}
        for r in TARGETS:
            qi = int(np.flatnonzero(others >= r / 100 - 1e-12).max())
            removed = np.zeros(len(pick), dtype=bool)
            removed[o[: qi * len(m) // GRID]] = True
            res[f"q{r}"] = point(removed[m], pick[m], rej[m])
            t = pooled[len(pooled) - need(r, len(pooled))]
            res[f"a{r}"] = point(score[m] < t, pick[m], rej[m])
        out[h] = res
    return out


def evaluate(name, score, d, tb, qual, rej, removed=None):
    pf = {}
    for fi in range(len(d["folders"])):
        m = d["folder_of"] == fi
        pk = d["pick"][m]
        if pk.sum() == 0 or (~pk).sum() == 0 or (removed is None and np.isnan(score[m]).any()):
            pf[fi] = None
            continue
        pf[fi] = binary(removed[m], pk, rej[m]) if removed is not None else graded(score[m], pk, tb[m], rej[m])
    if removed is None:
        for fi, v in label_free(score, d, tb, qual, rej).items():
            pf[fi].update(v)
    else:
        for fi in qual:
            for r in TARGETS:
                pf[fi][f"a{r}"] = None
    return pf


# ---------------------------------------------------------------- reject-trained variants


def choose_c_rn(d, train_mask, x, tb, qual_set):
    """`folder_gate.choose_c` with the inner validation folders' mean `RN@95` (qualifying folders) in place of the AUC."""
    idx = np.where(train_mask)[0]
    groups = d["folder_of"][idx]
    k = min(5, len(set(groups.tolist())))
    splits = list(GroupKFold(n_splits=k).split(idx, groups=groups))
    best, best_c = -1, None
    for c in gate.CS:
        vals = []
        for tr, va in splits:
            w, b = folder_gate.fit(d, idx[tr], x, c)
            s = x[idx[va]] @ w + b
            for fi in sorted(set(groups[va].tolist())):
                if fi not in qual_set:
                    continue
                m = groups[va] == fi
                vals.append(oracle_rn(s[m], d["pick"][idx[va][m]], tb[idx[va][m]], 95))
        v = float(np.mean(vals))
        if v > best:
            best, best_c = v, c
    return best_c


def r0_fold(d, held, mult, chooser, tb, qual_set):
    train_mask = d["folder_of"] != held
    x = folder_gate.design(d, train_mask, "full")
    dd = dict(d)
    w = d["weight"].copy()
    w[d["pick"]] *= mult
    dd["weight"] = w
    c = folder_gate.choose_c(dd, train_mask, x) if chooser == "auc" else choose_c_rn(dd, train_mask, x, tb, qual_set)
    wt, b = folder_gate.fit(dd, np.where(train_mask)[0], x, c)
    return held, c, x[d["folder_of"] == held] @ wt + b


def g0_fold(d, held, rescale=True):
    train_mask = d["folder_of"] != held
    x = folder_gate.design(d, train_mask, "full")
    x[:, : d["tech"].shape[1]][np.isnan(d["tech"])] = np.nan
    idx = np.where(train_mask)[0]
    w = d["weight"][idx]
    m = HistGradientBoostingClassifier(**HGB).fit(x[idx], d["pick"][idx].astype(int), sample_weight=w * len(idx) / w.sum() if rescale else w)
    return held, None, m.decision_function(x[d["folder_of"] == held])


# ---------------------------------------------------------------- data


def load_scores(path, d):
    with open(path, encoding="utf-8") as f:
        head = f.readline().rstrip("\n").split("\t")
        by = {}
        for line in f:
            r = line.rstrip("\n").split("\t")
            by[(r[0], r[1])] = r[2:]
    keys = [(f["folder"], f["file"]) for f in d["rows"]]
    missing = [k for k in keys if k not in by]
    if missing:
        raise SystemExit(f"{path}: {len(missing)} frames have no row")
    vals = np.array([[float(v) for v in by[k]] for k in keys])
    return {name: vals[:, j] for j, name in enumerate(head[2:])}, len(by)


def load_ear(dump, d):
    """The raw EAR of every frame from `features/<folder>.tsv` (gate.load keeps only the 0-1 `eyes_open`)."""
    by = {}
    feats = os.path.join(dump, "features")
    for name in os.listdir(feats):
        if name.endswith(".tsv"):
            with open(os.path.join(feats, name), newline="", encoding="utf-8") as f:
                for r in csv.DictReader(f, delimiter="\t"):
                    by[(name[:-4], r["file"])] = gate.num(r["ear"])
    return np.array([np.nan if by[(f["folder"], f["file"])] is None else by[(f["folder"], f["file"])] for f in d["rows"]])


def rules(d, tb, ear):
    rows = d["rows"]

    def col(key):
        return np.array([np.nan if f[key] is None else f[key] for f in rows])

    ef, yaw, pitch, sharp = col("ef"), col("yaw"), col("pitch"), col("sharp")
    with np.errstate(invalid="ignore"):
        closed = ear <= EYES_CLOSED_EAR
        low = ef < GOOD_EYE_FOCUS
        away = (np.abs(yaw) > GOOD_MAX_YAW) | (np.abs(pitch) > GOOD_MAX_PITCH)

    def sharp_bottom(q):
        out = np.zeros(len(rows), dtype=bool)
        for fi in range(len(d["folders"])):
            m = np.where(d["folder_of"] == fi)[0]
            v = m[~np.isnan(sharp[m])]
            out[v[bottom(sharp[v], tb[v])][: q * len(m) // 100]] = True
        return out

    s10, s20 = sharp_bottom(10), sharp_bottom(20)
    masks = {
        NOT_GOOD: ~d["tech"][:, 6].astype(bool),
        CLOSED: closed,
        FOCUS_LOW: low,
        AWAY: away,
        SHARP10: s10,
        SHARP20: s20,
        ANY: closed | low | away | s10,
    }
    count = closed.astype(int) + low.astype(int) + away.astype(int) + s10.astype(int)
    present = {"`ef` (AF face)": ~np.isnan(ef), "EAR (eyes judgment)": ~np.isnan(ear), "pose": ~np.isnan(yaw), "sharpness": ~np.isnan(sharp)}
    return masks, (-count).astype(float), present


# ---------------------------------------------------------------- report


def code(k):
    return k if "`" in k else f"`{k}`"


def f1(x):
    return "-" if x is None else f"{x:.1f}"


def pair(pts, target):
    if not pts:
        return "-"
    rec = float(np.mean([p["recall"] for p in pts]))
    rn = float(np.mean([p["rn"] for p in pts]))
    return f"{rec:.1f} / {rn:.1f} ({rec - target:+.1f})"


def main(dump, labels_dir, eval_dir, out_dir, jobs):
    start = time.time()
    os.makedirs(out_dir, exist_ok=True)
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    n, nf = len(d["rows"]), len(d["folders"])
    tb = np.random.default_rng(0).random(n)
    pick, folder_of = d["pick"], d["folder_of"]
    rej = d["reject"] & ~pick
    sizes = {fi: int((folder_of == fi).sum()) for fi in range(nf)}
    picks = {fi: int((pick & (folder_of == fi)).sum()) for fi in range(nf)}
    qual = [fi for fi in range(nf) if sizes[fi] >= fw.MIN_FRAMES and picks[fi] >= fw.MIN_PICKS]
    qual_set = set(qual)
    sub = [fi for fi in qual if picks[fi] >= MIN_SUB_PICKS]
    c1_names, c1 = ballgate.load_extras([os.path.join(labels_dir, "extras_c1.tsv")], d)
    c2_names, c2 = ballgate.load_extras([os.path.join(labels_dir, "extras_c2.tsv")], d)
    stored, stored_rows = load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    masks, count, present = rules(d, tb, load_ear(dump, d))
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    models, cs, times = {}, {}, {}

    def run(name, f):
        t = time.time()
        models[name], cs[name] = fw.collect(d, Parallel(n_jobs=jobs)(f(h) for h in range(nf)))
        times[name] = time.time() - t
        print(f"{name}: {times[name]:.0f} s", file=sys.stderr, flush=True)

    check = "R0 at pick weight 1, `C` by AUC (check)"
    run(check, lambda h: delayed(r0_fold)(d, h, 1, "auc", tb, qual_set))
    run(R0, lambda h: delayed(r0_fold)(d, h, PICK_WEIGHT, "rn", tb, qual_set))
    run(G0, lambda h: delayed(g0_fold)(d, h))
    g0_raw = "G0 on the raw weights (check)"
    run(g0_raw, lambda h: delayed(g0_fold)(d, h, False))

    with open(os.path.join(out_dir, "scores_reject.tsv"), "w", encoding="utf-8", newline="") as out:
        cols = {R0: models[R0], G0: models[G0], COUNT: count}
        out.write("folder\tfile\t" + "\t".join(cols) + "\n")
        for i, f in enumerate(d["rows"]):
            out.write(f"{f['folder']}\t{f['file']}\t" + "\t".join(f"{v[i]:.6g}" for v in cols.values()) + "\n")

    t = time.time()
    w0 = stored[W0]
    centered = w0.copy()
    for _, s, e in d["bursts"]:
        centered[s:e] -= w0[s:e].mean()
    tech = d["tech"]
    graded_scores = dict(stored)
    graded_scores[fw.CENTERED] = centered
    graded_scores.update(
        {
            fw.RANDOM: tb,
            fw.EARLIER: -d["order"],
            fw.LATER: d["order"],
            fw.SIZE: d["size"],
            fw.SHARP: np.where(np.isnan(d["sharp"]), -np.inf, d["sharp"]),
            fw.EYE: np.array([-np.inf if f["ef"] is None else f["ef"] for f in d["rows"]]),
            fw.TIER: tech[:, 6],
        }
    )
    order = list(graded_scores) + BINARY + [COUNT, R0, G0]
    pf = {k: evaluate(k, s, d, tb, qual, rej) for k, s in graded_scores.items()}
    for k in BINARY:
        pf[k] = evaluate(k, None, d, tb, qual, rej, removed=masks[k])
    pf[COUNT] = evaluate(COUNT, count, d, tb, qual, rej)
    pf[R0] = evaluate(R0, models[R0], d, tb, qual, rej)
    pf[G0] = evaluate(G0, models[G0], d, tb, qual, rej)
    pf_check = evaluate(check, models[check], d, tb, qual, rej)
    pf[g0_raw] = evaluate(g0_raw, models[g0_raw], d, tb, qual, rej)
    metric_time = time.time() - t
    for k in order:
        for fi in qual:
            if pf[k][fi] is None:
                raise SystemExit(f"{k}: no value on qualifying folder {d['folders'][fi]}")

    def vals(k, key, folders=qual):
        return {fi: pf[k][fi][key] for fi in folders if pf[k][fi][key] is not None}

    def mean(k, key, folders=qual):
        return folder_gate.mean_sd(vals(k, key, folders))

    def pooled(k, key):
        return fw.pooled(pf[k], key, qual, sizes)

    def is_binary(k):
        return k in BINARY

    def lf(k, key, folders=qual):
        if is_binary(k) and key.startswith("a"):
            return []
        return [pf[k][fi][key] for fi in folders]

    def rej_total(folders):
        return int(sum(int((rej & (folder_of == fi)).sum()) for fi in folders))

    nrej = rej_total(qual)

    print("## Data\n")
    rates = [picks[fi] / sizes[fi] for fi in range(nf)]
    gate.table(
        "| Count | Value |",
        [
            ("Folders / qualifying (>= 50 frames, >= 5 picks) / qualifying with >= 20 picks", f"{nf} / {len(qual)} / {len(sub)}"),
            ("Frames (with a vector) / picks / explicit rejects", f"{n} / {int(pick.sum())} / {int(d['reject'].sum())}"),
            ("Explicit rejects that are not picks / in qualifying folders / folders holding one", f"{int(rej.sum())} / {nrej} / {len({int(folder_of[i]) for i in np.flatnonzero(rej)})}"),
            ("Frames without a vector (dropped)", missing),
            ("Bursts (gap 1000 ms), singles included / singles", f"{len(d['bursts'])} / {int(d['single'].sum())}"),
            ("Folder pick rate min / median / max", f"{100 * min(rates):.1f}% / {100 * float(np.median(rates)):.1f}% / {100 * max(rates):.1f}%"),
            ("`scores.tsv` rows / matched to the dump's frames", f"{stored_rows} / {n}"),
            ("Frames with a value: " + ", ".join(present), " / ".join(str(int(v.sum())) for v in present.values())),
            ("Frames removed by each rule (all folders): " + ", ".join(k[6:] for k in BINARY), " / ".join(str(int(masks[k].sum())) for k in BINARY)),
            ("`c1` / `c2` columns (frames without a row)", f"{len(c1_names)} ({int(np.isnan(c1).all(axis=1).sum())}) / {len(c2_names)} ({int(np.isnan(c2).all(axis=1).sum())})"),
        ],
    )

    print("## Reference checks\n")
    ok = ~np.isnan(stored[F0_CTX])
    gate.table(
        "| Check | Expected | This run |",
        [
            ("`W0` mean frame AUC from `scores.tsv` (qualifying folders)", "0.666", gate.f3(mean(W0, "frame")[0])),
            ("`F0+ctx` mean frame AUC from `scores.tsv` (qualifying folders)", "0.666", gate.f3(mean(F0_CTX, "frame")[0])),
            ("Random (seed 0) mean `RN@95` %", "near 5", f1(mean(fw.RANDOM, "rn95")[0])),
            ("Random (seed 0) mean `pAUC95`", "near 0.5", gate.f3(mean(fw.RANDOM, "pauc")[0])),
            ("`R0` at pick weight 1 (`C` by AUC): max abs score difference from `F0+ctx`", "~5e-6 (6 significant digits)", f"{np.abs(models[check][ok] - stored[F0_CTX][ok]).max():.2e}"),
            ("`R0` at pick weight 1 (`C` by AUC): mean frame AUC / `RN@95` %", f"{gate.f3(mean(F0_CTX, 'frame')[0])} / {f1(mean(F0_CTX, 'rn95')[0])}",
             f"{gate.f3(folder_gate.mean_sd({fi: pf_check[fi]['frame'] for fi in qual})[0])} / {f1(folder_gate.mean_sd({fi: pf_check[fi]['rn95'] for fi in qual})[0])}"),
        ],
    )

    print("## Main table: bottom metrics (held out, qualifying folders)\n")
    print(
        f"Over the {len(qual)} qualifying folders: the mean (per-folder SD in brackets) and the frame-weighted mean (pooled). "
        "`RN@r`: the share of the folder's non-picks removed by the oracle cut that keeps >= r% of its picks; `RF@95` the share "
        "of all its frames removed there. `pAUC95`: McClish-standardized partial AUC over pick recall [0.95, 1], chance 0.5. "
        "Label-free pairs: the mean realized pick recall % / mean `RN` % (realized minus target recall in brackets); "
        "`quantile@r` removes the bottom share the other qualifying folders' held-out scores allow, `absolute@r` the frames "
        "below the other folders' pooled pick-score cut (sensitive to the score scale across folds). For a binary rule "
        "(`Rule: ...`), `RN@r` and `RF@r` are its single point in the folders where it keeps >= r% of the picks and 0 "
        "elsewhere, `pAUC95` and `absolute@r` do not apply, the `quantile@95` cell is the rule's own point (it needs no "
        "threshold), and its frame AUC is that of the keep / remove split. Frame AUC is recomputed from the same scores.\n"
    )
    rows = []
    for k in order:
        m95, s95, _ = mean(k, "rn95")
        m99, s99, _ = mean(k, "rn99")
        pa, ps, _ = mean(k, "pauc")
        rows.append(
            [k, fw.pm(m95, s95, digits=1), f1(pooled(k, "rn95")), f1(mean(k, "rf95")[0]), fw.pm(m99, s99, digits=1), f1(pooled(k, "rn99")),
             fw.pm(pa, ps), gate.f3(pooled(k, "pauc")), pair(lf(k, "q95"), 95), pair(lf(k, "a95"), 95), gate.f3(mean(k, "frame")[0])]
        )
    gate.table(
        "| Score | `RN@95` % | Pooled | `RF@95` % | `RN@99` % | Pooled | `pAUC95` | Pooled | `quantile@95` recall / `RN` (gap) | `absolute@95` recall / `RN` (gap) | Frame AUC |",
        rows,
    )

    print("## Main table, continued: reading aids and the 99% label-free pairs\n")
    print(
        f"Picks lost: the mean share of the folder's picks inside its bottom X% of frames by score. Rejects removed: of the "
        f"{nrej} explicit rejects (not picks) in the qualifying folders, the number removed at the oracle `RN@95` cut and at "
        "the `quantile@95` cut (a binary rule: its own point). The 99% label-free pairs as above.\n"
    )
    rows = []
    for k in order:
        lost = [f1(mean(k, f"lost{x}")[0]) for x in LOST_AT]
        rq = sum(pf[k][fi]["q95"]["rej"] for fi in qual)
        rows.append([k] + lost + [f"{sum(pf[k][fi]['rej95'] for fi in qual)} of {nrej}", f"{rq} of {nrej}", pair(lf(k, "q99"), 99), pair(lf(k, "a99"), 99)])
    gate.table(
        "| Score | Picks lost, bottom 10% | 20% | 30% | 50% | Rejects removed at `RN@95` | at `quantile@95` | `quantile@99` recall / `RN` (gap) | `absolute@99` recall / `RN` (gap) |",
        rows,
    )

    print("## Rule points\n")
    print(f"Each binary rule's single operating point, the mean over the {len(qual)} qualifying folders, and the folders where it keeps >= 95% / 99% of the picks.\n")
    rows = []
    for k in BINARY:
        pts = [pf[k][fi]["q95"] for fi in qual]
        rows.append(
            [k, f1(float(np.mean([p["recall"] for p in pts]))), f1(float(np.mean([p["rn"] for p in pts]))), f1(float(np.mean([p["rf"] for p in pts]))),
             f"{sum(1 for fi in qual if pf[k][fi]['reach95'])} of {len(qual)}", f"{sum(1 for fi in qual if pf[k][fi]['reach99'])} of {len(qual)}",
             f"{sum(p['rej'] for p in pts)} of {nrej}"]
        )
    gate.table("| Rule | Realized pick recall % | `RN` % | `RF` % | Folders reaching 95% | Folders reaching 99% | Rejects removed |", rows)

    print(f"## Sub-table: the {len(sub)} qualifying folders with >= {MIN_SUB_PICKS} picks\n")
    print("The same per-folder values (the label-free cuts still from the other qualifying folders), averaged over these folders only.\n")
    rows = []
    for k in order:
        m95, s95, _ = mean(k, "rn95", sub)
        m99, s99, _ = mean(k, "rn99", sub)
        pa, ps, _ = mean(k, "pauc", sub)
        rows.append([k, fw.pm(m95, s95, digits=1), fw.pm(m99, s99, digits=1), fw.pm(pa, ps), pair(lf(k, "q95", sub), 95), pair(lf(k, "q99", sub), 99)])
    gate.table("| Score | `RN@95` % | `RN@99` % | `pAUC95` | `quantile@95` recall / `RN` (gap) | `quantile@99` recall / `RN` (gap) |", rows)

    comparisons = [(k, W0) for k in order if k != W0]
    comparisons += [(f"{F0_CTX} {c}", F0_CTX) for c in fw.CUES]
    comparisons += [(R0, F0_CTX), (G0, F0_CTX)]
    print("## Paired differences (score minus comparison)\n")
    print(
        f"Over the {len(qual)} qualifying folders: the mean paired per-folder difference with the 1,000-draw folder bootstrap "
        "95% interval (seed 0, fresh per comparison and metric), in points for `RN`; the one-sided Wilcoxon signed-rank p on "
        "`RN@95` (zeros dropped); the folders where the score's `RN@95` is at or above the comparison's. Comparison bar: "
        f"`RN@95` gain >= +{BAR_GAIN:.1f} points, >= {BAR_FOLDERS} of {len(qual)} folders at or above, p < {ALPHA}.\n"
    )
    rows = []
    for v, base in comparisons:
        a95 = folder_gate.paired(vals(v, "rn95"), vals(base, "rn95"))
        a99 = folder_gate.paired(vals(v, "rn99"), vals(base, "rn99"))
        pa = folder_gate.paired(vals(v, "pauc"), vals(base, "pauc"))
        m95, lo95, hi95 = fw.boot_folders(a95)
        m99, lo99, hi99 = fw.boot_folders(a99)
        _, p95, _ = folder_gate.signed_rank(a95)
        above = sum(1 for x in a95 if x >= 0)
        cell = fw.ci(*fw.boot_folders(pa)) if pa else "-"
        checks = [m95 >= BAR_GAIN, above >= BAR_FOLDERS, p95 < ALPHA]
        rows.append([f"{code(v)} - {code(base)}", fw.ci(m95, lo95, hi95, 1), f"{p95:.4f}", f"{above} of {len(a95)}", fw.ci(m99, lo99, hi99, 1), cell,
                     f"{sum(checks)} of 3", "pass" if all(checks) else "fail"])
    gate.table("| Comparison | `RN@95`, points | p | Folders at or above | `RN@99`, points | `pAUC95` | Bars met | Result |", rows)

    print("## `G0` on the raw weights (check)\n")
    print(
        "`G0` fit with `sample_weight` the dump's per-frame weight as is (it sums to 1 per folder), not rescaled to mean 1 per "
        "training fold; everything else as `G0`. The plan's literal text; shown so the reader can see whether the comparison "
        "bar still passes without the rescale.\n"
    )
    rows = []
    for k in (G0, g0_raw):
        m95, s95, _ = mean(k, "rn95")
        m99, s99, _ = mean(k, "rn99")
        pa, ps, _ = mean(k, "pauc")
        rows.append([f"`{k}`", fw.pm(m95, s95, digits=1), fw.pm(m99, s99, digits=1), fw.pm(pa, ps), gate.f3(mean(k, "frame")[0])])
    gate.table("| Score | `RN@95` % | `RN@99` % | `pAUC95` | Frame AUC |", rows)
    rows = []
    for base in (W0, F0_CTX):
        a95 = folder_gate.paired(vals(g0_raw, "rn95"), vals(base, "rn95"))
        a99 = folder_gate.paired(vals(g0_raw, "rn99"), vals(base, "rn99"))
        pa = folder_gate.paired(vals(g0_raw, "pauc"), vals(base, "pauc"))
        m95, lo95, hi95 = fw.boot_folders(a95)
        m99, lo99, hi99 = fw.boot_folders(a99)
        _, p95, _ = folder_gate.signed_rank(a95)
        above = sum(1 for x in a95 if x >= 0)
        cell = fw.ci(*fw.boot_folders(pa)) if pa else "-"
        checks = [m95 >= BAR_GAIN, above >= BAR_FOLDERS, p95 < ALPHA]
        rows.append([f"`{g0_raw}` - {code(base)}", fw.ci(m95, lo95, hi95, 1), f"{p95:.4f}", f"{above} of {len(a95)}", fw.ci(m99, lo99, hi99, 1), cell,
                     f"{sum(checks)} of 3", "pass" if all(checks) else "fail"])
    gate.table("| Comparison | `RN@95`, points | p | Folders at or above | `RN@99`, points | `pAUC95` | Bars met | Result |", rows)

    best95 = max(order, key=lambda k: mean(k, "rn95")[0])
    best99 = max(order, key=lambda k: mean(k, "rn99")[0])
    print("## Best filter and the worth-a-filter bar\n")
    rows = []
    for label, k, r in (("Best at 95% (highest mean `RN@95`)", best95, 95), ("Best at 99% (highest mean `RN@99`)", best99, 99)):
        pts = lf(k, f"q{r}")
        rec = float(np.mean([p["recall"] for p in pts]))
        rn = float(np.mean([p["rn"] for p in pts]))
        met = rec >= FLOOR_RECALL and rn >= FLOOR_RN
        bar = ("met" if met else "not met") if r == 95 else "-"
        rows.append([label, f"`{k}`", f1(mean(k, f"rn{r}")[0]), f"{rec:.1f} / {rn:.1f} ({rec - r:+.1f})", bar])
    gate.table(
        f"| Reading | Score | Oracle `RN` % | Label-free `quantile` recall / `RN` (gap) | Worth-a-filter bar (`quantile@95` recall >= {FLOOR_RECALL:.0f}%, `RN` >= {FLOOR_RN:.0f}%) |",
        rows,
    )

    print("## Per folder (held out)\n")
    print(f"Best filter by mean `RN@95`: `{best95}`. Label-free pairs only for the qualifying folders.\n")
    rows = []
    for fi, folder in enumerate(d["folders"]):
        r = [f"`{folder}`", sizes[fi], picks[fi], f"{100 * picks[fi] / sizes[fi]:.1f}%", "yes" if fi in qual_set else "no"]
        for k in (W0, best95):
            v = pf[k][fi]
            r += ["-" if v is None else f1(v["rn95"]), pair([v["q95"]], 95) if fi in qual_set else "-"]
        rows.append(r)
    gate.table(f"| Folder | Frames | Picks | Pick rate | Qualifying | `{W0}` `RN@95` % | `{W0}` `quantile@95` | `{best95}` `RN@95` % | `{best95}` `quantile@95` |", rows)

    print("`C` chosen per fold (count of folds):\n")
    gate.table("| Model | " + " | ".join(str(c) for c in gate.CS) + " |", [[k] + [sum(1 for v in cs[k].values() if v == c) for c in gate.CS] for k in (R0, check)])

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [["Load (dump, cue files, `scores.tsv`, rules)", f"{load_time:.0f}"]] + [[k, f"{v:.0f}"] for k, v in times.items()] + [["Metrics", f"{metric_time:.0f}"]])
    print(f"Total {time.time() - start:.0f} s on {jobs} jobs.")


if __name__ == "__main__":
    if len(sys.argv) not in (5, 6):
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]) if len(sys.argv) > 5 else 8)
