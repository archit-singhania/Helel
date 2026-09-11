"""Full-size Helel-46M forward and quantization validation."""

from __future__ import annotations
import argparse
import json
from pathlib import Path

from .model import create_model
from .product import HELEL_46M, memory_profile


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    import mlx.core as mx
    import mlx.nn as nn

    model = create_model(HELEL_46M)
    tokens = mx.array([[1, 10, 20, 30, 2]])
    logits = model(tokens)
    mx.eval(logits)
    if logits.shape != (1, 5, HELEL_46M.vocabulary_size) or not bool(mx.all(mx.isfinite(logits)).item()):
        raise RuntimeError("full precision Helel-46M forward validation failed")
    nn.quantize(model, bits=4, group_size=64, class_predicate=lambda _path, module: isinstance(module, nn.Linear))
    quantized_logits = model(tokens)
    mx.eval(quantized_logits)
    if quantized_logits.shape != logits.shape or not bool(mx.all(mx.isfinite(quantized_logits)).item()):
        raise RuntimeError("quantized Helel-46M forward validation failed")
    report = {"parameters": HELEL_46M.parameter_count(), "logits_shape": list(logits.shape), "full_precision_finite": True, "int4_finite": True, "quantization_bits": 4, "group_size": 64, "estimated_memory": memory_profile()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
