# Core architecture

## Ownership

The Rust core owns local application services and coordinates filesystem, process, Git, search, patch, and persistence adapters. It does not own presentation, model training, or autonomous policy.

Core may depend on platform libraries and contract types. Desktop and agent components may depend on its public Rust traits and Tauri commands. Unit tests cover business rules; integration tests use temporary workspaces and real adapters where safe.
