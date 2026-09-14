#!/usr/bin/env bash
# Issue #1021 -- the CodeQL check on PR #1027 reported 99 alerts, and every one
# of them was produced by the *name* of a parameter or a local binding rather
# than by anything the code does.
#
# The failing check run (commit 800f5f7ff, https://github.com/link-assistant/formal-ai/runs/96753666549)
# was titled "99 new alerts including 98 critical severity security
# vulnerabilities". The 98 critical ones all carry the same message, "This
# hard-coded value is used as a salt", and the one high one says "This operation
# writes session_id to a log file".
#
# Neither is about cryptography or about a credential. Both come from a purely
# lexical rule in the queries:
#
#   * `rust/hard-coded-cryptographic-value` makes a sink out of every positional
#     argument passed to a parameter *literally named* `password`, `iv`, `nonce`
#     or `salt` -- see `HeuristicSinks` in
#     `rust/ql/lib/codeql/rust/security/HardcodedCryptographicValueExtensions.qll`.
#     `seeded_unit_interval(impulse, salt)` in `src/translation/selection.rs`
#     hashes with FNV-1a, a non-cryptographic hash, and the "salt" was only ever
#     a string that makes a draw reproducible. Every configuration literal that
#     reached that call was therefore reported as a hard-coded cryptographic salt.
#
#   * `rust/cleartext-logging` treats any name matching
#     `session.?(id|key)` as account information -- see `HeuristicNames` in
#     `shared/concepts/codeql/concepts/internal/SensitiveDataHeuristics.qll`.
#     `src/cli_improve.rs` printed FNV-1a digests of recorded sessions, which
#     are committed as evidence under `docs/case-studies/`, out of a binding
#     called `session_id`.
#
# The fix renames the things after what they are (`seed`, `agent_session_digests`)
# and changes no behaviour: the seed *strings* and the digest *values* are
# byte-identical before and after. `tests/unit/ci-cd/codeql_sink_heuristics.rs`
# is the regression guard, and this script is its falsification: it reverts the
# renames, shows the guard failing at the exact lines CodeQL flagged, and puts
# the tree back.
#
# Usage:
#   bash experiments/issue-1021-codeql-name-heuristics/falsify.sh
#
# Recorded result (2026-08-21), with the renames reverted:
#   no_function_parameter_is_named_after_a_hard_coded_cryptographic_sink
#     src/translation/selection.rs:324: parameter `salt`
#     src/translation/selection.rs:361: parameter `salt`
#     tests/source/translation/selection.rs:324: parameter `salt`
#     tests/source/translation/selection.rs:361: parameter `salt`
#   no_logging_macro_is_handed_a_name_that_reads_as_account_information
#     src/cli_improve.rs:89: session_id
# and with the renames in place: 12 passed, 0 failed.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(git -C "$here" rev-parse --show-toplevel)"
patch="$here/rename-away-from-the-sink-names.patch"
filter='ci_cd::codeql_sink_heuristics'

cd "$root"

# The script edits tracked files and puts them back with `git checkout`, which
# only has a state to put them back to if there is nothing uncommitted to lose.
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree is not clean. This script reverts the renames in place and"
  echo "restores them with \`git checkout\`, which would discard uncommitted work."
  exit 1
fi

if ! git apply -R --check "$patch" 2>/dev/null; then
  echo "The renames are not present in this tree in the shape the fixture records."
  echo "Either they were never applied, or the files moved on since"
  echo "$(basename "$patch") was captured. Regenerate the fixture with:"
  echo "  git show \$(git log -1 --format=%H -S seeded_unit_interval) -- <the renamed files>"
  exit 1
fi

touched="$(git apply --numstat "$patch" | cut -f3)"
restore() { git checkout -- $touched; }
trap restore EXIT INT TERM

echo "== reverting the renames: salt <- seed, session_id <- agent_session_digests =="
git apply -R "$patch"

echo
echo "== the guard against the pre-fix tree (expected: FAILED) =="
before="$(mktemp)"
cargo test --test unit "$filter" -- --nocapture >"$before" 2>&1 || true
sed -n '/^running [0-9]* test/,$p' "$before"
if grep -q '^test result: ok' "$before"; then
  echo
  echo "UNEXPECTED: the guard passes against the tree that produced the alerts."
  exit 1
fi

restore
trap - EXIT INT TERM

echo
echo "== the same guard against the fixed tree (expected: ok) =="
cargo test --test unit "$filter"
