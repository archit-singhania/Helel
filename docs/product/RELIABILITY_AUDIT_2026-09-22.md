# Local reliability audit — 2026-09-22

Verdict: the local preview has working tested components, but smooth end-to-end desktop operation is not yet certified. Earlier claims that every button and native workflow worked exceeded the evidence. Browser-only navigation cannot validate Tauri IPC, native dialogs, microphone permissions, or process cancellation.

## Fixes in this audit

- Ignore dependency/build/runtime directories at any depth in native watcher events, including nested node_modules and Python caches. Limit each drain to 512 events so a continuous producer cannot monopolize a poll.
- Bound rendered terminal history by both 2,000 entries and 262,144 UTF-16 code units, including a single oversized output chunk.
- Delete semantic-index directory prefixes literally. SQL LIKE previously interpreted underscores and percent signs in folder names as wildcards and could delete unrelated index records.
- Add regressions for all three cases.

## Remaining findings

| Priority | Finding | Consequence and next work |
| --- | --- | --- |
| P1 | Native commands are synchronous, including watcher waits, indexing, inference, and transcription | Long operations can stall native command handling. Move blocking work off the event thread with explicit concurrency and workspace ownership controls; simply making every command async risks races. |
| P1 | Agent execution holds the session mutex while running a tool | Pause/cancel can wait behind a slow tool. Add cancellable job ownership and release the session lock during execution. |
| Fixed for ordinary filesystem changes; limitations remain | Cached indexes on reopen | Reopen compares file membership, sizes, and millisecond modification times and rebuilds both stores on mismatch. Same-size edits with preserved timestamps are not detected; reconciliation/build still runs synchronously. |
| Partially fixed | Project opening | Active workspace is separate from recent-project settings. Startup opens once; overlapping UI opens are rejected, dirty edits require confirmation, and terminal activity blocks switching. Broader in-flight operation/workspace ownership still needs backend enforcement and acceptance tests. |
| Fixed in code; performance acceptance pending | Directory-only events | Directory events trigger one bounded full index rebuild per batch, covering descendants. Large repositories can still stall during that synchronous rebuild. |
| Fixed September 23 | Editor save acknowledgement | Single-file and bulk saves acknowledge submitted content only; newer edits remain dirty. Bulk failures are caught and successful earlier saves remain acknowledged. Concurrent saves and workspace-switch races still need acceptance coverage. |
| Fixed in code September 23; native acceptance pending | Voice recorder lifecycle | Unmount stops tracks, disconnects nodes, closes the audio context, and clears the timer. Late permission/transcription results are ignored; duplicate permission requests are blocked. Real microphone lifecycle testing remains pending. |
| Product gate | Current learned model previously scored 0% tool accuracy on five held-out cases | No evidence of useful autonomous coding quality; no new model training or evaluation was performed in this audit. |

## Evidence and limits

Verification completed and reviewed on September 23: `python3 scripts/verify.py` passed. Results: 35 Rust tests, 10 frontend tests, and 25 Python tests; formatting, Clippy, frontend lint/type checking, contracts, release/performance checks, frontend build, and the native Tauri debug build passed. The build still warns about a roughly 3.60 MB minified main JavaScript chunk (938 KB gzip); startup performance needs measurement in the native app. Verification log: `/private/tmp/helel-audit-20260922.log` (temporary local evidence).

The standalone 30-case HelelBench run passed 30/30 with zero crashes. Its runner writes predefined expected files directly; it is a fixture test, not an autonomous planner, compiler, MCP, or desktop acceptance test. Its reported tool-call rate is hard-coded and must not be used as learned-agent evidence.

Repository verification includes Rust tests, frontend static-render/state tests, Python tests, lint, type checking, contract checks, build checks, and a Tauri debug build. It does not test every button in a native window, real microphone capture, multi-hour stability, clean-machine installation, or successful learned coding tasks. No fresh interactive native acceptance run was completed in this audit.

Run the app with `pnpm tauri dev` from the repository root. The next acceptance pass should use a disposable project and cover open/reopen, edits during saves, external directory changes, long command cancellation, model startup, MCP discovery/calls, voice permission denial, switching views during recording, patch rollback, and restart. Measure input responsiveness while those operations run.

September 23 follow-up verification: `python3 scripts/verify.py` passed after the save and voice fixes (35 Rust, 11 frontend, 25 Python tests; lint/type/contracts/build checks and native Tauri debug build). Log: `/private/tmp/helel-20260923.log`. The added regression covers edits made during a save. Voice lifecycle changes were reviewed and compiled; native microphone acceptance is still pending.

Second September 23 follow-up: removed the 250 ms native watcher wait. The UI now waits 250 ms between nonblocking polls and discards obsolete watcher results after a workspace change. This removes repeated idle blocking but does not resolve long indexing/inference/tool operations. Added a core regression for offline edits, additions, and deletions.

Second follow-up verification passed: `python3 scripts/verify.py`, with 36 Rust tests, 11 frontend tests, 25 Python tests, static checks, contracts, and native debug build. Log: `/private/tmp/helel-20260923-next.log`. No interactive native responsiveness measurement was performed.
