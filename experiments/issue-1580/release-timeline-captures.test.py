"""Verify that the complete historical timeline can be replayed offline."""

import importlib.util
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

sys.path.insert(0, "scripts")
from wikidata_lino_codec import json_cache_file

SPEC = importlib.util.spec_from_file_location(
    "ground_release_timelines", Path("scripts/ground-release-timelines.py")
)
replay = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(replay)


class ReleaseTimelineCaptures(unittest.TestCase):
    def test_every_replay_entity_and_canonical_twin_is_beside_the_query(self):
        query = replay.FIXTURE.parent / "historical/spider-man-title-role-films.json"
        entity_dir = query.with_name("wikidata-entities")
        rows = replay.query_rows(query)
        self.assertEqual(len(rows), 12)
        self.assertEqual({path.stem for path in entity_dir.glob("*.json")}, {qid for qid, _ in rows})
        documents = [query] + [entity_dir / f"{qid}.json" for qid, _ in rows]
        for path in documents:
            with self.subTest(capture=path):
                payload = json.loads(path.read_bytes())
                self.assertEqual(
                    path.with_suffix(".lino").read_text(encoding="utf-8"),
                    json_cache_file(path.stem, payload),
                )
        with patch.object(replay, "get", side_effect=AssertionError("capture must be offline")):
            self.assertEqual(replay.main(["ground-release-timelines.py", "--check"]), 0)

    def test_missing_offline_capture_fails_before_network_use(self):
        with TemporaryDirectory() as folder:
            entity_dir = Path(folder)
            with patch.object(replay, "get", side_effect=AssertionError("capture must be offline")):
                with self.assertRaises(FileNotFoundError) as failure:
                    replay.fetch_entity("Q182212", entity_dir, offline=True)
        self.assertEqual(Path(failure.exception.filename).name, "Q182212.json")


if __name__ == "__main__":
    unittest.main()
