"""Step 3: the user's keep / discard labels of the sample folder, read from the
copies' sidecars and joined to the manifest.

Usage: python -I -B labels.py <out-dir>

<out-dir> is Step 1's data folder: `manifest.tsv` beside `sample/`, whose
copies carry the sidecars Riffle wrote while the user labeled. A copy is `keep`
when its deciding sidecar holds rating 1 (the `.dop`'s `Rating`, else the
XMP's `xmp:Rating`; the `.dop` decides when both exist), `discard` otherwise
(reviewed and unmarked). A Pick, a Reject, a color label or a rating other
than 1 is listed as unexpected, not dropped; the label follows the rating-1
mark alone. An optional `<out-dir>/skip.txt` (`name<TAB>reason` per line)
takes copies out of the sample. Writes `labels.tsv` into <out-dir> and prints
the counts (Markdown).

Only the manifest's design and dump-label columns are read: no score, cue or
model output, so the measurement of Step 4 stays blind.
"""

import csv
import os
import re
import sys
from collections import Counter

sys.dont_write_bytecode = True

KEPT = ["source", "folder", "file", "draw", "band", "inclusion_prob", "pick", "reject", "single", "burst_size", "capture_ms", "strict_label"]
BANDS = ["b05", "b10", "b20", "control"]
XMP_GOOD = re.compile(r'xmpDM:good\s*=\s*"([^"]*)"')
XMP_RATING = re.compile(r'xmp:Rating\s*=\s*"([^"]*)"')
XMP_LABEL = re.compile(r'(?:photoshop:LabelColor|xmp:Label)\s*=\s*"([^"]*)"')
DOP_SHOULD = re.compile(r"^\s*ShouldProcess\s*=\s*(-?\d+)\s*,", re.M)
DOP_RATING = re.compile(r"^\s*Rating\s*=\s*(-?\d+)\s*,", re.M)
DOP_LABEL = re.compile(r'^\s*ColorLabel\s*=\s*"([^"]*)"', re.M)


def read(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read()


def xmp_of(text):
    good = XMP_GOOD.search(text)
    rating = XMP_RATING.search(text)
    rating = rating.group(1) if rating else None
    flag = {"True": "Pick", "False": "Reject"}.get(good.group(1)) if good else None
    if flag is None and rating == "-1":
        flag = "Reject"
    return flag, rating, [m for m in XMP_LABEL.findall(text) if m]


def dop_of(text):
    should = DOP_SHOULD.search(text)
    rating = DOP_RATING.search(text)
    flag = {"0": "Pick", "1": "Reject"}.get(should.group(1)) if should else None
    return flag, rating.group(1) if rating else None, [m for m in DOP_LABEL.findall(text) if m]


def table(head, rows):
    print("| " + " | ".join(head) + " |")
    print("|" + "---|" * len(head))
    for r in rows:
        print("| " + " | ".join(str(c) for c in r) + " |")
    print()


def main(out_dir):
    sample = os.path.join(out_dir, "sample")
    with open(os.path.join(out_dir, "manifest.tsv"), encoding="utf-8", newline="") as f:
        manifest = [{k: r[k] for k in ["name"] + KEPT} for r in csv.DictReader(f, delimiter="\t")]
    skip = {}
    skip_path = os.path.join(out_dir, "skip.txt")
    if os.path.exists(skip_path):
        for line in read(skip_path).splitlines():
            if line.strip():
                name, _, reason = line.partition("\t")
                skip[name.strip()] = reason.strip()
    names = {r["name"] for r in manifest}
    unknown = sorted(n for n in skip if n not in names)
    if unknown:
        sys.exit("skip.txt names not in the manifest: " + ", ".join(unknown))
    listing = set(os.listdir(sample))
    expected = set()
    rows, unexpected, formats, mismatch, missing = [], [], Counter(), [], []
    for r in manifest:
        name = r["name"]
        stem = os.path.splitext(name)[0]
        if name not in listing:
            missing.append(name)
        xmps = [p for p in (name + ".xmp", stem + ".xmp") if p in listing]
        dop = name + ".dop" if name + ".dop" in listing else None
        expected.update(xmps + ([dop] if dop else []))
        found = {}
        for p in xmps:
            found.setdefault("xmp", xmp_of(read(os.path.join(sample, p))))
        if dop:
            found["dop"] = dop_of(read(os.path.join(sample, dop)))
        formats[" + ".join(sorted(found)) or "none"] += 1
        deciding = "dop" if "dop" in found else "xmp" if "xmp" in found else "-"
        rating = found[deciding][1] if deciding != "-" else None
        if "dop" in found and "xmp" in found and (found["dop"][1] == "1") != (found["xmp"][1] == "1"):
            mismatch.append((name, found["dop"][1] or "-", found["xmp"][1] or "-"))
        for kind, (fl, rt, labels) in sorted(found.items()):
            odd = []
            if fl:
                odd.append(fl.lower())
            if rt not in (None, "1"):
                odd.append(f"rating {rt}")
            if labels:
                odd.append("label " + "/".join(labels))
            if odd:
                unexpected.append((name, kind, ", ".join(odd)))
        if name in skip:
            continue
        rows.append({"name": name, "label": "keep" if rating == "1" else "discard", "source_sidecar": deciding, **{k: r[k] for k in KEPT}})
    orphans = sorted(p for p in listing if p not in names and p not in expected)

    print("# Step 3: labels\n")
    print(f"manifest rows {len(manifest)}, labeled {len(rows)}, skipped {len(skip)}, copies missing {len(missing)}, other files in sample {len(orphans)}\n")
    keep = sum(r["label"] == "keep" for r in rows)
    table(["label", "frames"], [("keep", keep), ("discard", len(rows) - keep), ("total", len(rows))])
    print("## Per band (counts only)\n")
    by = Counter((r["band"], r["label"]) for r in rows)
    table(["band", "keep", "discard", "total"], [(b, by[(b, "keep")], by[(b, "discard")], by[(b, "keep")] + by[(b, "discard")]) for b in BANDS])
    print("## Sidecars\n")
    table(["formats", "copies"], sorted(formats.items()))
    table(["deciding sidecar", "copies"], sorted(Counter(r["source_sidecar"] for r in rows).items()))
    print(f"dop / xmp disagreements on rating 1: {len(mismatch)}\n")
    if mismatch:
        table(["name", "dop rating", "xmp rating"], mismatch)
    print(f"unexpected pick / reject / label / rating other than 1: {len(unexpected)}\n")
    if unexpected:
        table(["name", "sidecar", "found"], unexpected)
    print("## Dump-label consistency\n")
    table(
        ["dump label", "sampled", "labeled keep", "labeled discard"],
        [(k, sum(r[k] == "1" for r in rows), sum(r[k] == "1" and r["label"] == "keep" for r in rows), sum(r[k] == "1" and r["label"] == "discard" for r in rows)) for k in ("pick", "reject")],
    )
    for name in missing:
        print(f"missing copy: {name}")
    for name in orphans:
        print(f"other file in sample: {name}")
    for name, reason in sorted(skip.items()):
        print(f"skipped: {name} ({reason})")

    with open(os.path.join(out_dir, "labels.tsv"), "w", encoding="utf-8", newline="") as out:
        w = csv.DictWriter(out, ["name", "label", "source_sidecar"] + KEPT, delimiter="\t", lineterminator="\n")
        w.writeheader()
        w.writerows(rows)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
