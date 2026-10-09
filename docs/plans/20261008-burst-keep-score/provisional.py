"""Distributions and mark rates of the provisional good-photo cuts (Step 4).

Usage: python provisional.py <dump dir> [eye_focus ear max_yaw max_pitch]

Reads the `riffle-cli features` dumps (`<folder>.tsv`, `<folder>.output.txt`)
of the re-dump, keeps the faced AF frames (`af` with a cue face), and prints
the percentiles of `eye_focus`, EAR, |yaw| and |pitch| (all, picks, non-picks),
then per folder the share each cut, their AND and today's `candidate` mark,
and the files of 2026-09-19 that pass the AND. The rule mirrors `goodPhoto`
in `crates/app/ui/src/focus.ts`: a focus candidate whose `eye_focus`, EAR and
pose all exist and pass.
"""

import csv
import math
import pathlib
import random
import sys

ROOT = pathlib.Path(sys.argv[1])
CUTS = [float(v) for v in sys.argv[2:6]] if len(sys.argv) >= 6 else [0.9, 0.2, 60.0, 45.0]
EYE_FOCUS, EAR, MAX_YAW, MAX_PITCH = CUTS


def num(v):
    return None if v in ("-", "err") else float(v)


def frames():
    out = {}
    for tsv in sorted(ROOT.glob("*.tsv")):
        folder = tsv.stem
        exported = set()
        listing = ROOT / f"{folder}.output.txt"
        if listing.exists():
            exported = set(listing.read_text().split())
        rows = []
        for r in csv.DictReader(tsv.open(encoding="utf-8"), delimiter="\t"):
            if r["af"] != "af" or r["cue_side"] in ("-", "err"):
                continue
            stem = r["file"].rsplit(".", 1)[0]
            flag = r["dop"] if r["dop"] != "-" else r["xmp"]
            pick = flag == "Pick" or any(
                e == stem or e.startswith(stem + "_") for e in exported
            )
            rows.append(
                {
                    "file": r["file"],
                    "pick": pick,
                    "candidate": r["state"] == "Candidate",
                    "eye_focus": num(r["eye_focus"]),
                    "ear": num(r["ear"]),
                    "yaw": num(r["yaw"]),
                    "pitch": num(r["pitch"]),
                }
            )
        out[folder] = rows
    return out


def passes(f):
    return {
        "eye_focus": f["eye_focus"] is not None and f["eye_focus"] >= EYE_FOCUS,
        "ear": f["ear"] is not None and f["ear"] >= EAR,
        "pose": f["yaw"] is not None
        and abs(f["yaw"]) <= MAX_YAW
        and abs(f["pitch"]) <= MAX_PITCH,
    }


def good(f):
    p = passes(f)
    return f["candidate"] and p["eye_focus"] and p["ear"] and p["pose"]


def percentile(values, q):
    values = sorted(values)
    if not values:
        return float("nan")
    k = (len(values) - 1) * q / 100
    lo, hi = math.floor(k), math.ceil(k)
    return values[lo] + (values[hi] - values[lo]) * (k - lo)


def pct(n, d):
    return f"{100 * n / d:.1f}%" if d else "-"


def main():
    data = frames()
    every = [f for rows in data.values() for f in rows]
    qs = [5, 10, 25, 50, 75, 90, 95]
    print(f"cuts: eye_focus >= {EYE_FOCUS}, EAR >= {EAR}, |yaw| <= {MAX_YAW}, |pitch| <= {MAX_PITCH}")
    print(f"faced AF frames: {len(every)}")
    print("\n| Feature | Set | n | " + " | ".join(f"p{q}" for q in qs) + " |")
    print("| --- | --- | ---: | " + " | ".join("---:" for _ in qs) + " |")
    for name, key in [("eye_focus", "eye_focus"), ("EAR", "ear"), ("abs(yaw)", "yaw"), ("abs(pitch)", "pitch")]:
        for label, sel in [("all", lambda f: True), ("picks", lambda f: f["pick"]), ("non-picks", lambda f: not f["pick"])]:
            vals = [abs(f[key]) if key in ("yaw", "pitch") else f[key] for f in every if sel(f) and f[key] is not None]
            cells = " | ".join(f"{percentile(vals, q):.3f}" if key in ("eye_focus", "ear") else f"{percentile(vals, q):.1f}" for q in qs)
            print(f"| {name} | {label} | {len(vals)} | {cells} |")

    for key, cut in [("eye_focus", EYE_FOCUS), ("ear", EAR)]:
        vals = sorted(f[key] for f in every if f[key] is not None)
        below = sum(v < cut for v in vals)
        print(f"{key} cut {cut} sits at p{100 * below / len(vals):.1f} of the {len(vals)} frames with a value")
    for key, cut in [("yaw", MAX_YAW), ("pitch", MAX_PITCH)]:
        vals = [abs(f[key]) for f in every if f[key] is not None]
        within = sum(v <= cut for v in vals)
        print(f"|{key}| cut {cut} sits at p{100 * within / len(vals):.1f} of the {len(vals)} frames with a pose")

    print("\n| Folder | Faced AF | `candidate` (before) | `eye_focus` | EAR | pose | AND (after) | Picks among AND | Picks among `candidate` |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    tot = {"n": 0, "cand": 0, "ef": 0, "ear": 0, "pose": 0, "good": 0, "gp": 0, "cp": 0}
    for folder, rows in data.items():
        c = {"n": len(rows), "cand": 0, "ef": 0, "ear": 0, "pose": 0, "good": 0, "gp": 0, "cp": 0}
        for f in rows:
            p = passes(f)
            c["cand"] += f["candidate"]
            c["ef"] += p["eye_focus"]
            c["ear"] += p["ear"]
            c["pose"] += p["pose"]
            c["good"] += good(f)
            c["gp"] += good(f) and f["pick"]
            c["cp"] += f["candidate"] and f["pick"]
        for k in tot:
            tot[k] += c[k]
        for name, c in [(f"`{folder}`", c)]:
            print(
                f"| {name} | {c['n']} | {pct(c['cand'], c['n'])} | {pct(c['ef'], c['n'])} | {pct(c['ear'], c['n'])} "
                f"| {pct(c['pose'], c['n'])} | {pct(c['good'], c['n'])} ({c['good']}) | {pct(c['gp'], c['good'])} | {pct(c['cp'], c['cand'])} |"
            )
    c = tot
    print(
        f"| **Total** | {c['n']} | {pct(c['cand'], c['n'])} | {pct(c['ef'], c['n'])} | {pct(c['ear'], c['n'])} "
        f"| {pct(c['pose'], c['n'])} | {pct(c['good'], c['n'])} ({c['good']}) | {pct(c['gp'], c['good'])} | {pct(c['cp'], c['cand'])} |"
    )

    marked = [f for f in data.get("2026-09-19", []) if good(f)]
    random.Random(20261009).shuffle(marked)
    print("\n2026-09-19, files that keep the icon (a seeded draw of 12):")
    for f in sorted(marked[:12], key=lambda f: f["file"]):
        print(
            f"  {f['file']}  eye_focus {f['eye_focus']:.3f}  EAR {f['ear']:.3f}  yaw {f['yaw']:.1f}  pitch {f['pitch']:.1f}  {'pick' if f['pick'] else 'non-pick'}"
        )


main()
