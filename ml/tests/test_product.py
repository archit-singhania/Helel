from pathlib import Path
import tempfile
import unittest

from helel_ml.product import DEFAULT_MIXTURE, HELEL_46M, curriculum_at, memory_profile, release_candidate_manifest, validate_mixture


class ProductModelTest(unittest.TestCase):
    def test_46m_configuration_mixture_and_curriculum(self) -> None:
        self.assertEqual(HELEL_46M.parameter_count(), 45_954_560)
        self.assertEqual(validate_mixture(DEFAULT_MIXTURE), DEFAULT_MIXTURE)
        self.assertEqual(curriculum_at(0.05).context_length, 512)
        self.assertEqual(curriculum_at(0.8).fim_rate, 0.70)
        self.assertLess(memory_profile()["int4_mib"], memory_profile()["fp16_mib"])

    def test_release_candidate_requires_artifacts_and_evaluations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkpoint, tokenizer = root / "model.safetensors", root / "tokenizer.json"
            checkpoint.write_bytes(b"weights")
            tokenizer.write_text("{}", encoding="utf-8")
            evaluations = {name: {"exact_match": 0.0} for name in ("completion", "repair", "fim", "repository_context")}
            manifest = release_candidate_manifest(checkpoint, tokenizer, evaluations, root / "release.json")
            self.assertEqual(manifest["model"], "Helel-46M")
            with self.assertRaisesRegex(ValueError, "missing evaluations"):
                release_candidate_manifest(checkpoint, tokenizer, {}, root / "bad.json")


if __name__ == "__main__":
    unittest.main()
