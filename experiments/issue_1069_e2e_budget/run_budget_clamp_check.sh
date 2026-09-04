#!/usr/bin/env bash
# Exercise the run budget issue #1069 added to issue #707's computer-use E2E.
#
# The incident (CI/CD run 33880485514) was that twenty sessions, each bounded by
# AGENT_TIMEOUT_SECONDS and nothing else, were entitled to more time than the
# step running them, so the runner ended the run and named the step instead of
# the scenario. Reading the script proves the clamp is written; running it
# proves the clamp fires. This harness stands in for the Formal AI server and
# the Agent CLI -- the two live dependencies -- and drives the real script under
# budgets small enough to expire, then asserts on what it said as it stopped.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT
mkdir -p "$WORK/bin" "$WORK/served"

# The script only asks the server for /health, so a file of that name answers it.
: >"$WORK/served/health"
cat >"$WORK/bin/formal-ai" <<'STUB'
#!/usr/bin/env bash
# Stands in for `formal-ai serve --host H --port P`. STARTUP_DELAY_SECONDS makes
# the run start with time already spent, the way a real server build does.
port=8907
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --port) port="$2"; shift 2 ;;
    *) shift ;;
  esac
done
sleep "${STARTUP_DELAY_SECONDS:-0}"
cd "$SERVE_ROOT"
exec python3 -m http.server "$port" --bind 127.0.0.1
STUB

# Stands in for the Agent CLI: a session costs SESSION_SECONDS and then emits
# the two markers the script greps for.
cat >"$WORK/bin/agent" <<'STUB'
#!/usr/bin/env bash
sleep "${SESSION_SECONDS:-1}"
printf '{"session_id":"ses_stub","type":"computer_use_complete"}\n'
STUB
chmod +x "$WORK/bin/formal-ai" "$WORK/bin/agent"

run_case() {
  local name="$1" log="$WORK/$1.log" status=0
  shift
  echo "== case: $name =="
  set +e
  env SERVE_ROOT="$WORK/served" BIN="$WORK/bin/formal-ai" AGENT="$WORK/bin/agent" \
    EVIDENCE_DIR="$WORK/$name-evidence" "$@" \
    "$ROOT/experiments/agent_cli_e2e/run_issue_707.sh" >"$log" 2>&1
  status=$?
  set -e
  sed 's/^/   | /' "$log"
  [[ "$status" -eq 1 ]] || {
    echo "!! $name: expected the script to fail on its own (exit 1), got $status" >&2
    exit 1
  }
  grep -q '^::error title=issue #707 computer-use record/replay::' "$log" || {
    echo "!! $name: no ::error annotation, so the job would report no cause" >&2
    exit 1
  }
}

# A run that has already spent its budget refuses to open a scenario it cannot
# finish, instead of starting one and letting the runner cut it off mid-session.
run_case budget-already-spent \
  PORT=8971 STARTUP_DELAY_SECONDS=3 SESSION_SECONDS=1 \
  AGENT_TIMEOUT_SECONDS=30 TEST_BUDGET_SECONDS=61 VERIFY_RESERVE_SECONDS=60
grep -q 'the 61s run budget was spent before record/active_customers started' \
  "$WORK/budget-already-spent.log" || {
  echo "!! the failure does not name the budget and the scenario it stopped at" >&2
  exit 1
}

# A run whose budget expires inside a scenario says so, and distinguishes that
# from the scenario outlasting its own session deadline: the numbers it prints
# are the ones needed to decide which clock to change.
run_case budget-expired-inside \
  PORT=8972 STARTUP_DELAY_SECONDS=0 SESSION_SECONDS=6 \
  AGENT_TIMEOUT_SECONDS=30 TEST_BUDGET_SECONDS=70 VERIFY_RESERVE_SECONDS=60
grep -q 'the 70s run budget expired inside record/' \
  "$WORK/budget-expired-inside.log" || {
  echo "!! a budget that ran out mid-session is not reported as such" >&2
  exit 1
}
grep -qE 'of its 30s left' "$WORK/budget-expired-inside.log" || {
  echo "!! the failure does not say how much of the session deadline was left" >&2
  exit 1
}
grep -qE '^== record 1/10: active_customers \(t\+[0-9]+s of 10s\) ==$' \
  "$WORK/budget-expired-inside.log" || {
  echo "!! scenarios do not report elapsed time against the run deadline" >&2
  exit 1
}

echo "== the run budget stops the run, names the scenario, and annotates the job =="
