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
| 5 | Processes, Git, patches | Done for local preview | PTY input/output/resize/cancel, transactional multi-file patches, deletion rollback, stage/commit UI, staged/unstaged diff. |
| 6 | Repository intelligence | Substantially done | Tree-sitter for Rust, Python, TypeScript/TSX and Java with SQLite/FTS5 persistence and exact definition/reference lookup. Event-driven full incremental semantic updates and LSP diagnostics remain. |
| 7 | Agent sessions | Done for local preview | Model-driven bounded loop, typed tools, real observations, persistence, retry budgets, pause/resume/cancel/completion. Coding quality depends on weights. |
| 8 | Dataset pipeline | Done | Licensed-source registry, normalization, redaction, deduplication, stable splits. A production corpus still requires owner approval. |
| 9 | Tokenizer | Done | Deterministic byte BPE and versioned artifacts. |
| 10 | Helel-22M model pipeline | Infrastructure done | Architecture, training/checkpoint/evaluation code; useful weights require local training. |
| 11 | Helel-46M product model | Infrastructure done | Architecture, curricula, quantization policy; useful weights require local training and evaluation. |
| 12 | Local inference protocol | Done for source checkout | Artifact validation, health check, context-aware generation limit, supervised JSONL streaming, project-local launcher and trained smoke checkpoint work. Tauri production resource bundling remains. |
| 13 | Agent hardening | Done for local preview | Trusted proposal bridge and desktop planner/executor loop use bounded real tool output, project validation policy, MCP discovery, budgets and per-action approval. More adversarial end-to-end cases remain useful. |
| 14 | Safety and recovery | Done for local preview | Hash-chained audit persistence, dirty-buffer patch blocking, task-scoped checkpoints, deletion handling, atomic apply and rollback are wired. Platform failure injection remains. |
| 15 | HelelBench and stack policy | Substantially done | 30 disposable source edit/repair/test/refactor/scaffold cases pass 30/30; seven framework recipes and stack commands exist. Learned-agent quality scoring awaits useful weights. |
| 16 | MCP, product UX, releases | Substantially done locally | MCP 2025-06-18 stdio initialization, discovery, calls, configuration UI and agent tool routing work; unified task display, Git actions, native watching, PTY and controls work. Model resource packaging, cross-platform testing and signing remain. |

## External prerequisites

- A useful private model requires a legally approved local corpus, owned compute time, and evaluation. Code can automate this, but the repository cannot manufacture trained weights.
- Clean-machine macOS testing, two-hour soak testing, accessibility testing, and Windows/Linux release checks require the respective user or CI machines.
- The source-checkout manual test gate is open for disposable repositories. Release acceptance remains closed until clean-machine, soak, accessibility and cross-platform checks pass.
