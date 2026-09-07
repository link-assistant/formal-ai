#!/usr/bin/env bash
# Issue #1079, defect D2: `cargo audit` reports `unmaintained` and `yanked`
# crates as *warnings*, and warnings do not change its exit status. The Security
# workflow therefore stayed green while `Cargo.lock` shipped a yanked crate.
#
# This script reproduces the exit status both ways on any lockfile. Run it with
# the repository's own lockfile to see the gap, and after `--deny warnings` is
# wired into `scripts/check-rust-dependencies.sh` to see the gate close.
#
#   bash experiments/issue-1079-cargo-audit-warning-exit-status.sh [Cargo.lock]
#
# Recorded output on `main` at f971b8205 (cargo-audit 0.22.2, advisory db with
# 1239 advisories):
#
#   plain            -> exit 0   "warning: 2 allowed warnings found"
#   --deny warnings  -> exit 1   "error: 2 denied warnings found!"
#
# The two warnings were `fxhash 0.2.1 unmaintained` (RUSTSEC-2025-0057) and
# `chacha20 0.10.1 yanked`. A yanked crate carries no advisory ID, so it cannot
# be silenced by `[advisories] ignore` -- only by `--no-yanked`, which disables
# the whole check. Yank findings must therefore always be fixed in the lockfile.

set -euo pipefail

lock="${1:-Cargo.lock}"

if ! command -v cargo-audit > /dev/null 2>&1; then
  echo "cargo-audit is not on PATH. Install the exact version CI uses with:"
  echo "  cargo install cargo-audit --locked --version 0.22.2"
  echo "or download the prebuilt binary CI installs:"
  echo "  https://github.com/rustsec/rustsec/releases/tag/cargo-audit%2Fv0.22.2"
  exit 127
fi

run() {
  local label="$1"
  shift
  local status=0
  "$@" > /tmp/issue-1079-audit.log 2>&1 || status=$?
  echo "=== ${label}: exit ${status}"
  tail -n 3 /tmp/issue-1079-audit.log
  echo
  return 0
}

run "cargo audit --file ${lock}" \
  cargo-audit audit --file "$lock" --color never
run "cargo audit --file ${lock} --deny warnings" \
  cargo-audit audit --file "$lock" --color never --deny warnings
