"""Preserve the WordNet evidence of published views, independent of other sources."""

from contextlib import redirect_stdout
import importlib.util
from io import StringIO
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("build_views", Path("scripts/build-views.py"))
views = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(views)


class WordnetViewCoverage(unittest.TestCase):
    def test_required_views_use_checked_in_wordnet_evidence(self):
        self.assertEqual(views.main(["--check"]), 0)

    def test_missing_required_source_fails_before_build_or_replay(self):
        with TemporaryDirectory() as folder:
            cache = Path(folder)
            for arguments in ([], ["--check"]):
                with self.subTest(arguments=arguments):
                    output = StringIO()
                    with (
                        patch.object(views, "WORDNET_DIR", cache),
                        patch.object(views, "build", return_value=0) as build,
                        patch.object(views, "check", return_value=0) as check,
                        redirect_stdout(output),
                    ):
                        self.assertEqual(views.main(arguments), 1)
                    self.assertIn("landmark: missing required WordNet senses or LiNo capture", output.getvalue())
                    build.assert_not_called()
                    check.assert_not_called()


if __name__ == "__main__":
    unittest.main()
