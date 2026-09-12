# Core architecture

## Ownership

The Rust core owns local application services and coordinates filesystem, process, Git, search, patch, and persistence adapters. It does not own presentation, model training, or autonomous policy.

Core may depend on platform libraries and contract types. Desktop and agent components may depend on its public Rust traits and Tauri commands. Unit tests cover business rules; integration tests use temporary workspaces and real adapters where safe.

## Workspace service

The Phase 2 workspace service accepts normalized relative paths only. It canonicalizes existing paths, canonicalizes parents for new paths, rejects symlinks during traversal, skips dependency/build directories, limits editor files to 2 MiB of UTF-8 text, and refuses recursive directory deletion. Tauri holds exactly one active workspace and exposes typed commands to the desktop.

## Local system service

The system service launches programs directly inside the active workspace. Interactive desktop processes use a native PTY with bounded output, input, resize, status, and cancellation. Command and patch decisions are appended to a durable hash-chained audit ledger. Git status, staged and unstaged diffs, explicit staging, and commits use fixed argument lists. Structured patches pass `git apply --check`, snapshot only their declared target paths, apply atomically, and retain a rollback checkpoint that preserves unrelated changes.

## Repository intelligence

The repository service retains the bounded lexical context index for planner retrieval and builds `.helel/index.sqlite` from Tree-sitter syntax trees for Rust, TypeScript/TSX, Python, and Java. SQLite stores source files, exact definitions, references, and an FTS5 context table. Full scans exclude dependencies, build output, VCS data, symlinks, binary files, and files above 2 MiB. Native workspace events replace periodic tree polling; the UI reports conflicts when an externally changed file has an unsaved editor buffer.
