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
MATRIX_OBSERVATIONS="$ARTIFACTS/contract-observations.jsonl"
: > "$MATRIX_OBSERVATIONS"
matrix_make_fixtures "$WORKDIR"

cleanup() {
  matrix_stop_stack
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

matrix_lock_installer "$CLIENT" | grep -q . \
  || matrix_fail "$CLIENT is missing from $LOCKFILE"

# Drive the binary `install_client.sh` put down, not whatever else on this host
# answers to the same command name. Two clients really do collide — cursor's
# vendor script drops an `agent` alias beside `cursor-agent`, shadowing the
# pinned `@link-assistant/agent` — and the collision surfaced as "the CLI never
# reached the server through the proxy", which reads like our defect.
LEG_BIN_DIR="$(matrix_client_bin_dir "$CLIENT")"
if [ -n "$LEG_BIN_DIR" ]; then
  PATH="$LEG_BIN_DIR:$PATH"
  export PATH
  matrix_note "driving $(readlink -f "$LEG_BIN_DIR/$(matrix_client_field "$CLIENT" command)")"
fi

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
  matrix_log_matches_re "$MATRIX_CLIENT_LOG" '[a-z]{3,}' \
    || matrix_fail "greeting: the CLI printed no prose answer"
  matrix_assert_output_lacks greeting "instructions" "$MATRIX_CLIENT_LOG"
  matrix_pass "greeting: answered in one bounded turn"
  matrix_record_case greeting
}

# ---------------------------------------------------------------------------
# read-file — issue #671's own finding
# ---------------------------------------------------------------------------
case_read_file() {
  matrix_start_stack read-file "$((BASE_PORT + 10))"
  # File delivery is an upstream capability: tool-bearing clients fetch the
  # fixture, while prompt-format clients receive its bytes in-band. Both the
  # invocation and the assertion come from the seed-defined contract.
  MATRIX_TRAILING_ARGS=()
  mapfile -t MATRIX_TRAILING_ARGS \
    < <(matrix_client_values "$CLIENT" verification.headless_args)
  matrix_run_headless read-file "$CLIENT" \
    "read the file alpha.txt and print its contents"
  matrix_assert_proxy_ok read-file
  matrix_assert_client_sandbox_worked read-file
  matrix_assert_output_contains read-file "$ALPHA_MARKER" "$MATRIX_CLIENT_LOG"
  # Two rounds is the honest shape: plan the read, then answer with it. The
  # bound is generous, but 281 rounds — the pre-fix behaviour — is not.
  matrix_assert_bounded_rounds read-file 12
  # A tool-bearing client must have actually executed a tool, not answered from
  # prose. `-s` matters: without slurping, jq's exit status reflects only the
  # *last* JSONL row, and the last row of a converged run is the final answer,
  # which carries no tool calls.
  if jq -es 'any(.[]; (.request_tools // []) | length > 0)' "$PROXY_LOG" > /dev/null; then
    [ "$(matrix_client_field "$CLIENT" verification.file_delivery)" = tool_call ] \
      || matrix_fail "read-file: the client advertised tools but its contract says file delivery is in-band"
    jq -es 'any(.[]; (.response_tool_calls // []) | length > 0)' "$PROXY_LOG" > /dev/null \
      || matrix_fail "read-file: no exchange planned a tool call"
    local required_response_tool
    while IFS= read -r required_response_tool; do
      jq -es --arg tool "$required_response_tool" \
        'any(.[]; any(.response_tool_calls[]?; .name == $tool))' \
        "$PROXY_LOG" > /dev/null \
        || matrix_fail "read-file: required response tool '$required_response_tool' was not invoked"
    done < <(matrix_client_values "$CLIENT" verification.required_response_tools)
    matrix_pass "read-file: a real tool call fetched $ALPHA_MARKER"
  else
    [ "$(matrix_client_field "$CLIENT" verification.file_delivery)" = in_band ] \
      || matrix_fail "read-file: $CLIENT advertised no tools but its contract requires a tool call"
    matrix_pass "read-file: prompt-format client supplied fixture bytes in-band"
  fi
  matrix_observe_case read_file \
    "read the file alpha.txt and print its contents" "$ALPHA_MARKER"
  matrix_record_case read-file
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
  matrix_log_matches_re "$MATRIX_CLIENT_LOG" '[a-z]{3,}' \
    || matrix_fail "summarize: the CLI printed no prose answer"
  matrix_pass "summarize: answered as a conversation request"
  matrix_record_case summarize
}

# ---------------------------------------------------------------------------
# interactive — issue #650 defect 2 and issue #713
# ---------------------------------------------------------------------------
case_interactive() {
  matrix_start_stack interactive "$((BASE_PORT + 30))"
  # The leading Enters answer the onboarding screens these TUIs open with, and
  # the count is deliberately generous: Claude Code opens three in a row (theme
  # picker, security notice, "is this a folder you trust?") where Codex opens
  # one, and a leg that under-counts them types the prompt into a dialog instead
  # of the composer. Any Enter left over lands on an *empty* composer, which is
  # issue #650 defect 2 — an empty message wedging the TUI — so a client that
  # never recovers from it fails here.
  # Then the real prompt — typed and submitted as two separate writes, because
  # Codex treats text immediately followed by CR as a bracketed paste and inserts
  # a newline instead of submitting; that swallowed Enter is what made the first
  # working TUI capture still record zero exchanges. Ctrl-C twice is the quit
  # path every one of these TUIs shares.
  # The wait after the prompt is on the *rendered answer*, not on a stopwatch:
  # see `matrix_keystrokes`.
  local preamble=() entry key value
  mapfile -t preamble \
    < <(matrix_client_values "$CLIENT" verification.interactive_preamble)
  while IFS= read -r entry; do
    key="${entry%%=*}"
    value="${entry#*=}"
    printf -v "$key" '%s' "$value"
    export "$key"
  done < <(matrix_client_values "$CLIENT" verification.interactive_env)
  matrix_run_interactive interactive "$CLIENT" \
    '10:\r' '4:\r' '4:\r' '4:\r' "${preamble[@]}" \
    '4:show me the exact text stored in alpha.txt' '3:\r' \
    "await:$ALPHA_MARKER" '2:\x03' '2:\x03'
  matrix_assert_launched interactive
  # Reaching the server at all from a TUI is the assertion issue #713 needed:
  # the launch-blocking bugs it recorded never produced a single exchange, while
  # 160 `--non-interactive` runs stayed green throughout.
  [ "$(matrix_proxy_rows)" -ge 1 ] \
    || matrix_fail "interactive: the TUI never reached the server"
  matrix_assert_bounded_rounds interactive 12
  # Same diagnosis as the headless read: this case also makes the client read a
  # file with its own tooling, so on a kernel that cannot host the client's
  # sandbox the marker goes missing for a reason that is not ours. Naming it in
  # both places is the point — the two cases must not report the same host
  # limitation differently.
  matrix_assert_client_sandbox_worked interactive
  # The answer has to come back through the TUI's own rendering, not just over
  # the wire — issue #671's comment asks for assertions on streamed output.
  matrix_assert_output_contains interactive "$ALPHA_MARKER" "$MATRIX_CLIENT_LOG"
  matrix_pass "interactive: TUI launched, submitted an empty message and answered"
  matrix_observe_case read_file \
    "show me the exact text stored in alpha.txt" "$ALPHA_MARKER"
  matrix_record_case interactive
}

# ---------------------------------------------------------------------------
# globally — issue #650 defect 4
# ---------------------------------------------------------------------------
case_globally() {
  local home_dir baseline after_setup after_undo status
  home_dir="$WORKDIR/home-$CLIENT"
  mkdir -p "$home_dir" "$ARTIFACTS/globally"
  MATRIX_CLIENT_LOG="$ARTIFACTS/globally/client.log"
  baseline="$(matrix_tree_digest "$home_dir")"

  # The alias must be accepted; issue #650 defect 4 was `--globally` being
  # rejected as an unknown flag.
  HOME="$home_dir" timeout 60 "$BIN" with --globally --no-start-server \
    --base-url "http://127.0.0.1:$BASE_PORT" "$CLIENT" \
    > "$MATRIX_CLIENT_LOG" 2>&1 < /dev/null
  status=$?
  matrix_log_matches_re "$MATRIX_CLIENT_LOG" "unexpected argument|unrecognized" \
    && matrix_fail "globally: the --globally alias was rejected (issue #650)"
  [ "$status" -eq 0 ] || matrix_fail "globally: --globally exited $status"

  after_setup="$(matrix_tree_digest "$home_dir")"
  [ "$after_setup" != "$baseline" ] \
    || matrix_fail "globally: --globally wrote nothing to the persistent config"
  matrix_pass "globally: --globally configured $CLIENT persistently"

  # Undo must restore the home exactly, backups included — a global setup that
  # leaves residue is how a user's own client config gets quietly rewritten.
  HOME="$home_dir" timeout 60 "$BIN" with --globally --undo --no-start-server \
    --base-url "http://127.0.0.1:$BASE_PORT" "$CLIENT" \
    >> "$MATRIX_CLIENT_LOG" 2>&1 < /dev/null
  status=$?
  [ "$status" -eq 0 ] || matrix_fail "globally: --globally --undo exited $status"

  after_undo="$(matrix_tree_digest "$home_dir")"
  [ "$after_undo" = "$baseline" ] \
    || matrix_fail "globally: --undo left the home directory changed:
$(diff <(printf '%s\n' "$baseline") <(printf '%s\n' "$after_undo") || true)"
  matrix_pass "globally: --undo restored the home directory exactly"
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

  local required forbidden
  while IFS= read -r required; do
    grep -qF -- "$required" <<< "$tools" \
      || matrix_fail "constraints: required advertised tool '$required' is missing"
    matrix_pass "constraints: advertised required tool $required"
  done < <(matrix_client_values "$CLIENT" verification.required_request_tools)
  while IFS= read -r forbidden; do
    matrix_assert_output_lacks constraints "$forbidden" "$MATRIX_CLIENT_LOG"
    matrix_pass "constraints: headless output lacks forbidden marker '$forbidden'"
  done < <(matrix_client_values "$CLIENT" verification.forbidden_output)

  matrix_record_case constraints
}

# ---------------------------------------------------------------------------
# launch — the leg shape for clients that are not prompt CLIs
# ---------------------------------------------------------------------------
# `t3code` is a web application server and the two OpenCode desktop rows are GUI
# applications, so there is no prompt to hand them and no TUI to type into. They
# still get a real leg: the app is launched through `formal-ai with` against our
# recording proxy and has to come up, and the *reason* it cannot be driven
# further is written as an assertion that fails when upstream changes it.
case_launch() {
  local shape="$1" port=$((BASE_PORT + 50))
  matrix_start_stack launch "$BASE_PORT"

  local launch_args=() raw ready required http_path extension_glob expected actual
  while IFS= read -r raw; do
    launch_args+=("$(matrix_expand_verification_template "$raw" "$port")")
  done < <(matrix_client_values "$CLIENT" verification.launch_args)
  extension_glob="$(matrix_client_field "$CLIENT" verification.extension_glob)"
  if [ -n "$extension_glob" ]; then
    extension_glob="$(matrix_expand_verification_template "$extension_glob" "$port")"
    compgen -G "$extension_glob" > /dev/null \
      || matrix_fail "launch: required extension is missing: $extension_glob"
  fi
  if matrix_client_bool "$CLIENT" verification.chromium_sandbox_fallback \
    && ! unshare --user --map-root-user true > /dev/null 2>&1; then
    matrix_note "launch: this host denies unprivileged user namespaces — adding --no-sandbox"
    launch_args+=(--no-sandbox --disable-gpu)
  fi
  matrix_launch_client launch "$CLIENT" "$shape" "${launch_args[@]}"
  if [ "$shape" = server ]; then
    ready="$(matrix_expand_verification_template \
      "$(matrix_client_field "$CLIENT" verification.launch_ready)" "$port")"
    [ -z "$ready" ] || matrix_await_log launch "$ready" 120
    matrix_assert_launched launch
    while IFS= read -r required; do
      required="$(matrix_expand_verification_template "$required" "$port")"
      matrix_log_matches "$MATRIX_CLIENT_LOG" "$required" \
        || matrix_fail "launch: required output is missing: $required"
    done < <(matrix_client_values "$CLIENT" verification.launch_required_output)
    http_path="$(matrix_client_field "$CLIENT" verification.launch_http_path)"
    [ -z "$http_path" ] \
      || [ "$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$port$http_path")" = 200 ] \
      || matrix_fail "launch: server surface did not answer 200 at $http_path"
    expected="$(matrix_client_values "$CLIENT" verification.launch_subcommands | sort | tr '\n' ' ')"
    if [ -n "$expected" ]; then
      actual="$("$(matrix_client_field "$CLIENT" command)" --help 2> /dev/null \
        | awk '/^SUBCOMMANDS/ { found = 1; next } found && NF { print $1 }' | sort | tr '\n' ' ')"
      [ "$actual" = "$expected" ] \
        || matrix_fail "launch: subcommands changed from '$expected' to '$actual' — check for a prompt path"
    fi
    matrix_pass "launch: server surface satisfied its seeded launch contract"
  else
    matrix_assert_still_running launch 20
    matrix_assert_launched launch
    matrix_supports_headless "$CLIENT" \
      && matrix_fail "launch: registry now claims a GUI headless mode — give it the full case list"
    matrix_pass "launch: GUI surface satisfied its seeded launch contract"
  fi

  # The point of the leg is that the *app* was pointed at our server, not merely
  # that a binary started — and a launch leg cannot prove that from traffic,
  # because a GUI sends no model request until a human types. What it can prove
  # is that the running process carries the wrapper's configuration.
  matrix_assert_launch_configured launch "$(matrix_client_field "$CLIENT" command)"

  # Whatever the client did on startup — a model catalog fetch, a health probe —
  # is recorded, so a reviewer sees which endpoints a desktop client touches.
  matrix_note "launch: $(matrix_proxy_rows) recorded exchanges on startup"
  matrix_record_case launch
  matrix_kill_launch
}

# ---------------------------------------------------------------------------
# mcp — the leg shape for clients that consume us as a *tool server*
# ---------------------------------------------------------------------------
# Cursor's integration is `default_protocol "mcp"`: the wrapper writes
# `.cursor/mcp.json` pointing at our `/mcp` endpoint, and Cursor's own model —
# behind Cursor's own credentials — calls us as a tool. There is no base URL to
# redirect and therefore no prompt this harness can put through it: without a
# `CURSOR_API_KEY` the CLI exits 1 with "Authentication required" before any
# turn. That constraint is asserted rather than skipped, and the surface Cursor
# would actually use is exercised directly over JSON-RPC through the proxy.
case_mcp() {
  matrix_start_stack mcp "$BASE_PORT"

  local rpc="$ARTIFACTS/mcp/rpc.json"
  mkdir -p "$ARTIFACTS/mcp"
  mcp_call() {
    curl -s -H 'Content-Type: application/json' -d "$1" "$BASE_URL/mcp" > "$rpc"
  }

  mcp_call '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"matrix","version":"1"}}}'
  [ "$(jq -r '.result.serverInfo.name' "$rpc")" = "formal-ai" ] \
    || matrix_fail "mcp: initialize did not identify our server: $(cat "$rpc")"
  matrix_pass "mcp: initialize handshake answered as formal-ai"

  mcp_call '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
  [ "$(jq -r '.result.tools[0].name' "$rpc")" = "formal_ai_chat" ] \
    || matrix_fail "mcp: tools/list did not advertise formal_ai_chat: $(cat "$rpc")"
  matrix_pass "mcp: tools/list advertises formal_ai_chat"

  # A real answer through the tool call Cursor would make, not merely a 200.
  #
  # The prompt is *not* the file question the CLI legs ask, and that is a
  # finding rather than a convenience: over MCP we are the tool, so there is no
  # tool loop back to us and no workspace of ours to read — `read the file
  # alpha.txt` answers "I could not determine …" here, which is the honest
  # answer for a tool server that was handed no bytes. A question the solver can
  # answer on its own is what proves the call reached the real solver.
  mcp_call '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"formal_ai_chat","arguments":{"prompt":"What is 2 + 2?"}}}'
  [ "$(jq -r '.result.isError' "$rpc")" = "false" ] \
    || matrix_fail "mcp: tools/call reported an error: $(cat "$rpc")"
  jq -r '.result.content[0].text' "$rpc" | grep -qF "4" \
    || matrix_fail "mcp: tools/call did not answer the question: $(cat "$rpc")"
  matrix_pass "mcp: tools/call reached the solver and answered correctly"

  # An unknown tool must be refused rather than silently answered — Cursor picks
  # the tool name, so a server that answers anything is a server that cannot be
  # trusted about which tool ran.
  mcp_call '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"not_a_tool","arguments":{"prompt":"hi"}}}'
  [ "$(jq -r '.error.code' "$rpc")" = "-32601" ] \
    || matrix_fail "mcp: an unknown tool name was not refused with -32601: $(cat "$rpc")"
  matrix_pass "mcp: an unknown tool name is refused (JSON-RPC -32601)"

  # Upstream constraint, asserted. If Cursor ever runs a turn without vendor
  # credentials, this fails and the leg must grow the full prompt case list.
  matrix_run_headless mcp "$CLIENT" "hi"
  [ "$MATRIX_CLIENT_STATUS" -ne 0 ] \
    || matrix_fail "mcp: $CLIENT ran a headless turn without vendor credentials — give it the full case list"
  local vendor_auth_error
  vendor_auth_error="$(matrix_client_field "$CLIENT" verification.vendor_auth_error)"
  [ -n "$vendor_auth_error" ] \
    || matrix_fail "mcp: no vendor credential constraint is defined"
  matrix_log_matches_ci "$MATRIX_CLIENT_LOG" "$vendor_auth_error" \
    || matrix_fail "mcp: $CLIENT failed for some reason other than missing credentials: $(matrix_strip_ansi "$MATRIX_CLIENT_LOG" | head -3)"
  matrix_pass "mcp: $CLIENT still requires vendor credentials for its own model (upstream constraint)"

  matrix_record_case mcp
}

FAILED=0
# Each case runs in a subshell so a failed assertion ends that case, not the
# leg: a red `read-file` must not hide whether `interactive` also broke. The
# stack is torn down here in the parent, where the pid file is readable.
run_case() {
  local name="$1"
  shift
  matrix_note "case $name"
  ("$@") || FAILED=1
  matrix_stop_stack
}

SHAPE="$(matrix_client_shape "$CLIENT")"
matrix_note "shape=$SHAPE"
if [ "$SHAPE" = cli ]; then
  if [ "$HEADLESS" = yes ]; then
    run_case greeting case_greeting
    run_case read-file case_read_file
    run_case summarize case_summarize
    run_case constraints case_constraints
  else
    matrix_note "$CLIENT has no headless invocation in the seed registry — interactive-only leg"
  fi
  run_case interactive case_interactive
elif [ "$SHAPE" = mcp ]; then
  run_case mcp case_mcp
else
  run_case launch case_launch "$SHAPE"
fi
run_case globally case_globally

matrix_stop_stack
[ "$FAILED" -eq 0 ] || { echo "!! $CLIENT leg failed" >&2; exit 1; }
echo "== $CLIENT leg OK =="
