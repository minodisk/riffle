"""How far the app's stored inputs are from Step 2's dump, for ranking.md.

Usage: python -I rank_compare.py <gate-dir> <app-dir>

<gate-dir> is Step 2's (`gate_dump.sh`), <app-dir> the app dump after
rank_dump.py. Prints, per technical column, how many frames agree at the
precision `riffle-cli features` prints (4 decimals, 1 for the pose), differ,
have the value in one dump only, or in neither; then the cosine between the
two dumps' embeddings of each file.
"""

import csv
import os
import sys

import numpy as np

DIGITS = {
    "sharpness": 4, "eye_focus": 4, "ear": 4, "eyes_open": 4, "yaw": 1, "pitch": 1,
    "roll": 1, "eye_offset": 4, "edge_gap": 4,
}


def rows(path):
    with open(path, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def vectors(path):
    out = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            p = parts[0].replace("\\", "/").split("/")
            out[(p[-2].lower(), p[-1])] = np.array(parts[1:], dtype=np.float64)
    return out


def main(gate, app):
    stats = {c: [0, 0, 0, 0] for c in [*DIGITS, "state"]}
    frames = 0
    for name in sorted(os.listdir(os.path.join(app, "features"))):
        if not name.endswith(".tsv"):
            continue
        step2 = {r["file"]: r for r in rows(os.path.join(gate, "features", name))}
        for r in rows(os.path.join(app, "features", name)):
            q = step2[r["file"]]
            frames += 1
            stats["state"][0 if r["state"] == q["state"] else 1] += 1
            for c, d in DIGITS.items():
                a, b = r[c], q[c]
                gone_a, gone_b = a in ("-", "err"), b in ("-", "err")
                if gone_a and gone_b:
                    stats[c][3] += 1
                elif gone_a != gone_b:
                    stats[c][2] += 1
                elif abs(float(a) - float(b)) <= 10**-d:
                    stats[c][0] += 1
                else:
                    stats[c][1] += 1
    print(f"{frames} frames\n")
    print("| Column | Equal | Differ | In one dump only | In neither |")
    print("| --- | ---: | ---: | ---: | ---: |")
    for c, s in stats.items():
        print(f"| {c} | " + " | ".join(str(x) for x in s) + " |")
    a, b = vectors(os.path.join(gate, "embed.tsv")), vectors(os.path.join(app, "embed.tsv"))
    keys = sorted(set(a) & set(b))
    cos = np.array([a[k] @ b[k] / np.linalg.norm(a[k]) / np.linalg.norm(b[k]) for k in keys])
    print(f"\nEmbeddings: {len(keys)} files in both, cosine min {cos.min():.9f}, mean {cos.mean():.9f}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
