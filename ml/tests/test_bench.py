import importlib.util, json, pathlib, tempfile, unittest

ROOT=pathlib.Path(__file__).resolve().parents[2]
SPEC=importlib.util.spec_from_file_location("helel_bench", ROOT/"scripts/helel_bench.py")
BENCH=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(BENCH)

class BenchTests(unittest.TestCase):
    def test_catalog_has_required_distribution_and_runs(self):
        catalog=json.loads((ROOT/"benchmarks/tasks.json").read_text())
        counts={name:sum(t["category"]==name for t in catalog["tasks"]) for name in ("edit","repair","test","refactor","scaffold")}
        self.assertEqual(counts, {"edit":10,"repair":5,"test":5,"refactor":5,"scaffold":5})
        report=BENCH.run(ROOT/"benchmarks/tasks.json")
        self.assertEqual(report["summary"]["passed"],30)
        self.assertIsNone(report["learnedModel"])

    def test_rejects_duplicate_or_short_catalog(self):
        with tempfile.TemporaryDirectory() as folder:
            path=pathlib.Path(folder)/"tasks.json"; path.write_text('{"tasks":[{"id":"x","category":"edit"}]}')
            with self.assertRaises(ValueError): BENCH.run(path)
