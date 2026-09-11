# Phase 8 audit

Audit date: 2026-09-11

## Result

The Helel-46M engineering pipeline is complete, while product weights remain pending. The exact model, curriculum, corpus mixture, evaluation gates, quantization path, memory profile, and release-candidate manifest are implemented and tested. No claim of learned coding quality is made.

## Built and validated

- 45,954,560 parameters; 13 layers; 512 dimensions; eight heads; 1,408 SwiGLU dimensions
- 8,192-token vocabulary and 2,048-token context configuration
- License-bound 70% source, 15% tests/repairs, 10% documentation, and 5% tool-trace mixture
- Staged 512/1,024/2,048 context and 25%/50%/70% FIM curriculum
- FP32, FP16, int8, and int4 weight memory estimates
- Four/eight-bit weight-only MLX quantization for linear layers
- Required completion, repair, FIM, and repository-context release evidence
- Checksummed release-candidate artifacts and reproducibility metadata
- Full-size finite logits shaped `(1, 5, 8192)` in full precision and int4

## Pending external work

- Review and approval of a production-scale licensed corpus
- The long-running pretraining job and its hardware/electricity budget
- Held-out quality results and a release-candidate checkpoint
- Quantized-versus-full-precision quality comparison on trained weights

## Cost audit

All implementation uses free local tooling. No paid corpus, API, cloud GPU, experiment service, or hosted model was introduced. A real training run will consume local or separately acquired compute resources.
