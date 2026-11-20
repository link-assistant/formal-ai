#!/usr/bin/env bash
# CI runs expensive families in separate bounded jobs. The default invocation
# preserves the complete three-process runner, with one Rust test thread each.
set -euo pipefail

contract_targets=(
  --test calendar_currency
  --test openai_forced_functions
  --test meta_executor_parity
  --test synthesis_contracts
  --test coding_contracts
  --test cnl_output
  --test cnl_parity
  --test unknown_reasoning
  --test data_files
  --test scheduling_contracts
  --test operational_contracts
  --test reasoning_contracts
  --test workflow_contracts
)
# Validate before invoking Cargo so misspelled matrix groups cannot replay all tests.
group="${1:-all}"
if [[ "$#" -gt 1 ]]; then
  echo 'Usage: ci-native-regressions.sh [all|core|coding|reasoning|hive|benchmarks|interfaces]' >&2
  exit 2
fi
case "$group" in
  all) ;;
  core)
    core_targets=()
    for ((index=0; index<${#contract_targets[@]}; index+=2)); do
      target="${contract_targets[index+1]}"
      if [[ "$target" != coding_contracts && "$target" != reasoning_contracts ]]; then
        core_targets+=(--test "$target")
      fi
    done
    cargo test --manifest-path rust/Cargo.toml --locked --no-fail-fast "${core_targets[@]}" -- --test-threads=1 --nocapture
    exit "$?"
    ;;
  coding|reasoning|hive|benchmarks|interfaces)
    family_arguments=()
    test_arguments=(--test-threads=1 --nocapture)
    case "$group" in
      coding) family_arguments=(--test coding_contracts) ;;
      reasoning)
        family_arguments=(--test reasoning_contracts)
        test_arguments=(--skip benchmarks:: --skip issue_1133_hive_mind_three_runs:: "${test_arguments[@]}")
        ;;
      hive) family_arguments=(--test reasoning_contracts issue_1133_hive_mind_three_runs::) ;;
      benchmarks) family_arguments=(--test reasoning_contracts benchmarks::) ;;
      interfaces) family_arguments=(--test interface_contracts) ;;
    esac
    cargo test --manifest-path rust/Cargo.toml --locked --no-fail-fast "${family_arguments[@]}" -- "${test_arguments[@]}"
    exit "$?"
    ;;
  *)
    echo 'Usage: ci-native-regressions.sh [all|core|coding|reasoning|hive|benchmarks|interfaces]' >&2
    exit 2
    ;;
esac
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
