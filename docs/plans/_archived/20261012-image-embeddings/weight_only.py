"""Re-export an FP32 ONNX file with smaller weights that a runtime widens back
to FP32 once at load: compute stays FP32, only the file shrinks.

- `int8`: the weight of every MatMul / Gemm / Conv (an initializer of at
  least 1024 values) becomes int8, symmetric, one scale per output channel,
  followed by a `DequantizeLinear`.
- `fp16`: the same weights become float16, followed by a `Cast` to float.

Usage: python -I weight_only.py <in.onnx> <out.onnx> <int8|fp16>
"""

import sys

import numpy as np
import onnx
from onnx import helper, numpy_helper


def output_axis(node, index):
    """The output-channel axis of the weight at input `index` of `node`."""
    if node.op_type == "Conv" and index == 1:
        return 0
    if node.op_type == "MatMul" and index == 1:
        return 1
    if node.op_type == "Gemm" and index == 1:
        trans_b = next((a.i for a in node.attribute if a.name == "transB"), 0)
        return 0 if trans_b else 1
    return None


def main(src, dst, mode):
    model = onnx.load(src)
    g = model.graph
    inits = {i.name: i for i in g.initializer}
    users = {}
    for node in g.node:
        for index, name in enumerate(node.input):
            users.setdefault(name, []).append((node, index))
    new_nodes = []
    done = 0
    for name, init in list(inits.items()):
        uses = users.get(name, [])
        axes = {output_axis(n, i) for n, i in uses}
        if not uses or None in axes or len(axes) != 1:
            continue
        w = numpy_helper.to_array(init)
        if w.dtype != np.float32 or w.size < 1024:
            continue
        (axis,) = axes
        g.initializer.remove(init)
        if mode == "int8":
            other = tuple(a for a in range(w.ndim) if a != axis)
            scale = np.abs(w).max(axis=other) / 127.0
            scale[scale == 0] = 1.0
            shape = [1] * w.ndim
            shape[axis] = -1
            q = np.clip(np.round(w / scale.reshape(shape)), -127, 127).astype(np.int8)
            g.initializer.extend(
                [
                    numpy_helper.from_array(q, name + "_q"),
                    numpy_helper.from_array(scale.astype(np.float32), name + "_scale"),
                    numpy_helper.from_array(np.zeros_like(scale, dtype=np.int8), name + "_zero"),
                ]
            )
            new_nodes.append(
                helper.make_node(
                    "DequantizeLinear",
                    [name + "_q", name + "_scale", name + "_zero"],
                    [name],
                    axis=axis,
                )
            )
        else:
            g.initializer.append(numpy_helper.from_array(w.astype(np.float16), name + "_h"))
            new_nodes.append(helper.make_node("Cast", [name + "_h"], [name], to=onnx.TensorProto.FLOAT))
        done += 1
    nodes = new_nodes + list(g.node)
    del g.node[:]
    g.node.extend(nodes)
    if mode == "int8" and all(o.version < 13 for o in model.opset_import if o.domain in ("", "ai.onnx")):
        raise SystemExit("per-axis DequantizeLinear needs opset 13")
    onnx.save(model, dst)
    print(f"{done} weights converted to {mode}")


if __name__ == "__main__":
    main(*sys.argv[1:])
