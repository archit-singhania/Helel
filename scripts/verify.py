#!/usr/bin/env python3
"""Run every Phase 0 quality gate from one cross-platform entry point."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
DESKTOP = ROOT / "apps" / "desktop"
JS_BIN = DESKTOP / "node_modules" / ".bin"


def run(label: str, command: list[str], env: dict[str, str] | None = None) -> None:
    print(f"\n==> {label}", flush=True)
    result = subprocess.run(command, cwd=ROOT, env=env, check=False)
    if result.returncode:
        raise SystemExit(f"{label} failed with exit code {result.returncode}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-tauri-build", action="store_true")
    args = parser.parse_args()
    python_env = os.environ.copy()
    python_env["PYTHONPATH"] = str(ROOT / "ml")

    checks = [
        ("Rust format", ["cargo", "fmt", "--all", "--", "--check"], None),
        ("Rust lint", ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], None),
        ("Rust tests", ["cargo", "test", "--workspace"], None),
        ("TypeScript lint", [str(JS_BIN / "eslint"), "--config", str(DESKTOP / "eslint.config.js"), str(DESKTOP), "--max-warnings", "0"], None),
        ("TypeScript typecheck", [str(JS_BIN / "tsc"), "-b", str(DESKTOP), "--pretty", "false"], None),
        ("Frontend tests", [str(JS_BIN / "vitest"), "run", "--root", str(DESKTOP)], None),
        ("Frontend build", [str(JS_BIN / "vite"), "build", str(DESKTOP)], None),
        ("Python import and tests", [sys.executable, "-m", "unittest", "discover", "-s", "ml/tests"], python_env),
        ("Contract validation", [sys.executable, "scripts/validate_contracts.py"], None),
    ]
    if not args.skip_tauri_build:
        checks.append(("Tauri development build", [str(JS_BIN / "tauri"), "build", "--debug", "--no-bundle"], None))
    for label, command, env in checks:
        run(label, command, env)
    print("\nHelel verification passed.")


if __name__ == "__main__":
    main()
