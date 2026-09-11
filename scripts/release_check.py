#!/usr/bin/env python3
"""Fail closed when v0.1 release invariants are not satisfied."""

from __future__ import annotations
import json
import os
from pathlib import Path
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "ml"))

from helel_ml.dataset import DatasetBuilder
from helel_ml.model import ModelConfig
from helel_ml.product import HELEL_46M
from helel_ml.tokenizer import ByteBPETokenizer


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def main() -> None:
    budgets = json.loads((ROOT / "benchmarks/budgets.json").read_text())
    tauri = json.loads((ROOT / "apps/desktop/src-tauri/tauri.conf.json").read_text())
    require(tauri["version"] == "0.1.0" and tauri["bundle"]["active"], "v0.1 bundle metadata is incomplete")
    csp = tauri["app"]["security"]["csp"]
    require("connect-src 'none'" in csp and "object-src 'none'" in csp, "release CSP must deny network and objects")
    require(ModelConfig.load(ROOT / "ml/configs/helel-46m.json") == HELEL_46M, "Helel-46M config drifted")
    require(45_000_000 <= HELEL_46M.parameter_count() <= 47_000_000, "Helel-46M parameter budget failed")
    start = time.perf_counter()
    with tempfile.TemporaryDirectory() as directory:
        builder = DatasetBuilder(ROOT / "ml/fixtures/sources.json", Path(directory))
        builder.build()
        dataset_seconds = time.perf_counter() - start
        documents, _ = builder.ingest()
        tokenizer_start = time.perf_counter()
        tokenizer, _ = ByteBPETokenizer.train((item.text for item in documents), 300)
        tokenizer_seconds = time.perf_counter() - tokenizer_start
        require(tokenizer.decode(tokenizer.encode("café 東京 🚀")) == "café 東京 🚀", "tokenizer round trip failed")
    require(dataset_seconds <= budgets["fixture_dataset_seconds"], "fixture dataset exceeded performance budget")
    require(tokenizer_seconds <= budgets["fixture_tokenizer_seconds"], "fixture tokenizer exceeded performance budget")
    frontend = ROOT / "apps/desktop/dist"
    if frontend.exists():
        size = sum(path.stat().st_size for path in frontend.rglob("*") if path.is_file())
        require(size <= budgets["maximum_frontend_bytes"], "frontend exceeded release size budget")
    binary = ROOT / "target/debug/helel-desktop"
    if binary.exists():
        require(binary.stat().st_size <= budgets["maximum_debug_binary_bytes"], "debug binary exceeded release size budget")
    production_files = [ROOT / "Cargo.toml", ROOT / "pyproject.toml", ROOT / "apps/desktop/package.json"]
    banned = ("openai", "anthropic", "telemetry", "sentry")
    dependency_text = "\n".join(path.read_text().lower() for path in production_files)
    require(not any(name in dependency_text for name in banned), "hosted or telemetry dependency detected")
    print(f"Release checks passed: dataset {dataset_seconds:.3f}s, tokenizer {tokenizer_seconds:.3f}s, parameters {HELEL_46M.parameter_count():,}.")


if __name__ == "__main__":
    main()
