# Core architecture

## Ownership

The Rust core owns local application services and coordinates filesystem, process, Git, search, patch, and persistence adapters. It does not own presentation, model training, or autonomous policy.

Core may depend on platform libraries and contract types. Desktop and agent components may depend on its public Rust traits and Tauri commands. Unit tests cover business rules; integration tests use temporary workspaces and real adapters where safe.

## Workspace service

The Phase 2 workspace service accepts normalized relative paths only. It canonicalizes existing paths, canonicalizes parents for new paths, rejects symlinks during traversal, skips dependency/build directories, limits editor files to 2 MiB of UTF-8 text, and refuses recursive directory deletion. Tauri holds exactly one active workspace and exposes typed commands to the desktop.
