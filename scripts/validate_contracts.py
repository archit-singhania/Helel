#!/usr/bin/env python3
"""Reject malformed JSON contract files as schemas are introduced."""

import json
from pathlib import Path

root = Path(__file__).resolve().parents[1] / "contracts"
for path in root.rglob("*.json"):
    with path.open(encoding="utf-8") as contract:
        json.load(contract)
print("Contract files are valid JSON.")
