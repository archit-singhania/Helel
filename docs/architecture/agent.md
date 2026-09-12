# Agent architecture

## Ownership

The agent owns task state, context selection, action validation, tool sequencing, observations, retries, and completion criteria. It does not directly access the operating system, render UI, train models, or bypass permission policy.

It depends on core tool interfaces, repository context, model runtime interfaces, and persisted state. Desktop may observe and control it; core tools must not depend on it. Public interfaces will be typed commands, events, and an auditable state machine. Scripted models and fake tools test behavior before a learned model is connected.

## Deterministic foundation

Phase 5 implements planning, gathering, executing, approval, verification, completion, failure, and cancellation states. Each transition is ordered by a step number and records a typed observation. Tool requests are versioned and limited to code search, file reads, Git inspection, direct process execution, and checked patches. Mutating tools pause at an approval boundary. Session ledgers persist in `.helel/agents.json` and reload with the workspace.

The current planner is scripted by design. A future local model may propose plans and typed requests, but it must use this state machine and cannot receive direct filesystem or process authority.

Phase 9 adds a local-model planner bridge. It serializes a bounded repository snapshot inside explicit untrusted-data delimiters, requires exactly one JSON proposal, validates the selected tool and its complete argument shape, and hands accepted proposals to the existing deterministic tool and approval policy. Generated text alone never executes an action.
## Phase 11 hardening

The orchestration core now tracks step, failure, generated-output, cancellation, and repeated-action limits independently of the planner. Local-model output enters through a strict versioned decoder and becomes an existing typed `ToolRequest`; it never gains direct process or filesystem authority. The desktop session driver still uses the Phase 5 deterministic sequence and must be migrated to the bounded loop before autonomous task completion is available in the UI.
