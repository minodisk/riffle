"""Step 1: the stratified sample of the dump's frames the user labels bad /
not bad, as shuffled RAW copies with their manifest.

Usage: python -I -B sample.py <app-dump-dir> <folder-wide-eval-dir> <reject-filter-eval-dir> <out-dir> [n]

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load` / `build` and folder_gate.py's `extend`; <folder-wide-eval-dir> holds
`scores.tsv`, <reject-filter-eval-dir> `scores_reject.tsv`. Writes
`manifest.tsv` and `strata.tsv` into <out-dir>, copies the sampled RAW files
(no sidecar) into `<out-dir>/sample/` and prints the tables (Markdown) and the
checks on stdout. `n` (default 1200) scales the allocation.
"""

import hashlib
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
sys.path.insert(0, os.path.join(ARCHIVED, "20261016-reject-filter-eval"))
import rejectfilter as rf  # noqa: E402

gate, folder_gate = rf.gate, rf.folder_gate

LIBRARY = r"D:\Photos\2026"
N_DEFAULT = 1200
ALLOCATION = [("closed", 120), ("focus_low", 120), ("away", 120), ("sharp10", 120), ("g0_bottom", 150), ("w0_bottom", 70), ("rest", 500)]
BOTTOM_SHARE = 15
REST_FLOOR = 8
SCORES = ["W0", "W0+off", "F0", "F0+ctx", "P0"]
REJECT_SCORES = ["R0", "G0", rf.COUNT]
SIDECARS = (".xmp", ".dop")


def under(path, root):
    p = os.path.normcase(os.path.realpath(path))
    r = os.path.normcase(os.path.realpath(root))
    return p == r or p.startswith(r.rstrip(os.sep) + os.sep)


def score_bottom(score, d, tb, share):
    out = np.zeros(len(score), dtype=bool)
    for fi in range(len(d["folders"])):
        m = np.where(d["folder_of"] == fi)[0]
        v = m[~np.isnan(score[m])]
        out[v[rf.bottom(score[v], tb[v])][: share * len(v) // 100]] = True
    return out


def allocate(sizes, total, floor):
    """`total` over the cells proportional to their sizes, at least `floor` (or the whole cell) each, largest remainder."""
    fixed = {}
    while True:
        free = [k for k in sizes if k not in fixed]
        left = total - sum(fixed.values())
        pop = sum(sizes[k] for k in free)
        quota = {k: left * sizes[k] / pop for k in free}
        low = [k for k in free if quota[k] < min(floor, sizes[k])]
        if not low:
            break
        for k in low:
            fixed[k] = min(floor, sizes[k])
    out = dict(fixed)
    out.update({k: int(q) for k, q in quota.items()})
    rest = total - sum(out.values())
    for k in sorted(quota, key=lambda k: (-(quota[k] - int(quota[k])), k))[:rest]:
        out[k] += 1
    assert all(out[k] <= sizes[k] for k in sizes)
    return out


def listing(folders):
    out = {}
    for folder in folders:
        for root, _, files in os.walk(os.path.join(LIBRARY, folder)):
            for name in files:
                p = os.path.join(root, name)
                st = os.stat(p)
                out[p] = (st.st_size, st.st_mtime_ns)
    return out


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while True:
            b = f.read(1 << 22)
            if not b:
                return h.hexdigest()
            h.update(b)


def cell(v, digits=6):
    if v is None:
        return ""
    if isinstance(v, (bool, np.bool_)):
        return "1" if v else "0"
    if isinstance(v, (float, np.floating)):
        return "nan" if np.isnan(v) else f"{v:.{digits}g}"
    return str(v)


def main(dump, eval_dir, reject_dir, out_dir, n_total):
    start = time.time()
    if under(out_dir, LIBRARY):
        raise SystemExit(f"refusing to write under the library {LIBRARY}: {out_dir}")
    sample_dir = os.path.join(out_dir, "sample")
    assert not under(sample_dir, LIBRARY)

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

    flags = {
        "closed": masks[rf.CLOSED],
        "focus_low": masks[rf.FOCUS_LOW],
        "away": masks[rf.AWAY],
        "sharp10": masks[rf.SHARP10],
        "g0_bottom": score_bottom(stored_rej["G0"], d, tb, BOTTOM_SHARE),
        "w0_bottom": score_bottom(stored["W0"], d, tb, BOTTOM_SHARE),
    }
    stratum = np.full(n, "rest", dtype=object)
    free = np.ones(n, dtype=bool)
    for name, _ in ALLOCATION[:-1]:
        hit = free & flags[name]
        stratum[hit] = name
        free &= ~hit

    scale = n_total / sum(a for _, a in ALLOCATION)
    alloc = [(k, int(round(a * scale))) for k, a in ALLOCATION]
    alloc[-1] = ("rest", n_total - sum(a for _, a in alloc[:-1]))

    drawn, prob, strata_rows = [], np.zeros(n), []
    for name, a in alloc[:-1]:
        idx = np.flatnonzero(stratum == name)
        take = min(a, len(idx))
        chosen = np.sort(rng.choice(idx, size=take, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = take / len(idx)
        strata_rows.append((name, "-", len(idx), take))
    rest_sizes = {fi: int(((stratum == "rest") & (folder_of == fi)).sum()) for fi in range(len(folders))}
    rest_alloc = allocate(rest_sizes, alloc[-1][1], REST_FLOOR)
    for fi in range(len(folders)):
        idx = np.flatnonzero((stratum == "rest") & (folder_of == fi))
        take = rest_alloc[fi]
        chosen = np.sort(rng.choice(idx, size=take, replace=False))
        drawn.extend(chosen.tolist())
        prob[idx] = take / len(idx)
        strata_rows.append(("rest", folders[fi], len(idx), take))
    drawn = np.array(drawn)
    assert len(set(drawn.tolist())) == len(drawn) == n_total
    assert (prob > 0).all()
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
            ("Frames firing each flag (before the priority order): " + ", ".join(flags), " / ".join(str(int(v.sum())) for v in flags.values())),
            ("`W0` / `G0` missing (frames)", f"{int(np.isnan(stored['W0']).sum())} / {int(np.isnan(stored_rej['G0']).sum())}"),
            ("Sample size", n_total),
        ],
    )

    print("## Strata\n")
    print("Disjoint, each frame in the first stratum that matches; `rest` drawn per folder (floor 8), the others as simple random samples.\n")
    trows = []
    for name, a in alloc:
        m = stratum == name
        trows.append([f"`{name}`", int(m.sum()), a, int((m & in_sample).sum()), f"{100 * (m & in_sample).sum() / m.sum():.2f}%"])
    gate.table("| Stratum | Population | Allocated | Drawn | Sampling rate |", trows)

    print("## Coverage\n")

    def cov(label, mask):
        return [label, int(mask.sum()), int((mask & in_sample).sum()), f"{100 * mask.mean():.1f}%", f"{100 * (mask & in_sample).sum() / n_total:.1f}%"]

    gate.table(
        "| Class | Dump frames | Sampled | Dump share | Sample share |",
        [cov("Singles (burst of one)", single), cov("Burst frames", ~single), cov("Picks", pick), cov("Explicit rejects", reject), cov("`photoTier` good", tier)]
        + [cov(f"Genre `{g}`", genre == g) for g in tops + ["Unknown"]],
    )
    print("Per folder:\n")
    frows = []
    for fi, folder in enumerate(folders):
        m = folder_of == fi
        frows.append(
            [f"`{folder}`", int(m.sum()), int((m & in_sample).sum()), rest_alloc[fi]]
            + [int((m & in_sample & (stratum == k)).sum()) for k, _ in alloc[:-1]]
            + [int((m & in_sample & pick).sum()), int((m & in_sample & reject).sum()), int((m & in_sample & single).sum())]
        )
    gate.table("| Folder | Frames | Sampled | `rest` | " + " | ".join(f"`{k}`" for k, _ in alloc[:-1]) + " | Picks | Rejects | Singles |", frows)

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
        head = ["name", "source", "folder", "file", "draw", "stratum", "inclusion_prob", "pick", "reject", "single", "burst_size", "capture_ms", "genre",
                "sharpness", "eye_focus", "ear", "yaw", "pitch", "roll", "tier", "closed", "focus_low", "away", "sharp10"] + SCORES + REJECT_SCORES
        out.write("\t".join(head) + "\n")
        for k, i in enumerate(order, start=1):
            f = rows[i]
            name = f"{k:04d}_{f['folder']}_{f['file']}"
            name_of[int(i)] = name
            vals = [name, sources[int(i)], f["folder"], f["file"], k, stratum[i], prob[i], pick[i], reject[i], single[i], int(size[i]), f["ms"], genre[i],
                    sharp[i], ef[i], ear[i], yaw[i], pitch[i], roll[i], tier[i], flags["closed"][i], flags["focus_low"][i], flags["away"][i], flags["sharp10"][i]]
            vals += [stored[s][i] for s in SCORES] + [stored_rej[s][i] for s in REJECT_SCORES]
            out.write("\t".join(cell(v) for v in vals) + "\n")
    with open(os.path.join(out_dir, "strata.tsv"), "w", encoding="utf-8", newline="") as out:
        out.write("stratum\tfolder\tpopulation\tdrawn\tinclusion_prob\n")
        for name, folder, pop, take in strata_rows:
            out.write(f"{name}\t{folder}\t{pop}\t{take}\t{take / pop:.6g}\n")

    before = listing(folders)
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
    mismatched = [name_of[int(i)] for i in order if os.path.getsize(sources[int(i)]) != os.path.getsize(os.path.join(sample_dir, name_of[int(i)]))
                  or sha256(sources[int(i)]) != sha256(os.path.join(sample_dir, name_of[int(i)]))]
    hash_time = time.time() - t
    present = sorted(os.listdir(sample_dir))
    sidecars = [p for p in present if p.lower().endswith(SIDECARS)]
    extra = sorted(set(present) - set(name_of.values()))
    after = listing(folders)
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
    if len(sys.argv) not in (5, 6):
        raise SystemExit(__doc__)
    main(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]) if len(sys.argv) > 5 else N_DEFAULT)
