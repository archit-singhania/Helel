# Agent architecture

## Ownership

The agent owns task state, context selection, action validation, tool sequencing, observations, retries, and completion criteria. It does not directly access the operating system, render UI, train models, or bypass permission policy.

It depends on core tool interfaces, repository context, model runtime interfaces, and persisted state. Desktop may observe and control it; core tools must not depend on it. Public interfaces will be typed commands, events, and an auditable state machine. Scripted models and fake tools test behavior before a learned model is connected.

## Deterministic foundation

Phase 5 implements planning, gathering, executing, approval, verification, completion, failure, and cancellation states. Each transition is ordered by a step number and records a typed observation. Tool requests are versioned and limited to code search, file reads, Git inspection, direct process execution, and checked patches. Mutating tools pause at an approval boundary. Session ledgers persist in `.helel/agents.json` and reload with the workspace.

The current planner is scripted by design. A future local model may propose plans and typed requests, but it must use this state machine and cannot receive direct filesystem or process authority.
