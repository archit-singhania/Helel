# Helel

Helel is an offline-first autonomous software engineering environment.

## Read first

- Architecture: `docs/architecture/index.md`
- Roadmap: `docs/ROADMAP.md`
- Security: `docs/architecture/security.md`
- Model: `docs/architecture/model.md`
- Active plans: `docs/exec-plans/active/`

## Hard constraints

- Production Helel must not depend on hosted LLM or inference APIs.
- Primary languages are Rust, TypeScript, and Python.
- The desktop stack is Tauri and React; local storage will use SQLite.
- The model is a custom decoder-only Transformer trained with local tooling.
- Keep network access and telemetry off by default.

## Workflow

Before changing code, read the relevant architecture document and active plan. Stay within the current phase. Keep architecture documents synchronized with architectural changes.

Before completion, run `python3 scripts/verify.py`. Do not claim completion if required checks fail.
