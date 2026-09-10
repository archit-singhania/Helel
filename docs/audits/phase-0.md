# Phase 0 audit

Audit date: 2026-09-11

## Result

Phase 0 is complete. The repository is a buildable foundation with no coding-agent product functionality yet.

## Built

- Cargo workspace with `helel-core` and the native Tauri application
- pnpm workspace with a React, TypeScript, Vite desktop frontend
- Python `helel_ml` package scaffold and import test
- Local launch screen and generated cross-platform application icons
- Contract locations for model, tool, and event schemas
- Architecture records for desktop, core, agent, model, storage, and security
- Concise repository instructions, roadmap, license, and phase execution plans
- Reproducible Rust and JavaScript lockfiles
- One verification entry point: `python3 scripts/verify.py`

## Verification evidence

The full verification command passed on macOS arm64. It ran:

- Rust format and Clippy with warnings denied
- Rust workspace unit and documentation tests
- TypeScript lint and strict typecheck
- Frontend unit test and production build
- Python import/unit test
- Contract JSON validation
- Tauri debug build without an installer bundle

The resulting native binary is generated at `target/debug/helel-desktop` and is intentionally ignored by Git.

## Deliberately left for later

- Phase 1 application layout, panels, settings, project picker, themes, and smoke tests
- Editor, files, terminal, Git, patches, indexing, and SQLite
- Agent state machine, tool permissions, model runtime, training, and benchmarks
- Windows and Linux build verification
- Installer packaging, code signing, notarization, and store distribution

## Cost audit

Repository dependencies and development tools are free and open source. The application currently uses no hosted AI API, cloud service, paid database, telemetry service, or paid runtime.

FOC does not include the computer and electricity used to develop or train models, internet access, optional Apple/Microsoft store accounts, certificates, code signing/notarization, or future GPU compute. A useful tiny model can be researched locally, but training a competitive model from scratch cannot realistically remain free at larger scales.

## Risks and decisions

- Only macOS has been built in this phase; cross-platform source structure is present, but Windows and Linux must be verified on those operating systems.
- Dependency versions are locked, but routine security and compatibility maintenance will still be needed.
- The current UI is a launch placeholder, not the Phase 1 IDE shell.
- Security boundaries are documented but are not implemented until local tools arrive in Phase 3.
