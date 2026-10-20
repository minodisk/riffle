"""Step 4: the fresh blind sample of the discard confirmation (the discard
winner's per-folder bottom 5 / 10 / 20% bands, `G0`'s bottom 20% outside the
winner's, and random controls), as shuffled RAW copies with their manifest.

Usage: python -I -B confirmsample.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <discard-labels-dir> <strict-labels-dir> <out-dir> [n] [--no-copy]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv`, <reject-filter-eval-dir> `scores_reject.tsv` (the stored `G0`),
<discard-labels-dir> and <strict-labels-dir> the two earlier samples'
`manifest.tsv`, whose frames leave the population (no `labels.tsv` is
opened), <out-dir> Step 2's `scores_discard.tsv` (the winner's score). Writes
`manifest.tsv` and `strata.tsv` into <out-dir>, copies the sampled RAW files
(no sidecar) into `<out-dir>/sample/` and prints the tables (Markdown) and the
checks on stdout. `n` (default 600) is split evenly over the five strata.
`--no-copy` stops after the tables and the manifest. A rerun draws the same
sample and copies only what is missing or differs, so the copy is resumable.
"""

import importlib.util
import json
import os
import shutil
import sys
import time

import numpy as np

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

_spec = importlib.util.spec_from_file_location("discard_sample", os.path.join(ARCHIVED, "20261018-discard-labels", "sample.py"))
ds = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ds)

rf, bs = bf.rf, ds.bs
gate, folder_gate = rf.gate, rf.folder_gate

WINNER = "G0 ⊕ ef"
LIBRARY = ds.LIBRARY
N_DEFAULT = 600
CUTS = ds.CUTS
STRATA = ["b05", "b10", "b20", "g0only", "control"]
CONTROL_FLOOR = ds.CONTROL_FLOOR
DRAW_SEED = 20261021
EARLIER = 1800
PL_EXPECTED = {WINNER: ("0.77", "2.10", "5.73"), "G0": ("0.68", "2.10", "5.18")}
SIDECARS = ds.SIDECARS


def main(dump, eval_dir, reject_dir, labels_dir, strict_dir, out_dir, n_total, copy):
    start = time.time()
    if bs.under(out_dir, LIBRARY):
        raise SystemExit(f"refusing to write under the library {LIBRARY}: {out_dir}")
    sample_dir = os.path.join(out_dir, "sample")
    assert not bs.under(sample_dir, LIBRARY)
    roots = {k: os.path.normcase(os.path.abspath(p)) for k, p in (("discard-labels", labels_dir), ("strict-labels", strict_dir))}
    opened = {k: set() for k in roots}

    def audit(event, args):
        if event == "open" and isinstance(args[0], str):
            p = os.path.normcase(os.path.abspath(args[0]))
            for k, r in roots.items():
                if p.startswith(r + os.sep):
                    opened[k].add(os.path.relpath(p, r))

    sys.addaudithook(audit)

    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n = len(rows)
    tb = np.random.default_rng(0).random(n)
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    sd, _ = rf.load_scores(os.path.join(out_dir, "scores_discard.tsv"), d)
    ear = rf.load_ear(dump, d)
    masks, _, _ = rf.rules(d, tb, ear)
    win, g0 = sd[WINNER], stored_rej["G0"]
    assert np.isfinite(win).all() and np.isfinite(g0).all()

    cw, cg = {}, {}
    for q in CUTS:
        for score, cut in ((win, cw), (g0, cg)):
            removed, out = bf.graded_cut(score, d, tb, q)
            assert not out.any()
            cut[q] = removed
    assert (cw[5] <= cw[10]).all() and (cw[10] <= cw[20]).all()
    band = np.full(n, "control", dtype=object)
    band[cg[20] & ~cw[20]] = "g0only"
    band[cw[20]] = "b20"
    band[cw[10]] = "b10"
    band[cw[5]] = "b05"
    g0_band = np.full(n, "above", dtype=object)
    g0_band[cg[20]] = "b20"
    g0_band[cg[10]] = "b10"
    g0_band[cg[5]] = "b05"

    def rank_share(score):
        out = np.zeros(n)
        for fi in range(len(folders)):
            m = np.where(folder_of == fi)[0]
            out[m[rf.bottom(score[m], tb[m])]] = np.arange(len(m)) / len(m)
        return out

    win_share, g0_share = rank_share(win), rank_share(g0)

    key = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    seen = np.zeros(n, dtype=bool)
    earlier = {}
    for label, path in (("strict", strict_dir), ("discard", labels_dir)):
        m = bf.read_tsv(os.path.join(path, "manifest.tsv"))
        idx = np.array([key[(r["folder"], r["file"])] for r in m])
        earlier[label] = (len(m), int(seen[idx].sum()))
        seen[idx] = True
    assert seen.sum() == EARLIER
    pop = ~seen

    per = n_total // len(STRATA)
    alloc = [(b, per) for b in STRATA[:-1]] + [("control", n_total - per * (len(STRATA) - 1))]
    rng = np.random.default_rng(DRAW_SEED)
    drawn, prob, strata_rows = [], np.zeros(n), []
    for name, a in alloc[:-1]:
        idx = np.flatnonzero(pop & (band == name))
        chosen = np.sort(rng.choice(idx, size=a, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = a / len(idx)
        strata_rows.append((name, "-", len(idx), a))
    ctrl = pop & (band == "control")
    ctrl_sizes = {fi: int((ctrl & (folder_of == fi)).sum()) for fi in range(len(folders))}
    ctrl_alloc = bs.allocate(ctrl_sizes, alloc[-1][1], CONTROL_FLOOR)
    for fi in range(len(folders)):
        idx = np.flatnonzero(ctrl & (folder_of == fi))
        if not len(idx):
            continue
        take = ctrl_alloc[fi]
        chosen = np.sort(rng.choice(idx, size=take, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = take / len(idx)
        strata_rows.append(("control", folders[fi], len(idx), take))
    drawn = np.array(drawn)
    assert len(set(drawn.tolist())) == len(drawn) == n_total
    assert (prob[pop] > 0).all() and (prob[seen] == 0).all()
    order = drawn[np.argsort(rng.random(len(drawn)), kind="stable")]
    in_sample = np.zeros(n, dtype=bool)
    in_sample[drawn] = True

    with open(folder_gate.GENRES, encoding="utf-8") as f:
        labels = json.load(f)["labels"]
    names = d["genre_labels"]
    tops = [lab["label"] for lab in labels if lab.get("parent") is None]
    top_cols = [names.index(t) for t in tops]
    soft_names = d["genre_names"]

    def genre_of(i):
        on = [t for t, c in zip(tops, top_cols) if d["genres"][i, c]]
        if not on:
            return "Unknown"
        return max(on, key=lambda t: (d["soft"][i, soft_names.index(t)], -tops.index(t)))

    genre = np.array([genre_of(i) for i in range(n)], dtype=object)
    single, size = d["single"], d["size"]
    pick, reject = d["pick"], d["reject"]

    def raw(k):
        return np.array([np.nan if f[k] is None else f[k] for f in rows])

    sharp, ef, yaw, pitch, roll = raw("sharp"), raw("ef"), raw("yaw"), raw("pitch"), raw("roll")
    tier = np.array([f["tier"] for f in rows])
    alld = np.ones(n, dtype=bool)

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / frames (with a vector) / frames without a vector (dropped)", f"{len(folders)} / {n} / {missing}"),
            ("Picks / explicit rejects / singles", f"{int(pick.sum())} / {int(reject.sum())} / {int(single.sum())}"),
            ("Earlier samples excluded (strict / discard; overlap) / population", f"{int(seen.sum())} ({earlier['strict'][0]} / {earlier['discard'][0]}; {earlier['discard'][1]}) / {int(pop.sum())}"),
            ("Winner", f"`{WINNER}` (`scores_discard.tsv`)"),
            ("Sample size / draw seed", f"{n_total} / {DRAW_SEED}"),
        ],
    )

    print("## Strata\n")
    print("The winner's per-folder bottom `floor(q * n / 100)` frames on the full dump (`b05`, `b10`, `b20`), ties by the seed-0 draw; "
          "`g0only` the frames in `G0`'s bottom 20% but not the winner's; `control` the rest, drawn per folder (floor " + str(CONTROL_FLOOR) + "); "
          "the others simple random samples.\n")
    brows = []
    for name, a in alloc:
        full = band == name
        m = pop & full
        brows.append([f"`{name}`", int(full.sum()), int((full & seen).sum()), int(m.sum()), a, int((m & in_sample).sum()),
                      f"{a / m.sum():.6g}" if name != "control" else f"{a / m.sum():.6g} (overall)"])
    gate.table("| Stratum | Dump frames | Removed by the exclusion | Population | Allocated | Drawn | `π` |", brows)
    print("`control` per folder:\n")
    gate.table("| Folder | Population | Drawn | `π` |", [[f"`{f}`", s, t, f"{t / s:.6g}"] for b, f, s, t in strata_rows if b == "control"])
    print("`G0`'s bands inside the strata (population):\n")
    grows = []
    for name in STRATA:
        m = pop & (band == name)
        grows.append([f"`{name}`"] + [int((m & (g0_band == g)).sum()) for g in ("b05", "b10", "b20", "above")])
    gate.table("| Stratum | `G0` `b05` | `G0` `b10` | `G0` `b20` | `G0` above 20% |", grows)

    print("## Pick share (label-free)\n")
    print("Dump picks inside each stratum and each cut, on the full dump and on the population.\n")
    prows = []
    for name in STRATA:
        for label, base in (("dump", alld), ("population", pop)):
            m = base & (band == name)
            prows.append([f"`{name}`", label, int(m.sum()), int((m & pick).sum()), f"{100 * (m & pick).sum() / m.sum():.2f}%",
                          f"{100 * (m & pick).sum() / (base & pick).sum():.2f}%"])
    gate.table("| Stratum | Over | Frames | Picks | Pick share of the stratum | Share of all picks |", prows)
    crows, pl = [], {}
    for score_name, cut in ((WINNER, cw), ("G0", cg)):
        for q in CUTS:
            for label, base in (("dump", alld), ("population", pop)):
                m = base & cut[q]
                loss = 100 * (m & pick).sum() / (base & pick).sum()
                if label == "dump":
                    pl[(score_name, q)] = f"{loss:.2f}"
                crows.append([f"`{score_name}` q{q}", label, int(m.sum()), f"{100 * m.sum() / base.sum():.2f}%", int((m & pick).sum()),
                              f"{100 * (m & pick).sum() / m.sum():.2f}%", f"{loss:.2f}%"])
    gate.table("| Cut | Over | Removed | Removed share | Picks removed | Pick share of the cut | Pick loss |", crows)

    print("## Coverage\n")

    def cov(label, mask):
        return [label, int(mask.sum()), int((mask & pop).sum()), int((mask & in_sample).sum()), f"{100 * mask.mean():.1f}%",
                f"{100 * (mask & pop).sum() / pop.sum():.1f}%", f"{100 * (mask & in_sample).sum() / n_total:.1f}%"]

    gate.table(
        "| Class | Dump frames | Population | Sampled | Dump share | Population share | Sample share |",
        [cov("Singles (burst of one)", single), cov("Burst frames", ~single), cov("Picks", pick), cov("Explicit rejects", reject), cov("`photoTier` good", tier)]
        + [cov(f"Genre `{g}`", genre == g) for g in tops + ["Unknown"]],
    )
    print("Per folder:\n")
    frows = []
    for fi, folder in enumerate(folders):
        m = folder_of == fi
        frows.append(
            [f"`{folder}`", int(m.sum()), int((m & pop).sum()), int((m & in_sample).sum())]
            + [int((m & in_sample & (band == k)).sum()) for k in STRATA]
            + [int((m & in_sample & pick).sum()), int((m & in_sample & reject).sum()), int((m & in_sample & single).sum())]
        )
    gate.table("| Folder | Frames | Population | Sampled | " + " | ".join(f"`{k}`" for k in STRATA) + " | Picks | Rejects | Singles |", frows)

    ht = float((1 / prob[drawn]).sum())
    checks = [
        ("Earlier samples' frames (1,200 strict + 600 discard, disjoint)", f"{EARLIER}, overlap 0", f"{int(seen.sum())}, overlap {earlier['discard'][1]}"),
        ("Population", "30659", str(int(pop.sum()))),
        ("Population frames in no stratum / strata population sum", "0 / " + str(int(pop.sum())), f"{int((pop & ~np.isin(band, STRATA)).sum())} / {sum(r[2] for r in strata_rows)}"),
        ("Sum of `1/π` over the sample", f"{int(pop.sum())}.0", f"{ht:.1f}"),
        ("Sampled frames in an earlier sample", "0", str(int((in_sample & seen).sum()))),
    ]
    for score_name, exp in PL_EXPECTED.items():
        checks.append((f"`{score_name}` `PL` % (full dump) q5 / q10 / q20", " / ".join(exp), " / ".join(pl[(score_name, q)] for q in CUTS)))
    for k in roots:
        checks.append((f"Files opened under the {k} directory", "`manifest.tsv` only", ", ".join(f"`{p}`" for p in sorted(opened[k])) or "none"))
    print("## Checks of the draw\n")
    gate.table("| Check | Expected | This run |", checks)
    ok = all(e == v for c, e, v in checks if not c.startswith("Files opened")) and all(opened[k] <= {"manifest.tsv"} for k in roots)
    if not ok:
        raise SystemExit("a check of the draw failed")

    sources = {int(i): os.path.join(LIBRARY, rows[i]["folder"], rows[i]["file"]) for i in drawn}
    absent = [p for p in sources.values() if not os.path.isfile(p)]
    if absent:
        raise SystemExit(f"{len(absent)} sources missing, first {absent[0]}")
    need = sum(os.path.getsize(p) for p in sources.values())
    free_bytes = shutil.disk_usage(os.path.splitdrive(os.path.abspath(out_dir))[0] + os.sep).free
    print("## Disk\n")
    gate.table("| Item | GB |", [("Sampled RAW files", f"{need / 1e9:.1f}"), ("Free on the output drive", f"{free_bytes / 1e9:.1f}")])
    sys.stdout.flush()

    name_of = {}
    with open(os.path.join(out_dir, "manifest.tsv"), "w", encoding="utf-8", newline="") as out:
        head = ["name", "source", "folder", "file", "draw", "band", "inclusion_prob", "pick", "reject", "single", "burst_size", "capture_ms", "genre",
                "sharpness", "eye_focus", "ear", "yaw", "pitch", "roll", "tier", "closed", "focus_low", "away", "sharp10"] + bs.SCORES + bs.REJECT_SCORES
        head += [WINNER, "winner_rank_share", "g0_rank_share", "g0_band"]
        out.write("\t".join(head) + "\n")
        for k, i in enumerate(order, start=1):
            f = rows[i]
            name = f"{k:04d}_{f['folder']}_{f['file']}"
            name_of[int(i)] = name
            vals = [name, sources[int(i)], f["folder"], f["file"], k, band[i], prob[i], pick[i], reject[i], single[i], int(size[i]), f["ms"], genre[i],
                    sharp[i], ef[i], ear[i], yaw[i], pitch[i], roll[i], tier[i], masks[rf.CLOSED][i], masks[rf.FOCUS_LOW][i], masks[rf.AWAY][i],
                    masks[rf.SHARP10][i]]
            vals += [stored[s][i] for s in bs.SCORES] + [stored_rej[s][i] for s in bs.REJECT_SCORES]
            vals += [win[i], win_share[i], g0_share[i], g0_band[i]]
            out.write("\t".join(bs.cell(v) for v in vals) + "\n")
    with open(os.path.join(out_dir, "strata.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("band\tfolder\tpopulation\tdrawn\tinclusion_prob\n")
        for name, folder, size_, take in strata_rows:
            out.write(f"{name}\t{folder}\t{size_}\t{take}\t{take / size_:.6g}\n")
    if not copy:
        print(f"Total {time.time() - start:.0f} s (no copy).")
        return

    os.makedirs(sample_dir, exist_ok=True)
    before = bs.listing(folders)
    t = time.time()
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
            print(f"copied {k} of {n_total} ({time.time() - t:.0f} s)", file=sys.stderr, flush=True)
    copy_time = time.time() - t

    t = time.time()
    mismatched = []
    for k, i in enumerate(order, start=1):
        src, dst = sources[int(i)], os.path.join(sample_dir, name_of[int(i)])
        if os.path.getsize(src) != os.path.getsize(dst) or bs.sha256(src) != bs.sha256(dst):
            mismatched.append(name_of[int(i)])
        if k % 50 == 0:
            print(f"hashed {k} of {n_total} ({time.time() - t:.0f} s)", file=sys.stderr, flush=True)
    hash_time = time.time() - t
    present = sorted(os.listdir(sample_dir))
    sidecars = [p for p in present if p.lower().endswith(SIDECARS)]
    extra = sorted(set(present) - set(name_of.values()))
    after = bs.listing(folders)
    changed = sorted(p for p in set(before) | set(after) if before.get(p) != after.get(p))

    print("## Checks of the copy\n")
    gate.table(
        "| Check | Expected | This run |",
        [
            ("Copies whose size or SHA-256 differs from the source", "0", len(mismatched)),
            ("`.xmp` / `.dop` files in `sample\\`", "0", len(sidecars)),
            ("Files in `sample\\` / manifest rows", f"{n_total} / {n_total}", f"{len(present)} / {len(name_of)}"),
            ("Files in `sample\\` not in the manifest", "0", len(extra)),
            ("Library files (the 27 folders, recursive) before / after / changed", "equal / 0", f"{len(before)} / {len(after)} / {len(changed)}"),
        ],
    )
    for p in (mismatched + sidecars + extra + changed)[:20]:
        print(f"- `{p}`")
    print("## Wall time\n")
    gate.table("| Stage | Seconds |", [("Copy (" + f"{copied} copied, {skipped} skipped)", f"{copy_time:.0f}"), ("SHA-256 of sources and copies", f"{hash_time:.0f}")])
    print(f"Total {time.time() - start:.0f} s.")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if a != "--no-copy"]
    if len(args) not in (6, 7):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*args[:6], int(args[6]) if len(args) > 6 else N_DEFAULT, "--no-copy" not in sys.argv[1:])
