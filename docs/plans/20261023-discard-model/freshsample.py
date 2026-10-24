"""Step 5: the fresh blind sample of the discard confirmation (the frozen
winner `LR/T`'s per-folder bottom 5 / 10 / 20% bands on the six fresh folders,
`ef`'s bottom 20% outside the winner's, and per-folder random controls), as
shuffled RAW copies with their manifest, by discardmodel.md's
"Pre-registration of the confirmation".

Usage: python -I -B freshsample.py <app-dump-dir> <face-choice-dir> <discard-labels-dir> <model-search-dir> <fresh-dump-dir> <out-dir> [n] [--no-copy]

<app-dump-dir> is the 27-folder dump and <face-choice-dir> its per-face dump
(`faces\\<folder>.tsv`); <discard-labels-dir> (round 1) and
<model-search-dir> (round 2) give the training labels, `manifest.tsv` and
`labels.tsv` only; <fresh-dump-dir> is Step 4's dump of the six fresh folders
(`features\\`, `embed.tsv`, `genre_text.tsv`, `faces\\`); <out-dir> the data
folder, holding Step 2's `scores_model.tsv` (read for the spot-fold check).
Refits `LR/T` once on all 27 dump folders, scores every fresh frame
(`scores_fresh.tsv`, with `ef` and `ef-fill` beside), cuts the strata, draws,
writes `manifest.tsv` and `strata.tsv`, copies the sampled RAW files (no
sidecar) into `<out-dir>/sample/` and prints the tables (Markdown) and the
checks on stdout. `n` (default 600) is split evenly over the five strata.
`--no-copy` stops after the tables and the files. A rerun draws the same sample
and copies only what is missing or differs, so the copy is resumable. No
fresh label exists; nothing under `sample\\` but the copies is opened.
"""

import os
import shutil
import sys
import time

import numpy as np

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import discardfit as dfit  # noqa: E402

bf, fcr, searchfit = dfit.bf, dfit.fcr, dfit.searchfit
rf, gate, folder_gate, bs = bf.rf, bf.gate, bf.folder_gate, bf.sample

WINNER = "LR/T"
LIBRARY = bs.LIBRARY
N_DEFAULT = 600
CUTS = (5, 10, 20)
STRATA = ["b05", "b10", "b20", "efonly", "control"]
CONTROL_FLOOR = 3
DRAW_SEED = 20261023
FRESH = {"2026-09-13-a": 3045, "2026-09-27-c": 1815, "2026-10-11": 3190, "2026-10-12a": 3185, "2026-10-12b": 1615, "2026-10-17": 1675}
POPULATION = 14525
PICKS, REJECTS = 1795, 68
MISSING_EF, MISSING_FILL = "9.00", "1.18"
DUMP_FRAMES = 32459
EF_GAP = 5.1e-5
SIDECARS = bs.SIDECARS
ROUNDS = dfit.ROUNDS


def face_values(path, d):
    """`af`, `af-whole` and the whole-set faces at or above the floor per frame, from a per-face dump."""
    rows = d["rows"]
    n = len(rows)
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    faces_of, extra = fcr.load_faces(path, index, n)
    af, af_whole, nconf = np.full(n, np.nan), np.full(n, np.nan), np.zeros(n, dtype=int)
    for i, fr in enumerate(faces_of):
        if fr is None:
            continue
        _, vals, nconf[i], _ = fcr.choose(fr)
        af[i], af_whole[i] = vals["af"], vals["af-whole"]
    return af, af_whole, nconf, extra, sum(fr is None for fr in faces_of)


def t_columns(d, ear_raw, ef, af_whole, nconf):
    """`T` as discardfit.py builds it, from `gate.build`'s first ten `tech` columns (no `folder_gate.extend`)."""
    no_ef = np.isnan(ef)
    ef_fill = np.where(no_ef, af_whole, ef)
    no_fill = np.isnan(ef_fill)
    touched = no_ef & ~no_fill
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
    return t, ef_fill


def same(a, b, tol):
    return int(((np.isnan(a) != np.isnan(b)) | (np.abs(np.nan_to_num(a) - np.nan_to_num(b)) > tol)).sum())


def max_gap(a, b):
    m = ~np.isnan(a) & ~np.isnan(b)
    return float(np.max(np.abs(a[m] - b[m]))) if m.any() else 0.0


def yn(ok):
    return "yes" if ok else "no"


def main(dump, face_dir, r1_dir, r2_dir, fresh, out_dir, n_total, copy):
    start = time.time()
    if bs.under(out_dir, LIBRARY):
        raise SystemExit(f"refusing to write under the library {LIBRARY}: {out_dir}")
    sample_dir = os.path.join(out_dir, "sample")
    assert not bs.under(sample_dir, LIBRARY)
    dirs = {1: r1_dir, 2: r2_dir}
    roots = {k: os.path.normcase(os.path.abspath(p)) for k, p in (("round-1", r1_dir), ("round-2", r2_dir), ("sample", sample_dir))}
    opened = {k: set() for k in roots}

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            for k, r in roots.items():
                if p.startswith(r + os.sep):
                    opened[k].add(os.path.relpath(p, r))

    sys.addaudithook(audit)

    # The 27-folder dump: `T` without `extend`, then Step 2's path beside it.
    frames, _ = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n = len(rows)
    af, af_whole, nconf, _, _ = face_values(os.path.join(face_dir, "faces"), d)
    ear = rf.load_ear(dump, d)
    t, _ = t_columns(d, ear, af, af_whole, nconf)
    folder_gate.extend(d, dump)
    stored_ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    t_step2, _ = t_columns(d, ear, stored_ef, af_whole, nconf)
    t_diff = same(t, t_step2, 1e-12)

    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    parts, rinfo = {k: [] for k in ("idx", "discard", "round", "w_clip")}, {}
    for rd in ROUNDS:
        man = bf.read_tsv(os.path.join(dirs[rd], "manifest.tsv"))
        lab = bf.read_tsv(os.path.join(dirs[rd], "labels.tsv"))
        by_name = {r["name"]: r for r in man}
        cap = searchfit.CAP_FACTOR * float(np.median([1 / float(r["inclusion_prob"]) for r in man]))
        idx = np.array([index[(r["folder"], r["file"])] for r in lab])
        w = np.array([1 / float(by_name[r["name"]]["inclusion_prob"]) for r in lab])
        dis = np.array([r["label"] == "discard" for r in lab])
        man_diff = sum(r["name"] not in by_name or (r["folder"], r["file"], r["band"], r["inclusion_prob"])
                       != tuple(by_name[r["name"]][k] for k in ("folder", "file", "band", "inclusion_prob")) for r in lab)
        rinfo[rd] = {"n": len(idx), "keep": int((~dis).sum()), "discard": int(dis.sum()), "labels": {r["label"] for r in lab}, "man_diff": man_diff,
                     "n_man": len(man), "cap": cap, "clipped": int((w > cap).sum())}
        parts["idx"].append(idx)
        parts["discard"].append(dis)
        parts["round"].append(np.full(len(idx), rd))
        parts["w_clip"].append(np.minimum(w, cap))
    L = {k: np.concatenate(v) for k, v in parts.items()}
    L["folder"] = folder_of[L["idx"]]

    spot = max(range(len(folders)), key=lambda fi: int((folder_of == fi).sum()))
    _, spot_c, spot_s = dfit.fold(folder_of, t, L, spot, WINNER)
    stored_lrt = {(r["folder"], r["file"]): float(r[WINNER]) for r in bf.read_tsv(os.path.join(out_dir, "scores_model.tsv"))}
    spot_rows = np.flatnonzero(folder_of == spot)
    spot_ref = np.array([stored_lrt[(rows[i]["folder"], rows[i]["file"])] for i in spot_rows])
    spot_gap = float(np.max(np.abs(spot_s - spot_ref)))

    y = L["discard"].astype(int)
    w_all = dfit.per_round_mean1(L["w_clip"], L["round"])
    fill = np.nanmean(t, axis=0)

    def standardize(cols, mu=None, sd=None):
        x = cols.copy()
        idx = np.where(np.isnan(x))
        x[idx] = np.take(fill, idx[1])
        return x if mu is None else (x - mu) / sd

    raw = standardize(t)
    mu, sd = raw.mean(axis=0), raw.std(axis=0)
    sd[sd == 0] = 1
    x = (raw - mu) / sd
    c = dfit.choose_c(x, L["idx"], y, w_all, L["round"], folder_of[L["idx"]])
    model = searchfit.lr("LR-L2", c).fit(x[L["idx"]], y, sample_weight=w_all)
    print(f"refit: C {c}", file=sys.stderr, flush=True)

    # The fresh folders.
    frames_f, missing_f = gate.load(fresh)
    e = gate.build(frames_f, gate.load_text(os.path.join(fresh, "genre_text.tsv")))
    rows_f, folders_f, fo = e["rows"], e["folders"], e["folder_of"]
    m = len(rows_f)
    af_f, afw_f, nconf_f, extra_f, nofaces_f = face_values(os.path.join(fresh, "faces"), e)
    ear_f = rf.load_ear(fresh, e)
    t_f, fill_f = t_columns(e, ear_f, af_f, afw_f, nconf_f)
    win = -model.decision_function(standardize(t_f, mu, sd))
    feat_ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows_f])
    ef_nan_diff = int((np.isnan(feat_ef) != np.isnan(af_f)).sum())
    ef_gap = max_gap(feat_ef, af_f)
    pick, reject = e["pick"], e["reject"]
    tb = np.random.default_rng(0).random(m)
    size = np.zeros(m, dtype=int)
    for _, s, en in e["bursts"]:
        size[s:en] = en - s
    single = size == 1

    cut_scores = {WINNER: win, "ef": np.where(np.isnan(af_f), -np.inf, af_f), "ef-fill": np.where(np.isnan(fill_f), -np.inf, fill_f), "random": tb}
    cuts, left_out = {}, 0
    for k, sc in cut_scores.items():
        for q in CUTS:
            removed, out = bf.graded_cut(sc, e, tb, q)
            left_out += int(out.sum())
            cuts[(k, q)] = removed
    cw = {q: cuts[(WINNER, q)] for q in CUTS}
    ce = {q: cuts[("ef", q)] for q in CUTS}
    nested = all(((a[5] <= a[10]) & (a[10] <= a[20])).all() for a in (cw, ce))
    band = np.full(m, "control", dtype=object)
    band[ce[20] & ~cw[20]] = "efonly"
    band[cw[20]] = "b20"
    band[cw[10]] = "b10"
    band[cw[5]] = "b05"
    ef_band = np.full(m, "above", dtype=object)
    ef_band[ce[20]] = "b20"
    ef_band[ce[10]] = "b10"
    ef_band[ce[5]] = "b05"

    def rank_share(score):
        out = np.zeros(m)
        for fi in range(len(folders_f)):
            idx = np.where(fo == fi)[0]
            out[idx[rf.bottom(score[idx], tb[idx])]] = np.arange(len(idx)) / len(idx)
        return out

    win_share, ef_share = rank_share(win), rank_share(cut_scores["ef"])

    per = n_total // len(STRATA)
    alloc = [(b, per) for b in STRATA[:-1]] + [("control", n_total - per * (len(STRATA) - 1))]
    rng = np.random.default_rng(DRAW_SEED)
    drawn, prob, strata_rows = [], np.zeros(m), []
    for name, a in alloc[:-1]:
        idx = np.flatnonzero(band == name)
        chosen = np.sort(rng.choice(idx, size=a, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = a / len(idx)
        strata_rows.append((name, "-", len(idx), a))
    ctrl = band == "control"
    ctrl_alloc = bs.allocate({fi: int((ctrl & (fo == fi)).sum()) for fi in range(len(folders_f))}, alloc[-1][1], CONTROL_FLOOR)
    for fi in range(len(folders_f)):
        idx = np.flatnonzero(ctrl & (fo == fi))
        if not len(idx):
            continue
        take = ctrl_alloc[fi]
        chosen = np.sort(rng.choice(idx, size=take, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = take / len(idx)
        strata_rows.append(("control", folders_f[fi], len(idx), take))
    drawn = np.array(drawn)
    order = drawn[np.argsort(rng.random(len(drawn)), kind="stable")]
    in_sample = np.zeros(m, dtype=bool)
    in_sample[drawn] = True
    ht = float((1 / prob[drawn]).sum())

    counts_f = {f: int((fo == fi).sum()) for fi, f in enumerate(folders_f)}
    checks = [
        ("27-folder dump frames", str(DUMP_FRAMES), str(n), n == DUMP_FRAMES),
        ("27-folder dump: `T` without `extend` (`af` value) vs Step 2's path (`extend`, stored `eye_focus`), frames differing (NaN pattern or > 1e-12)", "0", str(t_diff),
         t_diff == 0),
        (f"Spot fold `{folders[spot]}` ({len(spot_rows)} frames) of the `LR/T` recipe vs `scores_model.tsv`: largest difference (`C` {spot_c:g})", "<= 1e-9",
         f"{spot_gap:.2e}", spot_gap <= 1e-9),
    ]
    for rd in ROUNDS:
        ri = rinfo[rd]
        checks += [
            (f"Round {rd}: labels / keep / discard", f"600 / {ROUNDS[rd]['labels'][0]} / {ROUNDS[rd]['labels'][1]}", f"{ri['n']} / {ri['keep']} / {ri['discard']}",
             (ri["n"], ri["keep"], ri["discard"]) == (600, *ROUNDS[rd]["labels"]) and ri["labels"] <= {"keep", "discard"}),
            (f"Round {rd}: `labels.tsv` rows whose name, file, band or `π` differ from `manifest.tsv` / manifest rows", f"0 / {ri['n']}", f"{ri['man_diff']} / {ri['n_man']}",
             ri["man_diff"] == 0 and ri["n_man"] == ri["n"]),
        ]
    checks += [
        ("Fresh frames (with a vector) / without a vector", f"{POPULATION} / 0", f"{m} / {missing_f}", m == POPULATION and missing_f == 0),
        ("Fresh frames per folder", ", ".join(f"{v}" for v in FRESH.values()), ", ".join(str(counts_f.get(f, 0)) for f in FRESH), counts_f == FRESH),
        ("Fresh picks / rejects (`gate.load`)", f"{PICKS} / {REJECTS}", f"{int(pick.sum())} / {int(reject.sum())}", (int(pick.sum()), int(reject.sum())) == (PICKS, REJECTS)),
        ("Fresh frames without a face-dump row / face-dump rows outside the frames", "0 / 0", f"{nofaces_f} / {extra_f}", nofaces_f == 0 and extra_f == 0),
        ("`af` vs the features TSV's `eye_focus`: frames missing on one side only", "0", str(ef_nan_diff), ef_nan_diff == 0),
        ("`af` vs the features TSV's `eye_focus`: largest difference", f"<= {EF_GAP:.1e}", f"{ef_gap:.2e}", ef_gap <= EF_GAP),
        ("`ef` missing % / `ef-fill` missing % of the fresh frames (Step 4)", f"{MISSING_EF} / {MISSING_FILL}", f"{100 * np.isnan(af_f).mean():.2f} / {100 * np.isnan(fill_f).mean():.2f}",
         (f"{100 * np.isnan(af_f).mean():.2f}", f"{100 * np.isnan(fill_f).mean():.2f}") == (MISSING_EF, MISSING_FILL)),
        ("The winner finite on every fresh frame", "yes", yn(np.isfinite(win).all()), bool(np.isfinite(win).all())),
        ("Frames left out of a cut, over the scores and `q`", "0", str(left_out), left_out == 0),
        ("Bands nested (q5 within q10 within q20), the winner's and `ef`'s", "yes", yn(nested), nested),
        ("Frames in no stratum / strata population sum", f"0 / {POPULATION}", f"{int((~np.isin(band, STRATA)).sum())} / {sum(r[2] for r in strata_rows)}",
         not (~np.isin(band, STRATA)).any() and sum(r[2] for r in strata_rows) == POPULATION),
        ("Drawn frames / distinct", f"{n_total} / {n_total}", f"{len(drawn)} / {len(set(drawn.tolist()))}", len(drawn) == len(set(drawn.tolist())) == n_total),
        ("Every frame's `π` above zero", "yes", yn((prob > 0).all()), bool((prob > 0).all())),
        ("Sum of `1/π` over the sample", f"{POPULATION}.0", f"{ht:.1f}", abs(ht - POPULATION) < 0.05),
    ]
    allowed = {"round-1": {"labels.tsv", "manifest.tsv"}, "round-2": {"labels.tsv", "manifest.tsv"}}
    for k in allowed:
        got = sorted(opened[k])
        checks.append((f"Files opened under the {k} directory", "within " + ", ".join(f"`{p}`" for p in sorted(allowed[k])), ", ".join(f"`{p}`" for p in got) or "none",
                       set(got) <= allowed[k]))

    def sidecars_opened():
        return sorted(p for p in opened["sample"] if p.lower().endswith(SIDECARS))

    checks.append(("Sidecars opened under `sample\\`", "0", str(len(sidecars_opened())), not sidecars_opened()))

    print("# Discard model: the fresh blind sample\n")
    print("By discardmodel.md's \"Pre-registration of the confirmation\". No fresh label exists; every reading below is label-free.\n")
    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Fresh folders / frames (with a vector) / frames without a vector", f"{len(folders_f)} / {m} / {missing_f}"),
            ("Picks / explicit rejects / singles", f"{int(pick.sum())} / {int(reject.sum())} / {int(single.sum())}"),
            ("Winner", f"`{WINNER}`, refit once on the {len(folders)} dump folders ({n} frames, {len(L['idx'])} labels)"),
            *[(f"Round {rd}: cap (5 x the median sampled `1/π`) / labeled frames clipped", f"{rinfo[rd]['cap']:.3f} / {rinfo[rd]['clipped']}") for rd in ROUNDS],
            ("Sample size / draw seed", f"{n_total} / {DRAW_SEED}"),
        ],
    )

    print("## The refit\n")
    print(f"`C` = {c:g} by the inner `GroupKFold` (5 folds over the 27 folders, `gate.CS`, the weighted discard AUC). The mean fill and the "
          "standardization are the 27 folders'; the coefficients are on the standardized columns, on the discard log-odds (the score is their "
          "negation, higher keeps).\n")
    gate.table("| # | Column | Fill (dump mean) | Mean after the fill | SD | Coefficient (discard) |",
               [[j + 1, name, f"{fill[j]:.4f}", f"{mu[j]:.4f}", f"{sd[j]:.4f}", f"{model.coef_[0][j]:+.4f}"] for j, name in enumerate(dfit.T_NAMES)]
               + [["-", "intercept", "", "", "", f"{model.intercept_[0]:+.4f}"]])

    print("## Strata\n")
    print("The winner's per-folder bottom `floor(q * n / 100)` frames on the fresh folders (`b05`, `b10`, `b20`), ties by the seed-0 draw; "
          f"`efonly` the frames in `ef`'s bottom 20% but not the winner's; `control` the rest, drawn per folder (floor {CONTROL_FLOOR}); "
          "the others simple random samples.\n")
    gate.table("| Stratum | Population | Allocated | Drawn | `π` |",
               [[f"`{name}`", int((band == name).sum()), a, int((in_sample & (band == name)).sum()),
                 f"{a / (band == name).sum():.6g}" + (" (overall)" if name == "control" else "")] for name, a in alloc])
    print("`control` per folder:\n")
    gate.table("| Folder | Population | Drawn | `π` |", [[f"`{f}`", s, tk, f"{tk / s:.6g}"] for b, f, s, tk in strata_rows if b == "control"])

    print("## The `efonly` overlap\n")
    print("`ef`'s per-folder bands inside each stratum (frames of the fresh folders); `ef`'s bottom 20% is the first three columns.\n")
    gate.table("| Stratum | `ef` `b05` | `ef` `b10` | `ef` `b20` | `ef` above 20% | Frames |",
               [[f"`{name}`"] + [int(((band == name) & (ef_band == g)).sum()) for g in ("b05", "b10", "b20", "above")] + [int((band == name).sum())] for name in STRATA])
    both = int((cw[20] & ce[20]).sum())
    print(f"The winner's and `ef`'s bottom 20% share {both} frames of {int(cw[20].sum())} / {int(ce[20].sum())}.\n")

    print("## Label-free cuts on the fresh folders: removed share and pick loss\n")
    print("Per folder, the bottom `floor(q * n / 100)` frames by the score, a missing score last, ties by the seed-0 draw. Removed: the share of "
          "the fresh frames inside the cut (bar 3: >= 4.5%). `PL`: the share of the fresh folders' picks inside the cut (bar 2: <= 5%). Only the "
          "winner's q5 and q10 are eligible; the rest are readings.\n")
    lrows = []
    for k in cut_scores:
        for q in CUTS:
            r = cuts[(k, q)]
            pl, rem = 100 * (r & pick).sum() / pick.sum(), 100 * r.sum() / m
            lrows.append([f"`{k}`" + ("" if k == WINNER else " (reference)"), f"q{q}", int(r.sum()), f"{rem:.2f}", int((r & pick).sum()), f"{pl:.2f}",
                          yn(pl <= dfit.df.PL_BAR), yn(rem >= dfit.df.REMOVED_BAR)])
    gate.table("| Score | Cut | Removed | Removed % | Picks removed | `PL` % | `PL` <= 5% | Removed >= 4.5% |", lrows)
    print("Per folder, `PL` % at q5 / q10 / q20:\n")
    gate.table("| Folder | Frames | Picks | Pick % | `LR/T` | `ef` | Missing `ef` % |",
               [[f"`{f}`", int((fo == fi).sum()), int((pick & (fo == fi)).sum()), f"{100 * (pick & (fo == fi)).sum() / (fo == fi).sum():.2f}"]
                + [" / ".join(f"{100 * (cuts[(k, q)] & pick & (fo == fi)).sum() / max((pick & (fo == fi)).sum(), 1):.2f}" for q in CUTS) for k in (WINNER, "ef")]
                + [f"{100 * np.isnan(af_f[fo == fi]).mean():.2f}"] for fi, f in enumerate(folders_f)])

    print("## Pick share of the strata (label-free)\n")
    gate.table("| Stratum | Frames | Picks | Pick share of the stratum | Share of all picks | Without `eye_focus` |",
               [[f"`{name}`", int((band == name).sum()), int(((band == name) & pick).sum()), f"{100 * ((band == name) & pick).sum() / (band == name).sum():.2f}%",
                 f"{100 * ((band == name) & pick).sum() / pick.sum():.2f}%", int(((band == name) & np.isnan(af_f)).sum())] for name in STRATA])

    print("## Coverage per folder\n")
    gate.table("| Folder | Frames | Sampled | " + " | ".join(f"`{k}`" for k in STRATA) + " | Picks | Rejects | Singles |",
               [[f"`{f}`", int((fo == fi).sum()), int((in_sample & (fo == fi)).sum())]
                + [int((in_sample & (fo == fi) & (band == k)).sum()) for k in STRATA]
                + [int((in_sample & (fo == fi) & pick).sum()), int((in_sample & (fo == fi) & reject).sum()), int((in_sample & (fo == fi) & single).sum())]
                for fi, f in enumerate(folders_f)])

    print("## Checks of the draw\n")
    gate.table("| Check | Expected | This run | Result |", [(ck, ex, v, "pass" if ok else "FAIL") for ck, ex, v, ok in checks])
    sys.stdout.flush()
    if not all(ok for *_, ok in checks):
        raise SystemExit("a check of the draw failed")

    sources = {int(i): os.path.join(LIBRARY, rows_f[i]["folder"], rows_f[i]["file"]) for i in drawn}
    absent = [p for p in sources.values() if not os.path.isfile(p)]
    if absent:
        raise SystemExit(f"{len(absent)} sources missing, first {absent[0]}")
    need = sum(os.path.getsize(p) for p in sources.values())
    free_bytes = shutil.disk_usage(os.path.splitdrive(os.path.abspath(out_dir))[0] + os.sep).free
    print("## Disk\n")
    gate.table("| Item | GB |", [("Sampled RAW files", f"{need / 1e9:.1f}"), ("Free on the output drive", f"{free_bytes / 1e9:.1f}")])
    sys.stdout.flush()

    def raw_col(k):
        return np.array([np.nan if f[k] is None else f[k] for f in rows_f])

    with open(os.path.join(out_dir, "scores_fresh.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write(f"folder\tfile\t{WINNER}\tef\tef-fill\n")
        for i, f in enumerate(rows_f):
            out.write(f"{f['folder']}\t{f['file']}\t{float(win[i])!r}\t{float(af_f[i])!r}\t{float(fill_f[i])!r}\n")
    sharp, yaw, pitch, roll = raw_col("sharp"), raw_col("yaw"), raw_col("pitch"), raw_col("roll")
    name_of = {}
    with open(os.path.join(out_dir, "manifest.tsv"), "w", encoding="utf-8", newline="") as out:
        head = ["name", "source", "folder", "file", "draw", "band", "inclusion_prob", "pick", "reject", "single", "burst_size", "capture_ms",
                "sharpness", "eye_focus", "ef_fill", "ear", "yaw", "pitch", "roll", WINNER, "winner_rank_share", "ef_rank_share", "ef_band"]
        out.write("\t".join(head) + "\n")
        for k, i in enumerate(order, start=1):
            f = rows_f[i]
            name = f"{k:04d}_{f['folder']}_{f['file']}"
            name_of[int(i)] = name
            vals = [name, sources[int(i)], f["folder"], f["file"], k, band[i], prob[i], pick[i], reject[i], single[i], int(size[i]), f["ms"],
                    sharp[i], af_f[i], fill_f[i], ear_f[i], yaw[i], pitch[i], roll[i], win[i], win_share[i], ef_share[i], ef_band[i]]
            out.write("\t".join(bs.cell(v) for v in vals) + "\n")
    with open(os.path.join(out_dir, "strata.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("band\tfolder\tpopulation\tdrawn\tinclusion_prob\n")
        for name, folder, size_, take in strata_rows:
            out.write(f"{name}\t{folder}\t{size_}\t{take}\t{take / size_:.6g}\n")
    if not copy:
        print(f"Total {time.time() - start:.0f} s (no copy).")
        print("freshsample: done", file=sys.stderr, flush=True)
        return

    os.makedirs(sample_dir, exist_ok=True)
    before = bs.listing(list(FRESH))
    t0 = time.time()
    copied = skipped = 0
    for k, i in enumerate(order, start=1):
        src, dst = sources[int(i)], os.path.join(sample_dir, name_of[int(i)])
        st = os.stat(src)
        if os.path.isfile(dst):
            dt = os.stat(dst)
            if dt.st_size == st.st_size and dt.st_mtime_ns == st.st_mtime_ns:
                skipped += 1
                continue
        shutil.copy2(src, dst)
        copied += 1
        if k % 50 == 0:
            print(f"copied {k} of {n_total} ({time.time() - t0:.0f} s)", file=sys.stderr, flush=True)
    copy_time = time.time() - t0

    t0 = time.time()
    mismatched = []
    for k, i in enumerate(order, start=1):
        src, dst = sources[int(i)], os.path.join(sample_dir, name_of[int(i)])
        if os.path.getsize(src) != os.path.getsize(dst) or bs.sha256(src) != bs.sha256(dst):
            mismatched.append(name_of[int(i)])
        if k % 50 == 0:
            print(f"hashed {k} of {n_total} ({time.time() - t0:.0f} s)", file=sys.stderr, flush=True)
    hash_time = time.time() - t0
    present = sorted(os.listdir(sample_dir))
    sidecars = [p for p in present if p.lower().endswith(SIDECARS)]
    extra = sorted(set(present) - set(name_of.values()))
    after = bs.listing(list(FRESH))
    changed = sorted(p for p in set(before) | set(after) if before.get(p) != after.get(p))

    print("## Checks of the copy\n")
    gate.table(
        "| Check | Expected | This run |",
        [
            ("Copies whose size or SHA-256 differs from the source", "0", len(mismatched)),
            ("`.xmp` / `.dop` files in `sample\\`", "0", len(sidecars)),
            ("Sidecars opened under `sample\\`", "0", len(sidecars_opened())),
            ("Files in `sample\\` / manifest rows", f"{n_total} / {n_total}", f"{len(present)} / {len(name_of)}"),
            ("Files in `sample\\` not in the manifest", "0", len(extra)),
            ("Library files (the six folders, recursive) before / after / changed", "equal / 0", f"{len(before)} / {len(after)} / {len(changed)}"),
        ],
    )
    for p in (mismatched + sidecars + extra + changed)[:20]:
        print(f"- `{p}`")
    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [(f"Copy ({copied} copied, {skipped} skipped)", f"{copy_time:.0f}"), ("SHA-256 of sources and copies", f"{hash_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.")
    ok = not (mismatched or sidecars or sidecars_opened() or extra or changed) and len(present) == n_total
    print("freshsample: done" if ok else "freshsample: copy check failed", file=sys.stderr, flush=True)
    if not ok:
        raise SystemExit("a check of the copy failed")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if a != "--no-copy"]
    if len(args) not in (6, 7):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*args[:6], int(args[6]) if len(args) > 6 else N_DEFAULT, "--no-copy" not in sys.argv[1:])
