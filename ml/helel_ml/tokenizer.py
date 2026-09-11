"""Deterministic byte-level BPE tokenizer for Helel models."""

from __future__ import annotations
from collections import Counter
from dataclasses import dataclass
import json
from pathlib import Path
from typing import Iterable

SPECIAL_TOKENS = ("<pad>", "<bos>", "<eos>", "<fim_prefix>", "<fim_middle>", "<fim_suffix>")


@dataclass(frozen=True)
class TokenizerReport:
    vocabulary_size: int
    training_bytes: int
    training_tokens: int
    bytes_per_token: float


class ByteBPETokenizer:
    """Lossless UTF-8 tokenizer with deterministic merge tie-breaking."""

    def __init__(self, merges: list[tuple[int, int]] | None = None) -> None:
        self.merges = merges or []
        self._pieces: dict[int, bytes] = {index: bytes([index]) for index in range(256)}
        for left, right in self.merges:
            self._pieces[len(self._pieces)] = self._pieces[left] + self._pieces[right]

    @property
    def vocabulary_size(self) -> int:
        return len(SPECIAL_TOKENS) + len(self._pieces)

    @classmethod
    def train(cls, texts: Iterable[str], vocabulary_size: int = 512) -> tuple["ByteBPETokenizer", TokenizerReport]:
        if vocabulary_size < 256 + len(SPECIAL_TOKENS):
            raise ValueError("vocabulary_size must preserve all byte and special tokens")
        raw = [text.encode("utf-8") for text in texts if text]
        sequences = [list(item) for item in raw]
        merges: list[tuple[int, int]] = []
        while 256 + len(SPECIAL_TOKENS) + len(merges) < vocabulary_size:
            counts = Counter(pair for sequence in sequences for pair in zip(sequence, sequence[1:]))
            if not counts:
                break
            best_count = max(counts.values())
            if best_count < 2:
                break
            pair = min(pair for pair, count in counts.items() if count == best_count)
            token = 256 + len(merges)
            merges.append(pair)
            sequences = [_merge(sequence, pair, token) for sequence in sequences]
        tokenizer = cls(merges)
        byte_count = sum(map(len, raw))
        token_count = sum(map(len, sequences))
        report = TokenizerReport(tokenizer.vocabulary_size, byte_count, token_count, byte_count / max(1, token_count))
        return tokenizer, report

    def encode(self, text: str, *, bos: bool = False, eos: bool = False) -> list[int]:
        sequence = list(text.encode("utf-8"))
        for token, pair in enumerate(self.merges, start=256):
            sequence = _merge(sequence, pair, token)
        offset = len(SPECIAL_TOKENS)
        encoded = [token + offset for token in sequence]
        if bos:
            encoded.insert(0, SPECIAL_TOKENS.index("<bos>"))
        if eos:
            encoded.append(SPECIAL_TOKENS.index("<eos>"))
        return encoded

    def decode(self, tokens: Iterable[int]) -> str:
        output = bytearray()
        for token in tokens:
            output.extend(self.token_bytes(token))
        return output.decode("utf-8")

    def token_bytes(self, token: int) -> bytes:
        offset = len(SPECIAL_TOKENS)
        if token < offset:
            return b""
        piece = self._pieces.get(token - offset)
        if piece is None:
            raise ValueError(f"unknown token id: {token}")
        return piece

    def save(self, path: Path) -> None:
        payload = {"version": 1, "kind": "byte_bpe", "special_tokens": SPECIAL_TOKENS, "merges": self.merges, "vocabulary_size": self.vocabulary_size}
        path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    @classmethod
    def load(cls, path: Path) -> "ByteBPETokenizer":
        payload = json.loads(path.read_text(encoding="utf-8"))
        if payload.get("version") != 1 or payload.get("kind") != "byte_bpe" or tuple(payload.get("special_tokens", ())) != SPECIAL_TOKENS:
            raise ValueError("unsupported tokenizer format")
        return cls([tuple(map(int, pair)) for pair in payload["merges"]])


def _merge(sequence: list[int], pair: tuple[int, int], token: int) -> list[int]:
    output: list[int] = []
    index = 0
    while index < len(sequence):
        if index + 1 < len(sequence) and (sequence[index], sequence[index + 1]) == pair:
            output.append(token)
            index += 2
        else:
            output.append(sequence[index])
            index += 1
    return output
