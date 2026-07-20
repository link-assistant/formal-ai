# Shared plumbing for the issue-#671 multi-CLI end-to-end matrix.
#
# Sourced by `install_client.sh` and `run_leg.sh`. Nothing in here is specific
# to a single client: the per-client shape is read from
# `formal-ai clients --format json`, which is generated from the same
# `data/seed/client-integrations.lino` registry `formal-ai with` uses. A leg
# therefore cannot drift from the wrapper it is meant to prove.

set -uo pipefail

MATRIX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$MATRIX_DIR/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
LOCKFILE="${LOCKFILE:-$MATRIX_DIR/clients.lock}"

# Fixture markers from docs/testing/agentic-cli-tools.md. Asserting on these
# proves the CLI really read our workspace rather than answering from prose.
ALPHA_MARKER="ALPHA_MARKER_11111"
BETA_MARKER="BETA_MARKER_22222"
GAMMA_MARKER="GAMMA_33333"
NESTED_MARKER="NESTED_MARKER_44444"

matrix_fail() {
  echo "!! $*" >&2
  matrix_dump_logs
  exit 1
}

matrix_note() { echo "-- $*"; }
matrix_pass() { echo "ok $*"; }

# A missing harness dependency must stop the leg, not quietly shrink it. The
# first local run of this matrix had no `jq`, so every registry lookup returned
# empty, `supports_non_interactive` read as false, and Codex — which very much
# has a headless mode — silently degraded to an interactive-only leg that
# reported OK. A skip that announces itself as a pass is the exact failure mode
# issue #671 exists to end.
matrix_require_tools() {
  local missing=()
  for tool in jq curl timeout script sed awk; do
    command -v "$tool" > /dev/null 2>&1 || missing+=("$tool")
  done
  [ ${#missing[@]} -eq 0 ] \
    || matrix_fail "the matrix harness needs these missing tools: ${missing[*]}"
  [ -x "$BIN" ] || matrix_fail "no formal-ai binary at $BIN (cargo build --release --bin formal-ai)"
}

# --- lockfile -----------------------------------------------------------------

matrix_lock_installer() {
  awk -v id="$1" '$1 == id { print $2; exit }' "$LOCKFILE"
}

matrix_lock_spec() {
  awk -v id="$1" '$1 == id { print $3; exit }' "$LOCKFILE"
}

matrix_lock_ids() {
  awk '!/^#/ && NF >= 3 { print $1 }' "$LOCKFILE"
}

# --- registry -----------------------------------------------------------------

# Read one field of one client out of the seed-baked registry.
#
# An unknown client or field is a harness bug, not an empty answer: reading a
# missing `supports_non_interactive` as "false" would turn a broken lookup into
# a silently reduced leg.
# It returns non-zero rather than calling `matrix_fail`, because callers read it
# through command substitution — and `exit` inside a substitution only ends the
# substitution's subshell, which would let the very failure this guards against
# continue as an empty string.
matrix_client_field() {
  local id="$1" field="$2" registry value
  registry="$("$BIN" clients --format json)" || {
    echo "!! formal-ai clients --format json failed" >&2
    return 1
  }
  value="$(printf '%s' "$registry" | jq -er --arg id "$id" --arg field "$field" \
    '.[] | select(.id == $id) | .[$field] | if type == "array" then join(" ") else tostring end')" || {
    echo "!! client '$id' has no field '$field' in the seed registry" >&2
    return 1
  }
  printf '%s' "$value"
}

matrix_supports_headless() {
  local value
  value="$(matrix_client_field "$1" supports_non_interactive)" \
    || matrix_fail "cannot tell whether '$1' supports headless mode"
  case "$value" in
    true) return 0 ;;
    false) return 1 ;;
    *) matrix_fail "supports_non_interactive for '$1' was '$value', expected true or false" ;;
  esac
}

# --- stack --------------------------------------------------------------------
#
# `formal-ai serve --agent-mode` is the model provider, so no leg needs vendor
# credentials or a recorded upstream: the transcript we replay against *is* our
# own deterministic server. `formal-ai proxy` (PR #631) sits in front of it and
# records every exchange, which is what the per-case assertions read.

MATRIX_SERVE_PID=""
MATRIX_PROXY_PID=""
MATRIX_SERVE_LOG=""
MATRIX_PROXY_LOG=""
PROXY_LOG=""
BASE_URL=""

matrix_dump_logs() {
  for log in "$MATRIX_SERVE_LOG" "$MATRIX_PROXY_LOG" "${MATRIX_CLIENT_LOG:-}"; do
    [ -n "$log" ] && [ -f "$log" ] || continue
    echo "== $log ==" >&2
    tail -80 "$log" >&2 || true
  done
  if [ -n "$PROXY_LOG" ] && [ -f "$PROXY_LOG" ]; then
    echo "== proxy exchanges ==" >&2
    matrix_proxy_summary >&2 || true
  fi
}

matrix_stop_stack() {
  for pid in "$MATRIX_PROXY_PID" "$MATRIX_SERVE_PID"; do
    [ -n "$pid" ] || continue
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
  MATRIX_PROXY_PID=""
  MATRIX_SERVE_PID=""
}

# Start a fresh server + recording proxy for one case. Each case gets its own
# proxy log: the proxy holds an append handle open for its whole lifetime, so
# truncating or deleting a live log silently sends later rows to an unlinked
# inode instead of resetting the file.
matrix_start_stack() {
  local case_name="$1" port="$2"
  matrix_stop_stack
  local proxy_port=$((port + 1))
  MATRIX_SERVE_LOG="$ARTIFACTS/$case_name/serve.log"
  MATRIX_PROXY_LOG="$ARTIFACTS/$case_name/proxy-stderr.log"
  PROXY_LOG="$ARTIFACTS/$case_name/proxy.jsonl"
  mkdir -p "$ARTIFACTS/$case_name"

  FORMAL_AI_TRACE_REQUESTS=1 "$BIN" serve --agent-mode --host 127.0.0.1 --port "$port" \
    > "$MATRIX_SERVE_LOG" 2>&1 < /dev/null &
  MATRIX_SERVE_PID=$!
  matrix_await_listener "server" "$MATRIX_SERVE_PID" "$port" "$MATRIX_SERVE_LOG"

  "$BIN" proxy --listen "127.0.0.1:$proxy_port" \
    --upstream "http://127.0.0.1:$port" --log "$PROXY_LOG" --body \
    > "$MATRIX_PROXY_LOG" 2>&1 < /dev/null &
  MATRIX_PROXY_PID=$!
  matrix_await_listener "proxy" "$MATRIX_PROXY_PID" "$proxy_port" "$MATRIX_PROXY_LOG"

  BASE_URL="http://127.0.0.1:$proxy_port"
}

# Wait for the process *we* started, not for whatever happens to answer.
#
# A health probe alone is not enough: when a stale server from an earlier run
# still owns the port, our process dies with `AddrInUse` while the probe
# cheerfully succeeds against the stranger. The leg then drives a server whose
# proxy log it is not reading, and every assertion sees an empty transcript —
# a whole leg silently measuring nothing.
matrix_await_listener() {
  local role="$1" pid="$2" port="$3" log="$4"
  curl -fsS --retry 40 --retry-delay 1 --retry-connrefused \
    "http://127.0.0.1:$port/health" > /dev/null 2>&1 \
    || matrix_fail "$role never answered on port $port"
  kill -0 "$pid" 2> /dev/null \
    || matrix_fail "$role exited during startup on port $port: $(tail -1 "$log")"
  grep -qi "Address already in use" "$log" \
    && matrix_fail "$role could not bind port $port — another process owns it"
  return 0
}

# --- fixtures -----------------------------------------------------------------

matrix_make_fixtures() {
  local dir="$1"
  mkdir -p "$dir/subdir"
  printf '%s\nalpha second line\n' "$ALPHA_MARKER" > "$dir/alpha.txt"
  printf '# beta\n\n%s\n' "$BETA_MARKER" > "$dir/beta.md"
  printf '{"gamma": "%s"}\n' "$GAMMA_MARKER" > "$dir/gamma.json"
  printf '%s\n' "$NESTED_MARKER" > "$dir/subdir/nested.log"
}

# --- proxy assertions ---------------------------------------------------------

matrix_proxy_summary() {
  jq -c '{path, request_model, request_tools, status, response_model, response_tool_calls}' \
    "$PROXY_LOG" 2>/dev/null
}

# Model rounds only. The CLIs poll `/health` on startup, and counting those as
# rounds would both inflate the loop bound and let a leg that never asked the
# model anything look like it did.
matrix_proxy_rows() {
  [ -f "$PROXY_LOG" ] || { echo 0; return; }
  jq -r 'select(.path != "/health") | .path' "$PROXY_LOG" 2>/dev/null | wc -l | tr -d ' '
}

# Every recorded exchange must have reached our server and been answered.
matrix_assert_proxy_ok() {
  local rows
  rows="$(matrix_proxy_rows)"
  [ "$rows" -ge 1 ] || matrix_fail "$1: the CLI never reached the server through the proxy"
  local bad
  bad="$(jq -r 'select(.status >= 400) | "\(.status) \(.path)"' "$PROXY_LOG" | head -5)"
  [ -z "$bad" ] || matrix_fail "$1: proxy recorded failing exchanges: $bad"
  # Slurped, because jq's exit status otherwise reflects only the last row.
  jq -es --arg model "$MODEL" 'any(.[]; .request_model == $model)' "$PROXY_LOG" > /dev/null \
    || matrix_fail "$1: no exchange advertised model '$MODEL' (provenance)"
}

# The loop must converge. Issue #671's first real finding was a Codex leg that
# re-planned an identical `exec_command` 281 times because the planner could not
# see its own tool result; an unbounded round count is therefore a first-class
# failure, not a timeout to raise.
matrix_assert_bounded_rounds() {
  local case_name="$1" limit="$2" rows
  rows="$(matrix_proxy_rows)"
  [ "$rows" -le "$limit" ] \
    || matrix_fail "$case_name: $rows model rounds exceed the $limit-round bound (tool-call loop)"
  matrix_pass "$case_name: converged in $rows model rounds (bound $limit)"
}

matrix_assert_output_contains() {
  local case_name="$1" needle="$2" file="$3"
  matrix_strip_ansi "$file" | grep -qF -- "$needle" \
    || matrix_fail "$case_name: client output never contained '$needle'"
  matrix_pass "$case_name: client output carried '$needle'"
}

matrix_assert_output_lacks() {
  local case_name="$1" needle="$2" file="$3"
  matrix_strip_ansi "$file" | grep -qF -- "$needle" \
    && matrix_fail "$case_name: client output unexpectedly contained '$needle'"
  return 0
}

# TUI output is full of escape sequences; assertions run on the stripped text so
# a redraw cannot hide or fake a match.
matrix_strip_ansi() {
  sed -e 's/\x1b\[[0-9;?]*[a-zA-Z]//g' -e 's/\x1b][^\x07]*\x07//g' -e 's/\r/\n/g' "$1"
}

# --- driving the client -------------------------------------------------------

MATRIX_CLIENT_LOG=""

# One headless turn through `formal-ai with`, always with stdin closed: Codex
# otherwise blocks on "Reading additional input from stdin...".
matrix_run_headless() {
  local case_name="$1" client="$2" prompt="$3"
  shift 3
  MATRIX_CLIENT_LOG="$ARTIFACTS/$case_name/client.log"
  # MATRIX_CLIENT_ARGS is the escape hatch for a host whose kernel cannot host a
  # client's own sandbox — see `matrix_assert_client_sandbox_worked`. CI leaves
  # it empty so the real sandboxed path is what gets exercised.
  local extra=()
  [ -n "${MATRIX_CLIENT_ARGS:-}" ] && read -r -a extra <<< "$MATRIX_CLIENT_ARGS"
  timeout "${CASE_TIMEOUT:-180}" "$BIN" with \
    --no-start-server --base-url "$BASE_URL" --non-interactive \
    "$client" "${extra[@]}" "$@" "$prompt" \
    > "$MATRIX_CLIENT_LOG" 2>&1 < /dev/null
  MATRIX_CLIENT_STATUS=$?
  return 0
}

# A client's own sandbox failing to start is not a formal-ai defect, but it must
# never be mistaken for one — nor quietly tolerated. Codex shells out through
# bubblewrap, and on a host without unprivileged user namespaces `cat` returns
# `bwrap: loopback: Failed RTM_NEWADDR` *as the command's output*, which the
# server then faithfully quotes back as the file's contents. Naming it turns a
# baffling marker-missing failure into an actionable one.
matrix_assert_client_sandbox_worked() {
  local case_name="$1"
  matrix_strip_ansi "$MATRIX_CLIENT_LOG" | grep -qiE 'bwrap:|Failed RTM_NEWADDR|sandbox.*(denied|not permitted)' \
    && matrix_fail "$case_name: the client's own sandbox could not start on this host (needs unprivileged user namespaces; set MATRIX_CLIENT_ARGS to bypass it locally)"
  return 0
}

# One interactive turn through a real PTY. `script -qfec` is the mechanism the
# in-repo wrapper tests already use, and it is the only way to reach the TUI
# code paths: issue #713 recorded two launch-blocking interactive-only bugs that
# 160 `--non-interactive` verification runs could not see, and issue #746 showed
# a real TUI advertises tools (hosted `web_search`) that a hand-written request
# never does.
matrix_run_interactive() {
  local case_name="$1" client="$2" keys="$3"
  MATRIX_CLIENT_LOG="$ARTIFACTS/$case_name/client.log"
  local command="$BIN with --no-start-server --base-url $BASE_URL --interactive $client"
  printf '%b' "$keys" \
    | timeout "${CASE_TIMEOUT:-120}" script -qfec "$command" /dev/null \
      > "$MATRIX_CLIENT_LOG" 2>&1
  MATRIX_CLIENT_STATUS=$?
  return 0
}

# A launch failure is a wrapper bug, not a client quirk — these are the exact
# shapes issue #713 and issue #650 reported.
matrix_assert_launched() {
  local case_name="$1"
  for marker in "unexpected argument" "Not enough arguments" "unsupported tool" \
    "error: unrecognized" "command not found"; do
    matrix_strip_ansi "$MATRIX_CLIENT_LOG" | grep -qiF -- "$marker" \
      && matrix_fail "$case_name: launch rejected with '$marker'"
  done
  matrix_pass "$case_name: client launched cleanly"
}
