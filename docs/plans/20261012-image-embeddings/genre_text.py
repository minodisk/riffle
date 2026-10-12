"""Text vectors of the provisional genre prompts with the SigLIP 2 text tower.

Usage: python -I genre_text.py <text-dir> <genre_prompts.json> <out.tsv>

<text-dir> holds `onnx/text_model.onnx` and `tokenizer.json` of
onnx-community/siglip2-base-patch32-256-ONNX. Each prompt is lowercased,
tokenized without BOS, given an EOS and right-padded with <pad> to 64 tokens
(the SiglipProcessor defaults: `do_lower_case`, `add_eos_token`,
`max_length` 64). Writes one line per label, `label\tparent\tvec...`, the
vector being the mean of the label's L2-normalized prompt vectors, then
normalized again.
"""

import json
import os
import sys

import numpy as np
import onnxruntime
from tokenizers import Tokenizer

LENGTH = 64


def tokens(tok, text):
    ids = tok.encode(text.lower(), add_special_tokens=False).ids
    ids = ids[: LENGTH - 1] + [tok.token_to_id("<eos>")]
    return ids + [tok.token_to_id("<pad>")] * (LENGTH - len(ids))


def main(text_dir, prompts_path, out_path):
    tok = Tokenizer.from_file(os.path.join(text_dir, "tokenizer.json"))
    session = onnxruntime.InferenceSession(os.path.join(text_dir, "onnx", "text_model.onnx"))
    with open(prompts_path, encoding="utf-8") as f:
        labels = json.load(f)["labels"]
    with open(out_path, "w", encoding="utf-8", newline="") as out:
        for entry in labels:
            ids = np.array([tokens(tok, p) for p in entry["prompts"]], dtype=np.int64)
            (vecs,) = session.run(["pooler_output"], {"input_ids": ids})
            vecs = vecs / np.linalg.norm(vecs, axis=1, keepdims=True)
            v = vecs.mean(axis=0)
            v = v / np.linalg.norm(v)
            parent = entry["parent"] or "-"
            out.write(entry["label"] + "\t" + parent + "\t" + "\t".join(f"{x:.8g}" for x in v) + "\n")


if __name__ == "__main__":
    main(*sys.argv[1:4])
