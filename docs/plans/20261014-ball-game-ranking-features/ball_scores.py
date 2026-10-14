"""Candidate 1's columns for every frame of a gate directory: the cosine of
the frame's stored SigLIP 2 embedding with the `ball visible` and the
`contested carrier` text vectors, and each minus the cosine with its paired
negative (`no ball`, `single player`), the four columns of plan.md's
criterion.

Usage: python -I -B ball_scores.py <gate-dir> <ball_text.tsv> <extras_c1.tsv>

<gate-dir> holds `embed.tsv` (L2-normalized vectors, so the cosine is the dot
product). Writes ballgate.py's extras format, `folder\\tfile\\t<column>...`,
the first column being the `ball` one bar 4 reads.
"""

import os
import sys

import numpy as np

sys.dont_write_bytecode = True
sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_archived", "20261012-image-embeddings"),
)
import gate  # noqa: E402

COLUMNS = (
    ("ball visible", "ball visible", None),
    ("contested carrier", "contested carrier", None),
    ("ball visible minus no ball", "ball visible", "no ball"),
    ("contested carrier minus single player", "contested carrier", "single player"),
)


def main(gate_dir, text_path, out_path):
    names, _, text = gate.load_text(text_path)
    vecs = gate.load_embed(os.path.join(gate_dir, "embed.tsv"))
    keys = sorted(vecs)
    cos = np.array([vecs[k] for k in keys], dtype=np.float64) @ text.T
    col = {n: cos[:, i] for i, n in enumerate(names)}
    values = np.stack([col[a] - (col[b] if b else 0) for _, a, b in COLUMNS], axis=1)
    with open(out_path, "w", encoding="utf-8", newline="") as f:
        f.write("folder\tfile\t" + "\t".join(n for n, _, _ in COLUMNS) + "\n")
        for (folder, file), v in zip(keys, values):
            f.write(f"{folder}\t{file}\t" + "\t".join(f"{x:.7g}" for x in v) + "\n")
    for (n, _, _), c in zip(COLUMNS, values.T):
        print(f"{n}: mean {c.mean():.4f}, sd {c.std():.4f}, min {c.min():.4f}, max {c.max():.4f}")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    main(*sys.argv[1:4])
