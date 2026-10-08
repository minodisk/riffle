"""The data.md inventory table from the `riffle-cli features` dumps.

Usage: python -I inventory.py <dump-dir>

Reads `<folder>.tsv` and `<folder>.output.txt` per folder. A frame is one
base stem (the file stem with any `-DxO_...` suffix of a DxO DeepPRIME DNG
removed); when a folder holds both the camera file and its DeepPRIME DNG,
the camera file's row stands for the frame. An `Output/` stem names a frame
by its base stem, or by `<stem>_<n>` for the export of a PhotoLab virtual
copy. Picks: in a sidecar folder, the `.dop` flag (its first item, the
`dop` column) Pick or a virtual copy of the frame in `Output/` (a virtual
copy's pick is on a later item, which the `dop` column does not read), else
XMP Pick where the frame has no `.dop`; in a folder without sidecars, a
frame in `Output/`.
"""

import csv
import os
import re
import sys

ORDER = {"arw": 0, "dng-sidecar": 1, "dng-output": 2}


def base(stem):
    return re.sub(r"-DxO_.*$", "", stem)


def frames_of(rows):
    by = {}
    for r in rows:
        stem = os.path.splitext(r["file"])[0]
        b = base(stem)
        if b not in by or stem == b:
            by[b] = r
    return by


def num(v):
    return None if v in ("-", "err") else float(v)


def main(dump):
    out = []
    for name in sorted(os.listdir(dump)):
        if not name.endswith(".tsv"):
            continue
        folder = name[:-4]
        with open(os.path.join(dump, name), newline="", encoding="utf-8") as f:
            rows = list(csv.DictReader(f, delimiter="\t"))
        listing = os.path.join(dump, folder + ".output.txt")
        frames = frames_of(rows)
        outputs, copies = None, set()
        if os.path.exists(listing):
            outputs = set()
            with open(listing, encoding="utf-8") as f:
                for l in f:
                    s = base(l.strip())
                    m = re.match(r"^(.*)_\d+$", s)
                    if s and s not in frames and m and m.group(1) in frames:
                        s = m.group(1)
                        copies.add(s)
                    if s:
                        outputs.add(s)
        dng = all(r["file"].lower().endswith(".dng") for r in rows)
        sidecars = any(r["dop"] != "-" or r["xmp"] != "-" for r in rows)
        kind = "arw" if not dng else ("dng-sidecar" if sidecars else "dng-output")
        c = dict.fromkeys(
            "frames af cue judged pose dop xmp output copies label rejects disagree errors".split(), 0
        )
        for b, r in frames.items():
            c["frames"] += 1
            c["errors"] += "err" in r.values()
            c["af"] += r["af"] == "af"
            c["cue"] += num(r["cue_side"]) is not None
            side = num(r["judged_side"])
            c["judged"] += side is not None and side >= 60
            c["pose"] += num(r["yaw"]) is not None
            c["dop"] += r["dop"] == "Pick"
            c["xmp"] += r["xmp"] == "Pick"
            in_output = outputs is not None and b in outputs
            c["output"] += in_output
            c["copies"] += b in copies
            if kind == "dng-output":
                pick = in_output
            else:
                pick = (
                    r["dop"] == "Pick"
                    or b in copies
                    or (r["dop"] == "-" and r["xmp"] == "Pick")
                )
            c["label"] += pick
            flag = r["dop"] if r["dop"] != "-" else r["xmp"]
            c["rejects"] += flag == "Reject"
            if r["dop"] not in ("-", "err") and r["xmp"] not in ("-", "err"):
                c["disagree"] += r["dop"] != r["xmp"]
        out.append((ORDER[kind], folder, kind, c))
    cols = "frames af cue judged pose dop xmp output copies label rejects disagree errors".split()
    print("| Folder | Kind | Frames | AF | Cue face | Judged face | Pose | `.dop` picks | XMP picks | `Output/` | Virtual-copy picks | Picks (rule) | Rejects | XMP / `.dop` disagree | Errors |")
    print("| " + " | ".join(["---"] * 2 + ["---:"] * len(cols)) + " |")
    totals = {}
    for _, folder, kind, c in sorted(out):
        print(f"| `{folder}` | {kind} | " + " | ".join(str(c[k]) for k in cols) + " |")
        t = totals.setdefault(kind, dict.fromkeys(cols, 0))
        for k in cols:
            t[k] += c[k]
    for kind in sorted(totals, key=ORDER.get):
        print(f"| **Total** | {kind} | " + " | ".join(str(totals[kind][k]) for k in cols) + " |")


if __name__ == "__main__":
    main(sys.argv[1])
