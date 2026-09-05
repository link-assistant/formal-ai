"""Tests for scripts/create_github_release.py."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path


SCRIPT_PATH = (
    Path(__file__).resolve().parent.parent / "scripts" / "create_github_release.py"
)
spec = importlib.util.spec_from_file_location("create_github_release", SCRIPT_PATH)
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)  # type: ignore[union-attr]


def test_create_release_uses_tag_prefix_and_language_title(monkeypatch) -> None:
    """Release creation should separate tag format from display title."""
    commands = []

    def fake_run_command(cmd, check=True):
        commands.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(module, "run_command", fake_run_command)

    module.create_release(
        version="1.2.3",
        repository="owner/repo",
        release_notes="Release notes",
        prerelease=False,
        tag_prefix="python_v",
        language="Python",
    )

    assert commands == [
        [
            "gh",
            "release",
            "create",
            "python_v1.2.3",
            "--repo",
            "owner/repo",
            "--title",
            "[Python] 1.2.3",
            "--notes",
            "Release notes",
        ],
    ]


def test_create_release_caps_oversized_release_notes(monkeypatch) -> None:
    """Release creation should avoid sending oversized notes to GitHub."""
    commands = []
    oversized_notes = (
        "Important release summary\n\n"
        + ("Multi-byte change entry: café\n" * 3_000)
        + "\nTail marker that should be omitted"
    )

    def fake_run_command(cmd, check=True):
        commands.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(module, "run_command", fake_run_command)

    module.create_release(
        version="1.2.3",
        repository="owner/repo",
        release_notes=oversized_notes,
        prerelease=False,
        tag_prefix="python_v",
        language="Python",
    )

    release_notes = commands[0][commands[0].index("--notes") + 1]

    assert len(release_notes.encode("utf-8")) <= module.MAX_RELEASE_NOTES_BYTES
    assert release_notes.startswith("Important release summary")
    assert "Tail marker that should be omitted" not in release_notes
    assert "Release notes were truncated" in release_notes
    assert (
        "https://github.com/owner/repo/blob/python_v1.2.3/CHANGELOG.md" in release_notes
    )


def test_append_pypi_badge_if_missing_adds_static_version_badge() -> None:
    """Release notes should get a PyPI badge before gh creates the release."""
    body = module.append_pypi_badge_if_missing("Release notes", "1.2.3")

    assert "Release notes" in body
    assert "https://img.shields.io/badge/pypi-1.2.3-blue.svg" in body


def test_append_pypi_badge_if_missing_does_not_duplicate_badge() -> None:
    """Existing shields.io badges should be preserved without duplication."""
    existing = (
        "Release notes\n\n![PyPI](https://img.shields.io/badge/pypi-1.2.3-blue.svg)"
    )

    assert module.append_pypi_badge_if_missing(existing, "1.2.3") == existing
