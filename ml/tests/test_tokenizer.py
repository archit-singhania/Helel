from pathlib import Path
import tempfile
import unittest

from helel_ml.tokenizer import ByteBPETokenizer, SPECIAL_TOKENS


class TokenizerTest(unittest.TestCase):
    def test_training_is_deterministic_and_round_trips_unicode(self) -> None:
        corpus = ["fn greet(name: &str) -> String { name.to_owned() }\n", "café 東京 🚀\n"]
        first, first_report = ByteBPETokenizer.train(corpus, 300)
        second, second_report = ByteBPETokenizer.train(corpus, 300)
        self.assertEqual(first.merges, second.merges)
        self.assertEqual(first_report, second_report)
        for text in corpus:
            self.assertEqual(first.decode(first.encode(text, bos=True, eos=True)), text)
        self.assertGreater(first_report.bytes_per_token, 1.0)

    def test_serialization_preserves_ids_and_merges(self) -> None:
        tokenizer, _ = ByteBPETokenizer.train(["repeat repeat repeat\n"], 280)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tokenizer.json"
            tokenizer.save(path)
            loaded = ByteBPETokenizer.load(path)
            self.assertEqual(loaded.encode("repeat"), tokenizer.encode("repeat"))
            self.assertEqual(loaded.vocabulary_size, tokenizer.vocabulary_size)
            self.assertEqual(SPECIAL_TOKENS[3:], ("<fim_prefix>", "<fim_middle>", "<fim_suffix>"))


if __name__ == "__main__":
    unittest.main()
