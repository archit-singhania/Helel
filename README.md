# Helel

Helel is an offline-first autonomous software engineering environment. The repository now contains the Phase 7 model validation foundation: a Tauri IDE, secure local agent tools, reproducible data/tokenizer pipelines, and a locally trainable 22.8M-parameter decoder-only Transformer.

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

Phases 0 through 7 are complete. The Helel-46M product model begins in Phase 8. See [the roadmap](docs/ROADMAP.md) and [the Phase 7 audit](docs/audits/phase-7.md).

## Cost and privacy

All production dependencies are free and open source. The completed phases use no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
