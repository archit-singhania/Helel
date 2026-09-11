# Security architecture

## Ownership

Security owns workspace boundaries, path normalization, command risk classification, user approvals, network policy, secrets handling, and audit records. It does not decide which code change best satisfies a task.

Every filesystem, process, Git, and patch operation must pass through these checks. The agent and desktop depend on security decisions; security depends only on narrow core types and operating-system primitives. Tests must cover traversal, symlinks, command classification, scope escape, redaction, and denial behavior. Network use and telemetry remain disabled by default.

Phase 3 classifies direct program invocations as safe, modifying, or dangerous. Modifying and dangerous commands require explicit approval, destructive commands remain clearly identified, and read commands with parent or absolute paths are treated as dangerous. Processes inherit the active workspace as their working directory and through `HELEL_WORKSPACE`; shell expansion is unavailable. Patch writes use a validation pass before mutation, and each process or patch decision creates an audit entry.

The v0.1 desktop CSP denies all network connections, objects, frames, and base-URL changes. Its capability file grants only core window behavior and directory selection. Model requests have prompt, generation, stop-sequence, and temperature limits. Repository content is labeled untrusted, model proposal shapes are closed, and every modifying tool still passes through the core approval boundary.
