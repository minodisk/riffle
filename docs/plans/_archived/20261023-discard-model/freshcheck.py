"""Step 4: the label-free checks of the fresh dump freshdump.sh wrote.

Usage: python -I -B freshcheck.py <fresh-dump-dir>

Loads the dump through the archived gate.py's `load` / `build` (not
folder_gate.py's `extend`, which needs the app's `prior_scores.tsv`) and
the per-face dump through `facechoiceread.load_faces`, and prints, per fresh
folder (Markdown): the RAW count of the library listing freshdump.sh took,
the rows of each stage, the errors and `--check` mismatches from its logs,
the frames `gate.load` keeps, the trusted-AF share, the picks and rejects
`gate.load` derives (and the picks only `Output\\` makes), and the
missing-`eye_focus` share before and after the `af-whole` fill. No label
exists for these folders.
"""

import os
import re
import sys

import numpy as np

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261017-bad-photo-labels"))
import badfilter as bf  # noqa: E402

sys.path.insert(0, os.path.join(ARCHIVED, "20261023-face-choice"))
import facechoiceread as fcr  # noqa: E402

gate = bf.gate

FOLDERS = ["2026-10-11", "2026-10-12a", "2026-10-12b", "2026-10-17", "2026-09-13-a", "2026-09-27-c"]


def pct(a, b):
    return f"{100 * a / b:.2f}" if b else "-"


def log_count(path, pattern):
    with open(path, encoding="utf-8", errors="replace") as f:
        m = re.search(pattern, f.read())
    return int(m.group(1)) if m else None


def main():
    dump = sys.argv[1]
    logs = os.path.join(dump, "logs")
    raw = {f: 0 for f in FOLDERS}
    with open(os.path.join(logs, "listing.before.tsv"), encoding="utf-8") as f:
        for line in f:
            p = line.split("\t")[0].split("/")
            if len(p) == 2 and p[1].lower().endswith(".arw"):
                raw[p[0]] += 1

    feats = {f: bf.read_tsv(os.path.join(dump, "features", f + ".tsv")) for f in FOLDERS}
    embed_rows = {f: 0 for f in FOLDERS}
    with open(os.path.join(dump, "embed.tsv"), encoding="utf-8") as fh:
        for line in fh:
            embed_rows[line.split("\t")[0].replace("\\", "/").split("/")[-2]] += 1
    face_files = {f: len({r["file"] for r in bf.read_tsv(os.path.join(dump, "faces", f + ".tsv"))}) for f in FOLDERS}

    frames, missing = gate.load(dump)
    d = gate.build(frames, gate.load_text(os.path.join(dump, "genre_text.tsv")))
    rows = d["rows"]
    n = len(rows)
    index = {(f["folder"], f["file"]): i for i, f in enumerate(rows)}
    folder = np.array([f["folder"] for f in rows])
    pick = np.array([f["pick"] for f in rows])
    reject = np.array([f["reject"] for f in rows])
    by_row = {(r["folder"], r["file"]): r for f in FOLDERS for r in feats[f]}
    sidecar_pick = np.array([
        by_row[(f["folder"], f["file"])]["dop"] == "Pick"
        or (by_row[(f["folder"], f["file"])]["dop"] == "-" and by_row[(f["folder"], f["file"])]["xmp"] == "Pick")
        for f in rows
    ])
    ef = np.array([np.nan if f["ef"] is None else f["ef"] for f in rows])
    faces_of, extra = fcr.load_faces(os.path.join(dump, "faces"), index, n)
    no_faces = sum(fr is None for fr in faces_of)
    af, af_whole = np.full(n, np.nan), np.full(n, np.nan)
    has_af = np.zeros(n, dtype=bool)
    for i, fr in enumerate(faces_of):
        if fr is None:
            continue
        _, vals, _, _ = fcr.choose(fr)
        af[i], af_whole[i] = vals["af"], vals["af-whole"]
        has_af[i] = fr["af"]
    fill = np.where(np.isnan(ef), af_whole, ef)
    # The features TSV prints `eye_focus` to 4 decimals; the per-face dump keeps full precision.
    same = (np.isnan(ef) == np.isnan(af)) & (np.isnan(ef) | (np.abs(ef - af) <= 5.1e-5))
    both = ~np.isnan(ef) & ~np.isnan(af)
    gap = float(np.max(np.abs(ef[both] - af[both]))) if both.any() else 0.0

    print("| Folder | RAW | features rows | embed rows | faces files | features errors | embed failed | facechoice errors | `--check` mismatches | `gate.load` frames | trusted AF % | picks | from `Output\\` only | pick % | rejects | missing `eye_focus` % | after the fill % | `af` = features `eye_focus` (4 decimals) |")
    print("|---|" + "---:|" * 17)
    tot = {}
    for f in FOLDERS:
        m = folder == f
        k = int(m.sum())
        cells = {
            "raw": raw[f],
            "feat": len(feats[f]),
            "embed": embed_rows[f],
            "faces": face_files[f],
            "ferr": log_count(os.path.join(logs, f + ".features.err"), r"(\d+) with an error"),
            "efail": log_count(os.path.join(logs, f + ".embed.out"), r"(\d+) failed"),
            "cerr": log_count(os.path.join(logs, f + ".faces.err"), r"(\d+) errors"),
            "mis": log_count(os.path.join(logs, f + ".faces.err"), r"check: (\d+) mismatches"),
            "n": k,
            "af": int(has_af[m].sum()),
            "pick": int(pick[m].sum()),
            "out": int((pick & ~sidecar_pick)[m].sum()),
            "rej": int(reject[m].sum()),
            "noef": int(np.isnan(ef[m]).sum()),
            "nofill": int(np.isnan(fill[m]).sum()),
            "same": int(same[m].sum()),
        }
        for key, v in cells.items():
            tot[key] = tot.get(key, 0) + (v or 0)
        print(row(f, cells))
    print(row("**all**", tot))
    print()
    print(f"- Frames dropped by `gate.load` for want of a vector: {missing}")
    print(f"- Face-dump rows outside `gate.load`'s frames: {extra}; `gate.load` frames without a face-dump row: {no_faces}")
    print(f"- `gate.build` rows: {n}, folders: {len(d['folders'])}, bursts: {len(d['bursts'])}")
    print(f"- Largest |features `eye_focus` - `af`| where both exist: {gap:.2e} (the features TSV rounds to 4 decimals)")


def row(name, c):
    return (
        f"| {name} | {c['raw']} | {c['feat']} | {c['embed']} | {c['faces']} | {c['ferr']} | {c['efail']} | {c['cerr']} | {c['mis']} | {c['n']} "
        f"| {pct(c['af'], c['n'])} | {c['pick']} | {c['out']} | {pct(c['pick'], c['n'])} | {c['rej']} "
        f"| {pct(c['noef'], c['n'])} | {pct(c['nofill'], c['n'])} | {c['same']} |"
    )


if __name__ == "__main__":
    main()
