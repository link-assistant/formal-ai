#!/usr/bin/env python3
"""Small real-ZIP regressions for the shared artifact transfer adapter."""
import importlib.util
from pathlib import Path
import stat
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]


class Extraction(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location("artifact_extraction", ROOT / "scripts/extract-artifacts.py")
        cls.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.module)

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="artifact-extraction-")
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.raw = self.directory / "raw"
        self.raw.mkdir()
        self.destination = self.directory / "destination"

    def archive(self, location="artifact", entries=None):
        path = self.raw / location
        path.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(path, "w") as archive:
            for member, data in entries or [("nested/check.bin", bytes(range(256)))]:
                archive.writestr(member, data)
        return path

    def test_single_archive_bytes_and_existing_regular_overwrite(self):
        self.archive()
        self.destination.mkdir()
        (self.destination / "unrelated").write_bytes(b"retained")
        (self.destination / "nested").mkdir()
        (self.destination / "nested/check.bin").write_bytes(b"old cached executable")
        self.module.extract_artifacts(self.raw, self.destination, False)
        self.assertEqual((self.destination / "nested/check.bin").read_bytes(), bytes(range(256)))
        self.assertEqual((self.destination / "unrelated").read_bytes(), b"retained")

    def test_multiple_named_and_merged_layouts(self):
        self.archive("one/artifact", [("first.txt", b"one")])
        self.archive("two/artifact", [("second.txt", b"two")])
        self.module.extract_artifacts(self.raw, self.destination, False)
        self.assertEqual((self.destination / "one/first.txt").read_bytes(), b"one")
        self.assertEqual((self.destination / "two/second.txt").read_bytes(), b"two")
        merged = self.directory / "merged"
        self.module.extract_artifacts(self.raw, merged, True)
        self.assertEqual((merged / "first.txt").read_bytes(), b"one")
        self.assertEqual((merged / "second.txt").read_bytes(), b"two")

    def test_unsafe_paths_reject_the_whole_member_plan(self):
        for member in ["../escape", "/absolute", "C:/drive", "nested/../../escape", "back\\slash", "a/./alias"]:
            with self.subTest(member=member):
                self.archive(entries=[("safe", b"safe"), (member, b"bad")])
                with self.assertRaises(ValueError):
                    self.module.extract_artifacts(self.raw, self.destination, False)
                self.assertFalse((self.destination / "safe").exists())

    def test_symlink_and_special_members_reject_before_writes(self):
        for mode in [stat.S_IFLNK, stat.S_IFIFO, stat.S_IFCHR]:
            with self.subTest(mode=mode):
                member = zipfile.ZipInfo("special")
                member.create_system = 3
                member.external_attr = (mode | 0o777) << 16
                self.archive(entries=[("safe", b"safe"), (member, b"target")])
                with self.assertRaises(ValueError):
                    self.module.extract_artifacts(self.raw, self.destination, False)
                self.assertFalse((self.destination / "safe").exists())

    def test_existing_member_or_parent_symlinks_reject(self):
        self.archive()
        outside = self.directory / "outside"
        outside.mkdir()
        self.destination.mkdir()
        (self.destination / "nested").symlink_to(outside, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.module.extract_artifacts(self.raw, self.destination, False)
        self.assertFalse((outside / "check.bin").exists())
        (self.destination / "nested").unlink()
        (self.destination / "nested").mkdir()
        (self.destination / "nested/check.bin").symlink_to(outside / "check.bin")
        with self.assertRaises(ValueError):
            self.module.extract_artifacts(self.raw, self.destination, False)
        self.assertFalse((outside / "check.bin").exists())

    def test_duplicate_and_file_directory_conflicts_reject(self):
        for entries in [[("same", b"one"), ("SAME", b"two")], [("nested", b"one"), ("nested/file", b"two")]]:
            self.archive(entries=entries)
            with self.assertRaises(ValueError):
                self.module.extract_artifacts(self.raw, self.destination, False)
            self.assertFalse(self.destination.exists())

    def test_missing_corrupt_raw_and_member_crc_preserve_existing_file(self):
        with self.assertRaises(ValueError):
            self.module.extract_artifacts(self.raw, self.destination, False)
        path = self.raw / "artifact"
        path.write_bytes(b"not an archive")
        with self.assertRaises(zipfile.BadZipFile):
            self.module.extract_artifacts(self.raw, self.destination, False)
        with zipfile.ZipFile(path, "w"):
            pass
        with self.assertRaises(ValueError):
            self.module.extract_artifacts(self.raw, self.destination, False)
        self.assertFalse(self.destination.exists())
        path = self.archive(entries=[("file", b"unique fixture bytes")])
        data = path.read_bytes().replace(b"unique fixture bytes", b"corrupt fixture byte")
        self.assertEqual(len(data), path.stat().st_size)
        path.write_bytes(data)
        self.destination.mkdir()
        (self.destination / "file").write_bytes(b"old cached executable")
        with self.assertRaises(zipfile.BadZipFile):
            self.module.extract_artifacts(self.raw, self.destination, False)
        self.assertEqual((self.destination / "file").read_bytes(), b"old cached executable")
        self.assertEqual([p.name for p in self.destination.iterdir()], ["file"])


if __name__ == "__main__":
    unittest.main()
