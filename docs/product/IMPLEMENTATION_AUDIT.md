# Helel implementation audit

Audit date: 2026-09-20

## Current outcome

Helel is a verified offline developer preview with a model-driven bounded agent loop, supervised local inference, native watching and PTY support, transactional recovery, Tree-sitter/SQLite indexing, local stdio MCP lifecycle support, multilingual data tooling, and working macOS offline voice. It is not yet a Codex/Claude-quality agent: the first locally trained 22.8M candidate failed the leakage-free tool-selection gate, and clean-machine/cross-platform acceptance has not run.

## Sixteen-area status

| # | Area | Status | Evidence / remaining work |
|---:|---|---|---|
| 1 | Repository and architecture | Done | Workspace, contracts, architecture, verification script. |
| 2 | Tauri/React desktop shell | Done | Native desktop build and application layout. |
| 3 | Editor and IDE core | Done | Monaco, tabs, file tree, search and replace. |
| 4 | Secure filesystem | Done | Canonical workspace boundaries and traversal tests. |
| 5 | Processes, Git, patches | Done for local preview | PTY input/output/resize/cancel, transactional multi-file patches, deletion rollback, stage/commit UI, staged/unstaged diff. |
| 6 | Repository intelligence | Done for local preview | Tree-sitter for Rust, Python, TypeScript/TSX and Java with SQLite/FTS5 persistence, exact definition/reference lookup, persisted-index startup, single-pass full rebuilds, and incremental watcher updates including directory-prefix removal. LSP diagnostics remain advanced work. |
| 7 | Agent sessions | Done for local preview | Model-driven bounded loop, typed tools, real observations, persistence, step/output/failure/repetition/wall-time budgets, explicit retry, pause/resume/cancel/completion. Coding quality depends on weights. |
| 8 | Dataset pipeline | Done | Licensed-source registry, normalization, redaction, deduplication, stable splits, deterministic classification for more than 40 formats, per-language reporting, and seeded temperature balancing. A production corpus still requires owner approval. |
| 9 | Tokenizer | Done | Deterministic byte BPE and versioned artifacts. |
| 10 | Helel-22M model pipeline | Infrastructure validated; candidate rejected | Architecture, training/checkpoint/evaluation code and local MLX/Metal training work. The 22.8M SFT v3 candidate reached 40% valid proposals and 0% tool accuracy on leakage-free evaluation, so it is retained only as a research artifact. |
| 11 | Helel-46M product model | Infrastructure done | Architecture, curricula, quantization policy; useful weights require local training and evaluation. |
| 12 | Local inference protocol | Done for source checkout | Artifact compatibility and recorded SHA-256 validation, health check, context-aware generation limit, supervised JSONL streaming, project-local launcher and trained smoke checkpoint work. Tauri production resource bundling remains. |
| 13 | Agent hardening | Done for local preview | Trusted proposal bridge and desktop planner/executor loop use bounded real tool output, timed commands, project validation policy, MCP discovery, budgets and per-action approval. An integrated read/patch/validate/complete/rollback fixture passes. |
| 14 | Safety and recovery | Done for local preview | SHA-256 hash-chained audit persistence with legacy verification, dirty-buffer patch blocking, task-scoped checkpoints, deletion handling, atomic apply and rollback are wired. Platform failure injection remains. |
| 15 | HelelBench and stack policy | Substantially done | 30 disposable source edit/repair/test/refactor/scaffold cases pass 30/30; seven framework recipes and stack commands exist. Learned-agent quality scoring awaits useful weights. |
| 16 | MCP, product UX, voice, releases | Substantially done locally | MCP 2025-06-18 stdio initialization, discovery, calls, configuration UI and agent tool routing work; the premium task workspace includes action/patch review, command palette, retry and verified audit export. Startup project opens are single-flight, terminal rendering is bounded, and the ambient renderer is capped at 30 FPS. Bounded offline microphone capture, local multilingual Whisper transcription and macOS speech output work end to end. Model resource packaging, cross-platform testing and signing remain. |

## External prerequisites

- A useful private model requires a materially larger legally approved local corpus, owned compute time, and repeated leakage-free evaluation. The current trained candidate explicitly failed selection and is not the app default.
- Clean-machine macOS testing, two-hour soak testing, accessibility testing, and Windows/Linux release checks require the respective user or CI machines.
- The source-checkout manual test gate is open for disposable repositories. Release acceptance remains closed until clean-machine, soak, accessibility and cross-platform checks pass.
