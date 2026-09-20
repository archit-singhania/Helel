from dataclasses import asdict
from hashlib import sha256
import json
from pathlib import Path
import random
import tempfile
import unittest

from helel_ml.model import HELEL_22M, ModelConfig
from helel_ml.evaluation import EvaluationCase, exact_match
from helel_ml.tokenizer import ByteBPETokenizer
from helel_ml.training import TrainingConfig, fill_in_middle, language_balanced_texts, learning_rate, packed_sequences, verify_checkpoint


class ModelPipelineTest(unittest.TestCase):
    def test_22m_configuration_and_round_trip(self) -> None:
        HELEL_22M.validate()
        self.assertEqual(HELEL_22M.parameter_count(), 22_816_128)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config.json"
            HELEL_22M.save(path)
            self.assertEqual(ModelConfig.load(path), HELEL_22M)

    def test_fim_packing_and_schedule_are_deterministic(self) -> None:
        self.assertEqual(fill_in_middle(list(range(20)), random.Random(9)), fill_in_middle(list(range(20)), random.Random(9)))
        tokenizer, _ = ByteBPETokenizer.train(["abcdef " * 30], 280)
        left = packed_sequences(["abcdef " * 30], tokenizer, 15, 7, 1.0)
        right = packed_sequences(["abcdef " * 30], tokenizer, 15, 7, 1.0)
        self.assertEqual(left, right)
        self.assertTrue(all(len(item) == 16 for item in left))
        config = TrainingConfig(warmup_steps=2, maximum_steps=10)
        self.assertLess(learning_rate(0, config), learning_rate(1, config))
        self.assertGreater(learning_rate(2, config), learning_rate(10, config))

    def test_checkpoint_detects_config_mismatch_and_corruption(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            weights = root / "model.safetensors"
            optimizer = root / "optimizer.safetensors"
            weights.write_bytes(b"weights")
            optimizer.write_bytes(b"optimizer")
            metadata = {"version": 1, "step": 2, "loss": 1.0, "model_config": asdict(HELEL_22M), "training_config": asdict(TrainingConfig()), "files": {weights.name: sha256(weights.read_bytes()).hexdigest(), optimizer.name: sha256(optimizer.read_bytes()).hexdigest()}}
            (root / "checkpoint.json").write_text(json.dumps(metadata), encoding="utf-8")
            self.assertEqual(verify_checkpoint(root, HELEL_22M)["step"], 2)
            with self.assertRaisesRegex(ValueError, "mismatch"):
                verify_checkpoint(root, ModelConfig(layers=2))
            weights.write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "corruption"):
                verify_checkpoint(root, HELEL_22M)

    def test_evaluation_contract_covers_product_tasks(self) -> None:
        cases = [EvaluationCase("complete-1", "completion", "def add", "(a, b):"), EvaluationCase("repair-1", "repair", "fix: retrun", "return"), EvaluationCase("fim-1", "fim", "<fim_prefix>a<fim_suffix>c<fim_middle>", "b"), EvaluationCase("memorize-1", "memorization", "alpha", "beta")]
        for case in cases:
            case.validate()
        self.assertTrue(exact_match("return value\n", "return value"))
        with self.assertRaisesRegex(ValueError, "unsupported"):
            EvaluationCase("bad", "other", "x", "y").validate()

    def test_language_sampling_is_deterministic_and_reduces_dominance(self) -> None:
        records = [{"language": "python", "text": f"py-{index}"} for index in range(90)] + [{"language": "rust", "text": f"rs-{index}"} for index in range(10)]
        first, counts = language_balanced_texts(records, seed=7, temperature=0.5)
        second, repeated = language_balanced_texts(records, seed=7, temperature=0.5)
        self.assertEqual(first, second)
        self.assertEqual(counts, repeated)
        self.assertGreater(counts["rust"], 10)
        self.assertEqual(sum(counts.values()), len(records))


if __name__ == "__main__":
    unittest.main()
