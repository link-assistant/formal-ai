#!/usr/bin/env bash
# Drive scripts/preflight-credentials.sh through the verdicts it exists to tell
# apart (issue #1081, principle 16). No network: `fake-curl.sh` answers.
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
ln -s "$root/experiments/issue_1081_preflight/fake-curl.sh" "$work/curl"
export PATH="$work:$PATH"
export FAKE_CURL_LOG="$work/calls.log"
export CARGO_TOKEN=token GHCR_IMAGE=ghcr.io/link-assistant/formal-ai GITHUB_TOKEN=jobtoken
export DOCKERHUB_IMAGE=linkassistant/formal-ai DOCKERHUB_USERNAME=u DOCKERHUB_TOKEN=t
export PREFLIGHT_RETRIES=1 PREFLIGHT_RETRY_DELAY=0

run_case() { # run_case <name> <mode> <routes>
  : > "$FAKE_CURL_LOG"
  FAKE_CURL_ROUTES="$3" bash "$root/scripts/preflight-credentials.sh" --mode "$2" > "$work/out.txt" 2>&1
  echo "=== $1 (mode=$2) exit=$?"
  cat "$work/out.txt"
}

all_good='/me 200 {"user":{"login":"konard"}}
/owners 200 {"users":[{"login":"konard"}]}
/token 200 {"token":"bearer"}
blobs/uploads 202'

run_case "everything publishable" release "$all_good"
run_case "revoked crates.io token" release '/me 403
/token 200 {"token":"bearer"}
blobs/uploads 202'
run_case "token valid but not an owner" release '/me 200 {"user":{"login":"someone-else"}}
/owners 200 {"users":[{"login":"konard"}]}
/token 200 {"token":"bearer"}
blobs/uploads 202'
run_case "ghcr authenticates but cannot write" release '/me 200 {"user":{"login":"konard"}}
/owners 200 {"users":[{"login":"konard"}]}
/token 200 {"token":"bearer"}
blobs/uploads 403'
run_case "everything times out" release '/x 000'
run_case "revoked token on a pull request" report '/me 403
/token 200 {"token":"bearer"}
blobs/uploads 202'
echo "=== the blob upload session is cancelled"
grep -c 'DELETE' "$FAKE_CURL_LOG" || true
