#!/usr/bin/env bash
# Compile once, then overlap three isolated processes. The benchmark module has
# its own serialization lock; every process retains one Rust test thread.
set -euo pipefail

contract_targets=(
  --test calendar_currency
  --test openai_forced_functions
  --test meta_executor_parity
  --test synthesis_contracts
  --test cnl_output
  --test cnl_parity
  --test unknown_reasoning
  --test data_files
  --test scheduling_contracts
  --test operational_contracts
  --test reasoning_contracts
  --test workflow_contracts
)
native_targets=("${contract_targets[@]}" --test interface_contracts)
cargo test --manifest-path rust/Cargo.toml --locked --no-run "${native_targets[@]}"
benchmark_log="$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/formal-ai-native-benchmarks.XXXXXX.log")"
benchmark_pid=''
interface_pid=''
interface_log="$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/formal-ai-native-interfaces.XXXXXX.log")"
# Invoked indirectly by the EXIT trap below.
# shellcheck disable=SC2329
cleanup_native_regressions() {
  if [[ -n "$benchmark_pid" ]]; then
    kill "$benchmark_pid" 2>/dev/null || true
    wait "$benchmark_pid" 2>/dev/null || true
  fi
  if [[ -n "$interface_pid" ]]; then
    kill "$interface_pid" 2>/dev/null || true
    wait "$interface_pid" 2>/dev/null || true
  fi
  cat "$benchmark_log" "$interface_log"
  rm -f "$benchmark_log" "$interface_log"
}
trap 'cleanup_native_regressions' EXIT
cargo test --manifest-path rust/Cargo.toml --locked --test reasoning_contracts benchmarks:: -- --test-threads=1 --nocapture >"$benchmark_log" 2>&1 &
benchmark_pid=$!
cargo test --manifest-path rust/Cargo.toml --locked --test interface_contracts -- --test-threads=1 --nocapture >"$interface_log" 2>&1 &
interface_pid=$!
regression_status=0
cargo test --manifest-path rust/Cargo.toml --locked --no-fail-fast "${contract_targets[@]}" -- --skip benchmarks:: --test-threads=1 --nocapture || regression_status=$?
wait "$interface_pid" || regression_status=$?
interface_pid=''
wait "$benchmark_pid" || regression_status=$?
benchmark_pid=''
exit "$regression_status"
