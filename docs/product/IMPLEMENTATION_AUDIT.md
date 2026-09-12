# Helel implementation audit

Audit date: 2026-09-12

## Current outcome

Helel is a verified offline developer preview with a model-driven bounded agent loop, supervised local inference, native watching and PTY support, transactional recovery, Tree-sitter/SQLite indexing, and local stdio MCP lifecycle support. It is not yet a Codex/Claude-quality agent because no useful trained Helel weights are installed and clean-machine/cross-platform acceptance has not run.

## Sixteen-area status

| # | Area | Status | Evidence / remaining work |
|---:|---|---|---|
| 1 | Repository and architecture | Done | Workspace, contracts, architecture, verification script. |
| 2 | Tauri/React desktop shell | Done | Native desktop build and application layout. |
| 3 | Editor and IDE core | Done | Monaco, tabs, file tree, search and replace. |
| 4 | Secure filesystem | Done | Canonical workspace boundaries and traversal tests. |
| 5 | Processes, Git, patches | Substantially done | PTY input/output/resize/cancel, transactional patches, rollback, stage/commit APIs, staged/unstaged diff. Stage/commit UI remains. |
| 6 | Repository intelligence | Substantially done | Tree-sitter for four languages and SQLite/FTS5 persistence work. Incremental semantic updates and LSP diagnostics remain. |
| 7 | Agent sessions | Substantially done | Model-driven bounded loop, typed tools, persistence, retry, pause/resume/cancel/completion work. Quality depends on weights. |
| 8 | Dataset pipeline | Done | Licensed-source registry, normalization, redaction, deduplication, stable splits. A production corpus still requires owner approval. |
| 9 | Tokenizer | Done | Deterministic byte BPE and versioned artifacts. |
| 10 | Helel-22M model pipeline | Infrastructure done | Architecture, training/checkpoint/evaluation code; useful weights require local training. |
| 11 | Helel-46M product model | Infrastructure done | Architecture, curricula, quantization policy; useful weights require local training and evaluation. |
| 12 | Local inference protocol | Substantially done locally | Artifact validation, health check, supervised JSONL streaming, local launcher and smoke checkpoint work. Tauri production resource bundling remains. |
| 13 | Agent hardening | Substantially done | Trusted proposal bridge and desktop planner/executor loop are wired with budgets and approval. Adversarial end-to-end expansion remains. |
| 14 | Safety and recovery | Substantially done | Desktop audit persistence, native dirty-buffer notices, task-scoped checkpoints, atomic apply and rollback work. Broader failure injection remains. |
| 15 | HelelBench and stack policy | Partial | 30 disposable source edit/repair/test/refactor/scaffold cases pass; seven framework recipes and stack commands exist. Agent-driven build validators remain. |
| 16 | MCP, product UX, releases | Partial | MCP stdio registry, initialization, discovery and approved calls work; unified task display, native watching, PTY and controls work. MCP configuration UI, model packaging, cross-platform testing and signing remain. |

## External prerequisites

- A useful private model requires a legally approved local corpus, owned compute time, and evaluation. Code can automate this, but the repository cannot manufacture trained weights.
- Clean-machine macOS testing, two-hour soak testing, accessibility testing, and Windows/Linux release checks require the respective user or CI machines.
- The full manual test gate remains closed until the P0 acceptance criteria in `PRE_MANUAL_TESTING_PLAN.md` pass.
