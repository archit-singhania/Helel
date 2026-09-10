# P04 — Code intelligence

## Goal

Build local project understanding for deterministic navigation and later agent context retrieval.

## Delivered

1. Manifest, language, and framework detection
2. Bounded deterministic parsing for common Rust, TypeScript, JavaScript, and Python symbols
3. File, symbol, and identifier-reference indexes
4. Atomic versioned local persistence plus per-file updates after editor saves
5. Definition, reference, and visible workspace-symbol navigation
6. Ranked context retrieval capped by caller and hard limits
7. Workspace-boundary and round-trip tests
