"""Decision 1's evaluation: the archived ranking gate's shared model (the
reference) against the shared model plus a candidate's per-frame columns,
on the same frames, bursts, pairs, folds, `C` choice and metrics. The
criterion is pre-registered in plan.md.

Usage: python -I -B ballgate.py <gate-dir> <candidate>... [--labels <labels.tsv>]
       [--curve <name>] [--jobs <n>]

<gate-dir> is read by the archived gate.py's own `load` / `build` (the app
dump `D:\\Photos\\tests\\2026-10-12-rank-app\\`). Each <candidate> is one
extras file, or several joined by commas for a combination; an extras file
`extras_<name>.tsv` has a header `folder\\tfile\\t<column>...` and one row per
frame. The candidate's name is its files' names joined by `+`. A frame
without a row gets the training folders' mean of the column; more than 0.1%
missing is an error. Candidate columns are standardized on the training
folders like the technical ones and appended to the shared model's design
matrix; nothing else changes.

--labels reads the hand labels (`id\\tball\\tcontest`, ball one of `visible`
/ `none` / `unsure`, contest `yes` / `no` / `na`) with the `frames.tsv` next
to it (ball_sample.py) and gives the frame-level ROC AUC of every column
against both labels; bar 4 reads the candidate's first column whose name
starts with `ball`. --curve runs the learning curve for that candidate.
Prints Markdown tables on stdout.
"""

import csv
import os
import sys

import numpy as np
from joblib import Parallel, delayed
from sklearn.metrics import roc_auc_score

sys.dont_write_bytecode = True
sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_archived", "20261012-image-embeddings"),
)
import gate  # noqa: E402

MAX_MISSING = 0.001
BAR_AUC = 0.010
BAR_TOP1 = 0.020
BAR_FOLDERS = 13
BAR_LABEL_AUC = 0.75
REFERENCE = "Reference (shared model)"


def load_extras(paths, d):
    names, cols = [], []
    keys = [(f["folder"], f["file"]) for f in d["rows"]]
    for path in paths:
        with open(path, newline="", encoding="utf-8") as f:
            reader = csv.reader(f, delimiter="\t")
            head = next(reader)
            if head[:2] != ["folder", "file"] or len(head) < 3:
                raise SystemExit(f"{path}: the header must be folder, file, then the columns")
            by = {(r[0], r[1]): [gate.num(v) for v in r[2:]] for r in reader}
        missing = sum(1 for k in keys if k not in by)
        if missing > MAX_MISSING * len(keys):
            raise SystemExit(f"{path}: {missing} of {len(keys)} frames have no row")
        nan = [np.nan] * (len(head) - 2)
        block = np.array([[np.nan if v is None else v for v in by.get(k, nan)] for k in keys], dtype=np.float64)
        names += head[2:]
        cols.append(block)
    return names, np.hstack(cols)


def design(d, train_mask, extras):
    x = gate.design(d, train_mask, "shared")
    e = extras.copy()
    mean = np.nanmean(e[train_mask], axis=0)
    idx = np.where(np.isnan(e))
    e[idx] = np.take(mean, idx[1])
    mu, sd = e[train_mask].mean(axis=0), e[train_mask].std(axis=0)
    sd[sd == 0] = 1
    return np.hstack([x, (e - mu) / sd])


def run_fold(d, sc, held, train_folders, extras, tiebreak):
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    train_mask = np.isin(d["folder_of"], list(train_folders))
    x = design(d, train_mask, extras)
    train = [b for b in sc if folder_of_burst[b] in train_folders]
    c = gate.choose_c(d, train, x, folder_of_burst, tiebreak)
    w = gate.fit(d, train, x, c)
    return held, c, x[d["folder_of"] == held] @ w


def lofo(d, sc, extras, tiebreak, jobs):
    folds = sorted({d["bursts"][b][0] for b in sc})
    allf = set(range(len(d["folders"])))
    res = Parallel(n_jobs=jobs)(delayed(run_fold)(d, sc, h, allf - {h}, extras, tiebreak) for h in folds)
    score = np.full(len(d["rows"]), np.nan)
    cs = {}
    for held, c, s in res:
        score[d["folder_of"] == held] = s
        cs[held] = c
    return score, cs


def curve(d, sc, extras, tiebreak, jobs):
    """gate.curve's draws (same seeds), on the candidate's design."""
    folds = sorted({d["bursts"][b][0] for b in sc})
    tasks = []
    for k in gate.CURVE:
        for draw in range(gate.CURVE_DRAWS):
            for h in folds:
                rng = np.random.default_rng(1000 * k + 100 * draw + h)
                others = [f for f in range(len(d["folders"])) if f != h]
                tasks.append((k, draw, h, set(rng.choice(others, size=k, replace=False).tolist())))
    res = Parallel(n_jobs=jobs)(delayed(run_fold)(d, sc, h, tr, extras, tiebreak) for _, _, h, tr in tasks)
    out = {}
    for (k, draw, _, _), (held, _, s) in zip(tasks, res):
        score = out.setdefault((k, draw), np.full(len(d["rows"]), np.nan))
        score[d["folder_of"] == held] = s
    table = {}
    for k in gate.CURVE:
        ms = [gate.summarize(gate.burst_metrics(d, sc, out[(k, draw)], tiebreak)) for draw in range(gate.CURVE_DRAWS)]
        table[k] = (np.mean([m["wauc"] for m in ms]), np.mean([m["top1"] for m in ms]))
    return table


def load_labels(path, d):
    """Per frame index: (ball label, contest label), the unlabeled skipped."""
    with open(os.path.join(os.path.dirname(os.path.abspath(path)), "frames.tsv"), newline="", encoding="utf-8") as f:
        frame = {r["id"]: (r["folder"], r["file"]) for r in csv.DictReader(f, delimiter="\t")}
    index = {(f["folder"], f["file"]): i for i, f in enumerate(d["rows"])}
    out = {}
    with open(path, newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            ball, contest = (r.get("ball") or "").strip(), (r.get("contest") or "").strip()
            if not ball and not contest:
                continue
            if ball not in ("visible", "none", "unsure", "") or contest not in ("yes", "no", "na", ""):
                raise SystemExit(f"{path}: id {r['id']}: unknown label {ball!r} / {contest!r}")
            out[index[frame[r["id"]]]] = (ball, contest)
    return out


def label_auc(values, labels, which, positive, negative):
    idx = [i for i, l in labels.items() if l[which] in (positive, negative)]
    y = np.array([labels[i][which] == positive for i in idx])
    v = values[idx]
    keep = ~np.isnan(v)
    if keep.sum() < 2 or y[keep].all() or not y[keep].any():
        return None, int(y.sum()), int((~y).sum())
    return float(roc_auc_score(y[keep], v[keep])), int(y.sum()), int((~y).sum())


def main(argv):
    opts = {"--labels": None, "--curve": None, "--jobs": "8"}
    specs, it = [], iter(argv)
    for a in it:
        if a in opts:
            opts[a] = next(it)
        else:
            specs.append(a)
    if len(specs) < 2:
        raise SystemExit(__doc__)
    gate_dir, specs = specs[0], specs[1:]
    jobs = int(opts["--jobs"])

    frames, missing = gate.load(gate_dir)
    d = gate.build(frames, gate.load_text(os.path.join(gate_dir, "genre_text.tsv")))
    sc = gate.scorable(d)
    tiebreak = np.random.default_rng(0).random(len(d["rows"]))
    sports = d["genre"][:, d["genre_names"].index("sports")]
    sports_bursts = [b for b in sc if int(np.argmax(d["genre"][d["bursts"][b][1]])) == d["genre_names"].index("sports")]
    other_bursts = [b for b in sc if b not in set(sports_bursts)]
    rej_bursts = [b for b in sc if d["reject"][d["bursts"][b][1] : d["bursts"][b][2]].any()]

    cands = {}
    for spec in specs:
        paths = spec.split(",")
        name = "+".join(os.path.splitext(os.path.basename(p))[0].removeprefix("extras_") for p in paths)
        cands[name] = load_extras(paths, d)
    if opts["--curve"] and opts["--curve"] not in cands:
        raise SystemExit(f"--curve {opts['--curve']}: not one of {list(cands)}")
    labels = load_labels(opts["--labels"], d) if opts["--labels"] else None

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders", len(d["folders"])),
            ("Frames / picks / explicit rejects", f"{len(d['rows'])} / {d['pick'].sum()} / {d['reject'].sum()}"),
            ("Frames without a vector (dropped)", missing),
            ("Scorable bursts (sports / other by burst argmax)", f"{len(sc)} ({len(sports_bursts)} / {len(other_bursts)})"),
            ("Scorable bursts with an explicit reject", len(rej_bursts)),
        ]
        + [(f"`{n}` columns (frames without a row)", f"{', '.join(c)} ({int(np.isnan(e).all(axis=1).sum())})") for n, (c, e) in cands.items()],
    )

    scores, cs = {}, {}
    scores[REFERENCE], cs[REFERENCE] = gate.lofo(d, sc, "shared", tiebreak, jobs)
    for n, (_, e) in cands.items():
        scores[n], cs[n] = lofo(d, sc, e, tiebreak, jobs)
    vals = {k: gate.burst_metrics(d, sc, s, tiebreak) for k, s in scores.items()}
    summ = {k: gate.summarize(v) for k, v in vals.items()}
    pf = {k: (gate.per_folder(d, v, "wauc"), gate.per_folder(d, v, "top1")) for k, v in vals.items()}

    print("## Held-out results (leave one folder out)\n")
    rows = []
    for k, m in summ.items():
        rows.append([k, m["bursts"], gate.f3(m["wauc"]), gate.f3(gate.spread(pf[k][0])), gate.f3(m["uauc"]), gate.pc(m["top1"]), gate.pc(gate.spread(pf[k][1])), gate.pc(m["top3"])])
    gate.table("| Ranking | Bursts | Weighted pairwise AUC | Per-folder SD | Unweighted pairwise AUC | Top-1 | Per-folder SD | Top-3 |", rows)
    print("`C` chosen per fold (count of folds):\n")
    gate.table("| Model | " + " | ".join(str(c) for c in gate.CS) + " |", [[k] + [sum(1 for v in cs[k].values() if v == c) for c in gate.CS] for k in cs])

    print("## Per folder (held out)\n")
    ref_auc = pf[REFERENCE][0]
    big = [fi for fi, (n, _) in ref_auc.items() if n >= gate.MIN_SPREAD_BURSTS]
    rows = []
    for fi in sorted(ref_auc):
        r = [f"`{d['folders'][fi]}`", ref_auc[fi][0], gate.f3(ref_auc[fi][1]), gate.pc(pf[REFERENCE][1][fi][1])]
        for n in cands:
            a = pf[n][0][fi][1]
            r += [gate.f3(a), f"{a - ref_auc[fi][1]:+.3f}", gate.pc(pf[n][1][fi][1])]
        rows.append(r)
    gate.table("| Folder | Bursts | Reference AUC | Top-1 | " + " | ".join(f"`{n}` AUC | Delta | Top-1" for n in cands) + " |", rows)
    at_or_above = {n: sum(1 for fi in big if pf[n][0][fi][1] >= ref_auc[fi][1]) for n in cands}
    print(f"Folders with {gate.MIN_SPREAD_BURSTS} or more scorable bursts: {len(big)}. At or above the reference there: " + ", ".join(f"`{n}` {at_or_above[n]}" for n in cands) + ".\n")

    if labels is not None:
        print("## Columns against the hand labels (frame-level ROC AUC)\n")
        nb = sum(1 for l in labels.values() if l[0])
        print(f"{len(labels)} labeled frames ({nb} with a `ball` label). `unsure` / `na` excluded.\n")
        rows = []
        for n, (c, e) in cands.items():
            for j, col in enumerate(c):
                ab, pb, nb_ = label_auc(e[:, j], labels, 0, "visible", "none")
                ac, pc_, nc = label_auc(e[:, j], labels, 1, "yes", "no")
                rows.append([f"`{n}`", col, gate.f3(ab), f"{pb} / {nb_}", gate.f3(ac), f"{pc_} / {nc}"])
        gate.table("| Candidate | Column | AUC vs `ball` | Visible / none | AUC vs `contest` | Yes / no |", rows)

    print("## Criterion\n")
    rows = []
    ref = summ[REFERENCE]
    print(f"Reference: weighted AUC {gate.f3(ref['wauc'])}, Top-1 {gate.pc(ref['top1'])} (archived ranking.md: 0.710 / 54.9%).\n")
    for n, (c, e) in cands.items():
        da, dt = summ[n]["wauc"] - ref["wauc"], summ[n]["top1"] - ref["top1"]
        ok = [da >= BAR_AUC, dt >= BAR_TOP1, at_or_above[n] >= BAR_FOLDERS]
        ball = next((j for j, col in enumerate(c) if col.startswith("ball")), None)
        if labels is None or ball is None:
            b4, ok4 = ("no `--labels`" if labels is None else "no `ball` column"), None
        else:
            a4 = label_auc(e[:, ball], labels, 0, "visible", "none")[0]
            b4, ok4 = f"{gate.f3(a4)} (`{c[ball]}`)", a4 is not None and a4 >= BAR_LABEL_AUC
        verdict = "fail" if not all(ok) or ok4 is False else ("pass" if ok4 else "bars 1-3 pass, bar 4 pending")
        yn = lambda v: "-" if v is None else ("yes" if v else "no")
        rows.append([f"`{n}`", f"{da:+.3f}", yn(ok[0]), f"{100 * dt:+.1f}", yn(ok[1]), f"{at_or_above[n]} of {len(big)}", yn(ok[2]), b4, yn(ok4), verdict])
    gate.table(
        f"| Candidate | 1. AUC minus reference (>= {BAR_AUC:.3f}) | Met | 2. Top-1 minus reference, points (>= {100 * BAR_TOP1:.1f}) | Met | 3. Folders at or above (>= {BAR_FOLDERS}) | Met | 4. `ball` column AUC vs label (>= {BAR_LABEL_AUC}) | Met | Verdict |",
        rows,
    )

    print("## Reading aids (not in the criterion)\n")
    print("### Each column alone as a within-burst ranking\n")
    rows = []
    for n, (c, e) in cands.items():
        for j, col in enumerate(c):
            m = gate.summarize(gate.burst_metrics(d, sc, np.where(np.isnan(e[:, j]), -np.inf, e[:, j]), tiebreak))
            rows.append([f"`{n}`", col, gate.f3(m["wauc"]), gate.pc(m["top1"]), gate.pc(m["top3"])])
    gate.table("| Candidate | Column | Weighted AUC | Top-1 | Top-3 |", rows)

    print("### Columns gated by the burst's sports probability\n")
    gated = {}
    for n, (_, e) in cands.items():
        g = e.copy()
        mean = np.nanmean(g, axis=0)
        idx = np.where(np.isnan(g))
        g[idx] = np.take(mean, idx[1])
        gated[n] = gate.summarize(gate.burst_metrics(d, sc, lofo(d, sc, g * sports[:, None], tiebreak, jobs)[0], tiebreak))
    rows = [[f"`{n}` x P(sports)", gate.f3(m["wauc"]), f"{m['wauc'] - ref['wauc']:+.3f}", gate.pc(m["top1"]), f"{100 * (m['top1'] - ref['top1']):+.1f}"] for n, m in gated.items()]
    gate.table("| Model | Weighted AUC | Minus reference | Top-1 | Minus reference, points |", rows)

    print("### Sports and non-sports bursts\n")
    rows = []
    for k in scores:
        r = [k if k == REFERENCE else f"`{k}`"]
        for bs in (sports_bursts, other_bursts):
            m = gate.summarize({b: vals[k][b] for b in bs if b in vals[k]})
            r += [gate.f3(m["wauc"]) if m else "-", gate.pc(m["top1"]) if m else "-"]
        rows.append(r)
    gate.table(f"| Model | Sports AUC ({len(sports_bursts)}) | Top-1 | Other AUC ({len(other_bursts)}) | Top-1 |", rows)

    print("### Pick vs explicit reject\n")
    rows = []
    for k, s in scores.items():
        m = gate.summarize(gate.burst_metrics(d, rej_bursts, s, tiebreak, negatives=d["reject"]))
        rows.append([k if k == REFERENCE else f"`{k}`", m["bursts"] if m else 0, gate.f3(m["wauc"]) if m else "-", gate.f3(m["uauc"]) if m else "-"])
    gate.table("| Model | Bursts | Weighted AUC | Unweighted AUC |", rows)

    if opts["--curve"]:
        n = opts["--curve"]
        print(f"## Learning curve (`{n}`)\n")
        lc = curve(d, sc, cands[n][1], tiebreak, jobs)
        rows = [[k, gate.f3(lc[k][0]), gate.pc(lc[k][1])] for k in gate.CURVE]
        rows.append([len(set(d["bursts"][b][0] for b in sc)), gate.f3(summ[n]["wauc"]), gate.pc(summ[n]["top1"])])
        gate.table("| Training folders | Weighted AUC | Top-1 |", rows)

    print("## Weights of each candidate fit on every folder\n")
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    allm = np.ones(len(d["rows"]), dtype=bool)
    names = gate.TECH + gate.POSITION
    for n, (c, e) in cands.items():
        x = design(d, allm, e)
        cc = gate.choose_c(d, sc, x, folder_of_burst, tiebreak)
        w = gate.fit(d, sc, x, cc)
        rows = [[col, f"{w[i]:+.3f}"] for i, col in enumerate(names)]
        rows.append(["PCA dims (L2 norm of the 32)", f"{np.linalg.norm(w[len(names) : len(names) + gate.PCA_DIMS]):.3f}"])
        rows += [[col, f"{w[len(names) + gate.PCA_DIMS + j]:+.3f}"] for j, col in enumerate(c)]
        print(f"`{n}`: `C` = {cc} ({len(d['folders'])} folders); standardized columns.\n")
        gate.table("| Column | Weight |", rows)


if __name__ == "__main__":
    main(sys.argv[1:])
