"""Reproducible, license-aware local dataset construction."""

from __future__ import annotations
from dataclasses import asdict, dataclass
from hashlib import sha256
import json
from pathlib import Path
import re
import unicodedata

PIPELINE_VERSION = 1
ALLOWED_LICENSES = frozenset({"Apache-2.0", "MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC", "CC0-1.0", "Unlicense"})
TEXT_EXTENSIONS = frozenset({".c", ".cc", ".cpp", ".css", ".go", ".h", ".hpp", ".html", ".java", ".js", ".json", ".md", ".py", ".rs", ".sh", ".toml", ".ts", ".tsx", ".txt", ".yaml", ".yml"})
SECRET_PATTERNS = (
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"gh[pousr]_[A-Za-z0-9_]{20,}"),
    re.compile(r"(?i)(api[_-]?key|secret|token|password)\s*[:=]\s*['\"]?[^\s'\"]{8,}"),
    re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
)


@dataclass(frozen=True)
class SourceRecord:
    source_id: str
    path: str
    license: str
    origin: str
    revision: str
    include: tuple[str, ...] = ()

    @classmethod
    def from_dict(cls, value: dict[str, object]) -> "SourceRecord":
        required = ("source_id", "path", "license", "origin", "revision")
        if any(not isinstance(value.get(key), str) or not value[key] for key in required):
            raise ValueError("source entries require non-empty source_id, path, license, origin, and revision")
        license_name = str(value["license"])
        if license_name not in ALLOWED_LICENSES:
            raise ValueError(f"license is not allowlisted: {license_name}")
        includes = value.get("include", [])
        if not isinstance(includes, list) or not all(isinstance(item, str) for item in includes):
            raise ValueError("include must be a string list")
        return cls(*(str(value[key]) for key in required), tuple(includes))


@dataclass(frozen=True)
class DatasetDocument:
    document_id: str
    source_id: str
    path: str
    license: str
    text: str
    sha256: str
    redactions: int


@dataclass(frozen=True)
class BuildReport:
    read_files: int
    accepted_documents: int
    rejected_files: int
    exact_duplicates: int
    near_duplicates: int
    redactions: int
    split_counts: dict[str, int]


def normalize(text: str) -> str:
    text = unicodedata.normalize("NFC", text.replace("\r\n", "\n").replace("\r", "\n"))
    return "\n".join(line.rstrip() for line in text.split("\n")).strip() + "\n"


def redact_secrets(text: str) -> tuple[str, int]:
    count = 0
    for pattern in SECRET_PATTERNS:
        text, found = pattern.subn("<REDACTED_SECRET>", text)
        count += found
    return text, count


def _shingles(text: str) -> set[str]:
    tokens = re.findall(r"[A-Za-z_][A-Za-z0-9_]*|\S", text.lower())
    if len(tokens) < 5:
        return {" ".join(tokens)}
    return {" ".join(tokens[index : index + 5]) for index in range(len(tokens) - 4)}


def _near_duplicate(left: set[str], right: set[str]) -> bool:
    union = len(left | right)
    return bool(union) and len(left & right) / union >= 0.85


class DatasetBuilder:
    """Build deterministic JSONL splits from an explicit local registry."""

    def __init__(self, registry_path: Path, output_directory: Path) -> None:
        self.registry_path = registry_path.resolve()
        self.output_directory = output_directory.resolve()

    def load_registry(self) -> list[SourceRecord]:
        payload = json.loads(self.registry_path.read_text(encoding="utf-8"))
        if payload.get("version") != 1 or not isinstance(payload.get("sources"), list):
            raise ValueError("registry must contain version 1 and a sources list")
        records = [SourceRecord.from_dict(item) for item in payload["sources"]]
        if len({item.source_id for item in records}) != len(records):
            raise ValueError("source_id values must be unique")
        return sorted(records, key=lambda item: item.source_id)

    def ingest(self) -> tuple[list[DatasetDocument], BuildReport]:
        documents: list[DatasetDocument] = []
        exact: set[str] = set()
        fingerprints: list[set[str]] = []
        shingle_owners: dict[str, set[int]] = {}
        read_files = rejected = exact_duplicates = near_duplicates = redactions = 0
        for source in self.load_registry():
            root = (self.registry_path.parent / source.path).resolve()
            if not root.is_dir():
                raise ValueError(f"source path is not a directory: {source.path}")
            patterns = source.include or ("**/*",)
            paths = sorted({path for pattern in patterns for path in root.glob(pattern) if path.is_file()})
            for path in paths:
                read_files += 1
                if path.is_symlink() or path.suffix.lower() not in TEXT_EXTENSIONS or path.stat().st_size > 2 * 1024 * 1024:
                    rejected += 1
                    continue
                try:
                    text = normalize(path.read_text(encoding="utf-8"))
                except UnicodeDecodeError:
                    rejected += 1
                    continue
                text, found = redact_secrets(text)
                redactions += found
                if len(text) < 16 or len(text) > 2_000_000:
                    rejected += 1
                    continue
                digest = sha256(text.encode()).hexdigest()
                if digest in exact:
                    exact_duplicates += 1
                    continue
                shingles = _shingles(text)
                candidates: set[int] = set()
                for shingle in shingles:
                    candidates.update(shingle_owners.get(shingle, ()))
                if any(_near_duplicate(shingles, fingerprints[index]) for index in candidates):
                    near_duplicates += 1
                    continue
                relative = path.relative_to(root).as_posix()
                exact.add(digest)
                fingerprint_index = len(fingerprints)
                fingerprints.append(shingles)
                for shingle in shingles:
                    shingle_owners.setdefault(shingle, set()).add(fingerprint_index)
                documents.append(DatasetDocument(digest[:24], source.source_id, relative, source.license, text, digest, found))
        documents.sort(key=lambda item: item.document_id)
        counts = {name: 0 for name in ("train", "validation", "test")}
        for document in documents:
            counts[self.split_for(document.document_id)] += 1
        return documents, BuildReport(read_files, len(documents), rejected, exact_duplicates, near_duplicates, redactions, counts)

    @staticmethod
    def split_for(document_id: str) -> str:
        bucket = int(document_id[:8], 16) % 100
        return "train" if bucket < 90 else "validation" if bucket < 95 else "test"

    def build(self) -> dict[str, object]:
        documents, report = self.ingest()
        self.output_directory.mkdir(parents=True, exist_ok=True)
        checksums: dict[str, str] = {}
        split_ids: dict[str, set[str]] = {}
        for split in ("train", "validation", "test"):
            selected = [item for item in documents if self.split_for(item.document_id) == split]
            split_ids[split] = {item.document_id for item in selected}
            payload = "".join(json.dumps(asdict(item), sort_keys=True, ensure_ascii=False) + "\n" for item in selected)
            path = self.output_directory / f"{split}.jsonl"
            path.write_text(payload, encoding="utf-8")
            checksums[path.name] = sha256(payload.encode()).hexdigest()
        pairs = (("train", "validation"), ("train", "test"), ("validation", "test"))
        if any(split_ids[left] & split_ids[right] for left, right in pairs):
            raise RuntimeError("dataset split leakage detected")
        manifest = {"pipeline_version": PIPELINE_VERSION, "registry_sha256": sha256(self.registry_path.read_bytes()).hexdigest(), "sources": [asdict(item) for item in self.load_registry()], "report": asdict(report), "checksums": checksums}
        (self.output_directory / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return manifest
