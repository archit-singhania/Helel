from pathlib import Path
import tempfile
import unittest

from helel_ml.agent_model import parse_proposal
from helel_ml.tool_traces import LANGUAGES, generate, trace, training_records


class ToolTraceTest(unittest.TestCase):
    def test_generates_deterministic_multilingual_traces(self) -> None:
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            left = generate(Path(first))
            right = generate(Path(second))
            self.assertEqual(left, right)
            self.assertGreaterEqual(len(left["languages"]), 20)
            self.assertEqual(left["files"], right["files"])

    def test_every_embedded_action_matches_the_runtime_contract(self) -> None:
        for language, (path, symbol, command) in LANGUAGES.items():
            text = trace(language, path, symbol, command)
            for block in text.split("ASSISTANT_ACTION:\n")[1:]:
                parse_proposal(block.splitlines()[0])

    def test_training_records_pair_exact_runtime_prompts_with_actions(self) -> None:
        records = training_records()
        self.assertEqual(len(records), len(LANGUAGES) * 5)
        for record in records:
            self.assertIn("ASSISTANT_ACTION:\n", record["text"])
            parse_proposal(record["text"].split("ASSISTANT_ACTION:\n", 1)[1].strip())


if __name__ == "__main__":
    unittest.main()
