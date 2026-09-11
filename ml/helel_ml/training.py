"""Packed/FIM data preparation and local MLX training utilities."""

from __future__ import annotations
from dataclasses import asdict, dataclass
import json
import math
from pathlib import Path
import random
from typing import Iterable, Iterator

from .model import ModelConfig, create_model
from .tokenizer import ByteBPETokenizer


@dataclass(frozen=True)
class TrainingConfig:
    seed: int = 42
    batch_size: int = 2
    sequence_length: int = 128
    learning_rate: float = 3e-4
    minimum_learning_rate: float = 3e-5
    warmup_steps: int = 10
    maximum_steps: int = 100
    weight_decay: float = 0.1
    gradient_clip: float = 1.0
    fim_rate: float = 0.5


def fill_in_middle(tokens: list[int], rng: random.Random, prefix_id: int = 3, middle_id: int = 4, suffix_id: int = 5) -> list[int]:
    """Apply the standard prefix/suffix/middle rearrangement deterministically."""
    if len(tokens) < 3:
        return tokens[:]
    left, right = sorted(rng.sample(range(1, len(tokens)), 2))
    prefix, middle, suffix = tokens[:left], tokens[left:right], tokens[right:]
    return [prefix_id, *prefix, suffix_id, *suffix, middle_id, *middle]


def packed_sequences(texts: Iterable[str], tokenizer: ByteBPETokenizer, sequence_length: int, seed: int, fim_rate: float) -> list[list[int]]:
    if sequence_length < 2:
        raise ValueError("sequence_length must be at least two")
    rng = random.Random(seed)
    stream: list[int] = []
    for text in texts:
        tokens = tokenizer.encode(text, bos=True, eos=True)
        if rng.random() < fim_rate:
            tokens = fill_in_middle(tokens, rng)
        stream.extend(tokens)
    usable = len(stream) - (len(stream) % (sequence_length + 1))
    return [stream[index : index + sequence_length + 1] for index in range(0, usable, sequence_length + 1)]


def batches(sequences: list[list[int]], batch_size: int, seed: int) -> Iterator[list[list[int]]]:
    order = list(range(len(sequences)))
    random.Random(seed).shuffle(order)
    for start in range(0, len(order) - batch_size + 1, batch_size):
        yield [sequences[index] for index in order[start : start + batch_size]]


def learning_rate(step: int, config: TrainingConfig) -> float:
    if step < config.warmup_steps:
        return config.learning_rate * (step + 1) / max(1, config.warmup_steps)
    progress = min(1.0, (step - config.warmup_steps) / max(1, config.maximum_steps - config.warmup_steps))
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return config.minimum_learning_rate + (config.learning_rate - config.minimum_learning_rate) * cosine


def train(model_config: ModelConfig, training_config: TrainingConfig, sequences: list[list[int]], output: Path, resume_from: Path | None = None) -> list[dict[str, float]]:
    """Train locally with AdamW, global-norm clipping, and causal loss."""
    import mlx.core as mx
    import mlx.nn as nn
    import mlx.optimizers as optim

    mx.random.seed(training_config.seed)
    model = create_model(model_config)
    optimizer = optim.AdamW(learning_rate=training_config.learning_rate, weight_decay=training_config.weight_decay)
    start_step = 0
    if resume_from is not None:
        start_step = int(load_checkpoint(resume_from, model, optimizer, model_config)["step"])

    def loss_fn(active_model, inputs, targets):
        logits = active_model(inputs)
        return nn.losses.cross_entropy(logits, targets, reduction="mean")

    loss_and_grad = nn.value_and_grad(model, loss_fn)
    history: list[dict[str, float]] = []
    sequence_batches = list(batches(sequences, training_config.batch_size, training_config.seed))
    if not sequence_batches:
        raise ValueError("not enough packed sequences for one batch")
    for local_step in range(training_config.maximum_steps):
        step = start_step + local_step
        batch = sequence_batches[step % len(sequence_batches)]
        array = mx.array(batch)
        loss, gradients = loss_and_grad(model, array[:, :-1], array[:, 1:])
        gradients, total_norm = optim.clip_grad_norm(gradients, max_norm=training_config.gradient_clip)
        rate = learning_rate(step, training_config)
        optimizer.learning_rate = rate
        optimizer.update(model, gradients)
        mx.eval(model.parameters(), optimizer.state, loss, total_norm)
        history.append({"step": float(step + 1), "loss": float(loss.item()), "gradient_norm": float(total_norm.item()), "learning_rate": rate})
    output.mkdir(parents=True, exist_ok=True)
    save_checkpoint(output, model, model_config, training_config, optimizer, start_step + len(history), history[-1]["loss"])
    return history


def save_checkpoint(directory: Path, model, model_config: ModelConfig, training_config: TrainingConfig, optimizer, step: int, loss: float) -> None:
    """Write model weights and checksummed compatibility metadata."""
    from hashlib import sha256
    import mlx.core as mx
    from mlx.utils import tree_flatten

    weights = directory / "model.safetensors"
    optimizer_path = directory / "optimizer.safetensors"
    model.save_weights(str(weights))
    mx.save_safetensors(str(optimizer_path), dict(tree_flatten(optimizer.state)))
    metadata = {"version": 1, "step": step, "loss": loss, "model_config": asdict(model_config), "training_config": asdict(training_config), "files": {weights.name: sha256(weights.read_bytes()).hexdigest(), optimizer_path.name: sha256(optimizer_path.read_bytes()).hexdigest()}}
    (directory / "checkpoint.json").write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def load_checkpoint(directory: Path, model, optimizer, expected_config: ModelConfig) -> dict[str, object]:
    """Verify and restore model and optimizer state for deterministic resume."""
    import mlx.core as mx
    from mlx.utils import tree_unflatten

    metadata = verify_checkpoint(directory, expected_config)
    model.load_weights(str(directory / "model.safetensors"))
    optimizer.state = tree_unflatten(list(mx.load(str(directory / "optimizer.safetensors")).items()))
    mx.eval(model.parameters(), optimizer.state)
    return metadata


def verify_checkpoint(directory: Path, expected_config: ModelConfig | None = None) -> dict[str, object]:
    from hashlib import sha256

    metadata = json.loads((directory / "checkpoint.json").read_text(encoding="utf-8"))
    if metadata.get("version") != 1:
        raise ValueError("unsupported checkpoint version")
    if expected_config is not None and metadata.get("model_config") != asdict(expected_config):
        raise ValueError("checkpoint model configuration mismatch")
    for filename, expected in metadata["files"].items():
        path = directory / filename
        if not path.is_file() or sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"checkpoint corruption detected: {filename}")
    return metadata
