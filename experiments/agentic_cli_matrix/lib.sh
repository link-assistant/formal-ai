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
  # A failing case runs in a subshell, so the outer cleanup never sees the pid
  # of the application this case launched — without this the failed leg leaves
  # the app holding its port and poisons the next run.
  matrix_kill_launch
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

# Where an isolated (`MATRIX_ISOLATED_NPM=1`) install of one client lives.
matrix_client_prefix() {
  echo "${MATRIX_PREFIX_ROOT:-$HOME/.formal-ai-matrix/clients}/$1"
}

# --- what each leg actually drives --------------------------------------------
#
# A leg must run the binary *this harness installed*, not whatever `PATH`
# resolves the command name to. Two collisions on one machine proved the
# difference, and both reported as failures of our server:
#
#   * Cursor's vendor install script symlinks `cursor-agent` **and** `agent`
#     into `~/.local/bin`, which precedes `~/.bun/bin`. After the cursor leg
#     installed, every `agent` case died with `unknown option
#     '--no-summarize-session'` — surfaced by the harness as "the CLI never
#     reached the server through the proxy", i.e. blamed on us.
#   * A leftover `t3` under an old nvm Node 20 shadowed the `npm-native`
#     install, so the leg drove the copy that cannot load node-pty and t3 exited
#     before listening.
#
# So `install_client.sh` records the absolute path it installed, and every leg
# prepends a directory holding just that one command. CI never sees either
# collision — one client per runner — which is exactly why a single-machine run
# has to be the stricter of the two.

MATRIX_RESOLVED_DIR="${MATRIX_RESOLVED_DIR:-$HOME/.formal-ai-matrix/resolved}"

matrix_record_binary() {
  local client="$1" path="$2"
  [ -x "$path" ] || matrix_fail "$client: installed binary '$path' is missing or not executable"
  mkdir -p "$MATRIX_RESOLVED_DIR"
  printf '%s\n' "$path" > "$MATRIX_RESOLVED_DIR/$client"
  matrix_note "$client resolves to $path"
}

# A directory containing only this client's command, for the front of PATH.
# Empty output means the client was not installed through `install_client.sh`
# (a hand-installed CLI, or a runner that installed it some other way), and the
# caller falls back to PATH rather than failing — the harness pins what it
# installs, it does not refuse what it did not.
matrix_client_bin_dir() {
  local client="$1" record="$MATRIX_RESOLVED_DIR/$1" path command_name dir
  [ -f "$record" ] || return 0
  path="$(cat "$record")"
  [ -x "$path" ] || return 0
  command_name="$(matrix_client_field "$client" command)" || return 0
  dir="${MATRIX_BIN_ROOT:-$HOME/.formal-ai-matrix/bin}/$client"
  mkdir -p "$dir"
  ln -sf "$path" "$dir/$command_name"
  echo "$dir"
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

# Not every "client" is a terminal program that takes a prompt.
#
#   cli     — a prompt goes in, an answer comes out (codex, claude, aider, …).
#   server  — the client *is* a server with a web UI: `t3 --help` documents
#             `start`/`serve`/`auth`/`project`/`connect` and its only positional
#             argument is a working directory, so `t3 --no-browser 'say hi'`
#             reads the prompt as a *cwd* and exits 0 having answered nothing.
#             Driving a turn means a browser over its WebSocket API.
#   gui     — a desktop application: the VS Code extension host and OpenCode
#             Desktop's Electron shell (docs/case-studies/issue-762).
#   mcp     — the client does not use us as a *model* at all: it calls us as a
#             tool server over JSON-RPC while its own vendor model drives.
#             Read from the registry, so a client whose integration is MCP
#             cannot be handed prompt-shaped assertions that can never hold.
#
# These shapes get a launch leg rather than a prompt leg, plus a constraint
# assertion that fails the moment upstream grows a headless prompt path — the
# issue-#671 rule that a limitation is asserted, never skipped.
matrix_client_shape() {
  case "$1" in
    t3code) echo server ;;
    opencode-vscode | opencode-desktop) echo gui ;;
    *)
      if [ "$(matrix_client_field "$1" default_protocol)" = mcp ]; then
        echo mcp
      else
        echo cli
      fi
      ;;
  esac
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

# Started pids are tracked in a file, not only in variables.
#
# Each case runs in its own subshell so one failure does not abort the leg, and
# a variable assigned there never reaches the parent's exit trap. The first
# local run leaked one server and one proxy per case; the next run then found
# the ports taken. A pid file survives the subshell boundary, so the trap can
# actually clean up.
MATRIX_PID_FILE=""

matrix_track_pid() {
  [ -n "$MATRIX_PID_FILE" ] || MATRIX_PID_FILE="$ARTIFACTS/.stack-pids"
  echo "$1" >> "$MATRIX_PID_FILE"
}

matrix_stop_stack() {
  local file="${MATRIX_PID_FILE:-$ARTIFACTS/.stack-pids}"
  matrix_kill_launch
  if [ -f "$file" ]; then
    while read -r pid; do
      [ -n "$pid" ] || continue
      kill "$pid" 2> /dev/null || true
    done < "$file"
    rm -f "$file"
  fi
  for pid in "$MATRIX_PROXY_PID" "$MATRIX_SERVE_PID"; do
    [ -n "$pid" ] || continue
    wait "$pid" 2> /dev/null || true
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

  # The server runs in the *client's* working directory, exactly as
  # `formal-ai with` starts it. Anything it resolves against its own cwd — an
  # absolutised tool path, a relative fixture — then names the same file the
  # client would name. Started from the repository root instead, the `qwen` and
  # `agent` legs planned reads of files one directory tree away from the ones
  # they were asked about.
  # Each case gets its own shared memory. The server learns from what it is
  # asked, and with the default `$HOME/.formal-ai/memory.lino` every leg wrote
  # into the developer's real memory and read back the previous leg's turns —
  # the `aider` leg's format boilerplate ended up stored as a standing
  # requirement, and a re-run was answering partly from the run before it. A
  # matrix leg has to be hermetic to mean anything.
  ( cd "${WORKDIR:-$PWD}" \
      && FORMAL_AI_TRACE_REQUESTS=1 \
         FORMAL_AI_MEMORY_PATH="$ARTIFACTS/$case_name/memory.lino" \
         exec "$BIN" serve --agent-mode \
      --host 127.0.0.1 --port "$port" ) \
    > "$MATRIX_SERVE_LOG" 2>&1 < /dev/null &
  MATRIX_SERVE_PID=$!
  matrix_track_pid "$MATRIX_SERVE_PID"
  matrix_await_listener "server" "$MATRIX_SERVE_PID" "$port" "$MATRIX_SERVE_LOG"

  "$BIN" proxy --listen "127.0.0.1:$proxy_port" \
    --upstream "http://127.0.0.1:$port" --log "$PROXY_LOG" --body \
    > "$MATRIX_PROXY_LOG" 2>&1 < /dev/null &
  MATRIX_PROXY_PID=$!
  matrix_track_pid "$MATRIX_PROXY_PID"
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

# A stable content digest of a directory tree, used to prove `--undo` restores
# a home directory exactly rather than approximately.
matrix_tree_digest() {
  local dir="$1"
  [ -d "$dir" ] || { echo "<missing>"; return; }
  find "$dir" -type f -print0 \
    | sort -z \
    | xargs -0 -r sha256sum 2> /dev/null \
    | sed "s|$dir/||"
}

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
  matrix_log_matches "$file" "$needle" \
    || matrix_fail "$case_name: client output never contained '$needle'"
  matrix_pass "$case_name: client output carried '$needle'"
}

matrix_assert_output_lacks() {
  local case_name="$1" needle="$2" file="$3"
  matrix_log_matches "$file" "$needle" \
    && matrix_fail "$case_name: client output unexpectedly contained '$needle'"
  return 0
}

# TUI output is full of escape sequences; assertions run on the stripped text so
# a redraw cannot hide or fake a match.
matrix_strip_ansi() {
  sed -e 's/\x1b\[[0-9;?]*[a-zA-Z]//g' -e 's/\x1b][^\x07]*\x07//g' -e 's/\r/\n/g' "$1"
}

# Search stripped output *without* building a pipeline — every log assertion in
# this harness goes through these three.
#
# `set -o pipefail` and `grep -q` do not mix. grep exits at its first match and
# closes the pipe, `sed` is killed by SIGPIPE, and the pipeline's status becomes
# 141 — a failure — precisely when the needle was found *early* in a long log.
# The failure is therefore length-dependent, which is why it looked like a client
# defect: `agent` rendered ALPHA_MARKER_11111 three times into a 31 KB TUI log
# and its leg still reported "client output never contained" it, while the same
# assertion passed for clients whose logs were short enough that sed finished
# before grep exited. It also silently disarmed the negative checks (a real
# `bwrap:` match returned 141, so `&& matrix_fail` never fired) and made every
# `await:` step spin for the full timeout instead of returning on first render.
# Process substitution keeps grep's own exit status, which is the one that means
# "found".
matrix_log_matches() {
  grep -qF -- "$2" < <(matrix_strip_ansi "$1")
}

matrix_log_matches_ci() {
  grep -qiF -- "$2" < <(matrix_strip_ansi "$1")
}

matrix_log_matches_re() {
  grep -qiE -- "$2" < <(matrix_strip_ansi "$1")
}

# --- recorded transcripts -----------------------------------------------------
#
# Issue #671 asks for recorded, replayable sessions — in particular for claude,
# grok and aider, the three integrations PR #648 shipped without ever running.
# A transcript is the recorded exchange list with the bodies dropped: bodies
# carry the run's temp paths and session ids, so keeping them would make every
# committed transcript differ from the next run for reasons that mean nothing.
# What is left — path, model, advertised tools, status, planned tool calls — is
# exactly what the case assertions read, so `replay.sh` can re-check a committed
# transcript offline, with no CLI and no network.

RECORDED_DIR="${RECORDED_DIR:-$MATRIX_DIR/recorded}"

matrix_record_case() {
  local case_name="$1"
  [ "${MATRIX_RECORD:-0}" = 1 ] || return 0
  [ -f "$PROXY_LOG" ] || return 0
  mkdir -p "$RECORDED_DIR/$CLIENT"
  jq -c 'del(.request_body, .response_body)' "$PROXY_LOG" \
    > "$RECORDED_DIR/$CLIENT/$case_name.jsonl"
  matrix_note "recorded $RECORDED_DIR/$CLIENT/$case_name.jsonl"
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
  # MATRIX_TRAILING_ARGS goes *after* the prompt, because a client whose headless
  # flag takes the prompt as its value (aider's `--message`) consumes the first
  # user argument: putting `--file alpha.txt` first makes `--message` swallow
  # `--file` and aider exits with a usage error.
  local trailing=()
  [ -n "${MATRIX_TRAILING_ARGS:-}" ] && read -r -a trailing <<< "$MATRIX_TRAILING_ARGS"
  timeout "${CASE_TIMEOUT:-180}" "$BIN" with \
    --no-start-server --base-url "$BASE_URL" --non-interactive \
    "$client" "${extra[@]}" "$@" "$prompt" "${trailing[@]}" \
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
  matrix_log_matches_re "$MATRIX_CLIENT_LOG" 'bwrap:|Failed RTM_NEWADDR|sandbox.*(denied|not permitted)' \
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
  local case_name="$1" client="$2"
  shift 2
  MATRIX_CLIENT_LOG="$ARTIFACTS/$case_name/client.log"
  # A TUI needs a terminal with a size. `script` inherits the caller's, and a
  # CI step has no tty at all, so an unset size leaves the client rendering into
  # a 0x0 window and looking hung. Fixing the geometry also makes the captured
  # output stable enough to assert on.
  local command="stty rows 40 cols 120 2> /dev/null; $BIN with --no-start-server --base-url $BASE_URL --interactive $client ${MATRIX_CLIENT_ARGS:-}"
  matrix_keystrokes "$@" \
    | timeout "${CASE_TIMEOUT:-180}" script -qfec "$command" /dev/null \
      > "$MATRIX_CLIENT_LOG" 2>&1
  MATRIX_CLIENT_STATUS=$?
  return 0
}

# Type into the TUI on a human timescale, given `<seconds>:<keys>` steps.
#
# Writing every key at once and closing stdin does not work: the client reads
# EOF before it has finished starting, so nothing is ever submitted. The trailing
# settle keeps stdin open while the answer streams back — the streamed output is
# the thing issue #671's comment asks the matrix to assert on.
#
# A step spelled `await:<needle>` waits for the TUI to *render* that text instead
# of for a fixed number of seconds. A wall-clock wait cannot be both fast and
# reliable across clients: `qwen` needed three model rounds where `agent` needed
# two, and the leg quit the TUI a second before the answer appeared, reporting a
# missing marker that the very same log file grew moments later. Waiting on the
# rendered text makes the fast clients fast and the slow ones correct.
matrix_keystrokes() {
  local step waited
  for step in "$@"; do
    if [ "${step%%:*}" = await ]; then
      waited=0
      until matrix_log_matches "$MATRIX_CLIENT_LOG" "${step#*:}"; do
        [ "$waited" -lt "${MATRIX_TUI_AWAIT:-120}" ] || break
        sleep 1
        waited=$((waited + 1))
        # Press Enter periodically while waiting. Every one of these TUIs puts
        # the *approving* choice first in a tool-approval prompt — Claude Code
        # launches in "manual mode" and asks before each Read — so Enter is how
        # a human answers it, and on an idle composer Enter is the same harmless
        # empty submission issue #650 defect 2 was about.
        [ $((waited % ${MATRIX_TUI_POKE:-15})) -eq 0 ] && printf '\r' || true
      done
      continue
    fi
    sleep "${step%%:*}"
    printf '%b' "${step#*:}"
  done
  sleep "${MATRIX_TUI_SETTLE:-10}"
}

# Launch a server- or gui-shaped client in the background and keep it running.
#
# A desktop app or a web server never returns, so it cannot be driven the way a
# prompt CLI is: the leg starts it, waits for the evidence that it came up, and
# asserts against that. `xvfb-run` is used when the client needs a display and
# the host has one to give — the CI legs already run under it.
MATRIX_LAUNCH_PID=""

matrix_launch_client() {
  local case_name="$1" client="$2" shape="$3"
  shift 3
  MATRIX_CLIENT_LOG="$ARTIFACTS/$case_name/client.log"
  mkdir -p "$ARTIFACTS/$case_name"
  local launcher=()
  if [ "$shape" = gui ] && [ -z "${DISPLAY:-}" ]; then
    command -v xvfb-run > /dev/null 2>&1 \
      || matrix_fail "$case_name: $client needs a display and this host has neither DISPLAY nor xvfb-run"
    launcher=(xvfb-run -a)
  fi
  # An isolated HOME, because a desktop client onboards on first launch: run
  # against the developer's real home, OpenCode Desktop created a "Default
  # Project" directory there and would have skipped onboarding on every later
  # run — the second run would then no longer be testing what the first one did.
  local home_dir="$WORKDIR/home-launch"
  mkdir -p "$home_dir"
  HOME="$home_dir" "${launcher[@]}" "$BIN" with --no-start-server --base-url "$BASE_URL" \
    "$client" "$@" \
    > "$MATRIX_CLIENT_LOG" 2>&1 < /dev/null &
  MATRIX_LAUNCH_PID=$!
  matrix_track_pid "$MATRIX_LAUNCH_PID"
}

# Kill the launched application *and its children*.
#
# `formal-ai with` spawns the client, so killing the wrapper leaves the app
# behind: a leftover `t3 serve` from an earlier run still owned port 9010 and
# the next t3code leg died with `EADDRINUSE` — a stale process reported as a
# client defect. Children are killed first so nothing is reparented and missed.
matrix_kill_launch() {
  [ -n "${MATRIX_LAUNCH_PID:-}" ] || return 0
  local pids=() pid
  for pid in $(matrix_process_tree "$MATRIX_LAUNCH_PID"); do pids=("$pid" "${pids[@]}"); done
  for pid in "${pids[@]}"; do kill "$pid" 2> /dev/null || true; done
  MATRIX_LAUNCH_PID=""
}

# Wait for a line the client itself prints. A stopwatch would either be slow on
# every host or flaky on a loaded one; the printed readiness line is the client's
# own statement that it is up.
matrix_await_log() {
  local case_name="$1" needle="$2" limit="${3:-90}" waited=0
  until matrix_log_matches "$MATRIX_CLIENT_LOG" "$needle"; do
    kill -0 "$MATRIX_LAUNCH_PID" 2> /dev/null \
      || matrix_fail "$case_name: $CLIENT exited before printing '$needle'"
    [ "$waited" -lt "$limit" ] \
      || matrix_fail "$case_name: $CLIENT never printed '$needle' within ${limit}s"
    sleep 1
    waited=$((waited + 1))
  done
  matrix_pass "$case_name: client announced '$needle' after ${waited}s"
}

# Prove the *application* was configured to talk to our server.
#
# A launch leg cannot assert on a model exchange — a GUI client sends nothing
# until a human types — so the first version asserted "something reached the
# proxy", which the harness's own `/health` probe satisfied on its own. That is
# an assertion that can never fail, which is worse than none. What is really
# provable is that the wrapper's configuration reached the running process: it
# arrives either directly in the environment or through a config file named
# there, and both are readable from `/proc`.
matrix_assert_launch_configured() {
  local case_name="$1" command_name="$2" dir pid cmdline value
  # Only the process tree we launched. Matching *any* process whose command line
  # mentions the client's name looked equivalent and was not: `code` matched the
  # unrelated `claude --model claude-opus-4-8` process that happened to be
  # running on this host, and the assertion "passed" on its environment.
  for pid in $(matrix_process_tree "$MATRIX_LAUNCH_PID"); do
    dir="/proc/$pid"
    [ -r "$dir/environ" ] && [ -r "$dir/cmdline" ] || continue
    cmdline="$(tr '\0' ' ' < "$dir/cmdline" 2> /dev/null)" || continue
    case "$cmdline" in
      *"$BIN"*) continue ;; # the wrapper itself, not the client it launched
    esac
    if grep -qzF -- "$BASE_URL" "$dir/environ" 2> /dev/null; then
      matrix_pass "$case_name: pid $pid ($command_name) carries $BASE_URL in its environment"
      return 0
    fi
    # A client configured through a file: the environment names the path, and
    # the file is where our base URL actually is (`OPENCODE_CONFIG` names a
    # JSON file, `CODEX_HOME` a directory holding `config.toml`). The search is
    # deliberately shallow and skips the home directory itself — `HOME` is in
    # every environment, and grepping it recursively would match some unrelated
    # leftover and turn this assertion back into one that cannot fail.
    while read -r value; do
      local key="${value%%=*}"
      value="${value#*=}"
      # Only the variables a client is configured *through*. Without this the
      # first run matched `$HOME` (grepped recursively, hit an unrelated
      # leftover) and then the harness's own artifact directory — both make the
      # assertion unfailable, which is the bug this replaced in the first place.
      case "$key" in
        *CONFIG* | *_HOME | *SETTINGS*) ;;
        *) continue ;;
      esac
      case "$value" in
        "" | / | "$HOME" | "$HOME"/ | "$ARTIFACTS"*) continue ;;
      esac
      local hit=""
      if [ -f "$value" ]; then
        grep -qsF -- "$BASE_URL" "$value" && hit="$value"
      elif [ -d "$value" ]; then
        hit="$(grep -rlsF --include='*' -- "$BASE_URL" "$value" 2> /dev/null | head -1)"
      fi
      [ -n "$hit" ] || continue
      matrix_pass "$case_name: pid $pid ($command_name) was configured from $hit"
      return 0
    done < <(tr '\0' '\n' < "$dir/environ")
  done
  matrix_fail "$case_name: no process under the launched $command_name carries $BASE_URL — the wrapper's configuration never reached the application"
}

# The launched pid and every descendant, breadth-first. A desktop client forks:
# `formal-ai with` execs the AppImage's `AppRun`, which execs Electron, which
# spawns zygote, GPU and renderer children — and the configuration may only be
# visible on one of them.
matrix_process_tree() {
  local frontier=("$1") next=() pid child
  while [ "${#frontier[@]}" -gt 0 ]; do
    next=()
    for pid in "${frontier[@]}"; do
      echo "$pid"
      for child in $(pgrep -P "$pid" 2> /dev/null); do next+=("$child"); done
    done
    frontier=("${next[@]}")
  done
}

matrix_assert_still_running() {
  local case_name="$1" seconds="${2:-15}"
  sleep "$seconds"
  kill -0 "$MATRIX_LAUNCH_PID" 2> /dev/null \
    || matrix_fail "$case_name: $CLIENT exited within ${seconds}s of launch"
  matrix_pass "$case_name: client stayed up for ${seconds}s"
}

# A launch failure is a wrapper bug, not a client quirk — these are the exact
# shapes issue #713 and issue #650 reported.
matrix_assert_launched() {
  local case_name="$1"
  for marker in "unexpected argument" "Not enough arguments" "unsupported tool" \
    "error: unrecognized" "command not found"; do
    matrix_log_matches_ci "$MATRIX_CLIENT_LOG" "$marker" \
      && matrix_fail "$case_name: launch rejected with '$marker'"
  done
  matrix_pass "$case_name: client launched cleanly"
}
