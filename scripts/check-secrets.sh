#!/usr/bin/env bash
# check-secrets.sh
#
# Scans the files a change touches for committed credentials, using secretlint
# with the recommended preset (CI-CD-BEST-PRACTICES.md section 11 / issue #808).
#
# Usage:
#   BASE_REF=main bash scripts/check-secrets.sh     # scan the diff vs BASE_REF
#   bash scripts/check-secrets.sh --all             # scan every tracked file
#   bash scripts/check-secrets.sh path/to/file ...  # scan specific files
#
# Exit code 0 = no secrets found; non-zero = secretlint reported a finding.
#
# Why the diff and not the whole tree
# -----------------------------------
# The js template runs `secretlint "**/*"`. Measured on this repository that
# does not finish in a usable time: the glob walks the working tree (including
# `target/` and ~150 MB of committed case-study logs) and the scan was still
# running after 25 minutes locally. A gate that cannot be relied on to finish is
# a new CI failure mode, which is the opposite of what issue #808 asks for.
#
# Scanning what the change touches keeps the gate fast and catches the case that
# matters: a credential being *introduced*. Pre-existing files are not re-scanned
# on every pull request; run `--all` (no timeout) to audit the whole tree.
#
# FORMAL_AI_SECRETS_DEBUG=1 prints the resolved file list before scanning.
set -euo pipefail

cd "$(dirname "$0")/.."

debug() {
  if [ "${FORMAL_AI_SECRETS_DEBUG:-0}" = "1" ]; then
    printf '[check-secrets] %s\n' "$1" >&2
  fi
}

# Captured third-party CI/agent logs kept verbatim as evidence. They are
# reviewed when added and consist of redacted GitHub Actions output.
exclude() {
  grep -zv -e '^docs/case-studies/.*\.log$' -e '^dev/log/.*ci-logs/.*\.log$' || true
}

files=()
if [ "${1:-}" = "--all" ]; then
  echo "Scanning every tracked file (this is slow; there is no timeout here)."
  mapfile -d '' files < <(git ls-files -z | exclude)
elif [ "$#" -gt 0 ]; then
  files=("$@")
elif [ -n "${BASE_REF:-}" ]; then
  base="origin/${BASE_REF}"
  git rev-parse --verify --quiet "$base" >/dev/null || base="${BASE_REF}"
  # A push event's `before` SHA is all-zeroes for a new branch, and is unknown
  # to this clone after a force-push. Fall back to the previous commit rather
  # than failing the job on something that is not a secret problem.
  if ! git rev-parse --verify --quiet "${base}^{commit}" >/dev/null; then
    echo "Base ref '${BASE_REF}' does not resolve here; falling back to HEAD~1"
    base="HEAD~1"
  fi
  merge_base="$(git merge-base "$base" HEAD)"
  echo "Scanning files changed in ${merge_base}..HEAD"
  # -z for paths with spaces; ACMRT drops deletions, which have nothing to scan.
  mapfile -d '' files < <(git diff --name-only --diff-filter=ACMRT -z "$merge_base" HEAD | exclude)
else
  echo "Scanning uncommitted and untracked changes"
  mapfile -d '' files < <(
    { git diff --name-only --diff-filter=ACMRT -z HEAD; git ls-files -z --others --exclude-standard; } | exclude
  )
fi

if [ "${#files[@]}" -eq 0 ]; then
  echo "No files to scan."
  exit 0
fi

echo "Scanning ${#files[@]} file(s) for secrets..."
for f in "${files[@]}"; do
  debug "$f"
done

printf '%s\0' "${files[@]}" |
  xargs -0 npx --yes \
    -p secretlint \
    -p @secretlint/secretlint-rule-preset-recommend \
    secretlint

echo "No secrets found."
