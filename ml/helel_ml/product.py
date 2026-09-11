"""Helel-46M corpus mixture, curriculum, and quantization utilities."""

from __future__ import annotations
from dataclasses import asdict, dataclass
import json
from pathlib import Path
from typing import Iterable

from .model import ModelConfig

HELEL_46M = ModelConfig(vocabulary_size=8192, context_length=2048, dimensions=512, layers=13, heads=8, feed_forward_dimensions=1408)


@dataclass(frozen=True)
class MixtureSource:
    name: str
    weight: float
    license_policy: str = "phase6-allowlist"


DEFAULT_MIXTURE = (
    MixtureSource("source_code", 0.70),
    MixtureSource("tests_and_repairs", 0.15),
    MixtureSource("documentation", 0.10),
    MixtureSource("tool_traces", 0.05),
)


@dataclass(frozen=True)
class CurriculumStage:
    until_fraction: float
    context_length: int
    fim_rate: float
    learning_rate_scale: float


DEFAULT_CURRICULUM = (
    CurriculumStage(0.10, 512, 0.25, 1.0),
    CurriculumStage(0.60, 1024, 0.50, 1.0),
    CurriculumStage(1.00, 2048, 0.70, 0.5),
)


def validate_mixture(sources: Iterable[MixtureSource]) -> tuple[MixtureSource, ...]:
    result = tuple(sources)
    if not result or len({item.name for item in result}) != len(result):
        raise ValueError("mixture sources must be present and uniquely named")
    if any(item.weight <= 0 or item.license_policy != "phase6-allowlist" for item in result):
        raise ValueError("mixture weights must be positive and use the license allowlist")
    if abs(sum(item.weight for item in result) - 1.0) > 1e-9:
        raise ValueError("mixture weights must sum to one")
    return result


def curriculum_at(progress: float, stages: Iterable[CurriculumStage] = DEFAULT_CURRICULUM) -> CurriculumStage:
    if not 0 <= progress <= 1:
        raise ValueError("training progress must be between zero and one")
    for stage in stages:
        if progress <= stage.until_fraction:
            return stage
    raise ValueError("curriculum must cover the full run")


def memory_profile(config: ModelConfig = HELEL_46M) -> dict[str, float]:
    parameters = config.parameter_count()
    return {"parameters": float(parameters), "fp32_mib": parameters * 4 / 2**20, "fp16_mib": parameters * 2 / 2**20, "int8_mib": parameters / 2**20, "int4_mib": parameters / 2 / 2**20}


def quantize_checkpoint(source: Path, destination: Path, bits: int = 4, group_size: int = 64) -> dict[str, object]:
    """Quantize an MLX checkpoint and record measured size reduction."""
    if bits not in {4, 8} or group_size not in {32, 64, 128}:
        raise ValueError("supported quantization is 4/8-bit with 32/64/128 groups")
    import mlx.nn as nn
    from .model import create_model
    from .training import verify_checkpoint

    metadata = verify_checkpoint(source, HELEL_46M)
    model = create_model(HELEL_46M)
    model.load_weights(str(source / "model.safetensors"))
    nn.quantize(model, bits=bits, group_size=group_size, class_predicate=lambda _path, module: isinstance(module, nn.Linear))
    destination.mkdir(parents=True, exist_ok=True)
    weights = destination / "model.safetensors"
    model.save_weights(str(weights))
    report = {"version": 1, "source_step": metadata["step"], "bits": bits, "group_size": group_size, "source_bytes": (source / "model.safetensors").stat().st_size, "quantized_bytes": weights.stat().st_size}
    (destination / "quantization.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return report


def release_candidate_manifest(checkpoint: Path, tokenizer: Path, evaluations: dict[str, object], output: Path) -> dict[str, object]:
    """Create a release candidate only when required evidence is present."""
    from hashlib import sha256
    required = {"completion", "repair", "fim", "repository_context"}
    if not required <= evaluations.keys():
        raise ValueError(f"missing evaluations: {sorted(required - evaluations.keys())}")
    files = {"checkpoint": checkpoint, "tokenizer": tokenizer}
    if any(not path.is_file() for path in files.values()):
        raise ValueError("release candidate files are missing")
    manifest = {"version": 1, "model": "Helel-46M", "parameters": HELEL_46M.parameter_count(), "artifacts": {name: {"path": path.name, "sha256": sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size} for name, path in files.items()}, "evaluations": evaluations, "mixture": [asdict(item) for item in validate_mixture(DEFAULT_MIXTURE)], "curriculum": [asdict(item) for item in DEFAULT_CURRICULUM]}
    output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest
