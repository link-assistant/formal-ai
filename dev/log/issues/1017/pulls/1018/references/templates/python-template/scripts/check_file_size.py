#!/usr/bin/env python3
"""Check for files exceeding the maximum allowed line count.

Exits with error code 1 if any files exceed the limit.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import sys
from pathlib import Path

MAX_LINES = 1000
WARN_LINES = 900
FILE_EXTENSIONS = [".py"]
EXCLUDE_PATTERNS = [
    "node_modules",
    ".venv",
    "venv",
    "env",
    "__pycache__",
    ".git",
    "build",
    "dist",
    ".eggs",
    "*.egg-info",
]


class LineStatus(Enum):
    """Line-count classification for a checked file."""

    OK = "ok"
    WARNING = "warning"
    VIOLATION = "violation"


@dataclass(frozen=True)
class Finding:
    """A file-size warning or violation."""

    file: Path
    lines: int


@dataclass(frozen=True)
class CheckResult:
    """Collected warnings and violations from a file-size check."""

    warnings: list[Finding]
    violations: list[Finding]


def should_exclude(path: Path, exclude_patterns: list[str]) -> bool:
    """Check if a path should be excluded.

    Args:
        path: Path to check
        exclude_patterns: List of patterns to exclude

    Returns:
        True if path should be excluded
    """
    path_str = str(path)
    return any(pattern in path_str for pattern in exclude_patterns)


def find_python_files(directory: Path, exclude_patterns: list[str]) -> list[Path]:
    """Recursively find all Python files in a directory.

    Args:
        directory: Directory to search
        exclude_patterns: Patterns to exclude

    Returns:
        List of file paths
    """
    files = []
    for path in directory.rglob("*"):
        if should_exclude(path, exclude_patterns):
            continue
        if path.is_file() and path.suffix in FILE_EXTENSIONS:
            files.append(path)
    return sorted(files)


def count_lines(file_path: Path) -> int:
    """Count lines in a file.

    Args:
        file_path: Path to the file

    Returns:
        Number of lines
    """
    return len(file_path.read_text(encoding="utf-8").split("\n"))


def classify_line_count(line_count: int) -> LineStatus:
    """Classify a file by line count.

    Args:
        line_count: Number of lines in the file

    Returns:
        File-size status
    """
    if line_count > MAX_LINES:
        return LineStatus.VIOLATION
    if line_count > WARN_LINES:
        return LineStatus.WARNING
    return LineStatus.OK


def check_directory(directory: Path) -> CheckResult:
    """Check Python files under a directory for warning and hard limits.

    Args:
        directory: Directory to scan

    Returns:
        Collected warnings and violations
    """
    root = directory.resolve()
    warnings = []
    violations = []

    files = find_python_files(root, EXCLUDE_PATTERNS)
    for file in files:
        line_count = count_lines(file)
        finding = Finding(file=file.relative_to(root), lines=line_count)
        status = classify_line_count(line_count)

        if status == LineStatus.VIOLATION:
            violations.append(finding)
        elif status == LineStatus.WARNING:
            warnings.append(finding)

    return CheckResult(warnings=warnings, violations=violations)


def escape_annotation_property(value: str) -> str:
    """Escape a GitHub Actions annotation property value."""
    return (
        value.replace("%", "%25")
        .replace("\r", "%0D")
        .replace("\n", "%0A")
        .replace(":", "%3A")
        .replace(",", "%2C")
    )


def escape_annotation_message(value: str) -> str:
    """Escape a GitHub Actions annotation message."""
    return value.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def warning_annotation(finding: Finding) -> str:
    """Build a GitHub Actions warning annotation for a near-limit file."""
    message = (
        f"File has {finding.lines} lines (approaching limit of {MAX_LINES}). "
        f"Consider extracting code to keep at or below {WARN_LINES} lines and "
        "prevent concurrent PR merge limit violations."
    )

    return (
        f"::warning file={escape_annotation_property(finding.file.as_posix())}::"
        f"{escape_annotation_message(message)}"
    )


def print_warnings(warnings: list[Finding]) -> None:
    """Print non-failing file-size warnings."""
    if not warnings:
        return

    for warning in warnings:
        print(warning_annotation(warning))
        print(
            f"WARNING: {warning.file} has {warning.lines} lines "
            f"(approaching limit of {MAX_LINES}, warning threshold: {WARN_LINES})"
        )

    print()
    print(f"The following files are approaching the {MAX_LINES} line limit:")
    for warning in warnings:
        print(f"  {warning.file}")
    print(
        "\nConsider extracting code to prevent concurrent PR merge limit violations.\n"
    )


def print_violations(violations: list[Finding]) -> None:
    """Print hard-limit file-size violations."""
    if not violations:
        return

    print("✗ Found files exceeding the line limit:\n")
    for violation in violations:
        print(f"  {violation.file}: {violation.lines} lines (exceeds {MAX_LINES})")
    print(f"\nPlease refactor these files to be under {MAX_LINES} lines\n")


def main() -> None:
    """Main function."""
    cwd = Path.cwd()
    print(
        f"\nChecking Python files for maximum {MAX_LINES} lines "
        f"(warning above {WARN_LINES})...\n"
    )

    result = check_directory(cwd)
    print_warnings(result.warnings)

    if not result.violations:
        print("✓ All files are within the hard line limit\n")
        sys.exit(0)

    print_violations(result.violations)
    sys.exit(1)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        if "DEBUG" in sys.modules:
            import traceback

            traceback.print_exc()
        sys.exit(1)
