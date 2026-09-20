"""Command-line entry point for the local Phase 6 pipeline."""

from __future__ import annotations
import argparse
from dataclasses import asdict
import json
from pathlib import Path

from .dataset import DatasetBuilder
from .tokenizer import ByteBPETokenizer
from .training import language_balanced_texts


def main() -> None:
    parser = argparse.ArgumentParser(description="Build a licensed Helel dataset and tokenizer locally")
    parser.add_argument("--registry", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--vocab-size", type=int, default=512)
    parser.add_argument("--language-temperature", type=float, default=0.5)
    args = parser.parse_args()
    builder = DatasetBuilder(args.registry, args.output)
    manifest = builder.build()
    documents, _ = builder.ingest()
    texts, language_counts = language_balanced_texts([asdict(item) for item in documents], temperature=args.language_temperature)
    tokenizer, report = ByteBPETokenizer.train(texts, args.vocab_size)
    tokenizer.save(args.output / "tokenizer.json")
    tokenizer_report = {**asdict(report), "language_temperature": args.language_temperature, "sampled_languages": language_counts}
    (args.output / "tokenizer-report.json").write_text(json.dumps(tokenizer_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"dataset": manifest["report"], "tokenizer": tokenizer_report}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
