"""Print the inputs, outputs and op types of ONNX files.

Usage: python -I onnx_io.py <model.onnx>...
"""

import collections
import sys

import onnx


def describe(value):
    t = value.type.tensor_type
    dims = [d.dim_param or d.dim_value for d in t.shape.dim]
    return f"{value.name} {onnx.TensorProto.DataType.Name(t.elem_type)} {dims}"


def main(paths):
    for path in paths:
        model = onnx.load(path, load_external_data=False)
        g = model.graph
        print(f"== {path}")
        print(f"opset {[(o.domain or 'ai.onnx', o.version) for o in model.opset_import]}")
        for i in g.input:
            print(f"input  {describe(i)}")
        for o in g.output:
            print(f"output {describe(o)}")
        ops = collections.Counter(f"{n.domain + '.' if n.domain else ''}{n.op_type}" for n in g.node)
        print("ops " + ", ".join(f"{k} {v}" for k, v in sorted(ops.items())))


if __name__ == "__main__":
    main(sys.argv[1:])
