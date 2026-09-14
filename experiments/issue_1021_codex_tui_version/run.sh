#!/usr/bin/env bash
# Issue #1021: bisect the `E2E Tests (agent CLI <-> formal-ai)` failure that
# appeared on the pull-request branch on 2026-08-19 without any Formal AI change
# reaching the Codex TUI startup path.
#
# The job installs `@openai/codex` unpinned, so the client version drifts under
# the harness. This script runs the same Codex leg of `run_issue_819.sh` twice,
# once per version, from prefixes installed here rather than from `PATH`, and
# reports which versions clear Codex's "Do you trust the contents of this
# directory?" startup dialog.
#
# Usage: experiments/issue_1021_codex_tui_version/run.sh [version ...]
#        (defaults to the last version CI proved green and the one that broke)

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX_ROOT="${PREFIX_ROOT:-/tmp/formal-ai-codex-versions}"
PORT="${PORT:-8990}"
LOG_DIR="${LOG_DIR:-$ROOT/experiments/issue_1021_codex_tui_version/logs}"
VERSIONS=("$@")
if [ "${#VERSIONS[@]}" -eq 0 ]; then
  VERSIONS=(0.147.0 0.148.0)
fi

mkdir -p "$LOG_DIR"

if [ ! -x "$ROOT/target/release/formal-ai" ]; then
  echo "!! build the server first: cargo build --release --bin formal-ai" >&2
  exit 1
fi

status=0
port="$PORT"
for version in "${VERSIONS[@]}"; do
  prefix="$PREFIX_ROOT/$version"
  if [ ! -x "$prefix/node_modules/.bin/codex" ]; then
    echo "-- installing @openai/codex@$version into $prefix"
    npm install --prefix "$prefix" --ignore-scripts "@openai/codex@$version" \
      > "$LOG_DIR/install-$version.log" 2>&1 \
      || { echo "!! install of @openai/codex@$version failed" >&2; status=1; continue; }
  fi

  echo "-- running the Codex leg of issue #819 against codex $version"
  log="$LOG_DIR/run-$version.log"
  PATH="$prefix/node_modules/.bin:$PATH" \
    CLIENTS=codex TUI_CLIENTS=codex RUN_TUI=1 PORT="$port" \
    "$ROOT/experiments/agent_cli_e2e/run_issue_819.sh" > "$log" 2>&1
  result=$?
  port=$((port + 10))
  if [ "$result" -eq 0 ]; then
    echo "== codex $version: PASS"
  else
    echo "!! codex $version: FAIL (exit $result) -- see $log"
    grep -E "^(!!|== issue #819)" "$log" | tail -5
    status=1
  fi
done

exit "$status"
