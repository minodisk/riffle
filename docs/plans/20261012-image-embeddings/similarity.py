"""Step 4's threshold table: precision / recall of "same scene" at candidate
SigLIP 2 cosine thresholds, from the user's labels (similarity.md).

Usage: python -I similarity.py <dir>

<dir> holds pairs.py's `pairs.tsv` (id, folder, a, b, cosine, bin) and the
labeling page's `labels.tsv` (id, label). Prints the per-bin counts, then
for each threshold t the pairs with cosine >= t predicted "same": their
count, precision and recall, over the stratified pairs (the `burst`
anchors excluded) and over all pairs. Unsure labels are dropped.
"""

import csv
import os
import sys

THRESHOLDS = [0.80, 0.85, 0.90, 0.92, 0.93, 0.94, 0.95, 0.96, 0.965, 0.97, 0.975]


def read(path):
    with open(path, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def table(rows, title):
    same = sum(1 for r in rows if r["same"])
    print(f"\n{title}: {len(rows)} pairs, {same} same")
    print("| t | predicted same | of them same | precision | recall |")
    print("|---|---|---|---|---|")
    for t in THRESHOLDS:
        hit = [r for r in rows if r["cosine"] >= t]
        tp = sum(1 for r in hit if r["same"])
        prec = f"{tp / len(hit):.3f}" if hit else "-"
        rec = f"{tp / same:.3f}" if same else "-"
        print(f"| {t:.3f} | {len(hit)} | {tp} | {prec} | {rec} |")


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    d = sys.argv[1]
    labels = {r["id"]: r["label"] for r in read(os.path.join(d, "labels.tsv"))}
    rows = []
    for r in read(os.path.join(d, "pairs.tsv")):
        label = labels.get(r["id"])
        if label not in ("same", "different"):
            continue
        rows.append({"bin": r["bin"], "cosine": float(r["cosine"]), "same": label == "same"})

    bins = {}
    for r in rows:
        b = bins.setdefault(r["bin"], [0, 0])
        b[0 if r["same"] else 1] += 1
    print("| bin | same | different |")
    print("|---|---|---|")
    for b in sorted(bins, key=lambda k: (k == "burst", k)):
        print(f"| {b} | {bins[b][0]} | {bins[b][1]} |")

    top = sorted(rows, key=lambda r: -r["cosine"])[:20]
    print("\nTop 20 by cosine (S = same, d = different):")
    print(" ".join(f"{r['cosine']:.4f}{'S' if r['same'] else 'd'}{'*' if r['bin'] == 'burst' else ''}" for r in top))

    table([r for r in rows if r["bin"] != "burst"], "Stratified (anchors excluded)")
    table(rows, "All (anchors included)")


if __name__ == "__main__":
    main()
