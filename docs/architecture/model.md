# Model architecture

## Ownership

The model subsystem owns datasets, the tokenizer, decoder-only Transformer code, training, evaluation, checkpoints, and local inference. It does not own agent planning, tool execution, workspace access, or UI.

Training uses Python and an open-source tensor framework while weights and architecture remain Helel-owned. The local runtime exposes versioned token and generation contracts to the agent. Unit tests cover tensor shapes and serialization; evaluations cover completion, repair, fill-in-the-middle, and tool choice. No hosted inference API is permitted.
