# Model architecture

## Ownership

The model subsystem owns datasets, the tokenizer, decoder-only Transformer code, training, evaluation, checkpoints, and local inference. It does not own agent planning, tool execution, workspace access, or UI.

Training uses Python and an open-source tensor framework while weights and architecture remain Helel-owned. The local runtime exposes versioned token and generation contracts to the agent. Unit tests cover tensor shapes and serialization; evaluations cover completion, repair, fill-in-the-middle, and tool choice. No hosted inference API is permitted.

## Dataset and tokenizer foundation

Phase 6 accepts only explicitly registered local sources with an allowlisted SPDX license, origin, and immutable revision. It normalizes Unicode and newlines, removes high-confidence credential shapes, filters unsuitable files, removes exact and near duplicates through an inverted shingle index, and assigns documents to stable 90/5/5 splits from their content hashes. Versioned manifests retain source lineage and SHA-256 checksums for every split.

The Helel tokenizer is deterministic byte-level BPE with stable merge tie-breaking and reserved padding, sequence, and fill-in-the-middle tokens. Every UTF-8 string remains lossless. The serialized format is versioned and covered by a JSON Schema contract. Dataset outputs and tokenizer artifacts are generated locally and are excluded from Git by default.
