"""Step 4: every existing score, rule and cue against the user's bad / not-bad
labels, weighted back to the dump by the inverse inclusion probability, with
stratified and folder-cluster bootstrap intervals. The metrics, cuts and bars
are pre-registered in badfilter.md.

Usage: python -I -B badfilter.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv`, <reject-filter-eval-dir> `scores_reject.tsv`; <out-dir> is Step
1's data folder with `manifest.tsv`, `strata.tsv` and Step 3's `labels.tsv`.
Prints the result tables (Markdown) and saves them as `badfilter.out.md` in
<out-dir>.
"""

import csv
import os
import sys
import time

import numpy as np
from sklearn.metrics import roc_auc_score

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261016-reject-filter-eval"))
sys.path.insert(0, HERE)
import rejectfilter as rf  # noqa: E402
import sample  # noqa: E402

gate, folder_gate, fw = rf.gate, rf.folder_gate, rf.fw

QS = (5, 10, 20, 30, 40)
LOSS_CEIL = 5.0
RECALL_BAR = 50.0
RECALL_LO = 40.0
PRECISION_BAR = 50.0
CUE_FACTOR = 2.0
PAUC_FPR = 0.10
BOOT = 1000
STRATA = [k for k, _ in sample.ALLOCATION]
STORED = ["W0", "W0+off", "F0", "F0+ctx", "P0"]
STORED_REJ = ["R0", "G0", rf.COUNT]
G0Q95 = "G0 `quantile@95`"
RULES = [
    ("`eyes closed`", rf.CLOSED),
    ("`eye focus low`", rf.FOCUS_LOW),
    ("`face away`", rf.AWAY),
    ("`sharpness bottom 10%`", rf.SHARP10),
    ("`any of`", rf.ANY),
    ("`not good`", rf.NOT_GOOD),
]


class Tee:
    def __init__(self, stream):
        self.stream, self.parts = stream, []

    def write(self, s):
        self.parts.append(s)
        return self.stream.write(s)

    def flush(self):
        self.stream.flush()


# ---------------------------------------------------------------- data


def read_tsv(path):
    with open(path, encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def load(dump, eval_dir, reject_dir, out_dir):
    """The dump, its stored scores and rules, and the labeled sample as indices into the dump."""
    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    n = len(d["rows"])
    tb = np.random.default_rng(0).random(n)
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    ear = rf.load_ear(dump, d)
    masks, count, _ = rf.rules(d, tb, ear)
    index = {(f["folder"], f["file"]): i for i, f in enumerate(d["rows"])}
    labels = read_tsv(os.path.join(out_dir, "labels.tsv"))
    manifest = {r["name"]: r for r in read_tsv(os.path.join(out_dir, "manifest.tsv"))}
    s = {
        "name": [r["name"] for r in labels],
        "idx": np.array([index[(r["folder"], r["file"])] for r in labels]),
        "bad": np.array([r["label"] == "bad" for r in labels]),
        "pi": np.array([float(r["inclusion_prob"]) for r in labels]),
        "stratum": np.array([r["stratum"] for r in labels], dtype=object),
        "manifest": [manifest[r["name"]] for r in labels],
    }
    s["w"] = 1 / s["pi"]
    s["folder"] = d["folder_of"][s["idx"]]
    return d, tb, stored, stored_rej, ear, masks, count, s, missing


def recompute_strata(d, tb, stored, stored_rej, masks):
    """Step 1's stratum of every dump frame, from the same flags in the same priority order."""
    flags = {
        "closed": masks[rf.CLOSED],
        "focus_low": masks[rf.FOCUS_LOW],
        "away": masks[rf.AWAY],
        "sharp10": masks[rf.SHARP10],
        "g0_bottom": sample.score_bottom(stored_rej["G0"], d, tb, sample.BOTTOM_SHARE),
        "w0_bottom": sample.score_bottom(stored["W0"], d, tb, sample.BOTTOM_SHARE),
    }
    stratum = np.full(len(d["rows"]), "rest", dtype=object)
    free = np.ones(len(d["rows"]), dtype=bool)
    for name in STRATA[:-1]:
        hit = free & flags[name]
        stratum[hit] = name
        free &= ~hit
    return stratum


def cells_of(s, folders):
    """The design's cells: each stratum, and `rest` per folder."""
    out = []
    for name in STRATA[:-1]:
        out.append(np.flatnonzero(s["stratum"] == name))
    for fi in range(len(folders)):
        out.append(np.flatnonzero((s["stratum"] == "rest") & (s["folder"] == fi)))
    return [c for c in out if len(c)]


def strat_counts(s, folders):
    """The stratified bootstrap as a (draws, sampled frames) count matrix, `default_rng(0)`."""
    rng = np.random.default_rng(0)
    c = np.zeros((BOOT, len(s["idx"])))
    rows = np.arange(BOOT)[:, None]
    for cell in cells_of(s, folders):
        pick = cell[rng.integers(0, len(cell), size=(BOOT, len(cell)))]
        np.add.at(c, (np.broadcast_to(rows, pick.shape), pick), 1)
    return c


def cluster_counts(s, folders):
    """The folder-cluster bootstrap: resample the folders with replacement, `default_rng(0)`."""
    rng = np.random.default_rng(0)
    nf = len(folders)
    fc = np.zeros((BOOT, nf))
    draws = rng.integers(0, nf, size=(BOOT, nf))
    np.add.at(fc, (np.broadcast_to(np.arange(BOOT)[:, None], draws.shape), draws), 1)
    return fc[:, s["folder"]]


# ---------------------------------------------------------------- cuts


def graded_cut(score, d, tb, q):
    """Remove the bottom floor(q * n / 100) of each folder's frames by score, ties by the seed-0 draw; a folder with a NaN is left out."""
    removed = np.zeros(len(score), dtype=bool)
    out = np.zeros(len(score), dtype=bool)
    for fi in range(len(d["folders"])):
        m = np.where(d["folder_of"] == fi)[0]
        if np.isnan(score[m]).any():
            out[m] = True
            continue
        removed[m[rf.bottom(score[m], tb[m])][: q * len(m) // 100]] = True
    return removed, out


def g0_q95_cut(score, d, tb):
    """`G0`'s label-free `quantile@95` cut as `rf.label_free` takes it; a non-qualifying folder gets the share from all qualifying folders."""
    pick, folder_of = d["pick"], d["folder_of"]
    nf = len(d["folders"])
    sizes = [int((folder_of == fi).sum()) for fi in range(nf)]
    picks = [int((pick & (folder_of == fi)).sum()) for fi in range(nf)]
    qual = [fi for fi in range(nf) if sizes[fi] >= fw.MIN_FRAMES and picks[fi] >= fw.MIN_PICKS]
    order, rec = {}, {}
    for fi in range(nf):
        m = np.where(folder_of == fi)[0]
        order[fi] = m[rf.bottom(score[m], tb[m])]
        if fi in qual:
            cum = np.concatenate([[0], np.cumsum(pick[order[fi]])])
            rec[fi] = 1 - cum[(np.arange(rf.GRID + 1) * len(m)) // rf.GRID] / cum[-1]
    total = sum(rec.values())
    removed = np.zeros(len(score), dtype=bool)
    share = {}
    for fi in range(nf):
        others = (total - rec[fi]) / (len(qual) - 1) if fi in qual else total / len(qual)
        qi = int(np.flatnonzero(others >= 0.95 - 1e-12).max())
        removed[order[fi][: qi * sizes[fi] // rf.GRID]] = True
        share[fi] = qi / rf.GRID
    return removed, qual, share


def finite_by_folder(score, d):
    out = score.astype(np.float64).copy()
    for fi in range(len(d["folders"])):
        m = d["folder_of"] == fi
        out[m] = rf.finite(score[m])
    return out


# ---------------------------------------------------------------- metrics


def ratio(c, num, den):
    with np.errstate(invalid="ignore", divide="ignore"):
        return 100 * (c @ num) / (c @ den)


def rates(s, removed, ok, c=None):
    """`BR`, `NL`, `P` in percent on the sampled frames `ok`, weighted (or, with `c`, per bootstrap draw)."""
    w = s["w"] * ok
    bad = s["bad"]
    cols = [(w * (bad & removed), w * bad), (w * (~bad & removed), w * ~bad), (w * (bad & removed), w * removed)]
    if c is None:
        return [100 * a.sum() / b.sum() if b.sum() else np.nan for a, b in cols]
    return [ratio(c, a, b) for a, b in cols]


def unweighted(s, removed, ok):
    bad = s["bad"] & ok
    nb = ~s["bad"] & ok
    rem = removed & ok
    return [100 * (bad & rem).sum() / bad.sum(), 100 * (nb & rem).sum() / nb.sum(), 100 * (bad & rem).sum() / rem.sum() if rem.sum() else np.nan]


def quant(x):
    return float(np.nanquantile(x, 0.025)), float(np.nanquantile(x, 0.975))


def auc_pair(bad, score, w):
    keep = w > 0
    return float(roc_auc_score(bad[keep], -score[keep], sample_weight=w[keep])), float(roc_auc_score(bad[keep], -score[keep], max_fpr=PAUC_FPR, sample_weight=w[keep]))


def aucs(s, score, ok, c):
    bad, w = s["bad"][ok], s["w"][ok]
    sc = score[ok]
    point = auc_pair(bad, sc, w)
    unw = auc_pair(bad, sc, np.ones(len(sc)))
    boot = np.array([auc_pair(bad, sc, w * c[b][ok]) for b in range(BOOT)])
    return point, unw, quant(boot[:, 0]), quant(boot[:, 1])


# ---------------------------------------------------------------- report


def f1(x):
    return "-" if x is None or (isinstance(x, float) and np.isnan(x)) else f"{x:.1f}"


def iv(m, lo, hi, digits=1):
    return f"{m:.{digits}f} [{lo:.{digits}f}, {hi:.{digits}f}]"


def civ(m, lo, hi):
    return f"{m:+.1f} [{lo:+.1f}, {hi:+.1f}]"


def main(dump, eval_dir, reject_dir, out_dir):
    start = time.time()
    tee = Tee(sys.stdout)
    sys.stdout = tee
    d, tb, stored, stored_rej, ear, masks, count, s, missing = load(dump, eval_dir, reject_dir, out_dir)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n, ns, idx = len(rows), len(s["idx"]), s["idx"]
    bad, w = s["bad"], s["w"]
    load_time = time.time() - start
    print(f"loaded: {load_time:.0f} s", file=sys.stderr, flush=True)

    t = time.time()
    cs = strat_counts(s, folders)
    cc = cluster_counts(s, folders)

    stratum = recompute_strata(d, tb, stored, stored_rej, masks)
    strata_file = read_tsv(os.path.join(out_dir, "strata.tsv"))
    strata_diff = 0
    for r in strata_file:
        if r["stratum"] == "rest":
            fi = folders.index(r["folder"])
            pop = int(((stratum == "rest") & (folder_of == fi)).sum())
            drawn = int(((s["stratum"] == "rest") & (s["folder"] == fi)).sum())
        else:
            pop = int((stratum == r["stratum"]).sum())
            drawn = int((s["stratum"] == r["stratum"]).sum())
        strata_diff += pop != int(r["population"]) or drawn != int(r["drawn"]) or abs(drawn / pop - float(r["inclusion_prob"])) > 1e-5 * float(r["inclusion_prob"])
    pi_dump = np.zeros(n)
    for r in strata_file:
        m = (stratum == r["stratum"]) & ((folder_of == folders.index(r["folder"])) if r["stratum"] == "rest" else True)
        pi_dump[m] = int(r["drawn"]) / int(r["population"])
    pi_diff = int((np.abs(pi_dump[idx] - s["pi"]) > 1e-5 * s["pi"]).sum())
    stratum_diff = int((stratum[idx] != s["stratum"]).sum())

    def man_diff(col, vec):
        mv = np.array([float(r[col]) for r in s["manifest"]])
        dv = vec[idx]
        both = np.isnan(mv) & np.isnan(dv)
        return int((~both & ~(np.abs(mv - dv) <= 1e-5 * np.maximum(1, np.abs(dv)))).sum())

    w0 = stored["W0"]
    centered = w0.copy()
    for _, a, e in d["bursts"]:
        centered[a:e] -= w0[a:e].mean()
    tech = d["tech"]
    sharp = np.array([np.nan if f["sharp"] is None else f["sharp"] for f in rows])
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    yaw = np.array([np.nan if f["yaw"] is None else f["yaw"] for f in rows])
    pitch = np.array([np.nan if f["pitch"] is None else f["pitch"] for f in rows])
    tier = np.array([bool(f["tier"]) for f in rows])
    graded = {k: stored[k] for k in STORED}
    graded.update({k: stored_rej[k] for k in STORED_REJ})
    graded[fw.CENTERED] = centered
    graded[fw.RANDOM] = tb
    graded[fw.SHARP] = np.where(np.isnan(sharp), -np.inf, sharp)
    graded[fw.EYE] = np.where(np.isnan(ef), -np.inf, ef)
    graded[fw.TIER] = tech[:, 6]

    # Every (score, cut): the removed mask on the sampled frames and the frames it may be read on.
    cuts = {}
    for k, sc in graded.items():
        for q in QS:
            removed, out = graded_cut(sc, d, tb, q)
            cuts[(k, f"q{q}")] = (removed, out)
    g0_removed, qual, g0_share = g0_q95_cut(stored_rej["G0"], d, tb)
    cuts[(G0Q95, "q@95")] = (g0_removed, np.zeros(n, dtype=bool))
    for label, key in RULES:
        cuts[(label, "rule")] = (masks[key], np.zeros(n, dtype=bool))
    names = list(graded) + [G0Q95] + [label for label, _ in RULES]

    res = {}
    for (k, cut), (removed, out) in cuts.items():
        rem, ok = removed[idx], ~out[idx]
        point = rates(s, rem, ok)
        boot = rates(s, rem, ok, cs)
        clus = rates(s, rem, ok, cc)
        res[(k, cut)] = {
            "rem": rem, "ok": ok, "point": point, "unw": unweighted(s, rem, ok), "ci": [quant(b) for b in boot], "boot": boot,
            "cci": [quant(b) for b in clus], "cboot": clus, "dump": 100 * removed[~out].mean(), "out": int((~ok).sum()),
        }
        br, nl, p = point
        r = res[(k, cut)]
        r["pass"] = nl <= LOSS_CEIL and br >= RECALL_BAR and r["ci"][0][0] > RECALL_LO and p >= PRECISION_BAR
        r["cpass"] = r["pass"] and r["cci"][0][0] > RECALL_LO

    def cuts_of(k):
        return [c for (kk, c) in cuts if kk == k]

    primary = {k: min(cuts_of(k), key=lambda c: (abs(res[(k, c)]["point"][1] - LOSS_CEIL), cuts_of(k).index(c))) for k in names}

    auc = {}
    for k in names:
        if k in graded:
            sc = finite_by_folder(graded[k], d)[idx]
            ok = ~cuts[(k, "q5")][1][idx]
        else:
            c = cuts_of(k)[0]
            sc = (~res[(k, c)]["rem"]).astype(float)
            ok = res[(k, c)]["ok"]
        auc[k] = aucs(s, sc, ok, cs)
    metric_time = time.time() - t

    total_w = w.sum()
    base = 100 * (w * bad).sum() / total_w
    base_ci = quant(100 * (cs @ (w * bad)) / (cs @ w))
    base_cci = quant(100 * (cc @ (w * bad)) / (cc @ w))

    print("## Data\n")
    pick, reject = d["pick"][idx], d["reject"][idx]
    gate.table(
        "| Count | Value |",
        [
            ("Folders / dump frames (with a vector) / frames without a vector (dropped)", f"{len(folders)} / {n} / {missing}"),
            ("Sampled frames (`labels.tsv`) / bad / not bad", f"{ns} / {int(bad.sum())} / {int((~bad).sum())}"),
            ("Sum of `1/π` over the sampled frames", f"{total_w:.1f}"),
            ("Sampled picks / explicit rejects / singles", f"{int(pick.sum())} / {int(reject.sum())} / {int(d['single'][idx].sum())}"),
            ("Sampled frames without `W0` (folder `2026-07-23`)", int(np.isnan(w0[idx]).sum())),
        ],
    )

    print("## Bad base rate\n")
    print("The weighted (Horvitz-Thompson, `1/π`) share of the dump's frames labeled bad, the stratified bootstrap 95% interval, the folder-cluster interval, and the unweighted sample share.\n")
    gate.table(
        "| Reading | Weighted % | Stratified 95% | Folder-cluster 95% | Unweighted % |",
        [("Bad base rate", f"{base:.2f}", f"[{base_ci[0]:.2f}, {base_ci[1]:.2f}]", f"[{base_cci[0]:.2f}, {base_cci[1]:.2f}]", f"{100 * bad.mean():.2f}")],
    )

    print("## Reference checks\n")
    rnd = [res[(fw.RANDOM, f"q{q}")]["point"][0] for q in QS]
    lf = rf.label_free(stored_rej["G0"], d, tb, qual, d["reject"] & ~d["pick"])
    lf_rec = float(np.mean([lf[fi]["q95"]["recall"] for fi in qual]))
    lf_rn = float(np.mean([lf[fi]["q95"]["rn"] for fi in qual]))
    my = [rf.point(g0_removed[folder_of == fi], d["pick"][folder_of == fi], (d["reject"] & ~d["pick"])[folder_of == fi]) for fi in qual]
    my_rec, my_rn = float(np.mean([p["recall"] for p in my])), float(np.mean([p["rn"] for p in my]))
    my_rf = float(np.mean([p["rf"] for p in my]))
    gate.table(
        "| Check | Expected | This run |",
        [
            ("Strata (`strata.tsv` rows) whose population, drawn count or `π` differ when recomputed from the dump", "0", f"{strata_diff} of {len(strata_file)}"),
            ("Sampled frames whose recomputed stratum / `π` differ from `labels.tsv`", "0 / 0", f"{stratum_diff} / {pi_diff}"),
            ("Sampled frames whose `G0` / `W0` / `sharpness` in `manifest.tsv` differ from the dump's", "0 / 0 / 0", f"{man_diff('G0', stored_rej['G0'])} / {man_diff('W0', w0)} / {man_diff('sharpness', sharp)}"),
            ("Sum of `1/π` over the sampled frames", f"{n}", f"{total_w:.1f}"),
            ("Random (seed 0) weighted `BR` % at q = " + " / ".join(str(q) for q in QS), "near q", " / ".join(f1(v) for v in rnd)),
            ("Random (seed 0) weighted AUC / partial AUC", "near 0.5", f"{auc[fw.RANDOM][0][0]:.3f} / {auc[fw.RANDOM][0][1]:.3f}"),
            ("`G0` `quantile@95` over the qualifying folders: mean pick recall / non-pick removal % (`reject.md`: 94.7 / 16.8)", "94.7 / 16.8 (`rf.label_free` " + f"{lf_rec:.1f} / {lf_rn:.1f})", f"{my_rec:.1f} / {my_rn:.1f}"),
            ("`G0` `quantile@95`: mean frame share removed, qualifying folders / dump frames removed, all folders", "-", f"{my_rf:.1f}% / {int(g0_removed.sum())} ({100 * g0_removed.mean():.1f}%)"),
        ],
    )

    print("## Per stratum\n")
    srows = []
    for name in STRATA:
        m = s["stratum"] == name
        a = w * bad * m
        b = w * m
        srows.append([f"`{name}`", int((stratum == name).sum()), int(m.sum()), int((bad & m).sum()), iv(100 * a.sum() / b.sum(), *quant(ratio(cs, a, b))), f1(100 * (bad & m).sum() / m.sum())])
    gate.table("| Stratum | Population | Sampled | Bad (sampled) | Weighted bad rate % [stratified 95%] | Unweighted % |", srows)

    print("## Dump-label consistency\n")
    nonpick = ~pick
    gate.table(
        "| Reading | Value |",
        [
            ("Dump picks labeled bad: weighted share % / sampled count", f"{100 * (w * bad * pick).sum() / (w * pick).sum():.2f} / {int((bad & pick).sum())} of {int(pick.sum())}"),
            ("Sampled explicit dump rejects labeled bad", f"{int((bad & reject).sum())} of {int(reject.sum())}"),
            ("Weighted bad rate among the dump's non-picks % [stratified 95%]", iv(100 * (w * bad * nonpick).sum() / (w * nonpick).sum(), *quant(ratio(cs, w * bad * nonpick, w * nonpick)), 2)),
        ],
    )

    print("## Main table: bad recall, not-bad loss and precision per cut\n")
    print(
        "Weighted (`1/π`), with the stratified bootstrap 95% interval (1,000 draws, seed 0, within each stratum and each `rest` folder cell) "
        "and the unweighted sample value in brackets. `qX`: the folder's bottom X% by the score, cut on the full dump; `q@95`: `G0`'s "
        "label-free `quantile@95` share; `rule`: the rule's own point. Removed: the share of the dump's frames the cut removes (on the "
        "folders the score covers). Out: sampled frames in folders the score does not cover. Bar: the worth-a-filter bar (`NL` <= 5%, "
        "`BR` >= 50% with its 2.5% quantile above 40%, `P` >= 50%). Primary: the score's cut whose `NL` is nearest 5%.\n"
    )
    mrows = []
    for k in names:
        for c in cuts_of(k):
            r = res[(k, c)]
            cells = [f"{iv(r['point'][j], *r['ci'][j])} ({f1(r['unw'][j])})" for j in range(3)]
            mrows.append([k if "`" in k else f"`{k}`", c + (" (primary)" if primary[k] == c else ""), f1(r["dump"]), r["out"]] + cells + ["pass" if r["pass"] else "-"])
    gate.table("| Score | Cut | Removed % | Out | `BR` % | `NL` % | `P` % | Bar |", mrows)

    print("## Threshold-free: weighted AUC and partial AUC at not-bad loss <= 10%\n")
    print(
        "Bad vs not bad on the sampled frames, `1/π` weights, pooled over folders (missing values at the folder's lowest finite score minus 1); "
        "partial AUC McClish-standardized, chance 0.5. A rule's AUC is that of its keep / remove split. Stratified 95% interval; unweighted in brackets.\n"
    )
    arows = []
    for k in names:
        (pa, pp), (ua, up), lo_a, lo_p = auc[k]
        arows.append([k if "`" in k else f"`{k}`", f"{iv(pa, *lo_a, 3)} ({ua:.3f})", f"{iv(pp, *lo_p, 3)} ({up:.3f})"])
    gate.table("| Score | AUC | Partial AUC (`NL` <= 10%) |", arows)

    print("## Per cue: weighted bad rate per bin\n")
    print(
        f"The weighted bad rate of each bin (stratified 95% interval; sampled frames and bad among them), against the base rate {base:.2f}%. "
        f"Firing: the bin the cue's rule removes. Catches: the firing bin's rate is >= {CUE_FACTOR:.0f}x the base rate and its 2.5% quantile above the base rate.\n"
    )
    e, fv, ay, ap = ear[idx], ef[idx], np.abs(yaw[idx]), np.abs(pitch[idx])
    dec = np.zeros(n, dtype=int)
    for fi in range(len(folders)):
        m = np.where(folder_of == fi)[0]
        v = m[~np.isnan(sharp[m])]
        o = v[rf.bottom(sharp[v], tb[v])]
        dec[o] = np.arange(len(o)) * 10 // len(o) + 1
    dc = dec[idx]
    with np.errstate(invalid="ignore"):
        bins = [
            ("raw EAR", "<= 0.137 (closed)", e <= rf.EYES_CLOSED_EAR, True),
            ("raw EAR", "(0.137, 0.25]", (e > rf.EYES_CLOSED_EAR) & (e <= 0.25), False),
            ("raw EAR", "> 0.25", e > 0.25, False),
            ("raw EAR", "missing", np.isnan(e), False),
            ("`eye_focus`", "< 0.5", fv < 0.5, False),
            ("`eye_focus`", "[0.5, 0.9)", (fv >= 0.5) & (fv < rf.GOOD_EYE_FOCUS), False),
            ("`eye_focus`", "< 0.9 (the two lower bins)", fv < rf.GOOD_EYE_FOCUS, True),
            ("`eye_focus`", ">= 0.9", fv >= rf.GOOD_EYE_FOCUS, False),
            ("`eye_focus`", "missing (no AF face)", np.isnan(fv), False),
            ("`\\|yaw\\|`", "<= 35", ay <= rf.GOOD_MAX_YAW, False),
            ("`\\|yaw\\|`", "> 35", ay > rf.GOOD_MAX_YAW, True),
            ("`\\|yaw\\|`", "missing", np.isnan(ay), False),
            ("`\\|pitch\\|`", "<= 45", ap <= rf.GOOD_MAX_PITCH, False),
            ("`\\|pitch\\|`", "> 45", ap > rf.GOOD_MAX_PITCH, True),
            ("`\\|pitch\\|`", "missing", np.isnan(ap), False),
        ]
    bins += [("Sharpness decile", str(k), dc == k, k == 1) for k in range(1, 11)]
    bins += [("Sharpness decile", "missing", dc == 0, False), ("`tier`", "good", tier[idx], False), ("`tier`", "not good", ~tier[idx], True)]
    crow, cue_res = [], []
    for cue, label, m, firing in bins:
        a, b = w * bad * m, w * m
        if not m.any():
            crow.append([cue, label, 0, 0, "-", "-", "-", "yes" if firing else "", "-"])
            continue
        rate = 100 * a.sum() / b.sum()
        lo, hi = quant(ratio(cs, a, b))
        catches = rate >= CUE_FACTOR * base and lo > base
        if firing:
            cue_res.append((cue, label, rate, (lo, hi), quant(ratio(cc, a, b)), catches))
        crow.append([cue, label, int(m.sum()), int((bad & m).sum()), iv(rate, lo, hi), f1(100 * (bad & m).sum() / m.sum()), f"{rate / base:.2f}", "yes" if firing else "",
                     ("yes" if catches else "no") if firing else ""])
    gate.table("| Cue | Bin | Sampled | Bad | Weighted bad rate % | Unweighted % | x base | Firing | Catches |", crow)

    best = None
    for k in names:
        for c in cuts_of(k):
            r = res[(k, c)]
            if r["point"][1] <= LOSS_CEIL and (best is None or r["point"][0] > res[best]["point"][0]):
                best = (k, c)
    print("## Worth-a-filter bar\n")
    print("Per score: its primary cut, the best `BR` among its cuts with `NL` <= 5%, and whether any of its cuts passes the bar.\n")
    brows = []
    for k in names:
        live = [c for c in cuts_of(k) if res[(k, c)]["point"][1] <= LOSS_CEIL]
        top = max(live, key=lambda c: res[(k, c)]["point"][0]) if live else None
        passing = [c for c in cuts_of(k) if res[(k, c)]["pass"]]
        r = res[(k, primary[k])]
        brows.append([k if "`" in k else f"`{k}`", primary[k], f"{f1(r['point'][0])} / {f1(r['point'][1])} / {f1(r['point'][2])}",
                      "-" if top is None else f"{top}: {iv(res[(k, top)]['point'][0], *res[(k, top)]['ci'][0])}", ", ".join(passing) if passing else "no"])
    gate.table("| Score | Primary cut | Primary `BR` / `NL` / `P` % | Best `BR` % with `NL` <= 5% | Passes at |", brows)
    bk, bc = best
    rb = res[best]
    print(
        f"Best existing filter (highest weighted `BR` among the cuts with `NL` <= 5%): `{bk}` at `{bc}`, `BR` {iv(rb['point'][0], *rb['ci'][0])}%, "
        f"`NL` {iv(rb['point'][1], *rb['ci'][1])}%, `P` {iv(rb['point'][2], *rb['ci'][2])}%: {'passes' if rb['pass'] else 'does not pass'} the worth-a-filter bar.\n"
    )

    print("## Paired differences at the primary cut\n")
    print(
        "Score minus comparison, in points, with the stratified bootstrap 95% interval (the same draws for both). A graded score and a graded comparison "
        "are read at the score's primary cut; a rule or `G0` `quantile@95` against the comparison at the comparison's primary cut. Frames in folders "
        "either score does not cover are left out of both.\n"
    )
    prows = []
    for base_k in ("G0", "W0"):
        for k in names:
            if k == base_k:
                continue
            ck = primary[k]
            cb = ck if ck.startswith("q") and ck != "q@95" and base_k in graded else primary[base_k]
            ok = res[(k, ck)]["ok"] & res[(base_k, cb)]["ok"]
            a = rates(s, res[(k, ck)]["rem"], ok)
            b = rates(s, res[(base_k, cb)]["rem"], ok)
            ab = rates(s, res[(k, ck)]["rem"], ok, cs)
            bb = rates(s, res[(base_k, cb)]["rem"], ok, cs)
            prows.append([f"{k if '`' in k else f'`{k}`'} ({ck}) - `{base_k}` ({cb})"] + [civ(a[j] - b[j], *quant(ab[j] - bb[j])) for j in range(3)])
    gate.table("| Comparison | `BR`, points | `NL`, points | `P`, points |", prows)

    print("## Folder-cluster sensitivity\n")
    print("The 1,000-draw folder-cluster bootstrap (resample the 27 folders, seed 0) beside the stratified interval, for the base rate, each score at its primary cut and each cue's firing bin.\n")
    krows = [["Bad base rate", "-", f"{base:.1f} [{base_ci[0]:.1f}, {base_ci[1]:.1f}]", f"[{base_cci[0]:.1f}, {base_cci[1]:.1f}]", "", "", "", "", ""]]
    for k in names:
        r = res[(k, primary[k])]
        krows.append([k if "`" in k else f"`{k}`", primary[k]] + sum([[iv(r["point"][j], *r["ci"][j]), f"[{r['cci'][j][0]:.1f}, {r['cci'][j][1]:.1f}]"] for j in range(3)], []) + [
            ("pass" + ("" if r["cpass"] else ", fragile")) if r["pass"] else "-"])
    gate.table("| Reading | Cut | `BR` % (stratified) | Cluster | `NL` % (stratified) | Cluster | `P` % (stratified) | Cluster | Bar |", krows)
    gate.table(
        "| Cue firing bin | Bin | Weighted bad rate % (stratified) | Cluster | Catches |",
        [[cue, label, iv(rate, *ci), f"[{cc_[0]:.1f}, {cc_[1]:.1f}]", "yes" + ("" if cc_[0] > base else ", fragile") if catches else "no"] for cue, label, rate, ci, cc_, catches in cue_res],
    )

    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Load (dump, stored scores, rules, labels)", f"{load_time:.0f}"), ("Cuts, metrics and bootstraps", f"{metric_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.")
    sys.stdout = tee.stream
    with open(os.path.join(out_dir, "badfilter.out.md"), "w", encoding="utf-8", newline="") as out:
        out.write("".join(tee.parts))


if __name__ == "__main__":
    if len(sys.argv) != 5:
        raise SystemExit(__doc__)
    main(*sys.argv[1:5])
