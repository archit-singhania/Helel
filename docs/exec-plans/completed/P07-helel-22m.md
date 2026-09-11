# P07 — Helel-22M pipeline validation model

## Goal

Implement and validate the complete local model training path at approximately 22 million parameters before scaling it.

## Delivered

1. Versioned 22,816,128-parameter decoder-only Transformer configuration
2. Causal grouped tensor path with rotary positions, RMS normalization, and SwiGLU blocks
3. Deterministic packed dataset loader and fill-in-the-middle transformation
4. AdamW training, warmup/cosine schedule, and global gradient clipping
5. Safetensors checkpoints, optimizer resume, compatibility checks, and corruption detection
6. Completion, repair, FIM, memorization, loss, perplexity, and generation evaluations
7. Tiny local training/overfit/resume smoke test and full-size forward validation on Apple Metal

Product-scale corpus training and desktop inference remain outside this phase.
