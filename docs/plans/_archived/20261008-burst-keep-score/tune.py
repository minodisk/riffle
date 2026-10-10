"""The good-photo mark at the Step 5 cuts: the re-dump's mark rates and the 60-frame sample.

Usage: python tune.py <dump dir> <sample tsv> <sample folder> [max_yaw max_eye_offset min_edge_gap]

`<dump dir>` holds the `riffle-cli features` dumps of the six folders
(`<folder>.tsv`, `<folder>.output.txt`) taken with the `eye_offset` and
`edge_gap` columns; `<sample tsv>` is the same dump of the sample folder,
whose `.xmp` sidecars carry the user's 1-5 stars (`xmp:Rating`). The rule
mirrors `photoTier` in `crates/app/ui/src/focus.ts`: a focus candidate whose
`eye_focus`, EAR, pose, eye offset and edge gap all exist, whose eye offset is
at most `max_eye_offset` and edge gap at least `min_edge_gap`, is `good` at
the good cuts, else `fair` at the fair ones; both tiers share `max_yaw`.
Prints the distributions of the two new measures, the share of the faced AF
frames per folder, the sample's stars per tier (with the Step 4c rule beside
it), the 1-2 star frames still in a tier, a grid of the yaw cut, and the
three frames the user named.
"""

import csv
import math
import pathlib
import re
import statistics
import sys

DUMP = pathlib.Path(sys.argv[1])
SAMPLE = pathlib.Path(sys.argv[2])
SAMPLE_DIR = pathlib.Path(sys.argv[3])
MAX_YAW, MAX_EYE_OFFSET, MIN_EDGE_GAP = (
    [float(v) for v in sys.argv[4:7]] if len(sys.argv) >= 7 else [30.0, 0.1, 0.02]
)
GOOD = {"eye_focus": 0.998, "ear": 0.3, "pitch": 45.0}
FAIR = {"eye_focus": 0.99, "ear": 0.25, "pitch": 45.0}
NAMED = ["2026-07-11__DSC2638.ARW", "2026-07-11__DSC2827.ARW", "2026-09-19__DSC3345.ARW"]


def num(v):
    return None if v in ("-", "err") else float(v)


def frame(r):
    return {
        "file": r["file"],
        "candidate": r["state"] == "Candidate",
        "eye_focus": num(r["eye_focus"]),
        "ear": num(r["ear"]),
        "yaw": num(r["yaw"]),
        "pitch": num(r["pitch"]),
        "eye_offset": num(r["eye_offset"]),
        "edge_gap": num(r["edge_gap"]),
    }


def faced_af(path):
    for r in csv.DictReader(path.open(encoding="utf-8"), delimiter="\t"):
        if r["af"] == "af" and r["cue_side"] not in ("-", "err"):
            yield r


def clears(f, cuts, max_yaw, exclusions):
    ok = (
        f["candidate"]
        and f["eye_focus"] is not None
        and f["eye_focus"] >= cuts["eye_focus"]
        and f["ear"] is not None
        and f["ear"] >= cuts["ear"]
        and f["yaw"] is not None
        and abs(f["yaw"]) <= max_yaw
        and abs(f["pitch"]) <= cuts["pitch"]
    )
    if not exclusions:
        return ok
    max_offset, min_gap = exclusions
    return (
        ok
        and f["eye_offset"] is not None
        and f["eye_offset"] <= max_offset
        and f["edge_gap"] is not None
        and f["edge_gap"] >= min_gap
    )


def tier(f, max_yaw=MAX_YAW, exclusions=(MAX_EYE_OFFSET, MIN_EDGE_GAP)):
    if clears(f, GOOD, max_yaw, exclusions):
        return "good"
    if clears(f, FAIR, max_yaw, exclusions):
        return "fair"
    return "none"


def step4c(f):
    return tier(f, 60.0, None)


def pct(n, d):
    return f"{100 * n / d:.1f}%" if d else "-"


def percentile(values, q):
    values = sorted(values)
    k = (len(values) - 1) * q / 100
    lo, hi = math.floor(k), math.ceil(k)
    return values[lo] + (values[hi] - values[lo]) * (k - lo)


def ranks(v):
    order = sorted(range(len(v)), key=lambda i: v[i])
    out = [0.0] * len(v)
    i = 0
    while i < len(v):
        j = i
        while j + 1 < len(v) and v[order[j + 1]] == v[order[i]]:
            j += 1
        for k in range(i, j + 1):
            out[order[k]] = (i + j) / 2 + 1
        i = j + 1
    return out


def spearman(xs, ys):
    rx, ry = ranks(xs), ranks(ys)
    mx, my = statistics.mean(rx), statistics.mean(ry)
    cov = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    return cov / math.sqrt(sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry))


def stars(name):
    xmp = SAMPLE_DIR / (pathlib.Path(name).stem + ".xmp")
    m = re.search(r'xmp:Rating="(-?\d+)"', xmp.read_text(encoding="utf-8", errors="ignore"))
    return int(m.group(1)) if m else None


def folders():
    out = {}
    for tsv in sorted(DUMP.glob("*.tsv")):
        exported = set()
        listing = DUMP / f"{tsv.stem}.output.txt"
        if listing.exists():
            exported = set(listing.read_text().split())
        rows = []
        for r in faced_af(tsv):
            f = frame(r)
            stem = r["file"].rsplit(".", 1)[0]
            flag = r["dop"] if r["dop"] != "-" else r["xmp"]
            f["pick"] = flag == "Pick" or any(e == stem or e.startswith(stem + "_") for e in exported)
            rows.append(f)
        out[tsv.stem] = rows
    return out


def rates(data):
    every = [f for rows in data.values() for f in rows]
    qs = [1, 2, 5, 10, 25, 50, 75, 90, 95, 99]
    print(f"cuts: |yaw| <= {MAX_YAW} (both tiers), eye_offset <= {MAX_EYE_OFFSET}, edge_gap >= {MIN_EDGE_GAP}")
    print(f"faced AF frames: {len(every)}")
    print("\n| Measure | n | " + " | ".join(f"p{q}" for q in qs) + " |")
    print("| --- | ---: | " + " | ".join("---:" for _ in qs) + " |")
    for key in ["eye_offset", "edge_gap"]:
        vals = [f[key] for f in every if f[key] is not None]
        print(f"| {key} | {len(vals)} | " + " | ".join(f"{percentile(vals, q):.3f}" for q in qs) + " |")
    for key, cut, above in [("eye_offset", MAX_EYE_OFFSET, True), ("edge_gap", MIN_EDGE_GAP, False)]:
        vals = [f[key] for f in every if f[key] is not None]
        out = sum(v > cut if above else v < cut for v in vals)
        tiered = [f for f in every if f[key] is not None and tier(f, MAX_YAW, (9e9, -9e9)) != "none"]
        tiered_out = sum(f[key] > cut if above else f[key] < cut for f in tiered)
        print(
            f"{key} excludes {out} of {len(vals)} frames with a mesh ({pct(out, len(vals))}), "
            f"{tiered_out} of the {len(tiered)} the cuts alone tier ({pct(tiered_out, len(tiered))})"
        )

    print("\n| Folder | Faced AF | Good | Fair | Good + fair | Step 4c good + fair | Picks among good | Picks among fair |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    tot = {"n": 0, "good": 0, "fair": 0, "old": 0, "gp": 0, "fp": 0}
    for folder, rows in [*data.items(), ("**Total**", None)]:
        if rows is None:
            c = tot
        else:
            t = [tier(f) for f in rows]
            c = {
                "n": len(rows),
                "good": t.count("good"),
                "fair": t.count("fair"),
                "old": sum(step4c(f) != "none" for f in rows),
                "gp": sum(x == "good" and f["pick"] for x, f in zip(t, rows)),
                "fp": sum(x == "fair" and f["pick"] for x, f in zip(t, rows)),
            }
            for k in tot:
                tot[k] += c[k]
            folder = f"`{folder}`"
        print(
            f"| {folder} | {c['n']} | {pct(c['good'], c['n'])} ({c['good']}) | {pct(c['fair'], c['n'])} ({c['fair']}) "
            f"| {pct(c['good'] + c['fair'], c['n'])} | {pct(c['old'], c['n'])} | {pct(c['gp'], c['good'])} | {pct(c['fp'], c['fair'])} |"
        )


def sample():
    rows = []
    for r in csv.DictReader(SAMPLE.open(encoding="utf-8"), delimiter="\t"):
        f = frame(r)
        f["stars"] = stars(r["file"])
        rows.append(f)
    print(f"\nsample: {len(rows)} frames, {sum(f['stars'] is not None for f in rows)} rated")
    print("\n| Rule | Tier | n | Mean stars | 1 | 2 | 3 | 4 | 5 |")
    print("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for label, rule in [("Step 4c", step4c), ("Step 5", tier)]:
        for t in ["good", "fair", "none"]:
            s = [f["stars"] for f in rows if rule(f) == t and f["stars"]]
            mean = f"{statistics.mean(s):.2f}" if s else "-"
            print(f"| {label} | {t} | {len(s)} | {mean} | " + " | ".join(str(s.count(k)) for k in range(1, 6)) + " |")
    order = {"good": 2, "fair": 1, "none": 0}
    for label, rule in [("Step 4c", step4c), ("Step 5", tier)]:
        rated = [f for f in rows if f["stars"]]
        rho = spearman([order[rule(f)] for f in rated], [f["stars"] for f in rated])
        print(f"Spearman tier vs stars, {label}: {rho:.2f}")
    print("\n1-2 star frames in a tier at the Step 5 cuts:")
    for f in rows:
        if f["stars"] and f["stars"] <= 2 and tier(f) != "none":
            print(f"  {f['file']} {tier(f)} {f['stars']} stars  yaw {f['yaw']}  eye_offset {f['eye_offset']}  edge_gap {f['edge_gap']}")
    print("\nframes that left a tier (Step 4c -> Step 5):")
    for f in rows:
        if step4c(f) != tier(f):
            print(f"  {f['file']} {step4c(f)} -> {tier(f)}, {f['stars']} stars  yaw {f['yaw']}  eye_offset {f['eye_offset']}  edge_gap {f['edge_gap']}")
    print("\nthe frames the user named:")
    for f in rows:
        if f["file"] in NAMED:
            print(
                f"  {f['file']} -> {tier(f)}  eye_focus {f['eye_focus']}  EAR {f['ear']}  yaw {f['yaw']}  "
                f"pitch {f['pitch']}  eye_offset {f['eye_offset']}  edge_gap {f['edge_gap']}"
            )
    print("\n| \\|yaw\\| cut | Good n | Good mean | Fair n | Fair mean | Tiered 1-2 star |")
    print("| ---: | ---: | ---: | ---: | ---: | ---: |")
    for yaw in [20, 25, 30, 35, 40, 45, 60]:
        cells = []
        for t in ["good", "fair"]:
            s = [f["stars"] for f in rows if tier(f, yaw) == t and f["stars"]]
            cells += [str(len(s)), f"{statistics.mean(s):.2f}" if s else "-"]
        low = sum(1 for f in rows if f["stars"] and f["stars"] <= 2 and tier(f, yaw) != "none")
        print(f"| {yaw} | " + " | ".join(cells) + f" | {low} |")


def yaw_grid(data):
    every = [f for rows in data.values() for f in rows]
    print("\n| \\|yaw\\| cut | Good | Fair | Good + fair |")
    print("| ---: | ---: | ---: | ---: |")
    for yaw in [20, 25, 30, 35, 40, 45, 60]:
        t = [tier(f, yaw) for f in every]
        g, fa = t.count("good"), t.count("fair")
        print(f"| {yaw} | {pct(g, len(every))} | {pct(fa, len(every))} | {pct(g + fa, len(every))} |")


data = folders()
rates(data)
yaw_grid(data)
sample()
