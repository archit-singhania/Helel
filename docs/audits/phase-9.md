# Phase 9 audit

Audit date: 2026-09-11

## Result

The model-agent integration is complete at the protocol and policy boundary. Compatible local weights can generate streamed proposals, but model text cannot directly invoke tools or bypass approvals. The bundled app continues using its deterministic planner because no trained product checkpoint exists.

## Built

- Strict local artifact compatibility checks
- Bounded generation request and streaming event protocol
- Standard-IO transport without a network server
- Context truncation, generation caps, temperature bounds, stop sequences, and errors
- Repository-context isolation as explicitly untrusted serialized data
- Closed proposal schema with an allowlist of agent tools
- Exact per-tool argument validation and payload size limits
- Fake-generator tests for safe proposal routing and hostile repository text

## Remaining

- Install evaluated Helel-46M weights and tokenizer
- Add a packaged MLX sidecar before distributing the runtime to machines without Python
- Measure end-to-end token latency with trained quantized weights

## Cost audit

Inference is local and uses free MLX tooling. There is no hosted endpoint, API key, telemetry, or usage-based charge.
