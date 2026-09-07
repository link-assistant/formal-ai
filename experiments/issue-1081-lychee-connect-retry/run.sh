#!/usr/bin/env bash
# Show that lychee's --max-retries does not apply to a connection reset that
# happens during connect or the TLS handshake.
#
#   ./run.sh /path/to/lychee
#
# Expected, and what this measured on lychee 0.24.2:
#   https:// target -> 1 attempt   (the reset arrives during the TLS handshake:
#                                   reqwest reports is_connect(), and
#                                   `should_retry` answers false before it ever
#                                   consults `should_retry_io`)
#   http://  target -> 6 attempts  (the reset arrives after connect, on the
#                                   request, so it reaches `should_retry_io`,
#                                   where ConnectionReset is retryable)
set -uo pipefail

lychee="${1:-lychee}"
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
# Run from the scratch directory: lychee picks up a `.lycheeignore` from the
# working directory, and this repository's excludes localhost.
cd "$work" || exit 1
trap 'rm -rf "$work"' EXIT

run_case() { # run_case <scheme> <port> <mode>
  local scheme="$1" port="$2" mode="$3"
  local log="$work/$scheme-server.log"

  python3 "$here/reset-server.py" "$port" "$mode" > "$log" 2>&1 &
  local server=$!
  until grep -q listening "$log" 2>/dev/null; do sleep 0.1; done

  printf '[a](%s://127.0.0.1:%s/)\n' "$scheme" "$port" > "$work/input.md"

  local start
  start="$(date +%s)"
  "$lychee" --no-progress --verbose \
    --max-retries 5 --retry-wait-time 2 --timeout 10 \
    "$work/input.md" > "$work/$scheme-lychee.log" 2>&1
  local elapsed=$(( $(date +%s) - start ))

  kill "$server" 2>/dev/null
  wait "$server" 2>/dev/null

  local attempts
  attempts="$(grep -c '^attempt ' "$log")"
  printf '%s://127.0.0.1:%s (%s) -> %s connection attempt(s) in %ss\n' \
    "$scheme" "$port" "$mode" "$attempts" "$elapsed"
  grep -E 'ERROR|Error' "$work/$scheme-lychee.log" | head -2 | sed 's/^/    /'
}

echo "lychee: $("$lychee" --version)"
echo "--max-retries 5 --retry-wait-time 2 (so five retries cost at least 30s)"
echo
# The reset lands during the TLS handshake: reqwest reports is_connect().
run_case https 8443 immediate
# The same reset, on a connection that was already established: it reaches
# should_retry_io, where ConnectionReset is retryable.
run_case http 8080 after-request
