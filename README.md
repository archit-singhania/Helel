# Helel

Helel is an offline-first autonomous software engineering environment. The repository now contains the Phase 3 local system engine: a Tauri application with a local Monaco editor, secure workspace operations, streamed and cancellable processes, Git inspection, structured patches, and command auditing.

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

Phases 0 through 3 are complete. Code intelligence begins in Phase 4. See [the roadmap](docs/ROADMAP.md) and [the Phase 3 audit](docs/audits/phase-3.md).

## Cost and privacy

All production dependencies are free and open source. The completed phases use no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
