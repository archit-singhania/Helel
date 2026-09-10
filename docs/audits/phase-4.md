# Phase 4 audit

Audit date: 2026-09-11

## Result

Phase 4 is complete. Helel builds and persists a bounded local code index, detects the project shape, navigates indexed symbols, and retrieves ranked repository context.

## Built

- Language, manifest, and framework detection
- File metadata, symbol definition, and identifier reference indexes
- Deterministic parsing for common declarations in Rust, TypeScript, JavaScript, and Python
- Atomic, versioned `.helel/index.json` persistence
- Per-file index updates after editor saves
- Definition and reference commands with hard result caps
- Ranked line-level context retrieval with deterministic scoring and hard limits
- Agent-panel index summary and clickable workspace symbols

## Boundaries and known limits

- Full scans cap at 20,000 files and skip dependencies, build directories, VCS data, symlinks, binary files, and files above 2 MiB.
- Parsing is a deterministic lexical pass rather than a compiler-grade syntax tree, so complex declarations and semantic references can be missed.
- New, renamed, and deleted files trigger a rebuild on the next index request; editor saves update incrementally.
- JSON is the current portable persistence adapter. SQLite remains planned when scale and concurrency justify it.

## Verification

The completion gate covers index build, persistence round-trip, incremental file update, symbol lookup, reference lookup, and ranked context retrieval alongside the full repository suite.

## Cost audit

Indexing, persistence, and retrieval run locally with free and open-source libraries. No hosted search, embeddings API, database service, telemetry, or paid dependency was added.
