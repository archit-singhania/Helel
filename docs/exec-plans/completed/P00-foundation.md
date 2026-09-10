# P00 — Phase 0 foundation

## Goal

Create the buildable, documented, agent-friendly foundation for Helel without product functionality.

## Delivered

- Cargo and pnpm workspaces plus Python package
- Minimal React/Tauri desktop application
- Rust core boundary
- Contract directories
- Architecture and roadmap documents
- One cross-platform verification command
- Apache-2.0 license and ignore rules

## Acceptance

Run `python3 scripts/verify.py`. It checks Rust formatting, lint and tests; TypeScript lint, types, tests and build; Python import/tests; contract JSON; and a debug Tauri build without packaging.

## Out of scope

Editor, filesystem operations, terminal, Git, persistence, agent behavior, datasets, model training, inference, installers, signing, and release publishing.
