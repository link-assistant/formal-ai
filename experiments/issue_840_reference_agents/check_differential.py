#!/usr/bin/env python3
"""Fail when Formal AI regresses below the recorded issue #840 references."""

import json
import sys
from pathlib import Path


def abort(message):
    print(f"issue #840 differential gate: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_json(path, label):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        abort(f"cannot read {label} {path}: {error}")


def bash_commands(transcript):
    commands = []
    for line in transcript.splitlines():
        prefix = "[tool bash] "
        if not line.startswith(prefix):
            continue
        try:
            arguments = json.loads(line[len(prefix) :])
        except json.JSONDecodeError as error:
            abort(f"invalid bash arguments in ladder transcript: {error}")
        command = arguments.get("command")
        if not isinstance(command, str) or not command:
            abort("ladder transcript contains a bash call without a command")
        commands.append(command)
    return commands


def require(condition, message):
    if not condition:
        abort(message)


def main():
    if len(sys.argv) != 3:
        abort("usage: check_differential.py BASELINE_JSON RESULTS_JSON")

    baseline = load_json(sys.argv[1], "reference baseline")
    measured = load_json(sys.argv[2], "ladder results")

    require(baseline.get("schema_version") == 1, "unknown baseline schema")
    policy = baseline.get("policy", {})
    require(
        policy.get("quota_exhaustion") == "inconclusive",
        "quota exhaustion must be classified as inconclusive",
    )
    require(
        policy.get("inconclusive_is_eligible_for_comparison") is False,
        "inconclusive runs must be excluded from scored comparisons",
    )
    require(
        any(
            event.get("signal") == "provider 429 while retrying"
            and event.get("classification") == "inconclusive"
            for event in baseline.get("transient_observations", [])
        ),
        "the observed provider-429 policy is missing from the baseline",
    )

    agents = baseline.get("agents", [])
    require(
        sum(agent.get("status") == "pass" for agent in agents) >= 6,
        "reference baseline lost successful agent observations",
    )
    require(
        any(agent.get("status") == "inconclusive" for agent in agents),
        "reference baseline lost its inconclusive observation",
    )

    best = baseline.get("best_reference", {})
    require(best.get("status") == "pass", "best reference is not a passing run")
    require(best.get("found") is True, "best reference did not find the target")
    require(
        best.get("widened_after_empty") is True,
        "best reference did not recover after an empty observation",
    )
    require(
        best.get("simple_commands") is True,
        "best reference did not use one simple command per step",
    )
    require(
        best.get("named_discrepancy") is True,
        "best reference did not name the requested/actual discrepancy",
    )
    require(
        best.get("distinguished_target_kind") is True,
        "best reference did not distinguish the folder from the file decoy",
    )
    require(
        best.get("returned_decoy") is False,
        "best reference returned the PEM decoy",
    )

    summary = measured.get("summary", {})
    before = baseline.get("task_ladder_before", {})
    require(summary.get("total") == before.get("total"), "ladder task count changed")
    require(
        summary.get("passed", -1) > before.get("passed", -1),
        "Formal AI no longer improves on its recorded pre-change ladder score",
    )
    require(
        summary.get("passed") == summary.get("total"),
        f"task ladder is not green: {summary.get('passed')}/{summary.get('total')}",
    )

    rows = measured.get("results", [])
    local = next((row for row in rows if row.get("id") == baseline.get("task_id")), None)
    require(local is not None, "results omit the reference local-search task")
    require(local.get("pass") is True, "reference local-search task is marked failed")

    transcript = local.get("answer", "")
    assistant_output = local.get("assistant_output", "")
    require(isinstance(transcript, str), "local transcript is not text")
    require(isinstance(assistant_output, str), "local assistant output is not text")
    commands = bash_commands(transcript)
    require(len(commands) >= 2, "Formal AI did not observe and then widen")
    require(
        len(commands) <= best.get("command_count", 0),
        "Formal AI uses more commands than the best reference",
    )
    require("[tool websearch]" not in transcript, "local request routed to web search")

    forbidden_shell = (";", "&&", "-print -quit", "\n")
    require(
        all(not any(token in command for token in forbidden_shell) for command in commands),
        "Formal AI emitted a chained or early-exit shell command",
    )
    require(
        "-iname 'hive-mind-control-center'" in commands[0],
        "first command is not the exact-name observation",
    )
    require(
        "-iname '*hive*'" in commands[1],
        "second command does not widen after the empty exact result",
    )
    require(
        all("-type d" in command for command in commands[:2]),
        "local search did not preserve the requested directory kind",
    )

    answer_lower = assistant_output.lower()
    require("hive-control-center" in answer_lower, "answer omits the verified target")
    require(
        "hive-mind-control-center" in answer_lower and "no exact match" in answer_lower,
        "answer does not name the requested/actual discrepancy",
    )
    require(
        "private-key.pem" not in answer_lower,
        "answer leaked the more textually similar PEM-file decoy",
    )
    require(
        "desktop" in answer_lower,
        "answer does not state the scope that was actually searched",
    )

    print(
        "issue #840 differential gate: PASS "
        f"({summary['passed']}/{summary['total']} ladder nodes; "
        f"{len(commands)} commands vs {best['agent']}'s {best['command_count']})"
    )


if __name__ == "__main__":
    main()
