#!/usr/bin/env bash
# Run the inline `#[cfg(test)]` unit-test suites of every rust-script in scripts/.
#
# `cargo test` only builds the library crate, so these suites — including the
# regression guard on rebase-before-stage ordering in version-and-commit.rs —
# do not run unless they are invoked explicitly. Run this locally before
# touching anything under scripts/.
#
# Every suite is attempted even after one fails, so a single broken script does
# not hide the state of the others; the aggregate exit code is non-zero if any
# suite failed.
set -uo pipefail

cd "$(dirname "$0")/.."

# Match how CI compiles the rest of the tree, so dead-code and unused-import
# warnings introduced by the test harness surface here rather than in CI.
export RUSTFLAGS="${RUSTFLAGS:--Dwarnings}"

status=0
failed=()

for f in scripts/*.rs; do
  grep -q 'cfg(test)' "$f" || continue
  echo "::group::rust-script --test $f"
  if ! rust-script --test "$f"; then
    status=1
    failed+=("$f")
  fi
  echo "::endgroup::"
done

if [ "$status" -ne 0 ]; then
  echo "Failed script test suites:" >&2
  printf '  %s\n' "${failed[@]}" >&2
fi

exit "$status"
