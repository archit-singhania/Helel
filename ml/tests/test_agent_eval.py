import unittest

from helel_ml.agent_eval import score_outputs


class AgentEvaluationTest(unittest.TestCase):
    def test_scores_contract_validity_separately_from_tool_choice(self) -> None:
        outputs = [
            '{"rationale":"inspect","tool":"inspectGit","arguments":{}}',
            "not json",
            '{"rationale":"read","tool":"readFile","arguments":{"path":"a.py"}}',
        ]
        report = score_outputs(outputs, ["inspectGit", "readFile", "searchCode"])
        self.assertAlmostEqual(report["valid_proposal_rate"], 2 / 3)
        self.assertAlmostEqual(report["tool_accuracy"], 1 / 3)


if __name__ == "__main__":
    unittest.main()
