#!/usr/bin/env bash
# Drive every node of the issue #840 task ladder against a live `formal-ai serve`
# and record pass/fail per node, so the whole 24-task dataset can be re-measured
# after each change instead of re-reported by hand.
#
# Usage:
#   experiments/issue_840_task_ladder/run_ladder.sh
#
# Environment knobs:
#   BIN        Path to the release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT       Server port (default: 8771)
#   TASKS      Path to the ladder dataset (default: alongside this script)
#   OUT        Results JSON path (default: <scriptdir>/results.json)
#   ONLY       Optional substring filter on task id (e.g. ONLY=838 or ONLY=838.L4)
#   SANDBOX    Optional pre-existing sandbox dir; created and populated if unset
#
# A node PASSES when every `expect` substring appears in assistant output and
# no `forbid` substring does. Matching is case-insensitive. Raw tool results
# remain in the recorded transcript as observation evidence, but are not agent
# claims and therefore cannot by themselves trigger a forbidden-answer check.
# Nodes with an empty `expect` list are judged only by `forbid` (used where any
# sensible answer is acceptable but a specific failure mode must not occur).
#
# Exits 0 always: this is a measurement harness, not a gate. Read results.json
# (and the printed summary) for the outcome. `experiments/` is excluded from
# `any-code-changed` in scripts/detect-code-changes.rs, so this file does not
# gate CI.

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8771}"
TASKS="${TASKS:-$HERE/tasks.json}"
OUT="${OUT:-$HERE/results.json}"
ONLY="${ONLY:-}"
SANDBOX="${SANDBOX:-}"

if [ ! -x "$BIN" ]; then
  echo "formal-ai binary not found at $BIN (build with: cargo build --release)" >&2
  exit 0
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for JSON handling" >&2
  exit 0
fi

# --- sandbox reproducing the #838 desktop layout -----------------------------
if [ -z "$SANDBOX" ]; then
  SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-ladder.XXXXXX")"
  trap 'rm -rf "$SANDBOX"' EXIT
fi
python3 - "$TASKS" "$SANDBOX" <<'PY'
import json, os, sys
tasks, sandbox = sys.argv[1], sys.argv[2]
spec = json.load(open(tasks))["sandbox"]
for d in spec.get("dirs", []):
    os.makedirs(os.path.join(sandbox, d), exist_ok=True)
for f in spec.get("files", []):
    path = os.path.join(sandbox, f)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, "a").close()
PY
echo "sandbox: $SANDBOX"

# --- boot the server ---------------------------------------------------------
# BSD/macOS mktemp only substitutes a trailing XXXXXX template, so no suffix.
SERVER_LOG="$(mktemp "${TMPDIR:-/tmp}/formal-ai-ladder-server.XXXXXX")"
# Agent mode is required for the server to emit tool calls at all; without it
# every local-search node fails on "Running shell commands requires Agent mode"
# and measures the permission gate instead of the routing/answer quality.
FORMAL_AI_AGENT_MODE=1 "$BIN" serve --host 127.0.0.1 --port "$PORT" --agent-mode >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
cleanup_server() { kill "$SERVER_PID" 2>/dev/null || true; wait "$SERVER_PID" 2>/dev/null || true; }
trap 'cleanup_server; [ -z "${SANDBOX_KEEP:-}" ] && rm -rf "$SANDBOX"' EXIT

for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:$PORT/v1/models" >/dev/null 2>&1; then break; fi
  sleep 0.5
done
if ! curl -fsS "http://127.0.0.1:$PORT/v1/models" >/dev/null 2>&1; then
  echo "server never came up on port $PORT; log tail:" >&2
  tail -20 "$SERVER_LOG" >&2
  exit 0
fi

# --- drive every node --------------------------------------------------------
python3 - "$TASKS" "$OUT" "$PORT" "$ONLY" "$SANDBOX" <<'PY'
import json, os, subprocess, sys, urllib.request

tasks_path, out_path, port, only, sandbox = sys.argv[1:6]
data = json.load(open(tasks_path))
nodes = [t for t in data["tasks"] if not only or only in t["id"]]

# Mirrors the tool set opencode advertises, so routing is measured under the
# same conditions that produced #838 rather than in a tool-less chat context.
TOOLS = [
    {"type": "function", "function": {
        "name": "bash", "description": "Execute a shell command",
        "parameters": {"type": "object", "properties": {
            "command": {"type": "string"}}, "required": ["command"]}}},
    {"type": "function", "function": {
        "name": "websearch", "description": "Search the web",
        "parameters": {"type": "object", "properties": {
            "query": {"type": "string"}}, "required": ["query"]}}},
]

MAX_STEPS = 4

def post(messages):
    body = json.dumps({
        "model": "formal-ai", "messages": messages, "tools": TOOLS,
    }).encode()
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/v1/chat/completions",
        data=body, headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.load(r)

def ask(prompt):
    """Run a real agentic loop: execute returned shell commands in the sandbox
    and feed results back, exactly as opencode does. The transcript of every
    step is returned so assertions see both narration and tool output."""
    messages = [{"role": "user", "content": prompt}]
    transcript = []
    assistant_output = []
    try:
        for _ in range(MAX_STEPS):
            payload = post(messages)
            message = (payload.get("choices") or [{}])[0].get("message") or {}
            text = message.get("content") or ""
            if text:
                transcript.append(text)
                assistant_output.append(text)
            calls = message.get("tool_calls") or []
            if not calls:
                break
            messages.append({k: v for k, v in message.items() if v is not None})
            for call in calls:
                fn = call.get("function") or {}
                name = fn.get("name")
                try:
                    args = json.loads(fn.get("arguments") or "{}")
                except json.JSONDecodeError:
                    args = {}
                transcript.append(f"[tool {name}] {fn.get('arguments')}")
                if name == "bash" and args.get("command"):
                    proc = subprocess.run(
                        ["/bin/sh", "-c", args["command"]], cwd=sandbox,
                        capture_output=True, text=True, timeout=60,
                        env={**os.environ, "HOME": sandbox},
                    )
                    result = (proc.stdout + proc.stderr).strip() or "(no output)"
                elif name == "websearch":
                    query = str(args.get("query") or "").lower()
                    if "фуфломицин" in query or "fuflo" in query:
                        if "english" in query:
                            result = (
                                "Fuflomicin is a disparaging colloquial label "
                                "for a medicine without proven efficacy."
                            )
                        else:
                            result = (
                                "Фуфломицин — неодобрительное разговорное "
                                "название лекарства без доказанной эффективности."
                            )
                    elif "фбс" in query:
                        result = (
                            "ФБС: продавец хранит товар на своём складе, "
                            "собирает заказ и передаёт его маркетплейсу."
                        )
                    elif "фбо" in query:
                        result = (
                            "ФБО: товар хранится на складе маркетплейса, "
                            "который собирает и доставляет заказ."
                        )
                    else:
                        result = "No deterministic fixture evidence for this query."
                else:
                    result = "(tool not executed by harness)"
                transcript.append(f"[result] {result}")
                messages.append({
                    "role": "tool", "tool_call_id": call.get("id") or name,
                    "name": name, "content": result,
                })
    except Exception as exc:                      # noqa: BLE001 - harness
        return "\n".join(transcript), "\n".join(assistant_output), f"{type(exc).__name__}: {exc}"
    return "\n".join(transcript).strip(), "\n".join(assistant_output).strip(), None

results = []
for node in nodes:
    answer, assistant_output, error = ask(node["prompt"])
    low = assistant_output.lower()
    missing = [e for e in node.get("expect", []) if e.lower() not in low]
    leaked = [f for f in node.get("forbid", []) if f.lower() in low]
    ok = error is None and not missing and not leaked
    results.append({
        "id": node["id"], "level": node["level"], "seed": node["seed"],
        "lang": node["lang"], "prompt": node["prompt"], "note": node.get("note", ""),
        "answer": answer, "assistant_output": assistant_output, "error": error,
        "missing_expect": missing, "leaked_forbid": leaked, "pass": ok,
    })
    flag = "PASS" if ok else "FAIL"
    why = ""
    if error: why = f"  error={error}"
    elif missing: why = f"  missing={missing}"
    elif leaked: why = f"  leaked={leaked}"
    print(f"{flag}  {node['id']:<12} L{node['level']}  {node['prompt'][:58]!r}{why}")

passed = sum(1 for r in results if r["pass"])
summary = {
    "total": len(results), "passed": passed, "failed": len(results) - passed,
    "by_level": {}, "by_seed": {},
}
for r in results:
    lvl = summary["by_level"].setdefault(f"L{r['level']}", {"passed": 0, "total": 0})
    lvl["total"] += 1; lvl["passed"] += 1 if r["pass"] else 0
    sd = summary["by_seed"].setdefault(r["seed"], {"passed": 0, "total": 0})
    sd["total"] += 1; sd["passed"] += 1 if r["pass"] else 0

json.dump({"summary": summary, "results": results}, open(out_path, "w"), indent=2, ensure_ascii=False)
print()
print(f"TOTAL {passed}/{len(results)} passed")
for level in sorted(summary["by_level"]):
    v = summary["by_level"][level]
    print(f"  {level}: {v['passed']}/{v['total']}")
for seed in sorted(summary["by_seed"]):
    v = summary["by_seed"][seed]
    print(f"  #{seed}: {v['passed']}/{v['total']}")
print(f"\nwrote {out_path}")
PY
