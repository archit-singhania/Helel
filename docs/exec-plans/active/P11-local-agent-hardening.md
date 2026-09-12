# P11 local-agent hardening

## Goal

Turn the developer preview into a measurable, recoverable local coding-agent runtime. This plan implements the pre-manual-testing work in dependency order and records external prerequisites instead of representing them as code-complete.

## Current slice

1. Add HelelBench with 30 deterministic tasks and machine-readable reports.
2. Detect supported project stacks and derive explicit validation commands.
3. Add transactional multi-file patches with task-scoped rollback.
4. Add a persistent tamper-evident audit ledger.
5. Add a budgeted planner/executor loop and strict proposal decoder.
6. Add a local-only MCP protocol boundary.

## Exit checks

- New unit and integration tests pass.
- `python3 scripts/verify.py` passes.
- Architecture and readiness status describe actual behavior and remaining prerequisites.

