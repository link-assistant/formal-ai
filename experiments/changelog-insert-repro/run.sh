#!/usr/bin/env bash
# Reproduces the CHANGELOG.md shape defect described in README.md.
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
sandbox="$here/sandbox"

rm -rf "$sandbox"
mkdir -p "$sandbox/changelog.d"
cd "$sandbox"

# The exact CHANGELOG.md as it stood immediately before the v0.296.0 release.
git -C "$repo_root" show b2064b2a^:CHANGELOG.md > CHANGELOG.md
printf '[package]\nname = "x"\nversion = "0.296.0"\n' > Cargo.toml
printf -- '---\nbump: patch\n---\n\n### Fixed\n- A representative fragment.\n' > changelog.d/frag.md

rust-script "$repo_root/scripts/collect-changelog.rs" >/dev/null 2>&1

echo "--- marker region (want: marker, exactly ONE blank line, then the section) ---"
sed -n '8,12p' CHANGELOG.md | cat -A | sed 's/\$$/<EOL>/'
echo "--- trailing newline present? (want a final \\n) ---"
tail -c 12 CHANGELOG.md | od -c | tail -2
