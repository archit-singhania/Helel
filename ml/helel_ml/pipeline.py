"""Command-line entry point for the local Phase 6 pipeline."""

from __future__ import annotations
import argparse
from dataclasses import asdict
import json
from pathlib import Path

from .dataset import DatasetBuilder
from .tokenizer import ByteBPETokenizer


def main() -> None:
    parser = argparse.ArgumentParser(description="Build a licensed Helel dataset and tokenizer locally")
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--vocab-size", type=int, default=512)
    args = parser.parse_args()
    builder = DatasetBuilder(args.registry, args.output)
    manifest = builder.build()
    documents, _ = builder.ingest()
    tokenizer, report = ByteBPETokenizer.train((item.text for item in documents), args.vocab_size)
    tokenizer.save(args.output / "tokenizer.json")
    (args.output / "tokenizer-report.json").write_text(json.dumps(asdict(report), indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"dataset": manifest["report"], "tokenizer": asdict(report)}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
