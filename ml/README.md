# Helel model workspace

Phase 6 provides an offline dataset and tokenizer pipeline using only Python's standard library.

Run the committed fixture pipeline:

```sh
PYTHONPATH=ml python3 -m helel_ml.pipeline \
  --registry ml/fixtures/sources.json \
  --output ml/data/fixture \
  --vocab-size 512
```

Production source registries use the same format. Every source requires a stable ID, local path, allowlisted SPDX license, origin, immutable revision, and optional include globs. Review provenance and license compatibility before adding a source. The pipeline never downloads material.

Generated outputs contain three JSONL splits, `manifest.json`, `tokenizer.json`, and `tokenizer-report.json`. `ml/data/` is ignored because derived corpora may be large or redistributable only under their source terms.

Phase 7 adds the `ml/configs/helel-22m.json` validation model and an MLX training path for Apple silicon. Install the optional, free training dependency with `pip install -e '.[training]'`, then run `PYTHONPATH=ml python3 -m helel_ml.smoke --output ml/checkpoints/smoke --steps 2`. Checkpoints and generated datasets are ignored by Git.
