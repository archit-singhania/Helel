# Running Helel locally

## One-time preparation

From the repository root:

```sh
corepack enable
pnpm install --offline
python3 scripts/setup_local.py --train-smoke
python3 scripts/verify.py
```

The setup command creates `.venv/bin/helel-inference` and a small smoke checkpoint under `ml/checkpoints/smoke`. Both stay local and are ignored by Git. The smoke checkpoint proves training, artifact loading, Metal inference, and process supervision; it is deliberately too small to plan useful coding tasks.

## Start the real desktop app

```sh
pnpm tauri dev
```

Expected behavior:

1. Choose **Open project** and select a disposable Git repository.
2. Helel builds both its UI index and `.helel/index.sqlite` using Tree-sitter.
3. External file changes update the tree through the native watcher. Unsaved editor conflicts appear in the status bar.
4. Enter a direct command in Terminal. Helel shows the exact risk prompt, starts it in a PTY, streams output, accepts input, and allows cancellation.
5. Open Settings, choose **Use generated smoke model**, then **Start local model**. The status bar should report that the runtime is healthy.
6. For useful agent work, select a trained compatible checkpoint instead of the smoke model. Open Agent, enter an objective, and press **Continue**. Each model proposal is decoded in Rust. Read-only actions execute directly; process and patch actions require approval.
7. A successful patch creates `.helel/checkpoint.json`. Use **Source control → Rollback last task** to restore only task-owned paths.

## Train owned weights

Review and copy `ml/local-source.example.json`, set every source to an exact immutable revision, and add only material you own or are licensed to use. Then run:

```sh
PYTHONPATH=ml .venv/bin/python -m helel_ml.pipeline --registry /absolute/path/to/approved-sources.json --output ml/data/approved --vocab-size 4096
PYTHONPATH=ml .venv/bin/python -m helel_ml.train_cli --dataset ml/data/approved/train.jsonl --tokenizer ml/data/approved/tokenizer.json --output ml/checkpoints/helel-22m --model 22m --steps 1000 --sequence-length 1023
```

Inspect the held-out results before increasing training or starting 46M. A 46M run needs an 8,192-token tokenizer and `--model 46m`. Training uses local MLX/Metal and makes no hosted inference calls.

## MCP

Workspace MCP configuration is stored at `.helel/mcp.json`:

```json
[
  { "name": "local-tools", "program": "/absolute/path/to/server", "args": ["--stdio"], "enabled": true }
]
```

The Rust client uses local stdio, performs MCP 2025-06-18 initialization, discovers tools with `tools/list`, and routes calls through per-action approval. Network MCP transports are not enabled.

## Local test order

Use a disposable repository or committed branch. Test opening/editing, an external edit, PTY input/cancel, a patch and rollback, app restart/audit history, smoke-model health, a trained-model read action, a trained-model edit action, failing validation/retry, and MCP discovery/call. Record failures as HelelBench cases before testing valuable workspaces.
