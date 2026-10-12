"""Turn the app's ranking dump into a gate directory `gate.py` reads.

Usage: python -I rank_dump.py <gate-dir> <app-dir>

<app-dir> holds `app/<folder>.tsv` and `embed.tsv`, written by the ignored
test `rank_dump_of_real_folders` (`crates/app/src/commands.rs`) from what
`Index::rank_rows` reads: the second pass's stored values of the judged face
and the stored embedding. <gate-dir> is Step 2's (`gate_dump.sh`). Writes
<app-dir>/features/<folder>.tsv in the columns of `riffle-cli features`,
the analysis columns from the app and the sidecar flags (`xmp`, `dop`) from
Step 2's TSV of the same file, copies the `Output/` listings and
`genre_text.tsv`, and prints how the app's own flag compares with the gate's
labels.
"""

import csv
import os
import shutil
import sys

COLUMNS = [
    "folder", "file", "capture_time", "subsec", "xmp", "dop", "sharpness", "state",
    "eye_focus", "ear", "eyes_open", "yaw", "pitch", "roll", "eye_offset", "edge_gap",
]


def rows(path):
    with open(path, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def main(gate, app):
    os.makedirs(os.path.join(app, "features"), exist_ok=True)
    shutil.copy(os.path.join(gate, "genre_text.tsv"), os.path.join(app, "genre_text.tsv"))
    agree = {}
    for name in sorted(os.listdir(os.path.join(app, "app"))):
        folder = name[:-4]
        flags = {r["file"]: r for r in rows(os.path.join(gate, "features", name))}
        listing = os.path.join(gate, "features", folder + ".output.txt")
        if os.path.exists(listing):
            shutil.copy(listing, os.path.join(app, "features", folder + ".output.txt"))
        with open(os.path.join(app, "features", name), "w", newline="", encoding="utf-8") as out:
            out.write("\t".join(COLUMNS) + "\n")
            for r in rows(os.path.join(app, "app", name)):
                g = flags.get(r["file"])
                if g is None:
                    continue
                deciding = g["dop"] if g["dop"] != "-" else g["xmp"]
                key = (r["flag"], deciding)
                agree[key] = agree.get(key, 0) + 1
                r = {**r, "folder": folder, "xmp": g["xmp"], "dop": g["dop"]}
                out.write("\t".join(r[c] for c in COLUMNS) + "\n")
    print("| App flag | Sidecar flag the gate reads | Frames |")
    print("| --- | --- | ---: |")
    for (a, g), n in sorted(agree.items()):
        print(f"| {a} | {g} | {n} |")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
