"""Tests for the fresh-merge simulation used by pull-request CI."""

from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "simulate-fresh-merge.sh"


def test_fresh_merge_script_requires_base_ref() -> None:
    """The script should fail clearly instead of fetching an empty branch name."""
    result = subprocess.run(
        ["bash", str(SCRIPT)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode != 0
    assert "BASE_REF is required" in result.stderr
