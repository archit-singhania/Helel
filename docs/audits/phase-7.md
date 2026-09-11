# Phase 7 audit

Audit date: 2026-09-11

## Result

Phase 7 is complete as a pipeline-validation milestone. Helel now has a 22,816,128-parameter decoder-only model definition and a locally verified MLX path from packed tokenizer output through training, evaluation, checkpointing, corruption detection, and optimizer resume.

## Built

- Versioned Helel-22M configuration with exact parameter accounting
- Tied token embeddings and output projection
- Multi-head causal scaled-dot-product attention
- Rotary positional encoding, RMS normalization, residual connections, and SwiGLU feed-forward blocks
- Deterministic packed sequences and seeded fill-in-the-middle transformations
- AdamW, linear warmup, cosine decay, and global gradient-norm clipping
- Causal cross-entropy, perplexity, deterministic greedy generation, and exact-match scoring
- Completion, repair, fill-in-the-middle, and memorization evaluation contracts
- Safetensors model and optimizer checkpoints
- Versioned checkpoint metadata with model/training configurations and SHA-256 checksums
- Strict compatibility, missing-file, and corruption validation
- Model and optimizer restoration for resumed training
- JSON Schema contracts for model configuration and checkpoints

## Executed validation

- The full 22,816,128-parameter model produced finite logits with shape `(1, 5, 4096)` on local Apple Metal.
- A 101,440-parameter smoke model completed forward, backward, AdamW, clipping, checkpoint, restore, and resume paths.
- The 12-step memorization smoke reduced loss from `6.7262` to `2.4162` and resumed successfully at step 13.
- The portable Python suite covers configuration, parameter count, FIM, packing, schedules, evaluation contracts, checkpoint compatibility, and corruption handling.

## Boundaries and known limits

- Phase 7 validates the pipeline and architecture; it does not claim the randomly initialized 22M model has learned coding ability.
- Only a forward pass was run at full 22M scale. Training validation used the small smoke configuration to keep the gate fast.
- MLX targets Apple silicon. A portable CPU or CUDA tensor backend remains future work.
- The committed fixture corpus cannot support meaningful model quality evaluation.
- Large-scale training, product weights, quantization, and benchmark claims belong to Phase 8.

## Cost audit

MLX is free and MIT-licensed, and every operation runs locally. No hosted training service, inference API, experiment tracker, telemetry, paid model, or cloud storage was added. The smoke runs used existing local hardware; future production training may consume meaningful electricity and hardware time.
