"""Helel decoder-only Transformer configuration and lazy MLX implementation."""

from __future__ import annotations
from dataclasses import asdict, dataclass
import json
from pathlib import Path


@dataclass(frozen=True)
class ModelConfig:
    version: int = 1
    vocabulary_size: int = 4096
    context_length: int = 1024
    dimensions: int = 384
    layers: int = 12
    heads: int = 6
    feed_forward_dimensions: int = 1024
    rope_theta: float = 10_000.0
    rms_norm_epsilon: float = 1e-5

    def validate(self) -> None:
        if self.version != 1:
            raise ValueError("unsupported model configuration")
        if self.dimensions % self.heads:
            raise ValueError("dimensions must be divisible by heads")
        if min(self.vocabulary_size, self.context_length, self.dimensions, self.layers, self.heads, self.feed_forward_dimensions) <= 0:
            raise ValueError("model dimensions must be positive")

    def parameter_count(self) -> int:
        """Count tied-embedding Transformer parameters exactly."""
        embedding = self.vocabulary_size * self.dimensions
        attention = 4 * self.dimensions * self.dimensions
        feed_forward = 3 * self.dimensions * self.feed_forward_dimensions
        norms = 2 * self.dimensions
        return embedding + self.layers * (attention + feed_forward + norms) + self.dimensions

    def save(self, path: Path) -> None:
        path.write_text(json.dumps(asdict(self), indent=2, sort_keys=True) + "\n", encoding="utf-8")

    @classmethod
    def load(cls, path: Path) -> "ModelConfig":
        config = cls(**json.loads(path.read_text(encoding="utf-8")))
        config.validate()
        return config


HELEL_22M = ModelConfig()


def create_model(config: ModelConfig = HELEL_22M):
    """Create the MLX model lazily so metadata tools work without a Metal device."""
    config.validate()
    import mlx.core as mx
    import mlx.nn as nn

    class Attention(nn.Module):
        def __init__(self) -> None:
            super().__init__()
            self.heads = config.heads
            self.head_dimension = config.dimensions // config.heads
            self.scale = self.head_dimension**-0.5
            self.qkv = nn.Linear(config.dimensions, 3 * config.dimensions, bias=False)
            self.output = nn.Linear(config.dimensions, config.dimensions, bias=False)
            self.rope = nn.RoPE(self.head_dimension, traditional=False, base=config.rope_theta)

        def __call__(self, values, mask):
            batch, length, _ = values.shape
            qkv = self.qkv(values).reshape(batch, length, 3, self.heads, self.head_dimension)
            queries, keys, values = [qkv[:, :, index].transpose(0, 2, 1, 3) for index in range(3)]
            queries = self.rope(queries)
            keys = self.rope(keys)
            attended = mx.fast.scaled_dot_product_attention(queries, keys, values, scale=self.scale, mask=mask)
            return self.output(attended.transpose(0, 2, 1, 3).reshape(batch, length, config.dimensions))

    class FeedForward(nn.Module):
        def __init__(self) -> None:
            super().__init__()
            self.gate = nn.Linear(config.dimensions, config.feed_forward_dimensions, bias=False)
            self.up = nn.Linear(config.dimensions, config.feed_forward_dimensions, bias=False)
            self.down = nn.Linear(config.feed_forward_dimensions, config.dimensions, bias=False)

        def __call__(self, values):
            return self.down(nn.silu(self.gate(values)) * self.up(values))

    class Block(nn.Module):
        def __init__(self) -> None:
            super().__init__()
            self.attention_norm = nn.RMSNorm(config.dimensions, eps=config.rms_norm_epsilon)
            self.attention = Attention()
            self.feed_forward_norm = nn.RMSNorm(config.dimensions, eps=config.rms_norm_epsilon)
            self.feed_forward = FeedForward()

        def __call__(self, values, mask):
            values = values + self.attention(self.attention_norm(values), mask)
            return values + self.feed_forward(self.feed_forward_norm(values))

    class HelelModel(nn.Module):
        def __init__(self) -> None:
            super().__init__()
            self.embedding = nn.Embedding(config.vocabulary_size, config.dimensions)
            self.blocks = [Block() for _ in range(config.layers)]
            self.norm = nn.RMSNorm(config.dimensions, eps=config.rms_norm_epsilon)

        def __call__(self, tokens):
            values = self.embedding(tokens)
            mask = nn.MultiHeadAttention.create_additive_causal_mask(tokens.shape[1])
            for block in self.blocks:
                values = block(values, mask)
            return self.norm(values) @ self.embedding.weight.T

    return HelelModel()
