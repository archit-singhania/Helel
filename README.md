# Helel

Helel is an offline-first autonomous software engineering environment. The repository now contains the Phase 1 desktop shell: a Tauri application with a React/TypeScript workbench, local preferences, themes, recent projects, and a native folder picker, backed by a Rust workspace and Python ML foundation.

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

Phases 0 and 1 are complete. Editor functionality begins in Phase 2. See [the roadmap](docs/ROADMAP.md) and [the Phase 1 audit](docs/audits/phase-1.md).

## Cost and privacy

All production dependencies are free and open source. Phase 0 uses no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
