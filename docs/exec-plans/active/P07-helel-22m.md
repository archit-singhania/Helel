# P07 — Helel-22M pipeline validation model

## Goal

Implement and validate the complete local model training path at approximately 22 million parameters before scaling it.

## Planned slices

1. Decoder-only Transformer and configuration contracts
2. Causal attention, rotary positions, normalization, and feed-forward blocks
3. Packed dataset loader with fill-in-the-middle transformation
4. Deterministic training loop, optimizer, schedules, and gradient controls
5. Checkpoint save, resume, compatibility, and corruption detection
6. Completion, repair, memorization, and overfit evaluations
7. CPU smoke training and documented accelerator profiles

Product-scale training and desktop inference remain outside this phase.
