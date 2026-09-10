# Phase 3 audit

Audit date: 2026-09-11

## Result

Phase 3 is complete. Helel can run local commands inside the selected workspace, stream their output, cancel an active process, inspect Git state, and apply or reverse a checked unified patch.

## Built

- Shell-free process execution with argument arrays, workspace working directory, and `HELEL_WORKSPACE`
- Typed stdout, stderr, and exit events from Rust to the desktop
- Active-process tracking and cancellation from the desktop UI
- Terminal command parsing with quoted argument support
- Git branch, porcelain status, and unstaged diff view
- Structured patch validation, application, and reversal through `git apply`
- Safe, modifying, and dangerous command classification
- Approval prompts for modifying or dangerous operations
- In-memory audit entries for process, cancellation, and patch decisions
- Project-tree refresh every 2.5 seconds for external filesystem changes

## Security boundaries

- Commands run directly without shell interpolation or expansion.
- Every process starts in the registered workspace.
- Known destructive commands and path-escaping read arguments are dangerous.
- Modifying and dangerous commands are rejected by the backend without approval.
- Git inspection and patch operations use fixed subcommands and arguments.
- Patches must pass a dry validation before mutation and can be reversed.

## Verification

`CARGO_NET_OFFLINE=true python3 scripts/verify.py` passed six Rust tests, seven frontend tests, strict lint and type checks, Python and contract checks, the production frontend build, and the native macOS Tauri debug build. The verified application is at `target/debug/helel-desktop`.

## Left for Phase 4+

- Project and framework detection
- Incremental parsing and a persistent symbol/reference index
- Definition, reference, and workspace-symbol navigation
- Ranked local context retrieval for later agent use
- Persistent audit history and richer terminal sessions with PTY support
- Automatic clean-buffer reload and conflict notices for external edits
- Windows and Linux native verification

## Known limits

- The desktop exposes one active process at a time.
- Audit records reset when the application exits.
- Commands are streamed child processes rather than an interactive PTY shell.
- External changes refresh the project tree; open editor buffers are kept intact to avoid overwriting unsaved work.

## Cost audit

The implementation uses Rust, Tauri, React, Monaco, the local operating system, and the installed Git executable. All production dependencies are free and open source. No hosted AI API, paid service, telemetry, cloud storage, or usage-priced infrastructure was added.
