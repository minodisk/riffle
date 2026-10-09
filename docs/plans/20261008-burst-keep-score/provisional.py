"""Distributions and mark rates of the provisional good-photo cuts (Step 4).

Usage: python provisional.py <dump dir> [eye_focus ear max_yaw max_pitch
                                          [fair_eye_focus fair_ear fair_max_yaw fair_max_pitch]]

Reads the `riffle-cli features` dumps (`<folder>.tsv`, `<folder>.output.txt`)
of the re-dump, keeps the faced AF frames (`af` with a cue face), and prints
the percentiles of `eye_focus`, EAR, |yaw| and |pitch| (all, picks, non-picks),
then per folder the share each cut, their AND and today's `candidate` mark,
and the files of 2026-09-19 that pass the AND. The rule mirrors the `"good"` tier of
`photoTier` in `crates/app/ui/src/focus.ts`: a focus candidate whose `eye_focus`,
EAR and pose all exist and pass. The optional second set of cuts is the fair tier
(Step 4c), `photoTier` in the same file: a frame that is not good but passes
the looser cuts; its share and the good + fair share are printed per folder.
"""

import csv
import math
import pathlib
import random
import sys

ROOT = pathlib.Path(sys.argv[1])
CUTS = [float(v) for v in sys.argv[2:6]] if len(sys.argv) >= 6 else [0.9, 0.2, 60.0, 45.0]
EYE_FOCUS, EAR, MAX_YAW, MAX_PITCH = CUTS
FAIR = [float(v) for v in sys.argv[6:10]] if len(sys.argv) >= 10 else None


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


def passes(f, cuts=None):
    eye_focus, ear, max_yaw, max_pitch = cuts or CUTS
    return {
        "eye_focus": f["eye_focus"] is not None and f["eye_focus"] >= eye_focus,
        "ear": f["ear"] is not None and f["ear"] >= ear,
        "pose": f["yaw"] is not None
        and abs(f["yaw"]) <= max_yaw
        and abs(f["pitch"]) <= max_pitch,
    }


def good(f, cuts=None):
    p = passes(f, cuts)
    return f["candidate"] and p["eye_focus"] and p["ear"] and p["pose"]


def fair(f):
    return FAIR is not None and not good(f) and good(f, FAIR)


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
    if FAIR is not None:
        print("fair cuts: eye_focus >= {}, EAR >= {}, |yaw| <= {}, |pitch| <= {}".format(*FAIR))
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


def tiers():
    data = frames()
    print("\n| Folder | Faced AF | Good | Fair | Good + fair | Picks among good | Picks among fair |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    tot = {"n": 0, "good": 0, "fair": 0, "gp": 0, "fp": 0}
    for folder, rows in [*data.items(), ("**Total**", None)]:
        if rows is None:
            c = tot
        else:
            c = {
                "n": len(rows),
                "good": sum(good(f) for f in rows),
                "fair": sum(fair(f) for f in rows),
                "gp": sum(good(f) and f["pick"] for f in rows),
                "fp": sum(fair(f) and f["pick"] for f in rows),
            }
            for k in tot:
                tot[k] += c[k]
            folder = f"`{folder}`"
        print(
            f"| {folder} | {c['n']} | {pct(c['good'], c['n'])} ({c['good']}) | {pct(c['fair'], c['n'])} ({c['fair']}) "
            f"| {pct(c['good'] + c['fair'], c['n'])} | {pct(c['gp'], c['good'])} | {pct(c['fp'], c['fair'])} |"
        )


main()
if FAIR is not None:
    tiers()
