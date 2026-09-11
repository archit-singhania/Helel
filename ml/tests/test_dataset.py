import json
from pathlib import Path
import tempfile
import unittest

from helel_ml.dataset import DatasetBuilder, normalize, redact_secrets


FIXTURE_REGISTRY = Path(__file__).parents[1] / "fixtures" / "sources.json"


class DatasetPipelineTest(unittest.TestCase):
    def test_normalizes_and_redacts_credentials(self) -> None:
        self.assertEqual(normalize("cafe\u0301  \r\n"), "café\n")
        cleaned, count = redact_secrets("api_key = 'super-secret-value'\n")
        self.assertEqual(count, 1)
        self.assertNotIn("super-secret-value", cleaned)

    def test_build_is_reproducible_and_keeps_lineage(self) -> None:
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            left = DatasetBuilder(FIXTURE_REGISTRY, Path(first)).build()
            right = DatasetBuilder(FIXTURE_REGISTRY, Path(second)).build()
            self.assertEqual(left, right)
            self.assertEqual(left["report"]["accepted_documents"], 3)
            self.assertEqual(left["sources"][0]["license"], "Apache-2.0")
            for filename, checksum in left["checksums"].items():
                self.assertEqual(checksum, right["checksums"][filename])

    def test_rejects_unapproved_license(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "corpus").mkdir()
            registry = {"version": 1, "sources": [{"source_id": "bad", "path": "corpus", "license": "UNKNOWN", "origin": "test", "revision": "1"}]}
            (root / "sources.json").write_text(json.dumps(registry), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "not allowlisted"):
                DatasetBuilder(root / "sources.json", root / "output").load_registry()

    def test_removes_exact_and_near_duplicates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            corpus = root / "corpus"
            corpus.mkdir()
            base = "".join(f"value_{index} = calculate_total(values)\n" for index in range(30))
            (corpus / "one.py").write_text(base, encoding="utf-8")
            (corpus / "two.py").write_text(base, encoding="utf-8")
            (corpus / "three.py").write_text(base.replace("value_29", "result_29"), encoding="utf-8")
            registry = {"version": 1, "sources": [{"source_id": "dedup", "path": "corpus", "license": "MIT", "origin": "test", "revision": "1"}]}
            (root / "sources.json").write_text(json.dumps(registry), encoding="utf-8")
            _, report = DatasetBuilder(root / "sources.json", root / "output").ingest()
            self.assertEqual(report.exact_duplicates, 1)
            self.assertEqual(report.near_duplicates, 1)

    def test_split_assignment_is_stable_and_exclusive(self) -> None:
        assignments = {DatasetBuilder.split_for(f"{value:08x}" + "0" * 16) for value in range(1_000)}
        self.assertEqual(assignments, {"train", "validation", "test"})


if __name__ == "__main__":
    unittest.main()
