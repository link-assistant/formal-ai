#!/usr/bin/env bash
# Let Formal AI author a repository change through the real Agent CLI.
#
# Every other harness under experiments/agent_cli_e2e/ proves the round-trip and
# then throws the result away: the artifact stays in a scratch workspace and only
# the logs are kept. That is why self-authored work has so far needed a pull
# request of its own -- a human had to carry the bytes back into the repository,
# and the moment they did, the commit was theirs.
#
# This script closes that gap. It drives the same live loop (`formal-ai serve`
# plus the real `@link-assistant/agent` CLI), then lands the file the CLI wrote
# and commits it with the three canonical trailers the self-hosting metric reads:
#
#   Formal-AI-Session:      the resumable session id the run reported
#   Formal-AI-Model:        the model that produced the change (always formal-ai here)
#   Formal-AI-Evidence:     the directory holding that run's raw traces
#   Formal-AI-Pull-Request: the pull request the commit belongs to
#
# It opens no pull request and pushes nothing. The commit lands on whatever
# branch is checked out, so Formal AI's work rides along inside an ordinary pull
# request instead of needing a separate one (issue #1069).
#
# Usage:
#   scripts/author-change-with-formal-ai.sh \
#     --task "<prompt>" \
#     --produces <workspace-relative file the CLI must write> \
#     --into <repo-relative destination> \
#     [--produces <file> --into <destination>]...   (repeatable, in pairs) \
#     --evidence <repo-relative evidence directory> \
#     --pull-request <https://github.com/owner/repo/pull/N> \
#     --message "<commit subject>" \
#     [--seed <directory copied into the workspace first>] \
#     [--contains <text the artifact must contain>]... \
#     [--port <port>] [--no-commit]
set -euo pipefail

# The repository this authors in. Derived from the script's own location when
# it runs from the repository, but the composite action stages it into
# `$RUNNER_TEMP` before anything switches branches (so it runs *this* run's
# copy, not the bot branch's), and there `dirname $0/..` is the temp directory.
# `FORMAL_AI_REPO_ROOT` is how the action says where the checkout actually is;
# run 34294396281 died as `--seed is not a directory` without it.
if [[ -n "${FORMAL_AI_REPO_ROOT:-}" ]]; then
  ROOT="$FORMAL_AI_REPO_ROOT"
else
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8899}"

task=""
# Repeatable in pairs: a change that has to land with a changelog fragment
# beside it is two artifacts of one authoring run, not two runs (issue #1085).
produces=()
into=()
evidence=""
pull_request=""
message=""
seed=""
commit=1
contains=()

die() {
  echo "author-change-with-formal-ai: $*" >&2
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --task) task="$2"; shift 2 ;;
    --produces) produces+=("$2"); shift 2 ;;
    --into) into+=("$2"); shift 2 ;;
    --evidence) evidence="$2"; shift 2 ;;
    --pull-request) pull_request="$2"; shift 2 ;;
    --message) message="$2"; shift 2 ;;
    --seed) seed="$2"; shift 2 ;;
    --contains) contains+=("$2"); shift 2 ;;
    --port) PORT="$2"; shift 2 ;;
    --no-commit) commit=0; shift ;;
    *) die "unknown option: $1" ;;
  esac
done

[[ -n "$task" ]] || die "--task is required"
[[ "${#produces[@]}" -gt 0 ]] || die "--produces is required"
[[ "${#into[@]}" -le "${#produces[@]}" ]] || die "more --into than --produces"
while [[ "${#into[@]}" -lt "${#produces[@]}" ]]; do
  into+=("${produces[${#into[@]}]}")
done
[[ -n "$evidence" ]] || die "--evidence is required"
[[ -n "$pull_request" ]] || die "--pull-request is required"
[[ -n "$message" ]] || die "--message is required"

# The trailer is only meaningful if the metric can parse it, so reject a
# malformed pull-request reference here rather than at release time.
[[ "$pull_request" =~ ^https://github\.com/[^/]+/[^/]+/pull/[1-9][0-9]*$ ]] \
  || die "--pull-request must be a canonical GitHub pull-request URL: $pull_request"

command -v "$AGENT" >/dev/null || die "the @link-assistant/agent CLI is not on PATH"
[[ -x "$BIN" ]] || die "build first: cargo build --release --bin formal-ai"

out="$ROOT/$evidence"
mkdir -p "$out"

# The Agent CLI must not observe its own logs: issue #936 recorded a completion
# gate feeding back on itself when the live log sat inside the watched worktree.
# Keep the workspace, the server's memory, and the evidence directory distinct.
work="$(mktemp -d)"
state="$(mktemp -d)"
server_pid=""
cleanup() {
  [[ -n "$server_pid" ]] && kill "$server_pid" 2>/dev/null
  rm -rf "$work" "$state"
}
trap cleanup EXIT

if [[ -n "$seed" ]]; then
  seed_path="$seed"
  [[ "$seed_path" = /* ]] || seed_path="$ROOT/$seed"
  [[ -d "$seed_path" ]] || die "--seed is not a directory: $seed_path"
  cp -a "$seed_path"/. "$work"/
fi

printf '%s\n' "$task" >"$out/task.txt"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$state/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$out/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null \
  || die "formal-ai serve never came up on port $PORT"

agent_config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
# `--summarize-session` and `--generate-title` default to true, and both make a
# *second* model call that ignores `--model` and goes to the CLI's own default
# provider. With no credentials that provider answers "OpenCode's free tier can
# only be used in OpenCode", which aborted the run *after* Formal AI had already
# written the artifact -- so the loop looked broken when only the session
# bookkeeping was. Neither call is needed to author a change.
(
  cd "$work"
  PATH="$(dirname "$BIN"):$PATH" \
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$agent_config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
    --no-summarize-session --no-generate-title \
    --output-format stream-json --compact-json --disable-stdin --prompt "$task"
) >"$state/agent-stream.raw.log" 2>"$out/agent-stderr.log"

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$out/agent-stderr.log"
# Only the framed events are kept: the raw stream repeats them verbatim around
# the CLI's own progress chatter, and committing both would double the evidence
# for no extra proof.
grep '^{' "$state/agent-stream.raw.log" >"$out/agent-stream.jsonl"

session_id="$(grep -Eo '"session_id":"ses_[^"]+' "$out/agent-stream.jsonl" | tail -1 | cut -d'"' -f4)"
[[ -n "$session_id" ]] || die "the Agent CLI stream reported no resumable session id"
# One evidence file carries both markers the metric looks for: the literal
# `formal-ai` that identifies the producer, and the session the trailer names.
printf 'formal-ai session %s\n' "$session_id" >"$out/session-id.txt"
# Issue #1085 (D3.1): the metric attributes by the model that produced the
# tokens, so the evidence names it next to the session and the commit carries
# it as `Formal-AI-Model`. This loop only ever drives the formal-ai model.
model_version="$("$BIN" --version 2>/dev/null | awk '{ print $NF }')"
model="formal-ai/${model_version:-unknown}"
printf 'formal-ai model %s\n' "$model" >>"$out/session-id.txt"

for index in "${!produces[@]}"; do
  [[ -f "$work/${produces[index]}" ]] \
    || die "the Agent CLI did not write ${produces[index]}"
done
# Every `--contains` is checked against the whole set, because the text that
# proves the change landed lives in one of the artifacts, not in each of them.
for expected in ${contains[@]+"${contains[@]}"}; do
  found=0
  for produced in "${produces[@]}"; do
    if grep -Fq "$expected" "$work/$produced"; then
      found=1
      break
    fi
  done
  [[ "$found" -eq 1 ]] || die "no artifact contains: $expected"
done

destinations=()
for index in "${!produces[@]}"; do
  destination="${into[index]}"
  mkdir -p "$(dirname "$ROOT/$destination")"
  cp "$work/${produces[index]}" "$ROOT/$destination"
  destinations+=("$destination")
done
echo "Formal AI wrote ${destinations[*]} in session $session_id; evidence in $evidence"

if [[ "$commit" -eq 0 ]]; then
  echo "--no-commit: leaving ${destinations[*]} and $evidence staged for review"
  exit 0
fi

git -C "$ROOT" add -- "${destinations[@]}" "$evidence"
git -C "$ROOT" diff --cached --quiet && die "the run reproduced the committed bytes; nothing to author"
git -C "$ROOT" commit --quiet --message "$message" --message "$(
  printf 'Formal-AI-Session: %s\nFormal-AI-Model: %s\nFormal-AI-Evidence: %s\nFormal-AI-Pull-Request: %s\n' \
    "$session_id" "$model" "$evidence" "$pull_request"
)"
git -C "$ROOT" --no-pager log -1 --format='%h %s%n%b'
