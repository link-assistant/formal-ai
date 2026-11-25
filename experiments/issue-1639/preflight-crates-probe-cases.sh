#!/usr/bin/env bash
# Issue #1639: drive scripts/preflight-credentials.sh through the crates.io
# no-archive publish probe verdicts with the offline curl stub, mirroring the
# cases in rust/tests/unit/ci-cd/job_budget_fit/release_preflight.rs (which only
# run in the main-only Rust test shards).
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cp "$root/experiments/issue_1081_preflight/fake-curl.sh" "$work/curl"
chmod +x "$work/curl"
unset CARGO_REGISTRY_TOKEN CRATES_IO_API GITHUB_STEP_SUMMARY PREFLIGHT_VERBOSE
export PATH="$work:$PATH" CARGO_TOKEN=cargo-token CRATE_NAME=formal-ai
export GHCR_IMAGE=ghcr.io/link-assistant/formal-ai GITHUB_ACTOR=konard GITHUB_TOKEN=job-token
export DOCKERHUB_IMAGE=linkassistant/formal-ai DOCKERHUB_USERNAME=user DOCKERHUB_TOKEN=hub-token
export PREFLIGHT_RETRIES=1 PREFLIGHT_RETRY_DELAY=0
accepted='/crates/new 400 {"errors":[{"detail":"invalid tarball length"}]}'
rest='/token 200 {"token":"bearer"}
blobs/uploads 202'
run_case() { # run_case <name> <mode> <routes>
  : >"$work/calls.log"; : >"$work/config.log"
  FAKE_CURL_LOG="$work/calls.log" FAKE_CURL_CONFIG_LOG="$work/config.log" FAKE_CURL_ROUTES="$3" \
    bash "$root/scripts/preflight-credentials.sh" --mode "$2" >"$work/out.txt" 2>&1
  echo "=== $1 (mode=$2) exit=$?"
  cat "$work/out.txt"
  echo "--- token in argv: $(grep -c cargo-token "$work/calls.log"); token in -K config: $(grep -c 'Authorization: cargo-token' "$work/config.log"); /crates/new calls: $(grep -c /crates/new "$work/calls.log"); /me calls: $(grep -c /api/v1/me "$work/calls.log")"
}
run_case "everything publishable" release "$accepted
$rest"
run_case "cookie-only /me would say 403" release "/me 403 {\"errors\":[{\"detail\":\"authentication failed\"}]}
$accepted
$rest"
run_case "publish token denied" release "/crates/new 403 {\"errors\":[{\"detail\":\"this token does not have the required permissions\"}]}
$rest"
run_case "unexpected 400 detail" release "/crates/new 400 {\"errors\":[{\"detail\":\"missing or empty metadata fields\"}]}
$rest"
run_case "pull request (report mode)" report "$accepted
$rest"
run_case "everything unreachable" release '/ 000'
