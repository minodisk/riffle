"""The shipped ranking prior: Step 2's shared model refit on every folder of
a gate directory, with what `crates/core/src/rank.rs` needs to reproduce its
columns.

Usage: python -I rank_prior.py <gate-dir> <out.json> [scores.tsv]

<gate-dir> is read by gate.py's own `load` / `build` (for the shipped prior,
the app dump that rank_dump.py prepared, so the technical values are the
ones the app stores). As gate.py's last table does, this imputes the
technical columns with their means, fits the 32-dimension whitened PCA on
every frame, standardizes every column, chooses `C` by the grouped inner
cross-validation and fits the pairwise logistic on every scorable burst.
The JSON holds the imputation means, the PCA mean, the PCA components each
divided by the square root of its explained variance (the whitened
projection is then one dot product per dimension), the standardization, the
weights and `C`. `model`, `embedVersion` and `rankVersion` name the
embedding and the column layout it was made for (`embed::SHIPPED.name`,
`embed::EMBED_VERSION`, `rank::RANK_VERSION`); a unit test in rank.rs checks
them. With [scores.tsv], also writes each frame's score (`folder`, `file`,
the weights times its standardized columns) for the cross-check against
rank.rs. Re-running this script is the only way the prior changes.

The prior carries no per-genre offsets (rank.rs starts every genre's offset
at zero), so when the genre-ranking-offsets plan moved `RANK_VERSION` to 2
for the offsets in `ranking.json`, the shipped JSON's `rankVersion` was
edited to 2 by hand and this script was not re-run.
"""

import os
import sys

import numpy as np
from sklearn.decomposition import PCA

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gate  # noqa: E402

MODEL = "siglip2"
EMBED_VERSION = 1
RANK_VERSION = 2


def numbers(v, fmt):
    return "[" + ", ".join(fmt % x for x in v) + "]"


def main(gate_dir, out_path, scores_path=None):
    frames, missing = gate.load(gate_dir)
    d = gate.build(frames, gate.load_text(os.path.join(gate_dir, "genre_text.tsv")))
    sc = gate.scorable(d)
    tiebreak = np.random.default_rng(0).random(len(d["rows"]))

    tech = d["tech"].copy()
    tech_mean = np.nanmean(tech, axis=0)
    idx = np.where(np.isnan(tech))
    tech[idx] = np.take(tech_mean, idx[1])
    pca = PCA(n_components=gate.PCA_DIMS, whiten=True, random_state=0).fit(d["emb"])
    x = np.hstack([tech, d["pos"], pca.transform(d["emb"])])
    mu, sd = x.mean(axis=0), x.std(axis=0)
    sd[sd == 0] = 1
    x = (x - mu) / sd
    folder_of_burst = {b: d["bursts"][b][0] for b in sc}
    c = gate.choose_c(d, sc, x, folder_of_burst, tiebreak)
    w = gate.fit(d, sc, x, c)
    basis = pca.components_ / np.sqrt(pca.explained_variance_)[:, None]

    with open(out_path, "w", encoding="utf-8", newline="\n") as out:
        out.write("{\n")
        out.write(f'  "model": "{MODEL}",\n  "embedVersion": {EMBED_VERSION},\n  "rankVersion": {RANK_VERSION},\n')
        out.write(f'  "folders": {len(d["folders"])},\n  "bursts": {len(sc)},\n  "c": {c},\n')
        out.write(f'  "techMean": {numbers(tech_mean, "%.10g")},\n')
        out.write(f'  "mean": {numbers(mu, "%.10g")},\n')
        out.write(f'  "scale": {numbers(sd, "%.10g")},\n')
        out.write(f'  "weights": {numbers(w, "%.10g")},\n')
        out.write(f'  "pcaMean": {numbers(pca.mean_, "%.8g")},\n')
        out.write('  "pcaBasis": [\n' + ",\n".join("    " + numbers(r, "%.8g") for r in basis) + "\n  ]\n}\n")
    print(f"{len(d['rows'])} frames ({missing} without a vector), {len(sc)} scorable bursts, C = {c}")
    names = gate.TECH + gate.POSITION
    for i, n in enumerate(names):
        print(f"| {n} | {w[i]:+.3f} |")
    print(f"| PCA dims (L2 norm of the 32) | {np.linalg.norm(w[len(names):]):.3f} |")

    if scores_path:
        s = x @ w
        with open(scores_path, "w", encoding="utf-8", newline="\n") as out:
            out.write("folder\tfile\tscore\n")
            for i, f in enumerate(d["rows"]):
                out.write(f"{f['folder']}\t{f['file']}\t{s[i]:.9g}\n")


if __name__ == "__main__":
    if len(sys.argv) not in (3, 4):
        raise SystemExit(__doc__)
    main(*sys.argv[1:])
