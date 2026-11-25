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
# Exit code 0 = no secrets found; non-zero = a finding or scanner failure.
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
  local status=0
  grep -zv -e '^docs/case-studies/.*\.log$' -e '^dev/log/.*ci-logs/.*\.log$' || status=$?
  # grep's no-match status means a valid empty selection; actual errors fail.
  [ "$status" -le 1 ]
}

# mapfile alone loses the exit status of a process substitution. Wait for the
# producer before accepting either an empty or a partially emitted inventory.
read_inventory() {
  local descriptor producer
  exec {descriptor}< <("$@" | exclude)
  producer=$!
  mapfile -d '' files <&"$descriptor"
  exec {descriptor}<&-
  if ! wait "$producer"; then
    echo 'Unable to collect complete scanner inventory; no scan performed.' >&2
    return 1
  fi
}

uncommitted_files() {
  git diff --name-only --diff-filter=ACMRT -z HEAD &&
    git ls-files -z --others --exclude-standard
}

files=()
if [ "${1:-}" = "--all" ]; then
  echo "Scanning every tracked file (this is slow; there is no timeout here)."
  read_inventory git ls-files -z
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
  read_inventory git diff --name-only --diff-filter=ACMRT -z "$merge_base" HEAD
else
  echo "Scanning uncommitted and untracked changes"
  read_inventory uncommitted_files
fi

if [ "${#files[@]}" -eq 0 ]; then
  echo "No files to scan."
  exit 0
fi

echo "Scanning ${#files[@]} file(s) for secrets..."
for f in "${files[@]}"; do
  debug "$f"
done

# Pinned, not floating (issue #812): `npx --yes -p secretlint` resolved to
# whatever `latest` was at the moment the job ran, so the same commit could pass
# today and fail tomorrow with no change of ours, and a compromised publish would
# execute in CI unreviewed. Bump SECRETLINT_VERSION deliberately.
SECRETLINT_VERSION="${SECRETLINT_VERSION:-14.1.0}"
debug "secretlint ${SECRETLINT_VERSION}"

# Feed the exact existing selection through stdin rather than interpolating paths
# into npm's shell command. The pinned API is loaded once for all archive members.
export SECRETLINT_VERSION
scan_files() {
  if [ "$SECRETLINT_VERSION" = 14.1.0 ]; then
    # npm exec pins the root package, but resolves its semver transitives again.
    # Install the committed closure separately from application dependencies.
    scanner_directory="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-secretlint.XXXXXX")"
    trap 'rm -rf -- "$scanner_directory"' EXIT
    cp .github/secretlint/package.json .github/secretlint/package-lock.json "$scanner_directory/"
    # Installation must not consume the NUL filename stream intended for the scanner.
    npm ci --ignore-scripts --no-audit --no-fund --engine-strict --prefix "$scanner_directory" </dev/null >&2
    PATH="$scanner_directory/node_modules/.bin:$PATH" node scripts/check-secrets-files.mjs
  else
    # Preserve the explicit custom-version override; only the default is lock-qualified.
    npx --yes -p "secretlint@${SECRETLINT_VERSION}" node scripts/check-secrets-files.mjs
  fi
}
printf '%s\0' "${files[@]}" | scan_files

echo "No secrets found."
