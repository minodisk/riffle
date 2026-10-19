"""Step 1: the blind review sample of `G0`'s discard candidates (the per-folder
bottom 5 / 10 / 20% bands) and random controls, as shuffled RAW copies with
their manifest.

Usage: python -I -B sample.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <strict-labels-dir> <out-dir> [n]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv`, <reject-filter-eval-dir> `scores_reject.tsv`,
<strict-labels-dir> the bad-photo-labels sample's `manifest.tsv`, whose frames
leave the population. Writes `manifest.tsv` and `strata.tsv` into <out-dir>,
copies the sampled RAW files (no sidecar) into `<out-dir>/sample/` and prints
the tables (Markdown) and the checks on stdout. `n` (default 600) is split
evenly over the three bands and the controls.
"""

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

rf, bs = bf.rf, bf.sample
gate, folder_gate = rf.gate, rf.folder_gate

LIBRARY = bs.LIBRARY
N_DEFAULT = 600
CUTS = (5, 10, 20)
BANDS = ["b05", "b10", "b20", "control"]
CONTROL_FLOOR = 3
SCORES = bs.SCORES
REJECT_SCORES = bs.REJECT_SCORES
SIDECARS = bs.SIDECARS


def main(dump, eval_dir, reject_dir, strict_dir, out_dir, n_total):
    start = time.time()
    if bs.under(out_dir, LIBRARY):
        raise SystemExit(f"refusing to write under the library {LIBRARY}: {out_dir}")
    sample_dir = os.path.join(out_dir, "sample")
    assert not bs.under(sample_dir, LIBRARY)

    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    folder_gate.extend(d, dump)
    rows, folders, folder_of = d["rows"], d["folders"], d["folder_of"]
    n = len(rows)
    rng = np.random.default_rng(0)
    tb = rng.random(n)
    stored, _ = rf.load_scores(os.path.join(eval_dir, "scores.tsv"), d)
    stored_rej, _ = rf.load_scores(os.path.join(reject_dir, "scores_reject.tsv"), d)
    ear = rf.load_ear(dump, d)
    masks, _, _ = rf.rules(d, tb, ear)
    g0 = stored_rej["G0"]
    assert not np.isnan(g0).any()

    cut = {}
    for q in CUTS:
        removed, out = bf.graded_cut(g0, d, tb, q)
        assert not out.any()
        cut[q] = removed
    band = np.full(n, "control", dtype=object)
    band[cut[20]] = "b20"
    band[cut[10]] = "b10"
    band[cut[5]] = "b05"
    assert (cut[5] <= cut[10]).all() and (cut[10] <= cut[20]).all()
    share = np.zeros(n)
    for fi in range(len(folders)):
        m = np.where(folder_of == fi)[0]
        share[m[rf.bottom(g0[m], tb[m])]] = np.arange(len(m)) / len(m)

    key = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    strict = bf.read_tsv(os.path.join(strict_dir, "manifest.tsv"))
    seen = np.zeros(n, dtype=bool)
    for r in strict:
        seen[key[(r["folder"], r["file"])]] = True
    assert seen.sum() == len(strict)
    pop = ~seen

    per = n_total // len(BANDS)
    alloc = [(b, per) for b in BANDS[:-1]] + [("control", n_total - per * (len(BANDS) - 1))]

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
        take = ctrl_alloc[fi]
        chosen = np.sort(rng.choice(idx, size=take, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = take / len(idx)
        strata_rows.append(("control", folders[fi], len(idx), take))
    drawn = np.array(drawn)
    assert len(set(drawn.tolist())) == len(drawn) == n_total
    assert (prob[pop] > 0).all() and (prob[seen] == 0).all()
    order = drawn[np.argsort(rng.random(len(drawn)), kind="stable")]

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
    in_sample = np.zeros(n, dtype=bool)
    in_sample[drawn] = True

    def raw(key):
        return np.array([np.nan if f[key] is None else f[key] for f in rows])

    sharp, ef, yaw, pitch, roll = raw("sharp"), raw("ef"), raw("yaw"), raw("pitch"), raw("roll")
    tier = np.array([f["tier"] for f in rows])

    print("## Data\n")
    gate.table(
        "| Count | Value |",
        [
            ("Folders / frames (with a vector) / frames without a vector (dropped)", f"{len(folders)} / {n} / {missing}"),
            ("Picks / explicit rejects / singles", f"{int(pick.sum())} / {int(reject.sum())} / {int(single.sum())}"),
            ("Strict-sample frames excluded / population", f"{int(seen.sum())} / {int(pop.sum())}"),
            ("`W0` / `G0` missing (frames)", f"{int(np.isnan(stored['W0']).sum())} / {int(np.isnan(g0).sum())}"),
            ("Sample size", n_total),
        ],
    )

    print("## Bands\n")
    print("`G0`'s per-folder bottom `floor(q * n / 100)` frames on the full dump, ties by the seed-0 draw; "
          "the bands are simple random samples, `control` drawn per folder (floor " + str(CONTROL_FLOOR) + ").\n")
    brows = []
    for name, a in alloc:
        full = band == name
        m = pop & full
        brows.append([f"`{name}`", int(full.sum()), int((full & seen).sum()), int(m.sum()), a, int((m & in_sample).sum()),
                      f"{a / m.sum():.6g}" if name != "control" else f"{a / m.sum():.6g} (overall)"])
    gate.table("| Band | Dump frames | Removed by the exclusion | Population | Allocated | Drawn | `π` |", brows)

    print("## Pick share (label-free)\n")
    print("Dump picks inside each band and each cut, on the full dump and on the population.\n")
    prows = []
    for name in BANDS:
        for label, base in (("dump", np.ones(n, dtype=bool)), ("population", pop)):
            m = base & (band == name)
            prows.append([f"`{name}`", label, int(m.sum()), int((m & pick).sum()), f"{100 * (m & pick).sum() / m.sum():.2f}%",
                          f"{100 * (m & pick).sum() / (base & pick).sum():.2f}%"])
    gate.table("| Band | Over | Frames | Picks | Pick share of the band | Share of all picks |", prows)
    crows = []
    for q in CUTS:
        for label, base in (("dump", np.ones(n, dtype=bool)), ("population", pop)):
            m = base & cut[q]
            crows.append([f"`G0` q{q}", label, int(m.sum()), f"{100 * m.sum() / base.sum():.2f}%", int((m & pick).sum()),
                          f"{100 * (m & pick).sum() / m.sum():.2f}%", f"{100 * (m & pick).sum() / (base & pick).sum():.2f}%"])
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
            + [int((m & in_sample & (band == k)).sum()) for k in BANDS]
            + [int((m & in_sample & pick).sum()), int((m & in_sample & reject).sum()), int((m & in_sample & single).sum())]
        )
    gate.table("| Folder | Frames | Population | Sampled | " + " | ".join(f"`{k}`" for k in BANDS) + " | Picks | Rejects | Singles |", frows)

    sources = {int(i): os.path.join(LIBRARY, rows[i]["folder"], rows[i]["file"]) for i in drawn}
    absent = [p for p in sources.values() if not os.path.isfile(p)]
    if absent:
        raise SystemExit(f"{len(absent)} sources missing, first {absent[0]}")
    need = sum(os.path.getsize(p) for p in sources.values())
    free_bytes = shutil.disk_usage(os.path.splitdrive(os.path.abspath(out_dir))[0] + os.sep).free
    print("## Disk\n")
    gate.table("| Item | GB |", [("Sampled RAW files", f"{need / 1e9:.1f}"), ("Free on the output drive", f"{free_bytes / 1e9:.1f}")])
    sys.stdout.flush()

    os.makedirs(sample_dir, exist_ok=True)
    name_of = {}
    with open(os.path.join(out_dir, "manifest.tsv"), "w", encoding="utf-8", newline="") as out:
        head = ["name", "source", "folder", "file", "draw", "band", "inclusion_prob", "pick", "reject", "single", "burst_size", "capture_ms", "genre",
                "sharpness", "eye_focus", "ear", "yaw", "pitch", "roll", "tier", "closed", "focus_low", "away", "sharp10"] + SCORES + REJECT_SCORES
        head += ["g0_rank_share", "strict_label"]
        out.write("\t".join(head) + "\n")
        for k, i in enumerate(order, start=1):
            f = rows[i]
            name = f"{k:04d}_{f['folder']}_{f['file']}"
            name_of[int(i)] = name
            vals = [name, sources[int(i)], f["folder"], f["file"], k, band[i], prob[i], pick[i], reject[i], single[i], int(size[i]), f["ms"], genre[i],
                    sharp[i], ef[i], ear[i], yaw[i], pitch[i], roll[i], tier[i], masks[rf.CLOSED][i], masks[rf.FOCUS_LOW][i], masks[rf.AWAY][i],
                    masks[rf.SHARP10][i]]
            vals += [stored[s][i] for s in SCORES] + [stored_rej[s][i] for s in REJECT_SCORES]
            vals += [share[i], "-"]
            out.write("\t".join(bs.cell(v) for v in vals) + "\n")
    with open(os.path.join(out_dir, "strata.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("band\tfolder\tpopulation\tdrawn\tinclusion_prob\n")
        for name, folder, size_, take in strata_rows:
            out.write(f"{name}\t{folder}\t{size_}\t{take}\t{take / size_:.6g}\n")

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

    print("## Checks\n")
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
    if len(sys.argv) not in (6, 7):
        raise SystemExit(__doc__)
    sys.stdout.reconfigure(encoding="utf-8")
    main(*sys.argv[1:6], int(sys.argv[6]) if len(sys.argv) > 6 else N_DEFAULT)
