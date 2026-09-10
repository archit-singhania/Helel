# Helel

Helel is an offline-first autonomous software engineering environment. The repository now contains the Phase 2 IDE core: a Tauri application with a local Monaco editor, secure workspace tree, tabs, file operations, search/replace, diffs, diagnostics, themes, and recent projects.

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

Phases 0 through 2 are complete. Local processes and Git begin in Phase 3. See [the roadmap](docs/ROADMAP.md) and [the Phase 2 audit](docs/audits/phase-2.md).

## Cost and privacy

All production dependencies are free and open source. Phase 0 uses no hosted AI API, telemetry, cloud storage, or paid service. Hardware, electricity, internet access, code signing, and optional app-store accounts are not included in that statement.

## License

Apache-2.0. See [LICENSE](LICENSE).
