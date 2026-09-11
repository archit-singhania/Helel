# Helel

Helel v0.1 is an offline-first local software engineering environment. It combines a native Tauri IDE, secure and auditable tools, repository intelligence, deterministic agent sessions, reproducible model tooling, and a constrained local inference protocol.

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

The v0.1 engineering scope through Phase 10 is implemented. Helel-46M training remains gated on an approved production corpus and compute run; no useful weights are bundled. See [the roadmap](docs/ROADMAP.md), [Phase 8](docs/audits/phase-8.md), [Phase 9](docs/audits/phase-9.md), and [Phase 10](docs/audits/phase-10.md).

## Cost and privacy

All production dependencies are free and open source. The completed phases use no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
