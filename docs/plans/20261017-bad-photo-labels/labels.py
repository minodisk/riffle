"""Step 3: the user's bad / not-bad labels of the sample folder, read from the
copies' sidecars and joined to the manifest.

Usage: python -I -B labels.py <out-dir>

<out-dir> is Step 1's data folder: `manifest.tsv` beside `sample/`, whose
copies carry the sidecars Riffle wrote while the user labeled. A copy is `bad`
when its deciding sidecar holds a Reject (the `.dop`'s `ShouldProcess = 1`,
else the XMP's `xmpDM:good="False"`; the `.dop` decides when both exist, as
the archived `gate.load` does), `not_bad` otherwise. An optional
`<out-dir>/skip.txt` (`name<TAB>reason` per line) takes copies out of the
sample. Writes `labels.tsv` into <out-dir> and prints the counts (Markdown).

Only the manifest's design and dump-label columns are read: no score, cue or
model output, so the measurement of Step 4 stays blind.
"""

import csv
import os
import re
import sys
from collections import Counter

sys.dont_write_bytecode = True

KEPT = ["source", "folder", "file", "draw", "stratum", "inclusion_prob", "pick", "reject", "single", "burst_size", "capture_ms"]
STRATA = ["closed", "focus_low", "away", "sharp10", "g0_bottom", "w0_bottom", "rest"]
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
        flag = found[deciding][0] if deciding != "-" else None
        if "dop" in found and "xmp" in found and found["dop"][0] != found["xmp"][0]:
            mismatch.append((name, found["dop"][0] or "-", found["xmp"][0] or "-"))
        for kind, (fl, rating, labels) in sorted(found.items()):
            odd = []
            if fl == "Pick":
                odd.append("pick")
            if rating not in (None, "0", "-1"):
                odd.append(f"rating {rating}")
            if labels:
                odd.append("label " + "/".join(labels))
            if odd:
                unexpected.append((name, kind, ", ".join(odd)))
        if name in skip:
            continue
        rows.append({"name": name, "label": "bad" if flag == "Reject" else "not_bad", "source_sidecar": deciding, **{k: r[k] for k in KEPT}})
    orphans = sorted(p for p in listing if p not in names and p not in expected)

    print("# Step 3: labels\n")
    print(f"manifest rows {len(manifest)}, labeled {len(rows)}, skipped {len(skip)}, copies missing {len(missing)}, other files in sample {len(orphans)}\n")
    bad = sum(r["label"] == "bad" for r in rows)
    table(["label", "frames"], [("bad", bad), ("not_bad", len(rows) - bad), ("total", len(rows))])
    print("## Per stratum\n")
    by = Counter((r["stratum"], r["label"]) for r in rows)
    table(["stratum", "bad", "not_bad", "total"], [(s, by[(s, "bad")], by[(s, "not_bad")], by[(s, "bad")] + by[(s, "not_bad")]) for s in STRATA])
    print("## Sidecars\n")
    table(["formats", "copies"], sorted(formats.items()))
    table(["deciding sidecar", "copies"], sorted(Counter(r["source_sidecar"] for r in rows).items()))
    print(f"dop / xmp disagreements: {len(mismatch)}\n")
    if mismatch:
        table(["name", "dop", "xmp"], mismatch)
    print(f"unexpected pick / rating / label: {len(unexpected)}\n")
    if unexpected:
        table(["name", "sidecar", "found"], unexpected)
    print("## Dump-label consistency\n")
    table(
        ["dump label", "sampled", "labeled bad"],
        [(k, sum(r[k] == "1" for r in rows), sum(r[k] == "1" and r["label"] == "bad" for r in rows)) for k in ("pick", "reject")],
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
