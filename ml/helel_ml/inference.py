"""Strictly local Helel generation runtime and JSON-lines protocol."""

from __future__ import annotations
from dataclasses import asdict, dataclass
import json
from pathlib import Path
import sys
from typing import Iterator

from .model import ModelConfig, create_model
from .tokenizer import ByteBPETokenizer


@dataclass(frozen=True)
class GenerationRequest:
    request_id: str
    prompt: str
    maximum_new_tokens: int = 128
    temperature: float = 0.0
    stop: tuple[str, ...] = ()

    def validate(self, context_length: int) -> None:
        if not self.request_id or not self.prompt:
            raise ValueError("request_id and prompt are required")
        if not 1 <= self.maximum_new_tokens <= min(512, context_length):
            raise ValueError("maximum_new_tokens is outside the local safety limit")
        if not 0 <= self.temperature <= 2:
            raise ValueError("temperature must be between zero and two")
        if len(self.prompt.encode("utf-8")) > 2 * 1024 * 1024 or len(self.stop) > 8:
            raise ValueError("request exceeds local resource limits")


@dataclass(frozen=True)
class GenerationEvent:
    request_id: str
    kind: str
    text: str = ""
    token_count: int = 0
    error: str = ""


class LocalInferenceRuntime:
    """Loads local artifacts and generates without filesystem or tool authority."""

    def __init__(self, config_path: Path, tokenizer_path: Path, weights_path: Path) -> None:
        self.config = ModelConfig.load(config_path)
        self.tokenizer = ByteBPETokenizer.load(tokenizer_path)
        if self.tokenizer.vocabulary_size != self.config.vocabulary_size:
            raise ValueError("tokenizer and model vocabulary sizes differ")
        self.model = create_model(self.config)
        self.model.load_weights(str(weights_path))

    def stream(self, request: GenerationRequest) -> Iterator[GenerationEvent]:
        import mlx.core as mx

        request.validate(self.config.context_length)
        tokens = self.tokenizer.encode(request.prompt, bos=True)[-self.config.context_length :]
        generated = ""
        pending = bytearray()
        generated_count = 0
        for count in range(1, request.maximum_new_tokens + 1):
            logits = self.model(mx.array([tokens[-self.config.context_length :]]))[0, -1]
            token = int(mx.argmax(logits).item()) if request.temperature == 0 else int(mx.random.categorical(logits / request.temperature).item())
            if token == 2:
                break
            tokens.append(token)
            generated_count = count
            pending.extend(self.tokenizer.token_bytes(token))
            try:
                piece = pending.decode("utf-8")
            except UnicodeDecodeError:
                continue
            pending.clear()
            generated += piece
            yield GenerationEvent(request.request_id, "token", piece, count)
            if any(generated.endswith(stop) for stop in request.stop):
                break
        if pending:
            piece = pending.decode("utf-8", errors="replace")
            yield GenerationEvent(request.request_id, "token", piece, generated_count)
        yield GenerationEvent(request.request_id, "done", token_count=generated_count)


def serve(runtime: LocalInferenceRuntime) -> None:
    """Serve newline-delimited requests over local standard IO."""
    for line in sys.stdin:
        request_id = "unknown"
        try:
            payload = json.loads(line)
            request_id = str(payload.get("request_id", "unknown"))
            if payload.get("kind") == "health":
                print(json.dumps(asdict(GenerationEvent(request_id, "healthy")), sort_keys=True), flush=True)
                continue
            request = GenerationRequest(request_id=request_id, prompt=payload["prompt"], maximum_new_tokens=int(payload.get("maximum_new_tokens", 128)), temperature=float(payload.get("temperature", 0)), stop=tuple(payload.get("stop", ())))
            for event in runtime.stream(request):
                print(json.dumps(asdict(event), sort_keys=True), flush=True)
        except Exception as error:
            print(json.dumps(asdict(GenerationEvent(request_id, "error", error=str(error))), sort_keys=True), flush=True)


def main() -> None:
    import argparse
    parser = argparse.ArgumentParser(description="Run the local Helel JSON-lines inference process")
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--tokenizer", type=Path, required=True)
    parser.add_argument("--weights", type=Path, required=True)
    args = parser.parse_args()
    serve(LocalInferenceRuntime(args.config, args.tokenizer, args.weights))


if __name__ == "__main__":
    main()
