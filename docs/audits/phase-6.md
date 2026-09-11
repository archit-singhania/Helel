# Phase 6 audit

Audit date: 2026-09-11

## Result

Phase 6 is complete. Helel can transform explicitly registered, locally available, permissively licensed sources into reproducible training splits and train a deterministic lossless tokenizer without network access.

## Built

- Versioned source registry and JSON Schema with stable IDs, local paths, SPDX licenses, origins, revisions, and include globs
- Allowlist for Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, CC0-1.0, and Unlicense material
- UTF-8 validation, NFC normalization, newline normalization, trailing-space cleanup, size and extension filters
- High-confidence redaction for common cloud keys, GitHub tokens, credential assignments, and private-key headers
- SHA-256 exact deduplication and inverted five-token-shingle near-duplicate detection
- Deterministic content-hash 90/5/5 split assignment with leakage checks
- JSONL datasets with document IDs, licenses, source IDs, paths, checksums, and redaction counts
- Versioned manifests with registry checksum, lineage, quality statistics, split statistics, and output checksums
- Deterministic byte-level BPE tokenizer with lossless Unicode handling and stable merge tie-breaking
- Reserved padding, begin/end sequence, and fill-in-the-middle tokens
- Tokenizer serialization, loading, compression report, and JSON Schema contract
- Tiny repository-owned Apache-2.0 fixture corpus and CLI

## Verification

The Python suite covers normalization, credential redaction, rejected licenses, exact and near deduplication, split assignment, reproducible manifests, source lineage, tokenizer determinism, Unicode round trips, compression, and serialization. The final repository gate also covers Rust, TypeScript, frontend, contracts, production assets, and the native macOS build.

## Boundaries and known limits

- The pipeline never downloads a corpus; a human must review and register each local source.
- The license allowlist is a technical admission policy, not a legal conclusion about a particular dataset or use.
- Secret scanning intentionally favors high-confidence patterns and cannot prove a corpus contains no sensitive data.
- Near-duplicate matching uses lexical shingles and may miss structurally equivalent code with substantial renaming.
- The committed fixture is for validation and is far too small for model training.
- Phase 6 trains a tokenizer only. Neural model training begins in Phase 7.

## Cost audit

The pipeline uses Python's standard library and local files. It requires no paid corpus, hosted tokenizer, cloud database, telemetry, API key, or usage-priced service. Large-scale training later may have electricity and hardware costs, but no such cost was introduced here.
