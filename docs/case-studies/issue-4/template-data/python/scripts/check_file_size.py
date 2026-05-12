#!/usr/bin/env python3
"""Check for files exceeding the maximum allowed line count.

Exits with error code 1 if any files exceed the limit.
"""

from __future__ import annotations

import sys
from pathlib import Path

MAX_LINES = 1000
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
    return files


def count_lines(file_path: Path) -> int:
    """Count lines in a file.

    Args:
        file_path: Path to the file

    Returns:
        Number of lines
    """
    return len(file_path.read_text(encoding="utf-8").split("\n"))


def main() -> None:
    """Main function."""
    cwd = Path.cwd()
    print(f"\nChecking Python files for maximum {MAX_LINES} lines...\n")

    files = find_python_files(cwd, EXCLUDE_PATTERNS)
    violations = []

    for file in files:
        line_count = count_lines(file)
        if line_count > MAX_LINES:
            violations.append({"file": file.relative_to(cwd), "lines": line_count})

    if not violations:
        print("✓ All files are within the line limit\n")
        sys.exit(0)
    else:
        print("✗ Found files exceeding the line limit:\n")
        for violation in violations:
            print(
                f"  {violation['file']}: {violation['lines']} lines "
                f"(exceeds {MAX_LINES})"
            )
        print(f"\nPlease refactor these files to be under {MAX_LINES} lines\n")
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
