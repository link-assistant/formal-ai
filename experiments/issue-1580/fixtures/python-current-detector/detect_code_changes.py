#!/usr/bin/env python3
"""
Detect code changes for CI/CD pipeline.

This script detects what types of files have changed between two commits
and outputs the results for use in GitHub Actions workflow conditions.

Key behavior:
- For PRs: compares PR head against base branch
- For pushes: compares HEAD against the previous tip (or HEAD^ locally)
- Excludes certain folders and file types from "code changes" detection

Excluded from code changes (don't require changelog fragments):
- Markdown files (*.md) in any folder
- changelog.d/ folder (changelog metadata)
- dev/log/ folder (development logs)
- docs/ folder (documentation)
- experiments/ folder (experimental scripts)
- examples/ folder (example scripts)

Usage:
    python scripts/detect_code_changes.py

Environment variables (set by GitHub Actions):
    - GITHUB_EVENT_NAME: 'pull_request' or 'push'
    - GITHUB_BASE_SHA: Base commit SHA for PR
    - GITHUB_HEAD_SHA: Head commit SHA for PR
    - GITHUB_BEFORE_SHA: Previous tip for push events

Outputs (written to GITHUB_OUTPUT):
    - any-code-changed: 'true' if any code files changed outside excluded paths
    - docs-changed: 'true' if any markdown file or file under docs/ changed
"""

from __future__ import annotations

import os
import subprocess
import sys

EXCLUDED_FOLDERS = (
    "changelog.d/",
    "dev/log/",
    "docs/",
    "examples/",
    "experiments/",
)
CODE_EXTENSIONS = (".py", ".sh", ".json", ".toml", ".yml", ".yaml")


def git(*args: str) -> bytes:
    """Return stdout only when Git completes successfully."""
    return subprocess.run(["git", *args], stdout=subprocess.PIPE, check=True).stdout


def set_output(name: str, value: str) -> None:
    """Write output to GitHub Actions output file."""
    output_file = os.environ.get("GITHUB_OUTPUT")
    if output_file:
        with open(output_file, "a") as f:
            f.write(f"{name}={value}\n")
    print(f"{name}={value}")


def get_changed_files() -> list[str]:
    """Get the complete changed-file set or propagate the Git failure."""
    # Count both the deletion and addition when a path crosses a gate boundary.
    diff = ("diff", "--no-renames", "--name-only", "-z")
    if os.environ.get("GITHUB_EVENT_NAME") == "pull_request":
        base = os.environ.get("GITHUB_BASE_SHA")
        head = os.environ.get("GITHUB_HEAD_SHA")
        if not base or not head:
            raise ValueError(
                "PR comparison requires GITHUB_BASE_SHA and GITHUB_HEAD_SHA"
            )
        if git("rev-parse", "--is-shallow-repository").strip() == b"true":
            raise ValueError("PR comparison requires a full checkout (fetch-depth: 0)")
        for commit in (base, head):
            try:
                git("cat-file", "-e", f"{commit}^{{commit}}")
            except subprocess.CalledProcessError:
                git("fetch", "origin", commit)
        print(f"Comparing PR: {base}...{head}")
        output = git(*diff, f"{base}...{head}", "--")
    else:
        before = os.environ.get("GITHUB_BEFORE_SHA")
        # The raw commit records parent IDs even at a shallow boundary.
        headers = git("cat-file", "commit", "HEAD").split(b"\n\n", 1)[0]
        has_parent = any(line.startswith(b"parent ") for line in headers.splitlines())
        if before and before != "0" * 40:
            print(f"Comparing push: {before} to HEAD")
            output = git(*diff, before, "HEAD", "--")
        elif before or not has_parent:
            print("Listing the initial source tree at HEAD")
            output = git("ls-tree", "--name-only", "-r", "-z", "HEAD")
        else:
            print("Comparing HEAD^ to HEAD")
            output = git(*diff, "HEAD^", "HEAD", "--")
    return [os.fsdecode(path) for path in output.split(b"\0") if path]


def is_excluded_from_code_changes(file_path: str) -> bool:
    """Check if a file should be excluded from code changes detection."""
    if file_path.endswith(".md"):
        return True

    relative_path = file_path.removeprefix("python/")
    return relative_path.startswith(EXCLUDED_FOLDERS)


def is_docs_change(file_path: str) -> bool:
    """Check if a file is a documentation change (issue #72).

    Documentation-only pull requests skip the changelog-fragment requirement,
    so they need their own gate: markdown anywhere, or anything under docs/.
    """
    if file_path.endswith(".md"):
        return True
    return file_path.removeprefix("python/").startswith("docs/")


def detect_change_types(
    changed_files: list[str], *, event_name: str
) -> dict[str, bool]:
    """Classify changed files for job gating on an automatic event."""
    if event_name not in {"pull_request", "push"}:
        message = f"Unsupported automatic event: {event_name}"
        raise ValueError(message)

    code_changed = any(
        (
            file_path.endswith(CODE_EXTENSIONS)
            or file_path.startswith(".github/workflows/")
        )
        and not is_excluded_from_code_changes(file_path)
        for file_path in changed_files
    )
    docs_changed = any(is_docs_change(file_path) for file_path in changed_files)
    return {"any-code-changed": code_changed, "docs-changed": docs_changed}


def detect_changes() -> None:
    """Main function to detect changes."""
    print("Detecting file changes for CI/CD...\n")

    try:
        changed_files = get_changed_files()
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        sys.exit(f"Cannot determine changed files: {error}")

    print("Changed files:")
    if not changed_files:
        print("  (none)")
    else:
        for file in changed_files:
            print(f"  {file}")
    print()

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

    event_name = os.environ.get("GITHUB_EVENT_NAME", "push")
    outputs = detect_change_types(changed_files, event_name=event_name)
    for name, value in outputs.items():
        set_output(name, "true" if value else "false")

    print("\nChange detection completed.")


if __name__ == "__main__":
    detect_changes()
