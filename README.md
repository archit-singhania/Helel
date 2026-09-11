# Helel

Helel is an offline-first autonomous software engineering environment. The repository now contains the Phase 6 data foundation: a Tauri IDE with secure local tools, repository intelligence, deterministic agents, and a reproducible licensed dataset and tokenizer pipeline.

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

Phases 0 through 6 are complete. Helel-22M pipeline validation begins in Phase 7. See [the roadmap](docs/ROADMAP.md) and [the Phase 6 audit](docs/audits/phase-6.md).

## Cost and privacy

All production dependencies are free and open source. The completed phases use no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
