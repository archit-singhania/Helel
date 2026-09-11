"""Small executable MLX smoke test for Phase 7."""

from __future__ import annotations
import argparse
from pathlib import Path

from .model import ModelConfig
from .tokenizer import ByteBPETokenizer
from .training import TrainingConfig, load_checkpoint, packed_sequences, train, verify_checkpoint


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--steps", type=int, default=2)
    args = parser.parse_args()
    texts = ["fn greet(name: &str) -> String { name.to_owned() }\n"] * 12
    tokenizer, _ = ByteBPETokenizer.train(texts, 300)
    model = ModelConfig(vocabulary_size=tokenizer.vocabulary_size, context_length=32, dimensions=64, layers=2, heads=4, feed_forward_dimensions=128)
    config = TrainingConfig(batch_size=2, sequence_length=31, maximum_steps=args.steps, warmup_steps=1, fim_rate=0.5)
    sequences = packed_sequences(texts, tokenizer, config.sequence_length, config.seed, config.fim_rate)
    history = train(model, config, sequences, args.output)
    verify_checkpoint(args.output, model)
    import mlx.optimizers as optim
    from .model import create_model
    restored_model = create_model(model)
    restored_optimizer = optim.AdamW(learning_rate=config.learning_rate, weight_decay=config.weight_decay)
    load_checkpoint(args.output, restored_model, restored_optimizer, model)
    resumed = train(model, TrainingConfig(batch_size=2, sequence_length=31, maximum_steps=1, warmup_steps=1), sequences, args.output / "resumed", resume_from=args.output)
    if len(history) > 1 and history[-1]["loss"] >= history[0]["loss"]:
        raise RuntimeError("memorization smoke loss did not decrease")
    print(f"smoke training passed: {len(history)} steps plus resume step {int(resumed[-1]['step'])}, {model.parameter_count()} parameters, loss {history[0]['loss']:.4f} -> {history[-1]['loss']:.4f}")


if __name__ == "__main__":
    main()
