#!/usr/bin/env python3
"""Deterministic, offline HelelBench infrastructure runner."""
from __future__ import annotations
import argparse, json, pathlib, tempfile, time

ROOT = pathlib.Path(__file__).resolve().parents[1]

def execute(task: dict, workspace: pathlib.Path) -> dict:
    started = time.monotonic()
    target = workspace / f"{task['id']}.txt"
    before = "fixture\n"
    target.write_text(before, encoding="utf-8")
    # The scripted baseline represents the tool transport, mutation, and validator.
    target.write_text(f"completed:{task['category']}\n", encoding="utf-8")
    success = target.read_text(encoding="utf-8") == f"completed:{task['category']}\n"
    target.write_text(before, encoding="utf-8")
    rollback = target.read_text(encoding="utf-8") == before
    return {"id": task["id"], "category": task["category"], "success": success,
            "validationPassed": success, "validToolCallRate": 1.0,
            "rollbackPassed": rollback, "actions": 2,
            "runtimeMs": round((time.monotonic()-started)*1000, 3), "crashed": False}

def run(catalog: pathlib.Path) -> dict:
    data = json.loads(catalog.read_text(encoding="utf-8"))
    tasks = data["tasks"]
    if len(tasks) < 30 or len({t["id"] for t in tasks}) != len(tasks):
        raise ValueError("HelelBench requires at least 30 uniquely identified tasks")
    with tempfile.TemporaryDirectory(prefix="helel-bench-") as folder:
        results = [execute(task, pathlib.Path(folder)) for task in tasks]
    passed = sum(r["success"] and r["validationPassed"] for r in results)
    return {"schemaVersion": 1, "planner": "scripted-fixture", "learnedModel": None,
            "summary": {"total": len(results), "passed": passed, "successRate": passed/len(results)},
            "results": results}

def main() -> None:
    parser=argparse.ArgumentParser(); parser.add_argument("--catalog", type=pathlib.Path, default=ROOT/"benchmarks/tasks.json"); parser.add_argument("--output", type=pathlib.Path)
    args=parser.parse_args(); report=run(args.catalog); encoded=json.dumps(report, indent=2, sort_keys=True)
    if args.output: args.output.parent.mkdir(parents=True, exist_ok=True); args.output.write_text(encoded+"\n", encoding="utf-8")
    print(encoded)

if __name__ == "__main__": main()
