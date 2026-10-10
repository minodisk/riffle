"""The good-photo mark re-tuned on the user's stars of 180 frames (Step 7).

Usage: python retune.py <sample tsv> <meshfit tsv> <sample folder> <batch 1 manifest> <dump dir>...

`<sample tsv>` is the `riffle-cli features` dump of the sample folder, whose
`.xmp` sidecars carry the user's 1-5 stars (`xmp:Rating`; 1 = no subject,
2 = likely rejected, 3+ = a pass); `<meshfit tsv>` the `riffle-cli meshfit`
dump of the same folder, read for `yunet_eye_dist` only (its `before_`
columns are the scan's values); `<batch 1 manifest>` names the 60 frames of
batch 1 (the Step 5 sample), the other 120 being batch 2; each `<dump dir>`
holds `features` dumps of whole folders (`<folder>.tsv`). The rule mirrors
`photoTier` in `crates/app/ui/src/focus.ts`. Prints, per variant, on batch 1,
batch 2 and pooled: the frames marked, the share of 3+ stars among them, the
1- and 2-star frames among them, the share of the frames marked and the share
of the 3+ star frames marked; then the share of the faced AF frames marked
per folder for the variants that need no YuNet eye distance.
"""

import csv
import pathlib
import re
import sys

SAMPLE, MESHFIT, SAMPLE_DIR, BATCH1 = (pathlib.Path(a) for a in sys.argv[1:5])
DUMPS = [pathlib.Path(a) for a in sys.argv[5:]]
CURRENT = {"eye_focus": 0.99, "ear": 0.25, "yaw": 30, "pitch": 45, "offset": 0.10, "gap": 0.02, "gate": None}
CHOSEN = {**CURRENT, "eye_focus": 0.90, "yaw": 35}


def num(v):
    return None if v in ("-", "err", "") else float(v)


def frame(r):
    return {
        "candidate": r["state"] == "Candidate",
        "eye_focus": num(r["eye_focus"]),
        "ear": num(r["ear"]),
        "yaw": num(r["yaw"]),
        "pitch": num(r["pitch"]),
        "offset": num(r["eye_offset"]),
        "gap": num(r["edge_gap"]),
    }


def marks(f, c):
    if not f["candidate"] or None in (f["eye_focus"], f["ear"], f["yaw"], f["offset"], f["gap"]):
        return False
    if f["eye_focus"] < c["eye_focus"] or f["ear"] < c["ear"]:
        return False
    if abs(f["yaw"]) > c["yaw"] or abs(f["pitch"]) > c["pitch"] or f["gap"] < c["gap"]:
        return False
    if c["offset"] is not None and f["offset"] > c["offset"]:
        return c["gate"] is not None and f["dist"] < c["gate"]
    return True


def variants():
    out = [("current (Step 5b)", CURRENT)]
    out += [(f"\\|yaw\\| <= {y}", {**CURRENT, "yaw": y}) for y in (35, 40, 45)]
    out += [(f"eye_focus >= {e}", {**CURRENT, "eye_focus": e}) for e in (0.98, 0.97, 0.95, 0.90)]
    out += [(f"eye_offset <= {o}", {**CURRENT, "offset": o}) for o in (0.15, 0.20)]
    out += [("no eye_offset cut", {**CURRENT, "offset": None})]
    out += [(f"eye_offset only at eye dist >= {g}", {**CURRENT, "gate": g}) for g in (0.15, 0.20)]
    out += [("no edge_gap cut", {**CURRENT, "gap": -9e9})]
    for y in (35, 40):
        for e in (0.95, 0.90, 0.772):
            out.append((f"\\|yaw\\| <= {y}, eye_focus >= {e}", {**CURRENT, "yaw": y, "eye_focus": e}))
    out += [("\\|yaw\\| <= 35, eye_focus >= 0.90, EAR >= 0.22", {**CHOSEN, "ear": 0.22})]
    out += [("\\|yaw\\| <= 35, eye_focus >= 0.90, eye_offset <= 0.15", {**CHOSEN, "offset": 0.15})]
    out += [("\\|yaw\\| <= 35, eye_focus >= 0.90, gate 0.20", {**CHOSEN, "gate": 0.20})]
    return out


def stars(name):
    xmp = SAMPLE_DIR / (pathlib.Path(name).stem + ".xmp")
    return int(re.search(r'xmp:Rating="(-?\d+)"', xmp.read_text(encoding="utf-8", errors="ignore")).group(1))


def sample():
    batch1 = {r["name"] for r in csv.DictReader(BATCH1.open(encoding="utf-8"), delimiter="\t")}
    dist = {r["file"]: num(r["yunet_eye_dist"]) for r in csv.DictReader(MESHFIT.open(encoding="utf-8"), delimiter="\t")}
    rows = []
    for r in csv.DictReader(SAMPLE.open(encoding="utf-8"), delimiter="\t"):
        f = frame(r)
        f.update(file=r["file"], batch=1 if r["file"] in batch1 else 2, stars=stars(r["file"]), dist=dist[r["file"]])
        rows.append(f)
    return rows


def cell(rows, c):
    m = [f for f in rows if marks(f, c)]
    passed = sum(f["stars"] >= 3 for f in m)
    every = sum(f["stars"] >= 3 for f in rows)
    precision = f"{100 * passed / len(m):.1f}%" if m else "-"
    low = f"{sum(f['stars'] == 1 for f in m)} / {sum(f['stars'] == 2 for f in m)}"
    return f"{len(m)} | {precision} | {low} | {100 * len(m) / len(rows):.1f}% | {100 * passed / every:.1f}%"


def faced_af(path):
    for r in csv.DictReader(path.open(encoding="utf-8"), delimiter="\t"):
        if r["af"] == "af" and r["cue_side"] not in ("-", "err"):
            yield frame(r)


rows = sample()
sets = [("batch 1", [f for f in rows if f["batch"] == 1]), ("batch 2", [f for f in rows if f["batch"] == 2]), ("pooled", rows)]
for label, s in sets:
    n = len(s)
    by = [sum(f["stars"] == k for f in s) for k in range(1, 6)]
    print(f"{label}: {n} frames, stars 1-5: {by}")
head = " | ".join(f"{h} {k}" for h, _ in sets for k in ("n", "3+", "1 / 2", "marked", "3+ captured"))
print(f"\n| Variant | {head} |")
print("| --- |" + " ---: |" * 15)
for name, c in variants():
    print(f"| {name} | " + " | ".join(cell(s, c) for _, s in sets) + " |")

print("\n1-2 star frames the chosen cuts mark:")
for f in rows:
    if f["stars"] <= 2 and marks(f, CHOSEN):
        print(f"  {f['file']} {f['stars']} stars  eye_focus {f['eye_focus']}  yaw {f['yaw']}  EAR {f['ear']}")
print("\nframes whose mark changes, current -> chosen:")
for f in rows:
    if marks(f, CURRENT) != marks(f, CHOSEN):
        print(f"  {f['file']} -> {marks(f, CHOSEN)}, {f['stars']} stars  eye_focus {f['eye_focus']}  yaw {f['yaw']}")

folder_variants = [(n, c) for n, c in variants() if c["gate"] is None]
folders = {p.stem: list(faced_af(p)) for d in DUMPS for p in sorted(d.glob("*.tsv"))}
print("\n| Variant | " + " | ".join(f"`{k}`" for k in folders) + " | Total |")
print("| --- |" + " ---: |" * (len(folders) + 1))
print("| Faced AF | " + " | ".join(str(len(v)) for v in folders.values()) + f" | {sum(len(v) for v in folders.values())} |")
for name, c in folder_variants:
    shares = [100 * sum(marks(f, c) for f in v) / len(v) for v in folders.values()]
    total = sum(sum(marks(f, c) for f in v) for v in folders.values()) / sum(len(v) for v in folders.values())
    print(f"| {name} | " + " | ".join(f"{s:.1f}%" for s in shares) + f" | {100 * total:.1f}% |")
