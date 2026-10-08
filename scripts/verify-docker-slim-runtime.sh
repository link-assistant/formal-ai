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

port="${VERIFY_PORT:-18080}"
formal-ai serve --agent-mode --host 127.0.0.1 --port "$port" &
serve_pid=$!
trap 'kill "$serve_pid" 2>/dev/null || true' EXIT

ready=false
for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:$port/health" >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 1
done
[ "$ready" = true ] || fail "serve did not answer /health within 30s"

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

echo "verify-formal-ai-slim: CLI, health, chat and agent routing passed on $port"
