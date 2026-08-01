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
#   OUT       Results JSON (default: <scriptdir>/results.json for a full run;
#             filtered runs use results-partial-<filter>.json)
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
ONLY="${ONLY:-}"
TIMEOUT="${TIMEOUT:-300}"

if [ -n "${OUT:-}" ]; then
  OUT="$OUT"
elif [ -n "$ONLY" ]; then
  FILTER_SLUG="$(printf '%s' "$ONLY" | tr -c '[:alnum:]_.-' '_')"
  OUT="$HERE/results-partial-$FILTER_SLUG.json"
else
  OUT="$HERE/results.json"
fi

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
import glob, json, os, re, shutil, subprocess, sys, tempfile

prompts_path, out_path, only, timeout_s, binary, root = sys.argv[1:7]
timeout_s = int(timeout_s)
data = json.load(open(prompts_path))
all_tasks = data["tasks"]
tasks = [t for t in all_tasks if not only or only in t["id"]]

def reset_repo():
    """Every task starts from a clean tree so results are independent.

    Tasks operate on the REAL repository (that is the point -- we teach Formal
    AI on work we actually need done), so any edit an agent makes must be
    reverted before the next task runs. Untracked files created by a task are
    removed too, but `experiments/` is preserved so this harness cannot delete
    itself mid-run -- which the checkout must honour as well, or a filtered
    re-run (ONLY=...) silently reverts the results of the previous full run.
    """
    subprocess.run(["git", "checkout", "--", ".", ":(exclude)experiments"],
                   cwd=root, capture_output=True)
    subprocess.run(["git", "clean", "-fd", "--exclude=experiments"],
                   cwd=root, capture_output=True)

def reclaim_agent_scratch():
    """Delete the ~200 MB scratch home each `formal-ai with agent` run creates.

    The agent CLI copies its whole configuration into
    `$TMPDIR/formal-ai-agent-home-config-<pid>-<nanos>/` and never removes it.
    Over a 130-task run that is ~25 GB, and once the filesystem tightens the
    temporary server stops coming up at all: every task from that point on
    fails with `Formal AI server exited before listening: exit status: 101`,
    which the harness would otherwise record as an ordinary FAIL. Observed on
    Linux at task 88 of 130; reclaiming after each task keeps a long run's
    later results as trustworthy as its earlier ones.
    """
    tmp = os.environ.get("TMPDIR", "/tmp")
    for path in glob.glob(os.path.join(tmp, "formal-ai-agent-home-config-*")):
        shutil.rmtree(path, ignore_errors=True)

def snapshot_branches():
    """Record every ref before a task so `new_branch_for.sh` can spot new ones.

    L1 tasks ("solve the issue and open a pull request") are verified by a
    branch existing. Without a before-picture that check passes on branches
    that were already in the clone -- remote-tracking refs included -- which
    silently credits the agent with work other people did.
    """
    proc = subprocess.run(["git", "for-each-ref", "--format=%(refname)"],
                          cwd=root, capture_output=True, text=True)
    with open(os.path.join(os.path.dirname(out_path), ".branches-before"),
              "w") as handle:
        handle.write(proc.stdout)

def repo_is_clean():
    proc = subprocess.run(["git", "status", "--porcelain"], cwd=root,
                          capture_output=True, text=True)
    # This harness writes its own results.json inside experiments/, so changes
    # under experiments/ are expected and must not block a run.
    dirty = [line for line in proc.stdout.splitlines()
             if line.strip() and "experiments/" not in line]
    return not dirty

def resolve_expected_answer(task):
    """Resolve answer oracles from the current repository when requested.

    Release facts such as the crate version change independently of this
    benchmark. Keeping their old value in prompts.json turns a correct Agent
    answer into a false failure, so those tasks store a path and capture regex
    instead of a copied value.
    """
    source = task.get("expect_from_file")
    if source is None:
        return task.get("expect_answer"), ""
    try:
        path = os.path.join(root, source["path"])
        with open(path, "r", errors="replace") as handle:
            contents = handle.read()
        match = re.search(source["pattern"], contents, re.MULTILINE)
        if match is None:
            return None, f"pattern did not match {source['path']}"
        return match.group(source.get("group", 1)), ""
    except (KeyError, IndexError, OSError, re.error) as error:
        return None, f"could not resolve file-derived expectation: {error}"

if not repo_is_clean():
    print("refusing to run: working tree is dirty; commit or stash first", file=sys.stderr)
    raise SystemExit(0)

results = []

def summarize():
    by_level = {}
    for r in results:
        slot = by_level.setdefault(f"L{r['level']}", {"passed": 0, "total": 0})
        slot["total"] += 1
        slot["passed"] += 1 if r["pass"] else 0
    passed = sum(1 for r in results if r["pass"])
    return {"total": len(results), "passed": passed,
            "failed": len(results) - passed,
            "not_measured": sum(1 for r in results if r["not_measured"]),
            "by_level": by_level}

def write_results():
    with open(out_path, "w") as handle:
        json.dump({
                  "measurement": {
                      "dataset_total": len(all_tasks),
                      "measured_total": len(tasks),
                      "complete": not only and len(tasks) == len(all_tasks),
                      "filter": only or None,
                  },
                  "summary": summarize(), "results": results},
                  handle, indent=2, ensure_ascii=False)
for task in tasks:
    structural = ""
    reset_repo()
    expected, expectation_error = resolve_expected_answer(task)
    snapshot_branches()
    rust_targets = re.findall(
        r'\b((?:src|tests|scripts)/[\w./-]+\.rs)\b', task["prompt"],
    )
    rust_target_existed = {
        target: os.path.exists(os.path.join(root, target))
        for target in rust_targets
    }
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
    reclaim_agent_scratch()

    verified = subprocess.run(
        ["/bin/sh", "-c", task.get("verify", "true")],
        cwd=root, capture_output=True, text=True,
    ).returncode == 0

    # Structural sanity: a substring `verify` cannot tell real code from the
    # prompt echoed back into the file. Observed failure mode -- asked for a
    # Rust test file, the agent creates the file whose entire content is the
    # sentence fragment "one Rust test named X asserting ...". Compile every
    # requested .rs creation as a standalone library. This checks the exact
    # bytes rather than a proxy such as the presence of an `fn` substring.
    if verified:
        for created in rust_targets:
            path = os.path.join(root, created)
            if rust_target_existed[created] or not os.path.exists(path):
                continue
            descriptor, artifact = tempfile.mkstemp(
                prefix="formal-ai-ladder-", suffix=".rmeta",
            )
            os.close(descriptor)
            os.unlink(artifact)
            try:
                compiled = subprocess.run(
                    ["rustc", "--edition=2021", "--crate-type", "lib",
                     "--emit", "metadata", path, "-o", artifact],
                    cwd=root, capture_output=True, text=True,
                )
            finally:
                if os.path.exists(artifact):
                    os.unlink(artifact)
            if compiled.returncode != 0:
                verified = False
                detail = next(
                    (line.strip() for line in compiled.stderr.splitlines()
                     if line.strip()),
                    "rustc rejected the generated source",
                )
                structural = f"{created} does not compile: {detail}"
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
    answered = not expectation_error and (expected is None or (
        not refused and expected.lower() in output.lower()
    ))
    # An unfalsifiable `verify: true` plus a refusal is not a pass.
    # A refusal is never a pass, even when `verify` is a trivially true
    # placeholder (the L1 ceiling cases before their checks were tightened).
    # The temporary server never came up, so nothing about the model was
    # measured. Recording this as an ordinary FAIL is how a run silently turns
    # into fiction, so it is named and counted separately instead.
    not_measured = "exited before listening" in output
    ok = (verified and answered and not timed_out and not refused
          and not not_measured and not expectation_error)

    reason = ""
    structural = locals().get("structural", "")
    if not_measured:
        reason = "NOT MEASURED (server never started)"
    elif timed_out:
        reason = f"timeout after {timeout_s}s"
    elif expectation_error:
        reason = f"invalid answer expectation: {expectation_error}"
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
        "verified_effect": verified, "not_measured": not_measured,
        "expected_answer": expected,
        "output_tail": output[-2000:],
    })
    print(f"{'PASS' if ok else 'FAIL'}  {task['id']:<22} L{task['level']}  {reason}",
          flush=True)
    # Written after every task: a 130-task run takes half an hour, and an
    # all-or-nothing write loses the whole measurement if it is interrupted.
    write_results()

reset_repo()
write_results()

passed = sum(1 for r in results if r["pass"])
unmeasured = sum(1 for r in results if r["not_measured"])
print()
print(f"TOTAL {passed}/{len(results)} passed")
for level in sorted(summarize()["by_level"]):
    slot = summarize()["by_level"][level]
    print(f"  {level}: {slot['passed']}/{slot['total']}")
if unmeasured:
    print(f"\nWARNING: {unmeasured} task(s) were NOT MEASURED -- the temporary "
          "server never started. Treat this run as incomplete.")
if only:
    print(f"\nPARTIAL: measured {len(tasks)}/{len(all_tasks)} tasks with "
          f"filter {only!r}; the canonical 130-task score was not replaced.")
print(f"\nwrote {out_path}")
PY
