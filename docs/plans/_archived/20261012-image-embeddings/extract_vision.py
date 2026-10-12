"""Cut the image tower out of a combined CLIP ONNX file (text and vision in
one graph, as onnx-community's TinyCLIP export is), keeping only the path
from the image input to the image embedding.

Usage: python -I extract_vision.py <model.onnx> <out.onnx> [input] [output]
(input defaults to pixel_values, output to image_embeds)
"""

import sys

import onnx.utils


def main(src, dst, input_name="pixel_values", output_name="image_embeds"):
    onnx.utils.extract_model(src, dst, [input_name], [output_name])


if __name__ == "__main__":
    main(*sys.argv[1:])
