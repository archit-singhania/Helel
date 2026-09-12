# Security architecture

## Ownership

Security owns workspace boundaries, path normalization, command risk classification, user approvals, network policy, secrets handling, and audit records. It does not decide which code change best satisfies a task.

Every filesystem, process, Git, and patch operation must pass through these checks. The agent and desktop depend on security decisions; security depends only on narrow core types and operating-system primitives. Tests must cover traversal, symlinks, command classification, scope escape, redaction, and denial behavior. Network use and telemetry remain disabled by default.

Phase 3 classifies direct program invocations as safe, modifying, or dangerous. Modifying and dangerous commands require explicit approval, destructive commands remain clearly identified, and read commands with parent or absolute paths are treated as dangerous. Processes inherit the active workspace as their working directory and through `HELEL_WORKSPACE`; shell expansion is unavailable. Patch writes use a validation pass before mutation, and each process or patch decision creates an audit entry.

The v0.1 desktop CSP denies all network connections, objects, frames, and base-URL changes. Its capability file grants only core window behavior and directory selection. Model requests have prompt, generation, stop-sequence, and temperature limits. Repository content is labeled untrusted, model proposal shapes are closed, and every modifying tool still passes through the core approval boundary.

Phase 11 adds task-scoped checkpoints which restore only paths declared before a mutation, including deletion-only patches, while preserving unrelated working-tree changes. The desktop refuses direct and agent-driven patches while an editor buffer is dirty. The trusted Rust proposal decoder rejects unknown fields, incompatible sessions, replayed nonces, oversized payloads, and excessive command arguments.

The desktop writes filesystem mutations, process and PTY lifecycle operations, Git mutations, patch and rollback actions, model-runtime lifecycle operations, and MCP operations to `.helel/audit.jsonl`. The append-only ledger verifies its hash chain before accepting another record. MCP is restricted to explicitly configured local stdio children, performs the MCP 2025-06-18 initialization lifecycle, discovers tools, and requires approval for every tool call. MCP descriptions and results are labeled untrusted when added to a model prompt.
