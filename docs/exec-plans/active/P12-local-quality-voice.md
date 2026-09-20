# Phase 12 — local model quality and voice

## Objective

Improve the useful ceiling of Helel's small local model with deterministic, license-gated multilingual data preparation and add an entirely local voice path for creating tasks and hearing agent results.

## Constraints

- No hosted inference, speech, telemetry, scraping, or paid API.
- Training sources must be local, explicitly registered, revision-pinned, and license-allowlisted.
- Audio must remain on the device and temporary recordings must be deleted after transcription.
- Voice-triggered text must remain editable before it starts an agent task.
- Existing workspace, approval, audit, and process boundaries remain authoritative.

## Slices

1. Expand deterministic language detection and report per-language coverage.
2. Add seeded temperature-balanced multilingual sampling for tokenizer/training inputs.
3. Add multilingual dataset and sampling regression tests.
4. Add bounded local WAV capture and a configurable `whisper.cpp` CLI adapter.
5. Add local macOS speech output, audit records, and desktop controls.
6. Run the fixture pipeline, smoke training, learned-model evaluations, and full verification.
7. Build a legally approved multilingual corpus and train 22M/46M release candidates.

## Acceptance

- At least 30 common source/document formats are classified deterministically.
- Dataset manifests expose language counts and training reports expose sampled counts.
- No raw audio persists after successful or failed transcription.
- Recording is capped at 30 seconds and native audio input is size-bounded.
- Transcription invokes only an explicitly approved local executable with fixed arguments.
- Reduced or absent voice tooling leaves text input fully functional.
- `python3 scripts/verify.py` passes.

## Known external gate

Useful release weights require owner-approved local sources and enough Apple Silicon training time. Infrastructure and smoke training can be completed without this corpus; useful quality cannot be claimed until held-out learned-agent metrics pass.

## Implementation result — 2026-09-15

- Slices 1–6 are complete. The dataset classifies more than 40 formats, reports coverage, balances languages deterministically, and has regression tests.
- A 100-document Apache-2.0 snapshot from immutable revision `ad24e3561eada008f6714233e47df347d5d39940` validated the owned-data path. It is an infrastructure fixture, not a broad production corpus.
- The generated supervised set contains 100 canonical proposal traces across 20 language stacks. A 22.8M-parameter checkpoint was trained locally on MLX/Metal and evaluated on prompts with unseen symbols, paths, and wording.
- Checkpoint `helel-22m-agent-sft-v3` scored a 40% valid-proposal rate and 0% tool accuracy on the leakage-free five-case evaluation. It failed selection and is not used as the default desktop model.
- Offline voice is implemented with bounded Web Audio capture, local `whisper.cpp`, automatic language detection, editable transcripts, macOS speech output, audit records, and temporary-file deletion. The local multilingual base model checksum is `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe`.
- A synthetic 3.3-second speech test was transcribed correctly on Apple M3 using local Metal inference. The repository verification gate passed in full.
- Slice 7 remains open: approve a materially larger multilingual source registry, train 22M candidates until held-out proposal/tool metrics pass, and only then spend compute on 46M and quantization comparisons.
