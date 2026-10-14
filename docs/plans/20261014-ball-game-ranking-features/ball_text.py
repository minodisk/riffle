"""Candidate 1's text vectors: the four prompt ensembles of ball_prompts.json
through the SigLIP 2 text tower, as the archived genre_text.py makes the
genre vectors (lowercased, no BOS, EOS appended, right-padded with <pad> to
64; each prompt L2-normalized, the mean normalized again).

Usage: python -I -B ball_text.py <text-dir> <out.tsv>

<text-dir> holds `onnx/text_model.onnx` and `tokenizer.json` of
onnx-community/siglip2-base-patch32-256-ONNX. Writes `label\\t-\\tvec...` per
ensemble (genre_text.py's format).
"""

import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(
    0,
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_archived", "20261012-image-embeddings"),
)
import genre_text  # noqa: E402

PROMPTS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ball_prompts.json")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    genre_text.main(sys.argv[1], PROMPTS, sys.argv[2])
