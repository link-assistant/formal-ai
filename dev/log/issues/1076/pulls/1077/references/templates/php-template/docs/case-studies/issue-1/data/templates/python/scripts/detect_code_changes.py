#!/usr/bin/env python3
"""
Detect code changes for CI/CD pipeline.

This script detects what types of files have changed between two commits
and outputs the results for use in GitHub Actions workflow conditions.

Key behavior:
- For PRs: compares PR head against base branch
- For pushes: compares HEAD against HEAD^
- Excludes certain folders and file types from "code changes" detection

Excluded from code changes (don't require changelog fragments):
- Markdown files (*.md) in any folder
- changelog.d/ folder (changelog metadata)
- docs/ folder (documentation)
- experiments/ folder (experimental scripts)
- examples/ folder (example scripts)

Usage:
    python scripts/detect_code_changes.py

Environment variables (set by GitHub Actions):
    - GITHUB_EVENT_NAME: 'pull_request' or 'push'
    - GITHUB_BASE_SHA: Base commit SHA for PR
    - GITHUB_HEAD_SHA: Head commit SHA for PR

Outputs (written to GITHUB_OUTPUT):
    - py-changed: 'true' if any .py files changed
    - tests-changed: 'true' if any tests/ files changed
    - package-changed: 'true' if pyproject.toml changed
    - docs-changed: 'true' if any .md files changed
    - workflow-changed: 'true' if any .github/workflows/ files changed
    - any-code-changed: 'true' if any code files changed (excludes docs, changelogs, experiments, examples)
"""

from __future__ import annotations

import os
import subprocess
import sys


def exec_command(command: str) -> str:
    """Execute a shell command and return trimmed output."""
    try:
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error executing command: {command}", file=sys.stderr)
        print(f"stderr: {e.stderr}", file=sys.stderr)
        return ""


def set_output(name: str, value: str) -> None:
    """Write output to GitHub Actions output file."""
    output_file = os.environ.get("GITHUB_OUTPUT")
    if output_file:
        with open(output_file, "a") as f:
            f.write(f"{name}={value}\n")
    print(f"{name}={value}")


def get_changed_files() -> list[str]:
    """Get the list of changed files between two commits."""
    event_name = os.environ.get("GITHUB_EVENT_NAME", "local")

    if event_name == "pull_request":
        base_sha = os.environ.get("GITHUB_BASE_SHA")
        head_sha = os.environ.get("GITHUB_HEAD_SHA")

        if base_sha and head_sha:
            print(f"Comparing PR: {base_sha}...{head_sha}")
            try:
                # Ensure we have the base commit
                try:
                    subprocess.run(
                        f"git cat-file -e {base_sha}",
                        shell=True,
                        check=True,
                        capture_output=True,
                    )
                except subprocess.CalledProcessError:
                    print("Base commit not available locally, attempting fetch...")
                    subprocess.run(
                        f"git fetch origin {base_sha}",
                        shell=True,
                        check=False,
                    )

                output = exec_command(f"git diff --name-only {base_sha} {head_sha}")
                if output:
                    return [f for f in output.split("\n") if f]
            except Exception as e:
                print(f"Git diff failed: {e}", file=sys.stderr)

    # For push events or fallback
    print("Comparing HEAD^ to HEAD")
    try:
        output = exec_command("git diff --name-only HEAD^ HEAD")
        if output:
            return [f for f in output.split("\n") if f]
    except Exception:
        # If HEAD^ doesn't exist (first commit), list all files in HEAD
        print("HEAD^ not available, listing all files in HEAD")
        output = exec_command("git ls-tree --name-only -r HEAD")
        if output:
            return [f for f in output.split("\n") if f]

    return []


def is_excluded_from_code_changes(file_path: str) -> bool:
    """Check if a file should be excluded from code changes detection."""
    # Exclude markdown files in any folder
    if file_path.endswith(".md"):
        return True

    # Exclude specific folders from code changes
    excluded_folders = ["changelog.d/", "docs/", "experiments/", "examples/"]

    for folder in excluded_folders:
        if file_path.startswith(folder):
            return True

    return False


def detect_changes() -> None:
    """Main function to detect changes."""
    print("Detecting file changes for CI/CD...\n")

    changed_files = get_changed_files()

    print("Changed files:")
    if not changed_files:
        print("  (none)")
    else:
        for file in changed_files:
            print(f"  {file}")
    print()

    # Detect .py file changes
    py_changed = any(f.endswith(".py") for f in changed_files)
    set_output("py-changed", "true" if py_changed else "false")

    # Detect tests/ changes
    tests_changed = any(f.startswith("tests/") for f in changed_files)
    set_output("tests-changed", "true" if tests_changed else "false")

    # Detect pyproject.toml changes
    package_changed = "pyproject.toml" in changed_files
    set_output("package-changed", "true" if package_changed else "false")

    # Detect documentation changes (any .md file)
    docs_changed = any(f.endswith(".md") for f in changed_files)
    set_output("docs-changed", "true" if docs_changed else "false")

    # Detect workflow changes
    workflow_changed = any(f.startswith(".github/workflows/") for f in changed_files)
    set_output("workflow-changed", "true" if workflow_changed else "false")

    # Detect code changes (excluding docs, changelogs, experiments, examples folders, and markdown files)
    code_changed_files = [
        f for f in changed_files if not is_excluded_from_code_changes(f)
    ]

    print("\nFiles considered as code changes:")
    if not code_changed_files:
        print("  (none)")
    else:
        for file in code_changed_files:
            print(f"  {file}")
    print()

    # Check if any code files changed (.py, .toml, .yml, .yaml, or workflow files)
    import re

    code_pattern = re.compile(r"\.(py|toml|yml|yaml)$|\.github/workflows/")
    code_changed = any(code_pattern.search(f) for f in code_changed_files)
    set_output("any-code-changed", "true" if code_changed else "false")

    print("\nChange detection completed.")


if __name__ == "__main__":
    detect_changes()
