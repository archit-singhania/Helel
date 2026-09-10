# Helel

Helel is an offline-first autonomous software engineering environment. The repository now contains the Phase 5 deterministic agent foundation: a Tauri IDE with secure local tools, persisted repository intelligence, ranked context retrieval, and auditable agent sessions.

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

Phases 0 through 5 are complete. The dataset and tokenizer pipeline begins in Phase 6. See [the roadmap](docs/ROADMAP.md), [the Phase 4 audit](docs/audits/phase-4.md), and [the Phase 5 audit](docs/audits/phase-5.md).

## Cost and privacy

All production dependencies are free and open source. The completed phases use no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
