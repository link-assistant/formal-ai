#!/usr/bin/env bash
# Real Agent CLI reproduction for issue #771: a research turn whose fetch returns
# a whole scraped page, then a report turn that files it.
#
# The in-process unit tests pin the composed strings; only this harness proves
# the same shapes survive the OpenAI-compatible transport, the client's own tool
# execution, and the server's reading of the resulting history. Two properties
# are asserted on the argv a PATH-local fake gh records:
#
#   1. the transcribed turns stay inside attributed blockquote blocks, so no
#      line of a multi-line turn escapes and renders as top-level markdown;
#   2. the body stays far under GitHub's 65536-character issue limit even though
#      the fetched page is much larger than that.
#
# The prompts deliberately differ from run_issue_687.sh's wording so a pass
# proves general routing rather than one memorised phrasing.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8781}"
AGENT="${AGENT:-agent}"
LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/fake-bin"
GH_LOG="$WORKDIR/gh-invocations.log"
BODY_FILE="$WORKDIR/issue-body.md"
GITHUB_BODY_LIMIT=65536

mkdir -p "$FAKE_BIN"
cd "$WORKDIR"

cat > opencode.json <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$PORT/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  },
  "mcp": {
    "issue771": {
      "type": "local",
      "command": ["node", "$ROOT/experiments/agent_cli_e2e/mock-research-mcp.mjs"],
      "enabled": true
    }
  },
  "tools": {
    "websearch": false,
    "webfetch": false
  }
}
EOF

# Record argv and return GitHub's issue-URL shape without creating an issue. The
# body is captured to its own file as well: "$*" flattens argv and the shell has
# already removed the quoting by then, so a multi-line body cannot be recovered
# from the flattened log.
cat > "$FAKE_BIN/gh" <<EOF
#!/usr/bin/env bash
printf '%s\n' "\$*" >> "$GH_LOG"
while [ \$# -gt 0 ]; do
  if [ "\$1" = "--body" ]; then
    printf '%s' "\$2" > "$BODY_FILE"
    break
  fi
  shift
done
printf '%s\n' 'https://github.com/link-assistant/formal-ai/issues/999999'
EOF
chmod +x "$FAKE_BIN/gh"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$WORKDIR"' EXIT

if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
     "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  echo "!! server never came up on port $PORT"
  tail -80 "$LOG"
  exit 1
fi

fail() {
  echo "!! $*" >&2
  echo "== recorded gh invocations ==" >&2
  cat "$GH_LOG" >&2 2>/dev/null
  echo "== Agent CLI log ==" >&2
  tail -160 "$AGENT_LOG" >&2
  echo "== formal-ai server log ==" >&2
  tail -200 "$LOG" >&2
  exit 1
}

run_turn() {
  local label="$1"
  local prompt="$2"
  shift 2
  echo "== $label: $prompt ==" | tee -a "$AGENT_LOG"
  PATH="$FAKE_BIN:$PATH" timeout 180 "$AGENT" \
    --prompt "$prompt" \
    --disable-stdin \
    --model formal-ai/formal-ai \
    --no-summarize-session \
    "$@" >> "$AGENT_LOG" 2>&1 || fail "$label failed"
}

# The reported session: a question the local engine cannot answer, then a bare
# report request.
run_turn research "В каких странах есть частные космические компании?"

# The fixture's fetched page is far larger than the client's context budget, so
# the client prunes and summarizes the session before serving the next prompt.
# The turn that triggers that pruning is consumed by it rather than reaching us.
# That is client behaviour we do not control, and a real user simply asks again,
# so ask again. What the test pins is that a report request *does* land and that
# the body it produces is well-formed; not how many prompts the client spends
# reorganising its own history first.
attempt=1
while [ ! -f "$GH_LOG" ] && [ "$attempt" -le 3 ]; do
  run_turn "report (attempt $attempt)" "report" --continue --no-fork
  attempt=$((attempt + 1))
done

[ -f "$GH_LOG" ] || fail "report request did not execute gh after $((attempt - 1)) attempts"
grep -q 'issue create --repo link-assistant/formal-ai' "$GH_LOG" \
  || fail "gh invocation did not target the Formal AI repository"

# Requirement 2: the transcript stays inside attributed blocks and within the
# body limit.
[ -s "$BODY_FILE" ] || fail "the gh invocation carried no --body argument"
body="$(cat "$BODY_FILE")"

size="$(wc -m < "$BODY_FILE" | tr -d ' ')"
[ "$size" -lt "$GITHUB_BODY_LIMIT" ] \
  || fail "issue body was $size characters, over GitHub's $GITHUB_BODY_LIMIT limit"

grep -q '^\*\*user:\*\*$' <<<"$body" \
  || fail "issue body did not attribute the user turn to its role"
grep -q '^\*\*assistant:\*\*$' <<<"$body" \
  || fail "issue body did not attribute the assistant turn to its role"

# Every transcript line between the conversation heading and the footer must be
# blank, a role label, a quote line, or an italic notice -- never bare prose that
# would render as top-level markdown.
escaped="$(REPORT_BODY="$body" python3 - <<'PY'
import os

body = os.environ["REPORT_BODY"]
start = body.find("\n", body.find("### ")) + 1
end = body.rfind("\n\n")
for line in body[start:end].splitlines():
    if line.strip() and not line.startswith(("**", ">", "_")):
        print(line)
PY
)" || fail "could not scan the transcript"
[ -z "$escaped" ] \
  || fail "these transcript lines escaped their block and render as top-level markdown:
$escaped"

# Requirement 1: the answer under review is an extract, not the whole page. The
# extract's exact content is not part of this regression, so the size bound above
# is the assertion; the citation is reported for the log only.
grep -q 'Source:' <<<"$body" \
  && echo "-- transcribed answer cites its source" \
  || echo "-- note: no source citation in this run (search returned no URL)"

posts="$(grep -c 'POST /v1/chat/completions' "$LOG" || true)"
searches="$(grep -c 'agentic_outcome: planned ToolCalls.*websearch' "$LOG" || true)"
[ "$searches" -ge 1 ] || fail "the question never reached websearch"

echo "== issue #771 E2E OK: report body is $size characters, every turn attributed and contained ($posts rounds) =="
tail -40 "$AGENT_LOG"
