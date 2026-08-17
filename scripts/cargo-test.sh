#!/usr/bin/env bash
# Run `cargo test` under the repository's resource policy, then prune the build
# cache down to the artifacts the latest build actually produced.
#
# Two different machines run these tests and they want opposite things:
#
#   CI          ephemeral runners billed by the minute. Use every core, and
#               prune afterwards only so a warm cache restored on the next run
#               carries one build's worth of artifacts instead of many.
#   A laptop    shared with everything else the maintainer is doing. A bare
#               `cargo test` spawns one compile job and one test thread per CPU,
#               which pins all of them; `target/` then keeps every stale
#               artifact from every branch ever built, and this repository's
#               debug tree reaches several gigabytes.
#
# So parallelism is capped to half the CPUs unless `CI` is set, and the prune
# runs either way. Override with `CARGO_TEST_JOBS=<n>`; set
# `CARGO_TEST_NO_PRUNE=1` to keep the full cache for a debugging session.
#
# Usage: scripts/cargo-test.sh [any cargo test arguments]
#   scripts/cargo-test.sh                     # whole suite
#   scripts/cargo-test.sh --test unit issue_907
set -euo pipefail

cpu_count() {
  if command -v sysctl >/dev/null 2>&1 && sysctl -n hw.ncpu >/dev/null 2>&1; then
    sysctl -n hw.ncpu
  elif command -v nproc >/dev/null 2>&1; then
    nproc
  else
    echo 2
  fi
}

total_cpus=$(cpu_count)

if [ -n "${CARGO_TEST_JOBS:-}" ]; then
  jobs=$CARGO_TEST_JOBS
  policy="CARGO_TEST_JOBS override"
elif [ -n "${CI:-}" ]; then
  jobs=$total_cpus
  policy="CI: all cores"
else
  # Half, rounded down, but never below one -- a 1-CPU machine still has to run.
  jobs=$((total_cpus / 2))
  [ "$jobs" -lt 1 ] && jobs=1
  policy="local: half of $total_cpus cores"
fi

echo "cargo test: $jobs job(s) / test thread(s) ($policy)"

# Stamped before the build so the prune below can tell this run's artifacts
# from every earlier one, no matter how long the suite takes.
started=$(mktemp)
trap 'rm -f "$started"' EXIT

# `--jobs` caps compilation; RUST_TEST_THREADS caps the harness at run time.
# Capping only the first still lets the test binary saturate every core.
status=0
RUST_TEST_THREADS="$jobs" cargo test --jobs "$jobs" "$@" || status=$?

# Prune whether the suite passed or failed -- a red run leaves the same stale
# artifacts behind. The marker stamped before the build tells the pruner exactly
# which artifacts belong to this run.
"$(dirname "$0")/prune-build-cache.sh" "$started"

exit "$status"
