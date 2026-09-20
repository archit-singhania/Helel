"""Evaluate a local checkpoint against Helel's exact agent proposal contract."""

from __future__ import annotations
import argparse
from hashlib import sha256
import json
from pathlib import Path

from .agent_model import build_agent_prompt, parse_proposal
from .evaluation import generate
from .model import ModelConfig, create_model
from .tokenizer import ByteBPETokenizer


CASES = (
    ("locate", "Locate the calculateInvoice implementation before changing the billing flow.", [], [], "searchCode"),
    ("read", "Read internal/auth/token.go before fixing the token expiry bug.", [{"path": "internal/auth/token.go", "line": 47, "preview": "func expired(token Token) bool"}], [], "readFile"),
    ("git", "Before touching Sources/Store/Checkout.swift, inspect all current user changes.", [], [], "inspectGit"),
    ("verify", "The Python repair is applied. Verify it with the repository test command now.", [], ["updated tests/test_checkout.py"], "runCommand"),
    ("finish", "The requested Kotlin change is done and every check succeeded. Conclude the task.", [], ["Gradle tests passed", "working tree contains only the approved patch"], "complete"),
)


def score_outputs(outputs: list[str], expected_tools: list[str]) -> dict[str, object]:
    results = []
    for output, expected in zip(outputs, expected_tools, strict=True):
        try:
            proposal = parse_proposal(output)
            valid, tool = True, proposal.tool
        except (ValueError, json.JSONDecodeError):
            valid, tool = False, None
        results.append({"expected_tool": expected, "predicted_tool": tool, "valid": valid, "correct": tool == expected, "output": output[:2_000]})
    total = max(1, len(results))
    return {"valid_proposal_rate": sum(item["valid"] for item in results) / total, "tool_accuracy": sum(item["correct"] for item in results) / total, "cases": results}


def main() -> None:
    parser = argparse.ArgumentParser(description="Evaluate a local Helel agent checkpoint")
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--maximum-new-tokens", type=int, default=160)
    args = parser.parse_args()
    config = ModelConfig.load(args.checkpoint / "config.json")
    tokenizer = ByteBPETokenizer.load(args.checkpoint / "tokenizer.json")
    model = create_model(config)
    weights = args.checkpoint / "model.safetensors"
    model.load_weights(str(weights))
    outputs, expected = [], []
    for _, objective, context, observations, tool in CASES:
        prefix = '{"rationale":"'
        outputs.append(prefix + generate(model, tokenizer, build_agent_prompt(objective, context, observations) + prefix, args.maximum_new_tokens))
        expected.append(tool)
    report = {"version": 1, "checkpoint_sha256": sha256(weights.read_bytes()).hexdigest(), "parameters": config.parameter_count(), **score_outputs(outputs, expected)}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in report.items() if key != "cases"}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
