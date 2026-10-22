"""Step 1: draw the pick lockbox folders, label-free.

Usage: python -I -B lockbox.py <app-dump-dir> <out-dir>

<app-dump-dir> is the in-burst gate's app dump, read by the archived gate.py's
`load`. Reads only each folder's frame and pick counts: no label and no model
score. Draws `K` of the 27 folders (sorted by name) with
`numpy.random.default_rng(0).choice(27, K, replace=False)`, repeated on the
same generator until a draw holds 20-30% of the frames and at least
`MIN_QUALIFYING` qualifying folders (>= 50 frames, >= 5 picks); the first such
draw is the lockbox. Writes `lockbox.tsv` (`folder`, `frames`, `picks`,
`qualifying`, `lockbox`) into <out-dir>, prints the tables (Markdown) and saves
them as `lockbox.out.md` there.
"""

import os
import sys

import numpy as np

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.dirname(HERE)
ARCHIVED = PLANS if os.path.basename(PLANS) == "_archived" else os.path.join(PLANS, "_archived")
sys.path.insert(0, os.path.join(ARCHIVED, "20261012-image-embeddings"))
import gate  # noqa: E402

K = 7
SHARE_LO, SHARE_HI = 20.0, 30.0
MIN_QUALIFYING = 5
MIN_FRAMES, MIN_PICKS = 50, 5
MAX_DRAWS = 10000


def main(dump, out_dir):
    frames, missing = gate.load(dump)
    folders = sorted({f["folder"] for f in frames})
    sizes = np.array([sum(f["folder"] == name for f in frames) for name in folders])
    picks = np.array([sum(f["folder"] == name and f["pick"] for f in frames) for name in folders])
    qual = (sizes >= MIN_FRAMES) & (picks >= MIN_PICKS)
    total = sizes.sum()
    rng = np.random.default_rng(0)
    for draw in range(1, MAX_DRAWS + 1):
        idx = np.sort(rng.choice(len(folders), K, replace=False))
        share = 100 * sizes[idx].sum() / total
        if SHARE_LO <= share <= SHARE_HI and qual[idx].sum() >= MIN_QUALIFYING:
            break
    else:
        raise SystemExit(f"no draw in {MAX_DRAWS} met the constraints")
    box = np.zeros(len(folders), dtype=bool)
    box[idx] = True

    out = []
    out.append("## Data\n")
    out.append("| Count | Value |\n| --- | ---: |")
    out.append(f"| Folders / qualifying (>= {MIN_FRAMES} frames, >= {MIN_PICKS} picks) | {len(folders)} / {qual.sum()} |")
    out.append(f"| Frames / picks | {total} / {picks.sum()} |")
    out.append(f"| Frames without a vector (dropped by `gate.load`) | {missing} |")
    out.append("")
    out.append("## The draw\n")
    out.append(
        f"`numpy.random.default_rng(0).choice({len(folders)}, {K}, replace=False)` over the folders sorted by name, "
        f"repeated until {SHARE_LO:.0f}-{SHARE_HI:.0f}% of the frames and >= {MIN_QUALIFYING} qualifying folders; accepted at draw {draw}.\n"
    )
    out.append("| Set | Folders | Qualifying | Frames | Share of frames | Picks | Share of picks |")
    out.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for label, m in (("Lockbox", box), ("Exploration", ~box)):
        out.append(
            f"| {label} | {m.sum()} | {qual[m].sum()} | {sizes[m].sum()} | {100 * sizes[m].sum() / total:.1f}% "
            f"| {picks[m].sum()} | {100 * picks[m].sum() / picks.sum():.1f}% |"
        )
    out.append("")
    out.append("## Folders\n")
    out.append("| Folder | Frames | Picks | Qualifying | Lockbox |\n| --- | ---: | ---: | ---: | ---: |")
    for i, name in enumerate(folders):
        out.append(f"| `{name}` | {sizes[i]} | {picks[i]} | {'yes' if qual[i] else 'no'} | {'yes' if box[i] else 'no'} |")
    text = "\n".join(out) + "\n"
    print(text)
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "lockbox.out.md"), "w", encoding="utf-8") as f:
        f.write(text)
    with open(os.path.join(out_dir, "lockbox.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("folder\tframes\tpicks\tqualifying\tlockbox\n")
        for i, name in enumerate(folders):
            f.write(f"{name}\t{sizes[i]}\t{picks[i]}\t{int(qual[i])}\t{int(box[i])}\n")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main(*sys.argv[1:3])
