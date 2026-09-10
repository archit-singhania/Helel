import unittest

import helel_ml


class PackageTest(unittest.TestCase):
    def test_version_is_exposed(self) -> None:
        self.assertEqual(helel_ml.__version__, "0.1.0")


if __name__ == "__main__":
    unittest.main()
