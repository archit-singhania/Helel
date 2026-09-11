# Model architecture

## Ownership

The model subsystem owns datasets, the tokenizer, decoder-only Transformer code, training, evaluation, checkpoints, and local inference. It does not own agent planning, tool execution, workspace access, or UI.

Training uses Python and an open-source tensor framework while weights and architecture remain Helel-owned. The local runtime exposes versioned token and generation contracts to the agent. Unit tests cover tensor shapes and serialization; evaluations cover completion, repair, fill-in-the-middle, and tool choice. No hosted inference API is permitted.

## Dataset and tokenizer foundation

Phase 6 accepts only explicitly registered local sources with an allowlisted SPDX license, origin, and immutable revision. It normalizes Unicode and newlines, removes high-confidence credential shapes, filters unsuitable files, removes exact and near duplicates through an inverted shingle index, and assigns documents to stable 90/5/5 splits from their content hashes. Versioned manifests retain source lineage and SHA-256 checksums for every split.

The Helel tokenizer is deterministic byte-level BPE with stable merge tie-breaking and reserved padding, sequence, and fill-in-the-middle tokens. Every UTF-8 string remains lossless. The serialized format is versioned and covered by a JSON Schema contract. Dataset outputs and tokenizer artifacts are generated locally and are excluded from Git by default.

## Validation model

Phase 7 defines Helel-22M as a 22,816,128-parameter decoder-only Transformer with tied token embeddings, 12 layers, 384 hidden dimensions, six attention heads, 1,024 SwiGLU dimensions, RMS normalization, rotary positions, and causal scaled-dot-product attention. Its versioned configuration uses a 4,096-token vocabulary and 1,024-token context.

The MLX training path packs documents, applies deterministic fill-in-the-middle transformations, trains with AdamW, warmup plus cosine decay, and global gradient clipping, and writes safetensors checkpoints with compatibility metadata and SHA-256 corruption checks. Checkpoints restore both model and optimizer state. Evaluation interfaces cover causal loss, perplexity, greedy generation, completion, repair, FIM, memorization, and exact match.
