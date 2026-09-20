# Model architecture

## Ownership

The model subsystem owns datasets, the tokenizer, decoder-only Transformer code, training, evaluation, checkpoints, and local inference. It does not own agent planning, tool execution, workspace access, or UI.

Training uses Python and an open-source tensor framework while weights and architecture remain Helel-owned. The local runtime exposes versioned token and generation contracts to the agent. Unit tests cover tensor shapes and serialization; evaluations cover completion, repair, fill-in-the-middle, and tool choice. No hosted inference API is permitted.

## Dataset and tokenizer foundation

Phase 6 accepts only explicitly registered local sources with an allowlisted SPDX license, origin, and immutable revision. It normalizes Unicode and newlines, removes high-confidence credential shapes, filters unsuitable files, removes exact and near duplicates through an inverted shingle index, and assigns documents to stable 90/5/5 splits from their content hashes. Versioned manifests retain source lineage and SHA-256 checksums for every split.

The Helel tokenizer is deterministic byte-level BPE with stable merge tie-breaking and reserved padding, sequence, and fill-in-the-middle tokens. Every UTF-8 string remains lossless. The serialized format is versioned and covered by a JSON Schema contract. Dataset outputs and tokenizer artifacts are generated locally and are excluded from Git by default.

Phase 12 classifies more than 40 common programming and documentation formats in every accepted document and records per-language coverage in the manifest. Tokenizer and training inputs use seeded temperature sampling: dominant languages retain more samples while low-volume languages receive enough exposure to avoid being erased by a large TypeScript, Python, or Rust source. The sampler is deterministic, configurable, and preserves the source license, revision, redaction, deduplication, and split-lineage checks.

## Validation model

Phase 7 defines Helel-22M as a 22,816,128-parameter decoder-only Transformer with tied token embeddings, 12 layers, 384 hidden dimensions, six attention heads, 1,024 SwiGLU dimensions, RMS normalization, rotary positions, and causal scaled-dot-product attention. Its versioned configuration uses a 4,096-token vocabulary and 1,024-token context.

The MLX training path packs documents, applies deterministic fill-in-the-middle transformations, trains with AdamW, warmup plus cosine decay, and global gradient clipping, and writes safetensors checkpoints with compatibility metadata and SHA-256 corruption checks. Checkpoints restore both model and optimizer state. Evaluation interfaces cover causal loss, perplexity, greedy generation, completion, repair, FIM, memorization, and exact match.

## Product model and local inference

Helel-46M has 45,954,560 parameters, 13 layers, 512 hidden dimensions, eight heads, 1,408 SwiGLU dimensions, an 8,192-token vocabulary, and a 2,048-token context. The training policy defines licensed source-code, test/repair, documentation, and tool-trace mixtures plus staged context and FIM curricula. Release-candidate manifests require all artifacts and completion, repair, FIM, and repository-context evaluations.

Local inference loads version-compatible tokenizer, configuration, and safetensors files, applies strict request/token/resource limits, and streams JSON-line events over standard IO. Four-bit and eight-bit weight-only quantization target linear layers; tied embeddings remain full precision. No network transport exists. Useful product weights are not bundled until an approved corpus is trained and evaluated.
