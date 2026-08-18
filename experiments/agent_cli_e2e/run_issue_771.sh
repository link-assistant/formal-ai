#!/usr/bin/env bash
# Real Agent CLI reproduction for issue #771: a research turn whose fetch returns
# a whole scraped page, then a report turn that files it.
#
# The in-process unit tests pin the composed strings; only this harness proves
# the same shapes survive the OpenAI-compatible transport, the client's own tool
# execution, and the server's reading of the resulting history. Two properties
# are asserted on the argv a PATH-local fake gh records:
#
#   1. the transcribed turns stay inside a fenced code block, so no line of a
#      multi-line turn escapes and renders as top-level markdown;
#   2. the body stays far under GitHub's 65536-character issue limit even though
#      the fetched page is much larger than that.
#
# The prompts deliberately differ from run_issue_687.sh's wording so a pass
# proves general routing rather than one memorised phrasing.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
BIN_DIR="$(cd "$(dirname "$BIN")" && pwd)"
PORT="${PORT:-8781}"
AGENT="${AGENT:-agent}"
LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/fake-bin"
GH_LOG="$WORKDIR/gh-invocations.log"
BODY_FILE="$WORKDIR/issue-body.md"
GIST_DIR="$WORKDIR/gists"
GITHUB_BODY_LIMIT=65536

mkdir -p "$FAKE_BIN" "$GIST_DIR"
cd "$WORKDIR"

# The MCP timeouts below are load-bearing: without `tool_call_timeout`/
# `mcp_defaults` the Agent CLI computes its per-tool deadline as `NaN` and a
# call that would return in milliseconds aborts with `timed out after NaN
# seconds`. Observed on run_issue_781.sh in CI run 32107664418; the same shape
# is fixed here before it can fire. Values match run_issue_707.sh.
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
      "enabled": true,
      "tool_call_timeout": 120000
    }
  },
  "mcp_defaults": {
    "tool_call_timeout": 120000,
    "max_tool_call_timeout": 600000
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
if [ "\${1:-} \${2:-}" = "gist create" ]; then
  filename=''
  source_file=''
  while [ \$# -gt 0 ]; do
    if [ "\$1" = "--filename" ]; then
      filename="\$2"
      shift 2
      continue
    fi
    source_file="\$1"
    shift
  done
  cp "\$source_file" "$GIST_DIR/\$filename"
  printf 'https://gist.github.com/formal-ai/%s\n' "\$filename"
  exit 0
fi
while [ \$# -gt 0 ]; do
  if [ "\$1" = "--body" ]; then
    printf '%s' "\$2" > "$BODY_FILE"
    break
  elif [ "\$1" = "--body-file" ]; then
    cp "\$2" "$BODY_FILE"
    break
  fi
  shift
done
printf '%s\n' 'https://github.com/link-assistant/formal-ai/issues/999999'
EOF
chmod +x "$FAKE_BIN/gh"

# Private, empty memory per run so the chat handler's memory-fed planning and the
# `POST /v1/chat/completions` round count stay deterministic and independent of
# what earlier E2E scripts recorded into the shared ~/.formal-ai/memory.lino
# (issue #828). FORMAL_AI_DREAMING=0 stops background compaction of that file.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
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
  FORMAL_AI_BASE_URL="http://127.0.0.1:$PORT/v1" \
    FORMAL_AI_DIALOG_LOG_DIR="$WORKDIR/dialog-logs" \
    LINK_ASSISTANT_AGENT_DISABLE_AUTOUPDATE=1 \
    PATH="$FAKE_BIN:$BIN_DIR:$PATH" timeout 180 "$AGENT" \
    --prompt "$prompt" \
    --disable-stdin \
    --model formal-ai/formal-ai \
    --no-summarize-session \
    "$@" >> "$AGENT_LOG" 2>&1 || fail "$label failed"
}

# The reported session: a question the local engine cannot answer, then the
# report request and its two required confirmations.
run_turn research "В каких странах есть частные космические компании?"

# The report flow exercises both issue #822 confirmation gates in the same
# session after deterministic fixture-backed research.
run_turn report "report" --continue --no-fork
run_turn report_destination "GitHub issue" --continue --no-fork
run_turn report_context "Both logs" --continue --no-fork

[ -f "$GH_LOG" ] || fail "confirmed report request did not execute gh"
grep -q 'issue create --repo link-assistant/formal-ai' "$GH_LOG" \
  || fail "gh invocation did not target the Formal AI repository"
gist_count="$(grep -c '^gist create ' "$GH_LOG" || true)"
[ "$gist_count" -eq 3 ] \
  || fail "expected three separate context gists, found $gist_count"

# Requirement 2: the transcript stays inside fenced blocks and within the body
# limit; its complete contexts live behind the three links required by #989.
[ -s "$BODY_FILE" ] || fail "the gh invocation carried no --body argument"
body="$(cat "$BODY_FILE")"

size="$(wc -m < "$BODY_FILE" | tr -d ' ')"
[ "$size" -lt "$GITHUB_BODY_LIMIT" ] \
  || fail "issue body was $size characters, over GitHub's $GITHUB_BODY_LIMIT limit"

# Every transcribed turn must sit between an opening and a closing fence --
# never bare prose that would render as top-level markdown.
#
# Issue #839 gave this surface the web reporter's six-section document, so the
# body carries separate fenced dialog and reasoning-trace blocks. Issue #989
# moved harness, server, and merged context into three linked files, so no LiNo
# block belongs in the issue body. The property being pinned is unchanged and
# asserted more precisely: fences balance and no transcript line escapes into
# the surrounding Markdown.
escaped="$(REPORT_BODY="$body" python3 - <<'PY'
import os
import re

lines = os.environ["REPORT_BODY"].splitlines()
# A block opens with a run of backticks plus an optional language and closes
# with the identical run. The renderer lengthens the run until no content can
# terminate it early, so a line inside a block never looks like a fence.
fence_pattern = re.compile(r"^(`{3,})(\S*)\s*$")
# `U: `, `A (intent: unknown): `, `T: ` and the indented continuation rows of a
# multi-line turn -- the shapes issue #771 saw leak out of the block.
turn_pattern = re.compile(r"^(?:[UAT](?: \([^)]*\))?: |   \S)")

fence = None
outside = []
lino_blocks = 0
for line in lines:
    if fence is None:
        opening = fence_pattern.match(line)
        if opening:
            fence = opening.group(1)
            if opening.group(2) == "lino":
                lino_blocks += 1
            continue
        outside.append(line)
    elif line.rstrip() == fence:
        fence = None

problems = []
if fence is not None:
    problems.append("a fenced block was never closed")
if lino_blocks != 0:
    problems.append(f"expected linked context and no lino block, found {lino_blocks}")
leaked = [line for line in outside if turn_pattern.match(line)]
if leaked:
    problems.append(f"transcript lines outside any fence: {leaked[:3]}")
print("\n".join(problems))
PY
)" || fail "could not scan the transcript"
[ -z "$escaped" ] \
  || fail "the transcript escaped its fenced block:
$escaped"

shopt -s nullglob
harness_files=("$GIST_DIR"/formal-ai-harness-context-*.lino)
server_files=("$GIST_DIR"/formal-ai-server-context-*.lino)
[ "${#harness_files[@]}" -eq 1 ] \
  || fail "expected one captured harness context"
[ "${#server_files[@]}" -eq 1 ] \
  || fail "expected one captured server context"
[ -s "$GIST_DIR/context.lino" ] \
  || fail "expected one captured merged context"

if ! grep -q 'conversation' "${harness_files[0]}"; then
  grep -q 'context_export_failure' "${harness_files[0]}" \
    && grep -q 'source harness' "${harness_files[0]}" \
    || fail "the harness context was neither exported nor given an explicit diagnostic"
fi
grep -q 'server_logs' "${server_files[0]}" \
  || fail "the server context did not contain server logs"
grep -q 'conversation' "$GIST_DIR/context.lino" \
  || fail "the merged context did not contain the conversation"
grep -q 'server_logs' "$GIST_DIR/context.lino" \
  || fail "the merged context did not contain server logs"

for heading in '### Harness context' '### Server context' '### Merged context'; do
  grep -q "$heading" <<<"$body" \
    || fail "the report body omitted $heading"
done
for context_file in "${harness_files[0]}" "${server_files[0]}" "$GIST_DIR/context.lino"; do
  grep -q "https://gist.github.com/formal-ai/${context_file##*/}" <<<"$body" \
    || fail "the report body omitted the ${context_file##*/} link"
done

# Requirement 1: the answer under review is an extract, not the whole page. The
# extract's exact content is not part of this regression, so the size bound above
# is the assertion; the citation is reported for the log only.
grep -q 'Source:' <<<"$body" \
  && echo "-- transcribed answer cites its source" \
  || echo "-- note: no source citation in this run (search returned no URL)"

posts="$(grep -c 'POST /v1/chat/completions' "$LOG" || true)"
searches="$(grep -c 'agentic_outcome: planned ToolCalls.*websearch' "$LOG" || true)"
[ "$searches" -ge 1 ] || fail "the question never reached websearch"

echo "== issue #771 E2E OK: report body is $size characters and its three contexts are linked ($posts rounds) =="
tail -40 "$AGENT_LOG"
