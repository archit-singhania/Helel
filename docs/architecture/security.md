# Security architecture

## Ownership

Security owns workspace boundaries, path normalization, command risk classification, user approvals, network policy, secrets handling, and audit records. It does not decide which code change best satisfies a task.

Every filesystem, process, Git, and patch operation must pass through these checks. The agent and desktop depend on security decisions; security depends only on narrow core types and operating-system primitives. Tests must cover traversal, symlinks, command classification, scope escape, redaction, and denial behavior. Network use and telemetry remain disabled by default.
