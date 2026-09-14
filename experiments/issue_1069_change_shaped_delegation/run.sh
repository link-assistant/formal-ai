#!/usr/bin/env bash
# Probe one *change-shaped* delegation: Formal AI, driven through the real Agent
# CLI, must modify an existing tracked source file. The verifier reads only that
# tracked file, so the evidence-shaped escape the issue #1028 ladder allowed --
# writing a self-describing side file -- cannot pass here.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
PORT="${PORT:-8969}"
TARGET_FILE="src/orchestration/workspace.rs"
RUN_DIR="${RUN_DIR:-$(mktemp -d)}"
WORKSPACE="$RUN_DIR/repository"
SERVER_LOG="$RUN_DIR/formal-ai.log"
DISPATCH_REPORT="$RUN_DIR/dispatch-report.json"
OUTPUT="$WORKSPACE/.formal-ai-orchestration"
PULL_REQUEST_URL="${PULL_REQUEST_URL:-https://github.com/link-assistant/formal-ai/pull/1070}"

[ -x "$BIN" ] || { echo "build first: cargo build --bin formal-ai" >&2; exit 2; }
command -v agent >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }

TASK="In this repository, edit the existing tracked file ${TARGET_FILE}. It has a private function \`ignored\` whose \`matches!\` arm lists the directory names that are skipped when the workspace is walked: \".git\", \"target\", \".formal-ai\" and \".formal-ai-orchestration\". Add \"node_modules\" to that same list so dependency directories are skipped too. Change only that file, and keep it valid Rust."

mkdir -p "$WORKSPACE/$(dirname "$TARGET_FILE")" "$WORKSPACE/.baseline/$(dirname "$TARGET_FILE")"
cp "$ROOT/$TARGET_FILE" "$WORKSPACE/$TARGET_FILE"
cp "$ROOT/$TARGET_FILE" "$WORKSPACE/.baseline/$TARGET_FILE"
cp "$ROOT/experiments/issue_1069_change_shaped_delegation/verify.sh" "$WORKSPACE/verify.sh"
chmod +x "$WORKSPACE/verify.sh"

git -C "$WORKSPACE" init --quiet
git -C "$WORKSPACE" config user.name "Formal AI"
git -C "$WORKSPACE" config user.email "formal-ai@example.invalid"
git -C "$WORKSPACE" add -A
git -C "$WORKSPACE" commit --quiet -m "test: seed change-shaped delegation probe"
BASE_COMMIT="$(git -C "$WORKSPACE" rev-parse HEAD)"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$RUN_DIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
server_pid=$!
cleanup() {
  local status=$?
  kill "$server_pid" 2>/dev/null || true
  echo "run directory: $RUN_DIR" >&2
  exit "$status"
}
trap cleanup EXIT

for _ in $(seq 1 100); do
  curl -sf "http://127.0.0.1:$PORT/v1/models" >/dev/null && break
  sleep 0.2
done

set +e
"$BIN" agent dispatch \
  --incremental --cli agent --task "$TASK" \
  --workspace "$WORKSPACE" --output-dir "$OUTPUT" \
  --pull-request "$PULL_REQUEST_URL" --base-url "http://127.0.0.1:$PORT" \
  --allow-command bash --allow-command rustfmt --allow-command cmp --allow-command find \
  --verify '["bash","verify.sh"]' > "$DISPATCH_REPORT"
dispatch_status=$?
set -e
echo "dispatch exit: $dispatch_status"

echo "--- commits since $BASE_COMMIT ---"
git -C "$WORKSPACE" log --oneline "$BASE_COMMIT..HEAD" || true

# The probe's own assertion: at least one commit must carry a *modification* to
# the tracked file, with all three attribution trailers.
changed=0
for commit in $(git -C "$WORKSPACE" rev-list "$BASE_COMMIT..HEAD"); do
  for trailer in Formal-AI-Session Formal-AI-Evidence Formal-AI-Pull-Request; do
    git -C "$WORKSPACE" show -s --format=%B "$commit" | grep -q "^$trailer:" \
      || { echo "commit $commit is missing $trailer" >&2; exit 1; }
  done
  if git -C "$WORKSPACE" show --format= --name-status "$commit" | grep -q "^M[[:space:]]*$TARGET_FILE$"; then
    changed=1
  fi
done

if [ "$changed" -ne 1 ]; then
  echo "FAIL: no commit modified the tracked file $TARGET_FILE" >&2
  exit 1
fi
echo "PASS: change-shaped delegation modified $TARGET_FILE through the Agent CLI"
