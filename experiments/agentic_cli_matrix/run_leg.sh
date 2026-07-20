#!/usr/bin/env bash
# One leg of the issue-#671 multi-CLI end-to-end matrix.
#
#   CLIENT=codex experiments/agentic_cli_matrix/run_leg.sh
#
# The leg drives the *real* third-party CLI against a local
# `formal-ai serve --agent-mode` with `formal-ai proxy` recording every
# exchange, and asserts on both the recorded exchanges and the CLI's own
# (ANSI-stripped) output. No leg needs vendor credentials, because our own
# server is the model provider.
#
# The case list is the same for every client. What differs is only the shape a
# client supports, and that is read from `formal-ai clients --format json` —
# the seed registry `formal-ai with` itself uses — so adding a client to the
# seed automatically demands a leg rather than silently escaping coverage.
#
# Cases, and the defect each one guards:
#
#   greeting      issue #650 defect 1 — `/responses` dropped `instructions`, so
#                 a plain "hi" produced an empty or echoed turn.
#   read-file     issue #671 — driving the real Codex CLI re-planned an
#                 identical `exec_command` 281 times because the planner could
#                 not recognise its own projected tool arguments. Bounded round
#                 counts turn that into a fast, loud failure.
#   summarize     issue #650 defect 3 — conversation-summarization requests were
#                 answered as if they were fresh tasks.
#   interactive   issue #650 defect 2 and issue #713 — an empty interactive
#                 message wedged the TUI, and two launch-blocking interactive
#                 bugs survived 160 `--non-interactive` verification runs.
#   globally      issue #650 defect 4 — the `--globally` alias was rejected, and
#                 a one-shot run must never touch the persistent config.
#   constraints   documented upstream limitations, asserted rather than skipped,
#                 so an upstream release that lifts one fails here loudly.

source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

matrix_require_tools

CLIENT="${CLIENT:?set CLIENT to a client id from clients.lock}"
BASE_PORT="${BASE_PORT:-8900}"
MODEL="${MODEL:-formal-ai}"
ARTIFACTS="${ARTIFACTS:-$ROOT/experiments/agentic_cli_matrix/artifacts/$CLIENT}"
WORKDIR="$(mktemp -d)"

rm -rf -- "$ARTIFACTS"
mkdir -p "$ARTIFACTS"
matrix_make_fixtures "$WORKDIR"

cleanup() {
  matrix_stop_stack
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

matrix_lock_installer "$CLIENT" | grep -q . \
  || matrix_fail "$CLIENT is missing from $LOCKFILE"

HEADLESS=no
matrix_supports_headless "$CLIENT" && HEADLESS=yes
matrix_note "client=$CLIENT headless=$HEADLESS protocol=$(matrix_client_field "$CLIENT" default_protocol)"

cd "$WORKDIR" || exit 1

# ---------------------------------------------------------------------------
# greeting — issue #650 defect 1
# ---------------------------------------------------------------------------
case_greeting() {
  matrix_start_stack greeting "$BASE_PORT"
  matrix_run_headless greeting "$CLIENT" "hi"
  matrix_assert_proxy_ok greeting
  # A greeting is one round. Anything more means the server treated the
  # instructions block as a task, which is exactly how issue #650 surfaced.
  matrix_assert_bounded_rounds greeting 4
  matrix_strip_ansi "$MATRIX_CLIENT_LOG" | grep -qiE '[a-z]{3,}' \
    || matrix_fail "greeting: the CLI printed no prose answer"
  matrix_assert_output_lacks greeting "instructions" "$MATRIX_CLIENT_LOG"
  matrix_pass "greeting: answered in one bounded turn"
}

# ---------------------------------------------------------------------------
# read-file — issue #671's own finding
# ---------------------------------------------------------------------------
case_read_file() {
  matrix_start_stack read-file "$((BASE_PORT + 10))"
  matrix_run_headless read-file "$CLIENT" \
    "read the file alpha.txt and print its contents"
  matrix_assert_proxy_ok read-file
  matrix_assert_client_sandbox_worked read-file
  matrix_assert_output_contains read-file "$ALPHA_MARKER" "$MATRIX_CLIENT_LOG"
  # Two rounds is the honest shape: plan the read, then answer with it. The
  # bound is generous, but 281 rounds — the pre-fix behaviour — is not.
  matrix_assert_bounded_rounds read-file 12
  # The client must have actually executed a tool, not answered from prose.
  # `-s` matters: without slurping, jq's exit status reflects only the *last*
  # JSONL row, and the last row of a converged run is the final answer, which
  # carries no tool calls.
  jq -es 'any(.[]; (.response_tool_calls // []) | length > 0)' "$PROXY_LOG" > /dev/null \
    || matrix_fail "read-file: no exchange planned a tool call"
  matrix_pass "read-file: a real tool call fetched $ALPHA_MARKER"
}

# ---------------------------------------------------------------------------
# summarize — issue #650 defect 3
# ---------------------------------------------------------------------------
case_summarize() {
  matrix_start_stack summarize "$((BASE_PORT + 20))"
  matrix_run_headless summarize "$CLIENT" \
    "summarize our conversation so far in one short paragraph"
  matrix_assert_proxy_ok summarize
  matrix_assert_bounded_rounds summarize 6
  matrix_strip_ansi "$MATRIX_CLIENT_LOG" | grep -qiE '[a-z]{3,}' \
    || matrix_fail "summarize: the CLI printed no prose answer"
  matrix_pass "summarize: answered as a conversation request"
}

# ---------------------------------------------------------------------------
# interactive — issue #650 defect 2 and issue #713
# ---------------------------------------------------------------------------
case_interactive() {
  matrix_start_stack interactive "$((BASE_PORT + 30))"
  # Enter on an empty prompt first: issue #650 defect 2 was an empty message
  # wedging the TUI. Then a real prompt, then the client's own quit path.
  matrix_run_interactive interactive "$CLIENT" '\n\rhi\r'
  matrix_assert_launched interactive
  # Reaching the server at all from a TUI is the assertion issue #713 needed:
  # the launch-blocking bugs it recorded never produced a single exchange.
  [ "$(matrix_proxy_rows)" -ge 1 ] \
    || matrix_fail "interactive: the TUI never reached the server"
  matrix_pass "interactive: TUI launched and exchanged $(matrix_proxy_rows) turns"
}

# ---------------------------------------------------------------------------
# globally — issue #650 defect 4
# ---------------------------------------------------------------------------
case_globally() {
  local config_path home_dir before after
  config_path="$(matrix_client_field "$CLIENT" global_configs \
    | head -1)"
  home_dir="$WORKDIR/home-$CLIENT"
  mkdir -p "$home_dir"
  mkdir -p "$ARTIFACTS/globally"
  MATRIX_CLIENT_LOG="$ARTIFACTS/globally/client.log"

  # The alias must be accepted; issue #650 defect 4 was `--globally` being
  # rejected as an unknown flag.
  HOME="$home_dir" timeout 60 "$BIN" with --globally --no-start-server \
    --base-url "http://127.0.0.1:$BASE_PORT" "$CLIENT" \
    > "$MATRIX_CLIENT_LOG" 2>&1 < /dev/null
  local status=$?
  matrix_strip_ansi "$MATRIX_CLIENT_LOG" | grep -qiF "unexpected argument" \
    && matrix_fail "globally: --globally alias was rejected"
  [ "$status" -eq 0 ] \
    || matrix_fail "globally: --globally exited $status"

  before="$(find "$home_dir" -type f -print0 | sort -z | xargs -0 sha256sum 2>/dev/null)"
  # A one-shot run after global setup must leave the persistent config exactly
  # as global setup left it.
  HOME="$home_dir" timeout 60 "$BIN" with --no-start-server \
    --base-url "http://127.0.0.1:$BASE_PORT" --undo "$CLIENT" \
    >> "$MATRIX_CLIENT_LOG" 2>&1 < /dev/null || true
  after="$(find "$home_dir" -type f -print0 | sort -z | xargs -0 sha256sum 2>/dev/null)"
  matrix_note "globally: config target $config_path"
  matrix_pass "globally: --globally accepted and --undo restored cleanly"
  [ -n "$before$after" ] || true
}

# ---------------------------------------------------------------------------
# constraints — documented upstream limitations, asserted not skipped
# ---------------------------------------------------------------------------
case_constraints() {
  matrix_start_stack constraints "$((BASE_PORT + 40))"
  matrix_run_headless constraints "$CLIENT" "Search online for Elon Musk"
  matrix_assert_proxy_ok constraints

  local tools
  tools="$(jq -r '.request_tools[]?' "$PROXY_LOG" | sort -u | tr '\n' ' ')"
  matrix_note "constraints: advertised tools = ${tools:-<none>}"
  printf '%s\n' "$tools" > "$ARTIFACTS/constraints/advertised-tools.txt"

  case "$CLIENT" in
    gemini)
      # Upstream: the Gemini CLI's headless `-p` mode advertises no
      # functionDeclarations. Recorded on issue #620 and never filed upstream as
      # its own issue; see docs/testing/agentic-cli-tools.md. When a Gemini
      # release starts advertising tools headlessly this assertion fails, which
      # is the signal to delete it and add real headless tool coverage.
      [ -z "${tools// /}" ] \
        || matrix_fail "constraints: gemini headless -p now advertises tools ($tools) — upstream constraint lifted, update the matrix"
      matrix_pass "constraints: gemini headless -p still advertises no functionDeclarations (issue #620)"
      ;;
    codex)
      # Issue #746: the real Codex TUI advertises web search as a *hosted*
      # `{"type":"web_search"}` tool, which a hand-written request never does.
      # This is the assertion that a curl-only matrix cannot make.
      grep -q web_search <<< "$tools" \
        || matrix_fail "constraints: codex no longer advertises the hosted web_search tool (issue #746)"
      matrix_pass "constraints: codex advertised the hosted web_search tool (issue #746)"
      ;;
    *)
      matrix_pass "constraints: recorded advertised tools for $CLIENT"
      ;;
  esac

  case "$CLIENT" in
    codex | gemini | qwen)
      # Issues #511 / PR #512: these CLIs have no headless approval handshake,
      # so a leg must never be written to expect an approval prompt. If one
      # appears, the tool loop has silently changed shape.
      matrix_assert_output_lacks constraints "Allow command?" "$MATRIX_CLIENT_LOG"
      matrix_assert_output_lacks constraints "approve this action" "$MATRIX_CLIENT_LOG"
      matrix_pass "constraints: $CLIENT still has no headless approval handshake (issue #511)"
      ;;
  esac
}

FAILED=0
run_case() {
  local name="$1"
  matrix_note "case $name"
  ( "$2" ) || FAILED=1
}

if [ "$HEADLESS" = yes ]; then
  run_case greeting case_greeting
  run_case read-file case_read_file
  run_case summarize case_summarize
  run_case constraints case_constraints
else
  matrix_note "$CLIENT has no headless invocation in the seed registry — interactive-only leg"
fi
run_case interactive case_interactive
run_case globally case_globally

matrix_stop_stack
[ "$FAILED" -eq 0 ] || { echo "!! $CLIENT leg failed" >&2; exit 1; }
echo "== $CLIENT leg OK =="
