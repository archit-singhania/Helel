# Helel

Helel is an offline-first autonomous software engineering environment. This repository currently contains the Phase 0 foundation: a Tauri desktop shell, Rust core, React/TypeScript frontend, Python ML workspace, shared contracts, architecture records, and one verification command.

## Requirements

- Rust 1.85 or newer
- Node.js 22 or newer and pnpm 10 or newer
- Python 3.11 or newer
- Tauri system prerequisites for your operating system

## Setup

```sh
pnpm install
python3 scripts/verify.py
```

Run the browser frontend with `pnpm dev`, or the native desktop application with `pnpm tauri dev`.

## Project status

Phase 0 establishes buildable boundaries only. Product features begin in Phase 1. See [the roadmap](docs/ROADMAP.md) and [the Phase 0 plan](docs/exec-plans/completed/P00-foundation.md).

## Cost and privacy

All production dependencies are free and open source. Phase 0 uses no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
