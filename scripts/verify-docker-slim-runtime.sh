#!/usr/bin/env bash
# Smoke test the shipped sidecar binary, server, answers and agent tool route.
set -euo pipefail

fail() {
  echo "verify-formal-ai-slim: $*" >&2
  exit 1
}

command -v curl >/dev/null || fail "curl is required for the health check"
command -v node >/dev/null || fail "node is required for response validation"
formal-ai --version || fail "formal-ai --version did not answer"

smoke_home="$(mktemp -d)"
cleanup() {
  if [ -n "${serve_pid:-}" ]; then
    kill "$serve_pid" 2>/dev/null || true
    wait "$serve_pid" 2>/dev/null || true
  fi
  rm -rf "$smoke_home"
}
trap cleanup EXIT
cd "$smoke_home"

port="${VERIFY_PORT:-18080}"
FORMAL_AI_OFFLINE=1 FORMAL_AI_DREAMING=0 \
  FORMAL_AI_MEMORY_PATH="$smoke_home/memory.lino" \
  FORMAL_AI_SOURCE_CACHE_DIR="$smoke_home/sources" \
  formal-ai serve --agent-mode --host 127.0.0.1 --port "$port" &
serve_pid=$!

ready=false
for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:$port/health" >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 1
done
[ "$ready" = true ] || fail "serve did not answer /health within 30s"

node --input-type=module - "$port" <<'NODE'
import assert from 'node:assert/strict';

const base = `http://127.0.0.1:${process.argv[2]}`;
async function request(endpoint, payload) {
  const response = await fetch(base + endpoint, {
    method: 'POST', headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(payload), signal: AbortSignal.timeout(15_000),
  });
  const body = await response.text();
  assert.ok(response.ok, body);
  return JSON.parse(body);
}

for (const [prompt, expected] of [
  ['Hello', /Hi[!,]/u],
  ['Привет', /Здравствуйте/u],
  ['Report issue', /https:\/\/github\.com\/link-assistant\/formal-ai\/issues\/new$/u],
]) {
  const mcp = await request('/mcp', {
    jsonrpc: '2.0', id: 1, method: 'tools/call',
    params: {name: 'formal_ai_chat', arguments: {prompt}},
  });
  const telegram = await request('/telegram/webhook', {
    message: {message_id: 1, chat: {id: 7, type: 'private'}, text: prompt},
  });
  assert.match(mcp.result.content[0].text, expected, `MCP ${prompt}`);
  assert.match(telegram.text, expected, `Telegram ${prompt}`);
}

async function chat(content) {
  const result = await request('/v1/chat/completions', {
    model: 'formal-ai', messages: [{role: 'user', content}],
  });
  return result.choices[0].message.content;
}
for (const [prompt, terms, borrowedAnswer] of [
  ['How to buy extra baggage on IndiGo?', /baggage.*indigo/iu, /\b(?:15|20|30)\s*kg\b/iu],
  ['How to install Raku on Alpine Linux?', /raku.*alpine/iu, /^1\. /mu],
]) {
  const answer = await chat(prompt);
  assert.match(answer, terms);
  assert.doesNotMatch(answer, borrowedAnswer);
}
const capital = await chat('What is the capital of Portugal?');
assert.doesNotMatch(capital, /\bLisbon\b/iu);
NODE

curl --fail --silent --show-error --max-time 30 \
  -H 'Content-Type: application/json' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"2 + 2"}]}' \
  "http://127.0.0.1:$port/v1/chat/completions" \
  | node --input-type=module -e '
    import { readFileSync } from "node:fs";
    const response = JSON.parse(readFileSync(0, "utf8"));
    const answer = response.choices?.[0]?.message?.content ?? "";
    if (!/\b4\b/u.test(answer)) throw new Error("Arithmetic chat smoke failed: " + answer);
  '

curl --fail --silent --show-error --max-time 30 \
  -H 'Content-Type: application/json' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Create production-check.txt with the content '\''production-check'\''."}],"tools":[{"type":"function","function":{"name":"write","parameters":{"type":"object","properties":{"filePath":{"type":"string"},"content":{"type":"string"}}}}}]}' \
  "http://127.0.0.1:$port/v1/chat/completions" \
  | node --input-type=module -e '
    import { readFileSync } from "node:fs";
    const response = JSON.parse(readFileSync(0, "utf8"));
    const call = response.choices?.[0]?.message?.tool_calls?.[0]?.function;
    if (call?.name !== "write") throw new Error("Agent smoke did not plan the write tool");
    const argumentsList = JSON.parse(call.arguments);
    const path = argumentsList.filePath ?? argumentsList.file_path ?? argumentsList.path;
    if (path !== "production-check.txt" || argumentsList.content !== "production-check") {
      throw new Error("Agent smoke planned unexpected file bytes");
    }
  '

echo "verify-formal-ai-slim: CLI, health, cold source policy, chat, agent routing, greetings and issue forms passed on $port"
