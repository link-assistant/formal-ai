"""Regression tests for project package metadata."""

from __future__ import annotations

from pathlib import Path

from packaging.requirements import Requirement

ROOT = Path(__file__).resolve().parents[1]


def dev_requirement(name: str) -> Requirement:
    """Return one requirement from the project's ``dev`` dependency group."""
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    dev_group = pyproject.split("dev = [", maxsplit=1)[1].split("]", maxsplit=1)[0]
    requirement_lines = (line.strip().strip('",') for line in dev_group.splitlines())
    requirements = (Requirement(line) for line in requirement_lines if line)
    return next(requirement for requirement in requirements if requirement.name == name)


def test_mypy_dependency_supports_configured_python_target() -> None:
    """The dev extra must not install mypy versions that reject Python 3.9."""
    mypy = dev_requirement("mypy")

    assert "1.13.0" in mypy.specifier
    assert "2.0" not in mypy.specifier
