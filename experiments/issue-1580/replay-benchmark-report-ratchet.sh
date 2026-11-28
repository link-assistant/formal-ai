#!/usr/bin/env bash
# Replay the native integration case with a released or hosted CLI.
set -euo pipefail

if [[ $# != 1 || ! -x "$1" ]]; then
  printf 'Usage: %s /path/to/formal-ai\n' "$0" >&2
  exit 2
fi
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
binary=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT

export FORMAL_AI_BENCHMARK_REPLAY_SOURCE="$repository/rust/tests/integration/formal_ai_cli/benchmark_report.rs"
env "CARGO_BIN_EXE_formal-ai=$binary" rustc --edition=2024 --test -o "$scratch/replay" - <<'RS'
fn tmpdir() -> std::path::PathBuf {
    let directory = std::path::PathBuf::from(std::env::var_os("FORMAL_AI_REPLAY_WORKSPACE").unwrap());
    std::fs::create_dir(&directory).unwrap();
    directory
}
mod benchmark_report {
    include!(env!("FORMAL_AI_BENCHMARK_REPLAY_SOURCE"));
}
RS
FORMAL_AI_REPLAY_WORKSPACE="$scratch/workspace" "$scratch/replay" --test-threads=1 --nocapture
