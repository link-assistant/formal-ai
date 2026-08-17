#!/usr/bin/env python3
"""
Validate that PRs contain proper changelog fragments.

This script is the Python equivalent of validate-changeset.mjs from the JS template.
It ensures that pull requests include changelog documentation.

Usage:
    python scripts/validate_changeset.py

Exit codes:
    0 - Validation passed (fragment found or no source changes)
    1 - Validation failed (source changes without fragment)

Example CI usage:
    - name: Validate changelog fragment
      run: python scripts/validate_changeset.py
"""

import re
import sys
from pathlib import Path


def get_fragment_files(changelog_dir: Path) -> list[Path]:
    """Get list of changelog fragment files (excluding README and template)."""
    if not changelog_dir.exists():
        return []

    return [
        f
        for f in changelog_dir.glob("*.md")
        if f.name != "README.md" and not f.name.endswith(".j2")
    ]


def validate_fragment_content(fragment_path: Path) -> tuple[bool, str]:
    """
    Validate that a fragment has proper content.

    Returns (is_valid, error_message).
    """
    content = fragment_path.read_text().strip()

    if not content:
        return False, f"Fragment {fragment_path.name} is empty"

    # Check for at least one category heading
    category_pattern = re.compile(
        r"^###\s*(Added|Changed|Deprecated|Fixed|Removed|Security)",
        re.MULTILINE | re.IGNORECASE,
    )

    if not category_pattern.search(content):
        return False, (
            f"Fragment {fragment_path.name} missing category heading.\n"
            "Expected one of: ### Added, ### Changed, ### Deprecated, "
            "### Fixed, ### Removed, ### Security"
        )

    # Check for actual content (not just commented template)
    # Remove HTML comments
    content_without_comments = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)
    # Check if there's meaningful content after headings
    lines = [
        line.strip()
        for line in content_without_comments.split("\n")
        if line.strip() and not line.strip().startswith("#")
    ]

    if not lines:
        return False, (
            f"Fragment {fragment_path.name} has no content.\n"
            "Please add a description of your changes under the appropriate category."
        )

    return True, ""


def main() -> int:
    """Main entry point."""
    # Determine project root and changelog directory
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    changelog_dir = project_root / "changelog.d"

    print("Validating changelog fragments...")
    print()

    # Get fragment files
    fragments = get_fragment_files(changelog_dir)
    fragment_count = len(fragments)

    print(f"Found {fragment_count} changelog fragment(s)")

    if fragment_count == 0:
        print()
        print("WARNING: No changelog fragment found!")
        print()
        print("To document your changes, create a changelog fragment:")
        print()
        print("  # Using scriv (recommended):")
        print("  pip install 'scriv[toml]'")
        print("  scriv create")
        print()
        print("  # Or using the helper script:")
        print(
            "  python scripts/create_manual_changeset.py patch --description 'Your changes'"
        )
        print()
        print("See changelog.d/README.md for more information.")
        print()

        # This is currently a warning, not a failure
        # Change to "return 1" to make it required
        return 0

    if fragment_count > 1:
        print()
        print(
            f"WARNING: Found {fragment_count} fragments. Usually PRs should have only one."
        )
        print("Fragments found:")
        for f in fragments:
            print(f"  - {f.name}")
        print()

    # Validate each fragment
    all_valid = True
    for fragment in fragments:
        is_valid, error = validate_fragment_content(fragment)
        if is_valid:
            print(f"  [OK] {fragment.name}")
        else:
            print(f"  [FAIL] {error}")
            all_valid = False

    print()

    if all_valid:
        print("Changelog validation passed!")
        return 0
    else:
        print("Changelog validation FAILED!")
        print()
        print("Expected fragment format:")
        print()
        print("  ### Added")
        print("  - Description of new feature")
        print()
        print("  ### Changed")
        print("  - Description of change")
        print()
        print("  ### Fixed")
        print("  - Description of bug fix")
        print()
        return 1


if __name__ == "__main__":
    sys.exit(main())
