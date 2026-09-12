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

## Completion evidence

- HelelBench runs 30/30 disposable tasks with validation and rollback.
- The integrated Rust agent fixture performs read, transactional patch, validation, completion and rollback.
- Agent limits cover steps, output bytes, failures, repeated actions, 30-minute session wall time and 300-second child execution.
- Tree-sitter/SQLite definitions and references update after source modification and deletion.
- MCP performs 2025-06-18 initialization, discovery and an actual tool call over local stdio.
- The desktop exposes full pending-action review, retry, verified audit export, Git mutation controls, PTY controls and dirty-buffer patch blocking.
- `python3 scripts/verify.py` is the final required gate.

Useful trained weights, clean-machine acceptance, cross-platform inference and release signing are external follow-up programs documented in the product audit.
