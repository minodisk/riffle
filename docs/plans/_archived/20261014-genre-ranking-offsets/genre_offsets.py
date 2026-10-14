"""Step 1: the shared ranking model against shared + per-genre offsets,
held out by folder, on a gate directory (the app dump). The bar is in
offsets.md, written before the run.

Usage: python -I genre_offsets.py <gate-dir> <genres.json> [jobs]

<gate-dir> is read by the archived gate.py's own `load` / `build`;
<genres.json> is the shipped `crates/core/genres/siglip2.json`, which gives
each burst its genre by the rule `genre.rs` uses. Prints the tables of
offsets.md (Markdown) on stdout.
"""

import json
import math
import os
import sys

import numpy as np
from joblib import Parallel, delayed

sys.dont_write_bytecode = True
sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_archived", "20261012-image-embeddings"),
)
import gate  # noqa: E402

SHRINKS = (3, 10, 30, 100)
MIN_GENRE_BURSTS = 20
FALLBACK_BURSTS = 20
FALLBACK_DRAWS = 3
BOOT = 1000
POOLED_MARGIN = 0.002
FALLBACK_MARGIN = 0.005
UNKNOWN = "Unknown"
SCOPES = {"all": None, "tech": len(gate.TECH)}


# ---------------------------------------------------------------- genres


def burst_genres(d, path):
    """The top-level genre of every burst: the label on for the most of its
    frames (a sub-genre bringing its parent), ties to the earlier label,
    UNKNOWN when none is on."""
    with open(path, encoding="utf-8") as f:
        labels = json.load(f)["labels"]
    names = [lab["label"] for lab in labels]
    vec = np.array([lab["vector"] for lab in labels], dtype=np.float64)
    thr = np.array([lab["threshold"] for lab in labels])
    on = (d["emb"] @ vec.T) >= thr
    for i, lab in enumerate(labels):
        if lab["parent"] is not None:
            on[:, names.index(lab["parent"])] |= on[:, i]
    top = [i for i, lab in enumerate(labels) if lab["parent"] is None]
    out = []
    for _, s, e in d["bursts"]:
        counts = on[s:e][:, top].sum(axis=0)
        out.append(names[top[int(np.argmax(counts))]] if counts.max() > 0 else UNKNOWN)
    return [names[i] for i in top], out


# ---------------------------------------------------------------- model


def hier_design(d, train_mask, genre_of_row, tops, shrink, scope):
    x = gate.design(d, train_mask, "shared")
    if shrink is None:
        return x
    cols = x if SCOPES[scope] is None else x[:, : SCOPES[scope]]
    s = 1 / math.sqrt(shrink)
    extra = [(genre_of_row == g)[:, None] * cols * s for g in tops]
    return np.hstack([x] + extra)


def run_fold(d, sc, held, train, genre_of_row, tops, shrink, scope, tiebreak):
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    train_mask = d["folder_of"] != held
    x = hier_design(d, train_mask, genre_of_row, tops, shrink, scope)
    c = gate.choose_c(d, train, x, folder_of_burst, tiebreak)
    w = gate.fit(d, train, x, c)
    return held, c, x[d["folder_of"] == held] @ w


def lofo(d, sc, folds, trains, genre_of_row, tops, shrink, scope, tiebreak, jobs):
    res = Parallel(n_jobs=jobs)(
        delayed(run_fold)(d, sc, h, trains[h], genre_of_row, tops, shrink, scope, tiebreak) for h in folds
    )
    score = np.full(len(d["rows"]), np.nan)
    cs = {}
    for held, c, s in res:
        score[d["folder_of"] == held] = s
        cs[held] = c
    return score, cs


# ---------------------------------------------------------------- report


def wauc(vals, bursts):
    v = [vals[b][0] for b in bursts if b in vals]
    return float(np.mean(v)) if v else None


def top1(vals, bursts):
    v = [vals[b][3] for b in bursts if b in vals]
    return float(np.mean(v)) if v else None


def boot(vals, base, bursts, rng):
    bs = [b for b in bursts if b in vals and b in base]
    diff = np.array([vals[b][0] - base[b][0] for b in bs])
    draws = rng.integers(0, len(diff), size=(BOOT, len(diff)))
    means = diff[draws].mean(axis=1)
    return diff.mean(), np.quantile(means, 0.025), np.quantile(means, 0.975)


def signed(x):
    return f"{x:+.3f}"


def main(gate_dir, genres_path, jobs):
    frames, missing = gate.load(gate_dir)
    d = gate.build(frames, gate.load_text(os.path.join(gate_dir, "genre_text.tsv")))
    sc = gate.scorable(d)
    tiebreak = np.random.default_rng(0).random(len(d["rows"]))
    tops, bgenre = burst_genres(d, genres_path)
    genre_of_row = np.empty(len(d["rows"]), dtype=object)
    for b, (_, s, e) in enumerate(d["bursts"]):
        genre_of_row[s:e] = bgenre[b]
    folds = sorted({d["bursts"][b][0] for b in sc})
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    trains = {h: [b for b in sc if folder_of_burst[b] != h] for h in folds}

    print(f"{len(d['folders'])} folders, {len(d['rows'])} frames ({missing} without a vector), "
          f"{len(d['bursts'])} bursts, {len(sc)} scorable.\n")
    print("## Bursts per genre (shipped rule)\n")
    order = tops + [UNKNOWN]
    by_genre = {g: [b for b in sc if bgenre[b] == g] for g in order}
    gate.table(
        "| Genre | Bursts | Scorable bursts |",
        [(g, sum(1 for x in bgenre if x == g), len(by_genre[g])) for g in order],
    )
    big = [g for g in order if len(by_genre[g]) >= MIN_GENRE_BURSTS]
    ranked = sorted((g for g in tops), key=lambda g: -len(by_genre[g]))
    second = ranked[1]

    def evaluate(score):
        return gate.burst_metrics(d, sc, score, tiebreak)

    variants = {}
    cs = {}
    score, cs["shared"] = lofo(d, sc, folds, trains, genre_of_row, tops, None, "all", tiebreak, jobs)
    variants["shared"] = evaluate(score)
    for k in SHRINKS:
        name = f"all, OFFSET_SHRINK {k}"
        score, cs[name] = lofo(d, sc, folds, trains, genre_of_row, tops, k, "all", tiebreak, jobs)
        variants[name] = evaluate(score)

    def cut_trains(draw):
        out = {}
        for h in folds:
            rng = np.random.default_rng(100 * draw + h)
            pool = [b for b in trains[h] if bgenre[b] == second]
            keep = set(rng.choice(pool, size=min(FALLBACK_BURSTS, len(pool)), replace=False).tolist())
            out[h] = [b for b in trains[h] if bgenre[b] != second or b in keep]
        return out

    cuts = [cut_trains(draw) for draw in range(FALLBACK_DRAWS)]

    def fallback(shrink, scope):
        out = []
        for cut in cuts:
            score, _ = lofo(d, sc, folds, cut, genre_of_row, tops, shrink, scope, tiebreak, jobs)
            out.append(wauc(evaluate(score), by_genre[second]))
        return out

    fb = {"shared": fallback(None, "all")}
    for k in SHRINKS:
        fb[f"all, OFFSET_SHRINK {k}"] = fallback(k, "all")

    def passes(name):
        dp = wauc(variants[name], sc) - wauc(variants["shared"], sc)
        df = abs(np.mean(fb[name]) - np.mean(fb["shared"]))
        return dp >= -POOLED_MARGIN and df <= FALLBACK_MARGIN

    def best(scope):
        ok = [k for k in SHRINKS if passes(f"{scope}, OFFSET_SHRINK {k}")]
        return max(ok, key=lambda k: wauc(variants[f"{scope}, OFFSET_SHRINK {k}"], sc)) if ok else None

    chosen, scope = best("all"), "all"
    tech_shrinks = [chosen] if chosen is not None else list(SHRINKS)
    for k in tech_shrinks:
        name = f"tech, OFFSET_SHRINK {k}"
        score, cs[name] = lofo(d, sc, folds, trains, genre_of_row, tops, k, "tech", tiebreak, jobs)
        variants[name] = evaluate(score)
        fb[name] = fallback(k, "tech")
    if chosen is None:
        chosen, scope = best("tech"), "tech"

    print("## Pooled (held out, leave one folder out)\n")
    rows = []
    for name, vals in variants.items():
        m = gate.summarize(vals)
        rows.append([name, m["bursts"], gate.f3(m["wauc"]), gate.f3(m["uauc"]), gate.pc(m["top1"]), gate.pc(m["top3"]),
                     "-" if name == "shared" else signed(m["wauc"] - wauc(variants["shared"], sc))])
    gate.table("| Variant | Bursts | Weighted AUC | Unweighted AUC | Top-1 | Top-3 | Minus shared |", rows)

    print("`C` chosen per fold (count of folds):\n")
    gate.table("| Variant | " + " | ".join(str(c) for c in gate.CS) + " |",
               [[n] + [sum(1 for v in cs[n].values() if v == c) for c in gate.CS] for n in cs])

    print(f"## Per genre (genres with at least {MIN_GENRE_BURSTS} scorable bursts)\n")
    rows = []
    for name, vals in variants.items():
        r = [name]
        for g in big:
            r += [gate.f3(wauc(vals, by_genre[g])), gate.pc(top1(vals, by_genre[g]))]
        rows.append(r)
    gate.table("| Variant | " + " | ".join(f"{g} ({len(by_genre[g])}) AUC | Top-1" for g in big) + " |", rows)

    print("## Paired per-burst difference (variant minus shared), bootstrap 95%\n")
    rows = []
    for name, vals in variants.items():
        if name == "shared":
            continue
        r = [name]
        for group in [sc] + [by_genre[g] for g in big]:
            m, lo, hi = boot(vals, variants["shared"], group, np.random.default_rng(0))
            r.append(f"{signed(m)} [{signed(lo)}, {signed(hi)}]")
        rows.append(r)
    gate.table("| Variant | Pooled | " + " | ".join(big) + " |", rows)

    print(f"## Fallback: `{second}` training bursts cut to {FALLBACK_BURSTS}\n")
    rows = []
    for name, v in fb.items():
        rows.append([name] + [gate.f3(x) for x in v] + [gate.f3(np.mean(v)),
                     "-" if name == "shared" else signed(np.mean(v) - np.mean(fb["shared"]))])
    gate.table("| Variant | " + " | ".join(f"Draw {i}" for i in range(FALLBACK_DRAWS)) + " | Mean | Minus shared |", rows)

    print("## Decision\n")
    rows = []
    for name in variants:
        if name == "shared":
            continue
        dp = wauc(variants[name], sc) - wauc(variants["shared"], sc)
        df = np.mean(fb[name]) - np.mean(fb["shared"])
        rows.append([name, signed(dp), signed(df), "pass" if passes(name) else "fail"])
    gate.table(f"| Variant | Pooled minus shared (>= -{POOLED_MARGIN}) | Fallback minus shared (\\|x\\| <= {FALLBACK_MARGIN}) | Result |", rows)
    if chosen is None:
        print("Drop: no variant passes.")
    else:
        print(f"Keep on: OFFSET_SHRINK = {chosen}, offsets on {'all columns' if scope == 'all' else 'the technical columns'}.")


if __name__ == "__main__":
    if len(sys.argv) not in (3, 4):
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 8)
