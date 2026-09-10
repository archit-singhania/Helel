# P05 — Deterministic agent

## Goal

Add an auditable agent state machine that can use local tools without bypassing security policy.

## Delivered

1. Typed lifecycle states and ordered transitions
2. Versioned code-search, file-read, Git, command, and patch tool requests
3. Observation recording, failure handling, cancellation, and terminal-state protection
4. Per-action approval gates for modifying tools
5. Local session-ledger persistence and workspace reload
6. Desktop controls for starting, advancing, reviewing, and cancelling sessions
7. State, approval, persistence, and contract tests

Learned planning and model inference remain outside this phase.
