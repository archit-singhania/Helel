# Core architecture

## Ownership

The Rust core owns local application services and coordinates filesystem, process, Git, search, patch, and persistence adapters. It does not own presentation, model training, or autonomous policy.

Core may depend on platform libraries and contract types. Desktop and agent components may depend on its public Rust traits and Tauri commands. Unit tests cover business rules; integration tests use temporary workspaces and real adapters where safe.

## Workspace service

The Phase 2 workspace service accepts normalized relative paths only. It canonicalizes existing paths, canonicalizes parents for new paths, rejects symlinks during traversal, skips dependency/build directories, limits editor files to 2 MiB of UTF-8 text, and refuses recursive directory deletion. Tauri holds exactly one active workspace and exposes typed commands to the desktop.

## Local system service

The Phase 3 system service launches programs directly, without a command shell, inside the active workspace. It streams standard output and error as typed desktop events, tracks running children by ID, supports cancellation, and records command decisions in an in-memory audit log. Git status and diffs use fixed argument lists. Structured patches must pass `git apply --check` before they can be applied or reversed.

## Repository intelligence

The Phase 4 indexer detects languages, manifests, and common frameworks; extracts deterministic source symbols and identifier references; and stores a versioned index in `.helel/index.json`. Full scans exclude dependencies, build output, VCS data, symlinks, binary files, and files above 2 MiB. Editor saves update only the affected file record. Definition, reference, workspace-symbol, and ranked context queries have explicit result limits.
