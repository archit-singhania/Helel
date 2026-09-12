# Helel implementation audit

Audit date: 2026-09-12

## Current outcome

Helel is a verified offline developer preview with an IDE, trusted local tools, model-training infrastructure, and the first pre-manual hardening foundations. It is not yet a fully autonomous Codex/Claude-class app: no useful trained weights are installed, the desktop still drives a scripted agent, and several OS integrations remain incomplete.

## Sixteen-area status

| # | Area | Status | Evidence / remaining work |
|---:|---|---|---|
| 1 | Repository and architecture | Done | Workspace, contracts, architecture, verification script. |
| 2 | Tauri/React desktop shell | Done | Native desktop build and application layout. |
| 3 | Editor and IDE core | Done | Monaco, tabs, file tree, search and replace. |
| 4 | Secure filesystem | Done | Canonical workspace boundaries and traversal tests. |
| 5 | Processes, Git, patches | Partial | Shell-free streaming/cancel and patch checks work; PTY, stage/commit UI, and task diff remain. |
| 6 | Repository intelligence | Partial | Persistent lexical symbols/context work; Tree-sitter/LSP and SQLite/FTS5 remain. |
| 7 | Agent sessions | Partial | Durable sessions and typed tools work; desktop still uses the scripted sequence. |
| 8 | Dataset pipeline | Done | Licensed-source registry, normalization, redaction, deduplication, stable splits. A production corpus still requires owner approval. |
| 9 | Tokenizer | Done | Deterministic byte BPE and versioned artifacts. |
| 10 | Helel-22M model pipeline | Infrastructure done | Architecture, training/checkpoint/evaluation code; useful weights require local training. |
| 11 | Helel-46M product model | Infrastructure done | Architecture, curricula, quantization policy; useful weights require local training and evaluation. |
| 12 | Local inference protocol | Partial | Bounded JSONL Python runtime exists; packaging, lifecycle supervision, and a real checkpoint remain. |
| 13 | Agent hardening | Partial | Budget state and strict proposal bridge exist in Rust; full planner/executor loop and desktop wiring remain. |
| 14 | Safety and recovery | Partial | Task-scoped rollback and durable hash-chained ledger primitives exist; desktop audit migration, dirty-buffer coordination, and failure injection remain. |
| 15 | HelelBench and stack policy | Partial | 30 deterministic infrastructure cases, report format, stack detection and commands work; real edit/repair/scaffold repositories and agent-driven execution remain. |
| 16 | MCP, product UX, releases | Early | Local MCP JSON-RPC validation boundary exists; stdio server lifecycle/tool discovery, unified agent UI, native watching, PTY, framework skills, cross-platform validation, and signing remain. |

## External prerequisites

- A useful private model requires a legally approved local corpus, owned compute time, and evaluation. Code can automate this, but the repository cannot manufacture trained weights.
- Clean-machine macOS testing, two-hour soak testing, accessibility testing, and Windows/Linux release checks require the respective user or CI machines.
- The full manual test gate remains closed until the P0 acceptance criteria in `PRE_MANUAL_TESTING_PLAN.md` pass.
