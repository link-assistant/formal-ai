#!/usr/bin/env bash
# Drive the issue #847 coding-task ladder through `formal-ai with agent`, i.e.
# the exact path Hive Mind uses: Hive Mind -> Agent CLI -> formal-ai serve
# --agent-mode (link-assistant/hive-mind#2059).
#
# The point is to find the complexity level at which Formal AI can actually
# complete a coding task, by starting at "solve this issue and open a PR" and
# splitting downward until tasks stop failing.
#
# Usage:
#   experiments/issue_847_coding_ladder/run_coding_ladder.sh
#   ONLY=L4 experiments/issue_847_coding_ladder/run_coding_ladder.sh
#
# Environment knobs:
#   BIN       Path to the release binary (default: target/release/formal-ai)
#   PROMPTS   Dataset path (default: alongside this script)
#   OUT       Results JSON (default: <scriptdir>/results.json)
#   ONLY      Substring filter on task id (e.g. ONLY=L4, ONLY=L1.solve)
#   TIMEOUT   Per-task seconds (default: 300)
#
# Tasks run against the REAL repository and every edit is reverted with
# `git checkout -- .` before the next task, so the tree must be clean to start.
#
# A task PASSES when its `verify` shell snippet exits 0 AND, where
# `expect_answer` is present, that string appears in the agent's output.
# Verification is by observed effect, never by the agent's narration --
# "I created the file" with no file is a FAIL.
#
# Exits 0 always: measurement harness, not a CI gate. `experiments/` is
# excluded from any-code-changed (scripts/detect-code-changes.rs) and from
# shellcheck (.github/workflows/release.yml), though note issue #846: that
# exclusion is bypassed on direct pushes to main.

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PROMPTS="${PROMPTS:-$HERE/prompts.json}"
OUT="${OUT:-$HERE/results.json}"
ONLY="${ONLY:-}"
TIMEOUT="${TIMEOUT:-300}"

if [ ! -x "$BIN" ]; then
  echo "formal-ai binary not found at $BIN (build with: cargo build --release)" >&2
  exit 0
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required" >&2
  exit 0
fi

echo "binary:  $($BIN --version 2>/dev/null)"
echo "agent:   $(agent --version 2>/dev/null || echo 'not on PATH')"
echo "repo:    $ROOT"
echo

python3 - "$PROMPTS" "$OUT" "$ONLY" "$TIMEOUT" "$BIN" "$ROOT" <<'PY'
import json, os, re, subprocess, sys

prompts_path, out_path, only, timeout_s, binary, root = sys.argv[1:7]
timeout_s = int(timeout_s)
data = json.load(open(prompts_path))
tasks = [t for t in data["tasks"] if not only or only in t["id"]]

def reset_repo():
    """Every task starts from a clean tree so results are independent.

    Tasks operate on the REAL repository (that is the point -- we teach Formal
    AI on work we actually need done), so any edit an agent makes must be
    reverted before the next task runs. Untracked files created by a task are
    removed too, but `experiments/` is preserved so this harness cannot delete
    itself mid-run.
    """
    subprocess.run(["git", "checkout", "--", "."], cwd=root, capture_output=True)
    subprocess.run(["git", "clean", "-fd", "--exclude=experiments"],
                   cwd=root, capture_output=True)

def repo_is_clean():
    proc = subprocess.run(["git", "status", "--porcelain"], cwd=root,
                          capture_output=True, text=True)
    # This harness writes its own results.json inside experiments/, so changes
    # under experiments/ are expected and must not block a run.
    dirty = [line for line in proc.stdout.splitlines()
             if line.strip() and "experiments/" not in line]
    return not dirty

if not repo_is_clean():
    print("refusing to run: working tree is dirty; commit or stash first", file=sys.stderr)
    raise SystemExit(0)

results = []
for task in tasks:
    structural = ""
    reset_repo()
    cmd = [binary, "with", "agent", "--non-interactive", "-p", task["prompt"]]
    try:
        proc = subprocess.run(
            cmd, cwd=root, capture_output=True, text=True, timeout=timeout_s,
        )
        output = proc.stdout + proc.stderr
        timed_out = False
    except subprocess.TimeoutExpired as exc:
        output = (exc.stdout or "") + (exc.stderr or "")
        if isinstance(output, bytes):
            output = output.decode("utf-8", "replace")
        timed_out = True

    verified = subprocess.run(
        ["/bin/sh", "-c", task.get("verify", "true")],
        cwd=root, capture_output=True, text=True,
    ).returncode == 0

    # Structural sanity: a substring `verify` cannot tell real code from the
    # prompt echoed back into the file. Observed failure mode -- asked for a
    # Rust test file, the agent creates the file whose entire content is the
    # sentence fragment "one Rust test named X asserting ...". grep for the
    # test name then passes. Any .rs the task created must actually parse as
    # Rust to the extent of containing a function item.
    if verified:
        for created in re.findall(r'\b((?:src|tests|scripts)/[\w./-]+\.rs)\b', task["prompt"]):
            path = os.path.join(root, created)
            if not os.path.exists(path):
                continue
            with open(path, "r", errors="replace") as handle:
                body = handle.read()
            if "fn " not in body:
                verified = False
                structural = f"{created} has no `fn` item (prompt echoed as content?)"
                break

    # Judge `expect_answer` against the assistant's ANSWER only. The agent CLI
    # emits verbose JSON logs on the same streams, and matching against the raw
    # combined output produced false positives (a bare "1." or "yes" occurs
    # incidentally in log payloads while the model actually refused).
    # The refusal text is produced by the SERVER and does not reach the agent
    # CLI's stdout, so scanning `output` alone reported refusals as successes.
    # The CLI prints the server log path on exit; read it back and look there.
    haystack = output
    for line in output.splitlines():
        if "server log:" in line:
            log_path = line.split("server log:", 1)[1].strip()
            try:
                with open(log_path, "r", errors="replace") as handle:
                    haystack += handle.read()
            except OSError:
                pass
    # Two distinct non-answers must both count as failure:
    #   * the unknown-prompt refusal ("I could not determine ...")
    #   * a misroute that answers a different question than the one asked
    #     (e.g. a decomposition request formalized as write_program(rust, missing))
    lowered = haystack.lower()
    refused = ("could not determine" in lowered
               or "не смог определить" in lowered
               or "i do not have a template for language" in lowered
               or "supported languages:" in lowered)
    expected = task.get("expect_answer")
    answered = expected is None or (
        not refused and expected.lower() in output.lower()
    )
    # An unfalsifiable `verify: true` plus a refusal is not a pass.
    # A refusal is never a pass, even when `verify` is a trivially true
    # placeholder (the L1 ceiling cases before their checks were tightened).
    ok = verified and answered and not timed_out and not refused

    reason = ""
    structural = locals().get("structural", "")
    if timed_out:
        reason = f"timeout after {timeout_s}s"
    elif not verified:
        reason = structural or "verify failed (no observable effect)"
    elif refused:
        reason = "refused (unknown-prompt fallback)"
    elif not answered:
        reason = f"answer missing {expected!r}"

    results.append({
        "id": task["id"], "level": task["level"], "seed": task.get("seed"),
        "prompt": task["prompt"], "note": task.get("note", ""),
        "pass": ok, "reason": reason, "timed_out": timed_out, "refused": refused,
        "verified_effect": verified,
        "output_tail": output[-2000:],
    })
    print(f"{'PASS' if ok else 'FAIL'}  {task['id']:<22} L{task['level']}  {reason}")

reset_repo()

passed = sum(1 for r in results if r["pass"])
by_level = {}
for r in results:
    slot = by_level.setdefault(f"L{r['level']}", {"passed": 0, "total": 0})
    slot["total"] += 1
    slot["passed"] += 1 if r["pass"] else 0

json.dump(
    {"summary": {"total": len(results), "passed": passed,
                 "failed": len(results) - passed, "by_level": by_level},
     "results": results},
    open(out_path, "w"), indent=2, ensure_ascii=False,
)

print()
print(f"TOTAL {passed}/{len(results)} passed")
for level in sorted(by_level):
    slot = by_level[level]
    print(f"  {level}: {slot['passed']}/{slot['total']}")
print(f"\nwrote {out_path}")
PY
