"""Local loss, generation, completion, repair, and FIM evaluations."""

from __future__ import annotations
from dataclasses import dataclass
import math
from typing import Iterable

from .tokenizer import ByteBPETokenizer


@dataclass(frozen=True)
class EvaluationCase:
    case_id: str
    kind: str
    prompt: str
    expected: str

    def validate(self) -> None:
        if self.kind not in {"completion", "repair", "fim", "memorization"}:
            raise ValueError(f"unsupported evaluation kind: {self.kind}")
        if not self.case_id or not self.expected:
            raise ValueError("evaluation cases require an ID and expected output")


def exact_match(prediction: str, expected: str) -> bool:
    return prediction.replace("\r\n", "\n").strip() == expected.replace("\r\n", "\n").strip()


def evaluate_cases(model, tokenizer: ByteBPETokenizer, cases: Iterable[EvaluationCase], maximum_new_tokens: int = 64) -> dict[str, object]:
    results = []
    for case in cases:
        case.validate()
        prediction = generate(model, tokenizer, case.prompt, maximum_new_tokens)
        results.append({"case_id": case.case_id, "kind": case.kind, "exact_match": exact_match(prediction, case.expected), "prediction": prediction})
    return {"cases": results, "exact_match": sum(item["exact_match"] for item in results) / max(1, len(results))}


def evaluate_loss(model, sequences: list[list[int]]) -> dict[str, float]:
    import mlx.core as mx
    import mlx.nn as nn

    if not sequences:
        raise ValueError("evaluation requires packed sequences")
    array = mx.array(sequences)
    loss = nn.losses.cross_entropy(model(array[:, :-1]), array[:, 1:], reduction="mean")
    mx.eval(loss)
    value = float(loss.item())
    return {"loss": value, "perplexity": math.exp(min(20.0, value))}


def generate(model, tokenizer: ByteBPETokenizer, prompt: str, maximum_new_tokens: int = 64, temperature: float = 0.0) -> str:
    import mlx.core as mx

    tokens = tokenizer.encode(prompt, bos=True)
    for _ in range(maximum_new_tokens):
        logits = model(mx.array([tokens]))[0, -1]
        if temperature <= 0:
            next_token = int(mx.argmax(logits).item())
        else:
            next_token = int(mx.random.categorical(logits / temperature).item())
        if next_token == 2:
            break
        tokens.append(next_token)
    return tokenizer.decode(tokens)[len(prompt) :]
