#!/usr/bin/env bash
# Run several legs of the issue-#671 matrix locally, one after another.
#
#   experiments/agentic_cli_matrix/run_matrix.sh                 # every locked client
#   experiments/agentic_cli_matrix/run_matrix.sh codex claude    # just these
#   MATRIX_RECORD=1 experiments/agentic_cli_matrix/run_matrix.sh # re-record transcripts
#
# In CI each client is its own job and they run in parallel; this is the same
# work serialised for a single machine. Ports come from the lockfile position so
# the two agree by construction — `tests/unit/issue_671_matrix_coverage.rs`
# asserts the workflow's `base_port` values match this formula, because a leg
# that quietly shares a port with another leg is how the first local run ended
# up asserting against a stranger's server.

source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

MATRIX_PORT_BASE="${MATRIX_PORT_BASE:-8900}"
MATRIX_PORT_STRIDE="${MATRIX_PORT_STRIDE:-60}"
LOGS="${LOGS:-$MATRIX_DIR/artifacts/logs}"
mkdir -p "$LOGS"

matrix_base_port() {
  awk -v id="$1" -v base="$MATRIX_PORT_BASE" -v stride="$MATRIX_PORT_STRIDE" \
    '!/^#/ && NF >= 3 { if ($1 == id) { print base + n * stride; exit } n++ }' "$LOCKFILE"
}

targets=("$@")
PLAN="$(matrix_plan)"
if [ ${#targets[@]} -eq 0 ]; then
  mapfile -t targets < <(printf '%s' "$PLAN" | jq -r '.[].client')
fi

failed=()
for client in "${targets[@]}"; do
  port="$(printf '%s' "$PLAN" | jq -r --arg client "$client" \
    '.[] | select(.client == $client) | .base_port')"
  [ -n "$port" ] || matrix_fail "$client is not in $LOCKFILE"
  log="$LOGS/$client.log"
  # Extra flags for one client come from `MATRIX_ARGS_<CLIENT>`, e.g.
  # `MATRIX_ARGS_CODEX=--dangerously-bypass-approvals-and-sandbox` on a host
  # whose kernel cannot run Codex's own bubblewrap sandbox. CI sets none of
  # these, so CI always exercises the real sandboxed path.
  var="MATRIX_ARGS_$(printf '%s' "$client" | tr 'a-z-' 'A-Z_')"
  matrix_note "leg $client (base port $port) -> $log"
  # An isolated install of this client, if one exists, wins over the shared
  # global tree — see the `MATRIX_ISOLATED_NPM` note in `install_client.sh`.
  prefix="$(matrix_client_prefix "$client")/node_modules/.bin"
  leg_path="$PATH"
  [ -d "$prefix" ] && leg_path="$prefix:$PATH"
  CLIENT="$client" BASE_PORT="$port" MATRIX_CLIENT_ARGS="${!var:-}" PATH="$leg_path" \
    "$MATRIX_DIR/run_leg.sh" > "$log" 2>&1 \
    || failed+=("$client")
  grep -aE '^(ok|!!) ' "$log" | sed "s/^/  [$client] /" || true
done

if [ ${#failed[@]} -gt 0 ]; then
  echo "!! failed legs: ${failed[*]}" >&2
  exit 1
fi

observations=()
for client in "${targets[@]}"; do
  observation="$ROOT/experiments/agentic_cli_matrix/artifacts/$client/contract-observations.jsonl"
  [ -s "$observation" ] && observations+=("$observation")
done
if [ ${#observations[@]} -gt 0 ]; then
  report="$LOGS/client-contract-learning.lino"
  "$BIN" clients learn "${observations[@]}" > "$report" \
    || matrix_fail "could not learn from successful real-client observations"
  matrix_note "human-gated client-contract learning report -> $report"
else
  matrix_note "selected surfaces produced no prompt observations to learn from"
fi
echo "== all legs OK: ${targets[*]} =="
