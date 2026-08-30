#!/usr/bin/env bash
# Ask a running `formal-ai serve --agent-mode` for the plan it makes of one
# prompt, with the same fourteen tools the Agent CLI advertises.
#
# `examples/issue_1066_ladder_node_offline` replays the planner in process; this
# asks the binary over HTTP instead, so a difference between the two is a
# difference in the serving path rather than in the planner.
#
# Usage: bash experiments/issue_1066_served_route/probe.sh <prompt-file> [workdir]
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PROMPT_FILE="${1:?usage: probe.sh <prompt-file> [workdir]}"
WORK="${2:-$(mktemp -d)}"
OUT="${OUT:-/tmp/issue-1066-served-route}"
PORT="${PORT:-8${RANDOM:0:3}}"

[[ -x "$BIN" ]] || { echo "build first: cargo build --release" >&2; exit 2; }
mkdir -p "$OUT" "$WORK"

( cd "$WORK" && FORMAL_AI_DREAMING=0 FORMAL_AI_TRACE_REQUESTS=1 \
    "$BIN" serve --agent-mode --host 127.0.0.1 --port "$PORT" ) \
  > "$OUT/server.log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null' EXIT
for _ in $(seq 1 100); do
  curl -fsS "http://127.0.0.1:$PORT/v1/models" >/dev/null 2>&1 && break
  read -r -t 0.2 < /dev/null 2>/dev/null || true
done

python3 - "$PROMPT_FILE" > "$OUT/request.json" <<'PY'
import json, sys
prompt = open(sys.argv[1]).read()
tools = ["bash", "batch", "codesearch", "edit", "glob", "grep", "list", "read",
         "task", "todoread", "todowrite", "webfetch", "websearch", "write"]
json.dump({
    "model": "formal-ai",
    "messages": [{"role": "user", "content": prompt}],
    "tools": [{"type": "function",
               "function": {"name": name, "description": name,
                            "parameters": {"type": "object", "properties": {}}}}
              for name in tools],
}, sys.stdout)
PY

curl -fsS -X POST "http://127.0.0.1:$PORT/v1/chat/completions" \
  -H 'content-type: application/json' --data-binary @"$OUT/request.json" \
  > "$OUT/response.json"
status=$?
echo "curl exit=$status; response in $OUT/response.json, traces in $OUT/server.log"
grep -o '\[trace\] general_change_plan=[a-z_]*' "$OUT/server.log" | sort | uniq -c
python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(json.dumps(d["choices"][0]["message"], indent=2)[:4000])' "$OUT/response.json"
