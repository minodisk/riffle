"""The shipped genre data file: each label's prompt ensemble, its text vector
from the SigLIP 2 text tower and its calibrated threshold.

Usage: python -I genre_vectors.py <text-dir> <genre_prompts.json> <out.json>

<text-dir> holds `onnx/text_model.onnx` and `tokenizer.json` of
onnx-community/siglip2-base-patch32-256-ONNX. The prompts are tokenized as
genre_text.py does (lowercased, no BOS, EOS appended, right-padded with
<pad> to 64). A label's vector is the mean of its L2-normalized prompt
vectors, normalized again, so `crates/core/src/genre.rs` scores a frame by
one dot product per label. `threshold` (on that cosine) is copied from
genre_prompts.json, where genre_calibrate.py's chosen values are written by
hand; a label without one gets `null`, which genre.rs refuses, so only a
calibrated file ships. `model` and
`embedVersion` name the image embedding the vectors were made for
(`embed::SHIPPED.name`, `embed::EMBED_VERSION`); a unit test in genre.rs
checks them. Re-running this script is the only way the vectors change.
"""

import json
import os
import sys

import numpy as np
import onnxruntime
from tokenizers import Tokenizer

LENGTH = 64
MODEL = "siglip2"
EMBED_VERSION = 1


def tokens(tok, text):
    ids = tok.encode(text.lower(), add_special_tokens=False).ids
    ids = ids[: LENGTH - 1] + [tok.token_to_id("<eos>")]
    return ids + [tok.token_to_id("<pad>")] * (LENGTH - len(ids))


def main(text_dir, prompts_path, out_path):
    tok = Tokenizer.from_file(os.path.join(text_dir, "tokenizer.json"))
    session = onnxruntime.InferenceSession(os.path.join(text_dir, "onnx", "text_model.onnx"))
    with open(prompts_path, encoding="utf-8") as f:
        entries = json.load(f)["labels"]
    rows = []
    for entry in entries:
        ids = np.array([tokens(tok, p) for p in entry["prompts"]], dtype=np.int64)
        (vecs,) = session.run(["pooler_output"], {"input_ids": ids})
        vecs = vecs / np.linalg.norm(vecs, axis=1, keepdims=True)
        v = vecs.mean(axis=0)
        v = v / np.linalg.norm(v)
        head = {
            "label": entry["label"],
            "parent": entry["parent"],
            "threshold": entry.get("threshold"),
            "prompts": entry["prompts"],
        }
        body = json.dumps(head, ensure_ascii=False)[:-1]
        vector = ", ".join(f"{x:.7g}" for x in v.astype(np.float32))
        rows.append(f'    {body}, "vector": [{vector}]}}')
    with open(out_path, "w", encoding="utf-8", newline="\n") as out:
        out.write("{\n")
        out.write(f'  "model": "{MODEL}",\n  "embedVersion": {EMBED_VERSION},\n')
        out.write('  "labels": [\n' + ",\n".join(rows) + "\n  ]\n}\n")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    main(*sys.argv[1:4])
