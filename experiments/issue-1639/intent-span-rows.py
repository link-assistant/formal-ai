"""Issue #1639: check every issue-1588 intent-span fixture row against a
formal-ai binary with the Rust test's rules (offline, isolated caches), so
every failing row shows at once instead of the first one the test stops at.

usage: python3 -I intent-span-rows.py <formal-ai binary> <repo root>
"""
import json, os, resource, subprocess, sys, tempfile

binary, repo = sys.argv[1], sys.argv[2]
rows = json.load(open(os.path.join(repo, "data/parity/issue-1588-intent-spans.json")))

def limit():
    resource.setrlimit(resource.RLIMIT_AS, (2_000_000 * 1024,) * 2)

failed = 0
for index, row in enumerate(rows):
    with tempfile.TemporaryDirectory() as cache:
        env = dict(os.environ, FORMAL_AI_OFFLINE="1", FORMAL_AI_CACHE_DIR=cache,
                   FORMAL_AI_SOURCE_CACHE_DIR=cache)
        proc = subprocess.run([binary, "chat", "--format", "responses", "--prompt", row["prompt"]],
                              env=env, capture_output=True, text=True, timeout=120,
                              preexec_fn=limit, cwd=repo)
    try:
        response = json.loads(proc.stdout)
    except ValueError:
        print(f"{index}\tFAIL\tno JSON\t{row['prompt']}")
        failed += 1
        continue
    message = next(o for o in response["output"] if o.get("type") == "message")
    steps = [s["detail"] for s in message.get("thinking_steps", []) if s.get("step") == "dispatch_handler"]
    intent = steps[-1] if steps else "?"
    answer = "".join(c.get("text", "") for c in message.get("content", []))
    kind = row["kind"]
    problems = []
    if kind == "document" and intent in ("unit_conversion", "calendar_create_event"):
        problems.append(f"intent {intent}")
    if kind == "event" and (intent != "calendar_create_event" or "BEGIN:VEVENT" not in answer):
        problems.append(f"intent {intent}")
    if kind == "conversion" and intent != "unit_conversion":
        problems.append(f"intent {intent}")
    if kind == "date" and intent != "calendar_date_offset":
        problems.append(f"intent {intent}")
    if "answer" in row and answer != row["answer"]:
        problems.append(f"answer {answer[:120]!r}")
    if "text" in row and row["text"] not in answer:
        problems.append(f"text missing {row['text']!r}")
    failed += bool(problems)
    print(f"{index}\t{'FAIL' if problems else 'ok'}\t{kind}/{intent}\t{row['prompt'][:60]}\t{'; '.join(problems)}")
print(f"failed rows: {failed}")
sys.exit(1 if failed else 0)
