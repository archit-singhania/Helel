# P06 — Dataset and tokenizer pipeline

## Goal

Create a reproducible, legally traceable local pipeline for training data and the Helel tokenizer.

## Delivered

1. Versioned local source registry with SPDX license allowlist, origin, and revision
2. Unicode/newline normalization, file quality filters, and credential redaction
3. SHA-256 exact deduplication and scalable shingle-index near deduplication
4. Stable content-hash train, validation, and test split isolation
5. Deterministic lossless byte-level BPE training, encoding, decoding, and evaluation
6. Dataset lineage, split checksums, quality counts, and tokenizer reports
7. Apache-2.0 fixture corpus and continuous verification tests

Large corpus acquisition and model training remain outside this phase.
