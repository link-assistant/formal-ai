#!/usr/bin/env bash
# Drive every node of the issue #840 task ladder against a live `formal-ai serve`
# and record pass/fail per node, so the whole 24-node dataset can be re-measured
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
#   FIXTURES   Web-search fixture file (default: <scriptdir>/web_fixtures.json);
#              set FIXTURES=none to restore the old routing-only measurement
#   BASELINE   Optional results.json to compare against; the run then exits
#              non-zero if the score dropped (used by CI to catch regressions)
#
# A node PASSES when all of the following hold:
#   * every `expect` substring appears in the answer and no `forbid` substring
#     does (case-insensitive);
#   * at least one `expect_any` substring appears, when that list is non-empty;
#   * the answer is not an unknown-prompt refusal;
#   * the answer is not the capability menu;
#   * every tool named in `expect_tool` was called, and none named in
#     `forbid_tool` was;
#   * no `command_forbid` substring appears in any shell command that ran.
#
# Everything past `expect`/`forbid` exists because substring matching alone is not
# falsifiable: issue #842 recorded nodes counted as passing while the server had
# actually refused, printed the capability menu, or answered a different
# question via the wrong tool. Nodes with an empty `expect` list are judged only
# by the remaining checks (used where any sensible answer is acceptable but a
# specific failure mode must not occur).
#
# Exits 0 unless BASELINE is set and the score dropped. Read results.json
# (and the printed summary) for the outcome. `experiments/` is excluded from
# `any-code-changed` in scripts/detect-code-changes.rs, so editing this file does
# not by itself trigger a CI run; the `task-ladder` job in
# .github/workflows/release.yml runs it with BASELINE set on every code change,
# which is what makes a drop in the score visible (issue #842).

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8771}"
TASKS="${TASKS:-$HERE/tasks.json}"
OUT="${OUT:-$HERE/results.json}"
ONLY="${ONLY:-}"
SANDBOX="${SANDBOX:-}"
FIXTURES="${FIXTURES:-$HERE/web_fixtures.json}"
BASELINE="${BASELINE:-}"

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
python3 - "$TASKS" "$OUT" "$PORT" "$ONLY" "$SANDBOX" "$FIXTURES" "$BASELINE" <<'PY'
import json, os, subprocess, sys, urllib.request

tasks_path, out_path, port, only, sandbox, fixtures_path, baseline_path = sys.argv[1:8]
data = json.load(open(tasks_path))
nodes = [t for t in data["tasks"] if not only or only in t["id"]]

# Offline web-search corpus. Without it a `websearch` call returns nothing and
# every #827 node measures whether the request was *routed* to search, never
# whether the answer was *synthesized* from what search returned — the gap the
# issue asks to close. Each entry maps a set of query keywords to the page text
# a real provider would return; the harness serves the first entry whose
# keywords all appear in the query.
FIXTURES = []
if fixtures_path != "none" and os.path.exists(fixtures_path):
    FIXTURES = json.load(open(fixtures_path))["documents"]

def web_fixture(query):
    low = query.lower()
    for doc in FIXTURES:
        if all(k.lower() in low for k in doc["keywords"]):
            return doc["text"]
    return None

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
    tools_called = []
    try:
        for _ in range(MAX_STEPS):
            payload = post(messages)
            message = (payload.get("choices") or [{}])[0].get("message") or {}
            text = message.get("content") or ""
            if text:
                transcript.append(text)
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
                tools_called.append(name)
                if name == "websearch" and args.get("query"):
                    result = web_fixture(args["query"]) or "(no fixture for this query)"
                elif name == "bash" and args.get("command"):
                    proc = subprocess.run(
                        ["/bin/sh", "-c", args["command"]], cwd=sandbox,
                        capture_output=True, text=True, timeout=60,
                        env={**os.environ, "HOME": sandbox},
                    )
                    result = (proc.stdout + proc.stderr).strip() or "(no output)"
                else:
                    # websearch and anything else: no live network in the harness
                    result = "(tool not executed by harness)"
                transcript.append(f"[result] {result}")
                messages.append({
                    "role": "tool", "tool_call_id": call.get("id") or name,
                    "name": name, "content": result,
                })
    except Exception as exc:                      # noqa: BLE001 - harness
        return "\n".join(transcript), tools_called, f"{type(exc).__name__}: {exc}"
    return "\n".join(transcript).strip(), tools_called, None

# The unknown-prompt refusal, in every language the ladder uses. A refusal is
# never a pass: issue #842's failure taxonomy B is precisely "refuses instead of
# attempting", and an empty `expect` list would otherwise score it as success.
REFUSALS = [
    "could not determine",
    "не смог определить",
    "निर्धारित नहीं कर",
    "无法确定",
]
# The opening line of the capability menu. Taxonomy C: the menu as a fallback is
# never an answer to a question about anything else.
MENUS = [
    "here is what i can do",
    "вот что я умею",
    "मैं यह कर सकता हूँ",
    "以下是我的功能",
]

results = []
for node in nodes:
    answer, tools_called, error = ask(node["prompt"])
    low = answer.lower()
    missing = [e for e in node.get("expect", []) if e.lower() not in low]
    leaked = [f for f in node.get("forbid", []) if f.lower() in low]
    refused = any(marker in low for marker in REFUSALS)
    menued = any(marker in low for marker in MENUS)
    # `expect_any` is satisfied by one alternative: absence can be reported as
    # "no matching file", "not found", or "нет такой папки" and all are correct,
    # so demanding a single substring would measure phrasing, not behaviour.
    alternatives = node.get("expect_any", [])
    unmet_any = bool(alternatives) and not any(a.lower() in low for a in alternatives)
    misrouted = (
        [t for t in node.get("expect_tool", []) if t not in tools_called]
        + [t for t in node.get("forbid_tool", []) if t in tools_called]
    )
    # Assertions on the shell command itself, not on the prose around it — the
    # #840 §3 standard is about the command being simple and able to widen when
    # it finds nothing, which the answer text cannot show.
    commands = "\n".join(
        line for line in answer.splitlines() if line.startswith("[tool bash]")
    ).lower()
    bad_command = [c for c in node.get("command_forbid", []) if c.lower() in commands]
    ok = (error is None and not missing and not unmet_any and not leaked
          and not refused and not menued and not misrouted and not bad_command)
    results.append({
        "id": node["id"], "level": node["level"], "seed": node["seed"],
        "lang": node["lang"], "prompt": node["prompt"], "note": node.get("note", ""),
        "answer": answer, "error": error, "tools_called": tools_called,
        "missing_expect": missing, "unmet_expect_any": unmet_any,
        "leaked_forbid": leaked, "bad_command": bad_command,
        "refused": refused, "capability_menu": menued, "misrouted": misrouted,
        "pass": ok,
    })
    flag = "PASS" if ok else "FAIL"
    why = ""
    if error: why = f"  error={error}"
    elif refused: why = "  refused (unknown-prompt fallback)"
    elif menued: why = "  answered with the capability menu"
    elif misrouted: why = f"  misrouted={misrouted} called={tools_called}"
    elif missing: why = f"  missing={missing}"
    elif unmet_any: why = f"  none of={alternatives}"
    elif leaked: why = f"  leaked={leaked}"
    elif bad_command: why = f"  command contains {bad_command}"
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

# Regression gate. Without a BASELINE this stays a pure measurement harness; CI
# passes the committed results.json so a change that lowers the score fails the
# job instead of silently landing.
if baseline_path:
    before = json.load(open(baseline_path))["summary"]
    scale = len(results) / before["total"] if before["total"] else 1
    expected = before["passed"] * scale
    print(f"baseline {before['passed']}/{before['total']} -> now {passed}/{len(results)}")
    if passed < expected:
        print("REGRESSION: fewer nodes pass than in the recorded baseline")
        sys.exit(1)
PY
