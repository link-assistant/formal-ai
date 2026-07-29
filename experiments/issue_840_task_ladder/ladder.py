#!/usr/bin/env python3
"""Drive and judge the deterministic issue #840/#842 task ladder."""

import argparse
import json
import os
import subprocess
import sys
import urllib.request
from pathlib import Path


TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "bash",
            "description": "Execute a shell command",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "websearch",
            "description": "Search the web",
            "parameters": {
                "type": "object",
                "properties": {"query": {"type": "string"}},
                "required": ["query"],
            },
        },
    },
]
MAX_STEPS = 4

# Unknown-prompt fallbacks cannot pass merely because a node has no positive
# prose assertion. These are signature fragments of the seeded generic
# fallbacks in the four supported language families, not broad negative words.
REFUSALS = (
    "could not determine",
    "i couldn't determine",
    "i don't know how to answer",
    "не смог определить",
    "я пока не знаю, как ответить",
    "निर्धारित नहीं कर",
    "मुझे अभी इसका उत्तर देना नहीं आता",
    "无法确定",
    "我无法从本地",
    "我还不知道如何回答这个问题",
)
CAPABILITY_MENUS = (
    "here is what i can do",
    "вот что я умею",
    "मैं यह कर सकता हूँ",
    "以下是我的功能",
)
VIOLATION_FIELDS = (
    "error",
    "missing_expect",
    "unmet_expect_any",
    "leaked_forbid",
    "missing_tools",
    "forbidden_tools",
    "bad_commands",
    "refused",
    "capability_menu",
)


def fixture_for_query(query, documents):
    """Return the first fixture whose every keyword occurs in the query."""
    folded = str(query).casefold()
    for document in documents:
        if all(str(keyword).casefold() in folded for keyword in document["keywords"]):
            return document["text"]
    return None


def judge(node, assistant_output, tools_called, bash_commands, error=None):
    """Judge one node on assistant claims, routes, and emitted command shape."""
    folded = assistant_output.casefold()
    missing = [
        expected
        for expected in node.get("expect", [])
        if str(expected).casefold() not in folded
    ]
    alternatives = node.get("expect_any", [])
    unmet_any = bool(alternatives) and not any(
        str(alternative).casefold() in folded for alternative in alternatives
    )
    leaked = [
        forbidden
        for forbidden in node.get("forbid", [])
        if str(forbidden).casefold() in folded
    ]
    called = [str(name).casefold() for name in tools_called]
    missing_tools = [
        name
        for name in node.get("expect_tool", [])
        if str(name).casefold() not in called
    ]
    forbidden_tools = [
        name
        for name in node.get("forbid_tool", [])
        if str(name).casefold() in called
    ]
    command_text = "\n".join(bash_commands).casefold()
    bad_commands = [
        fragment
        for fragment in node.get("command_forbid", [])
        if str(fragment).casefold() in command_text
    ]
    refused = any(marker in folded for marker in REFUSALS)
    capability_menu = any(marker in folded for marker in CAPABILITY_MENUS)
    passed = not any(
        (
            error,
            missing,
            unmet_any,
            leaked,
            missing_tools,
            forbidden_tools,
            bad_commands,
            refused,
            capability_menu,
        )
    )
    return {
        "missing_expect": missing,
        "unmet_expect_any": unmet_any,
        "leaked_forbid": leaked,
        "missing_tools": missing_tools,
        "forbidden_tools": forbidden_tools,
        "bad_commands": bad_commands,
        "refused": refused,
        "capability_menu": capability_menu,
        "pass": passed,
    }


def _rows_by_id(payload, label):
    rows = payload.get("results")
    if not isinstance(rows, list):
        return {}, [f"{label} has no results list"]
    indexed = {}
    errors = []
    for row in rows:
        task_id = row.get("id") if isinstance(row, dict) else None
        if not isinstance(task_id, str) or not task_id:
            errors.append(f"{label} contains a row without a stable id")
        elif task_id in indexed:
            errors.append(f"{label} contains duplicate node {task_id}")
        else:
            indexed[task_id] = row
    return indexed, errors


def ratchet_errors(baseline, measured):
    """Return per-ID regression errors while permitting appended green nodes."""
    before, errors = _rows_by_id(baseline, "baseline")
    current, current_errors = _rows_by_id(measured, "measurement")
    errors.extend(current_errors)
    for task_id, previous in before.items():
        if task_id not in current:
            errors.append(f"missing baseline node {task_id}")
        elif previous.get("pass") is True and current[task_id].get("pass") is not True:
            errors.append(f"baseline node {task_id} regressed")
    for task_id, row in current.items():
        if row.get("pass") is not True:
            errors.append(f"current node {task_id} is failing")
    return errors


def learning_proposals(measured):
    """Derive review-only learning candidates from observed failed nodes."""
    candidates = []
    for result in measured.get("results", []):
        if result.get("pass") is True:
            continue
        violations = {
            field: result[field]
            for field in VIOLATION_FIELDS
            if result.get(field)
        }
        candidates.append(
            {
                "id": result.get("id"),
                "seed": result.get("seed"),
                "level": result.get("level"),
                "lang": result.get("lang"),
                "prompt": result.get("prompt"),
                "evidence": {
                    "assistant_output": result.get("assistant_output", ""),
                    "tools_called": result.get("tools_called", []),
                    "bash_commands": result.get("bash_commands", []),
                    "error": result.get("error"),
                },
                "violations": violations,
                "decision": "awaiting_human_review",
                "promotion_gate": (
                    "all task-ladder nodes pass by stable id and a human approves"
                ),
            }
        )
    return {
        "schema_version": 1,
        "record_type": "task_ladder_learning_proposals",
        "promotion": "awaiting_human_review",
        "candidates": candidates,
    }


def load_documents(path):
    if path == "none":
        return []
    return json.loads(Path(path).read_text(encoding="utf-8")).get("documents", [])


class AgentLoop:
    def __init__(self, port, sandbox, documents):
        self.port = port
        self.sandbox = sandbox
        self.documents = documents

    def post(self, messages):
        body = json.dumps(
            {"model": "formal-ai", "messages": messages, "tools": TOOLS}
        ).encode()
        request = urllib.request.Request(
            f"http://127.0.0.1:{self.port}/v1/chat/completions",
            data=body,
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(request, timeout=120) as response:
            return json.load(response)

    def ask(self, prompt):
        """Execute the same tool loop as an OpenAI-compatible agent client."""
        messages = [{"role": "user", "content": prompt}]
        transcript = []
        assistant_output = []
        tools_called = []
        bash_commands = []
        try:
            for _ in range(MAX_STEPS):
                payload = self.post(messages)
                message = (payload.get("choices") or [{}])[0].get("message") or {}
                text = message.get("content") or ""
                if text:
                    transcript.append(text)
                    assistant_output.append(text)
                calls = message.get("tool_calls") or []
                if not calls:
                    break
                messages.append(
                    {key: value for key, value in message.items() if value is not None}
                )
                for call in calls:
                    function = call.get("function") or {}
                    name = str(function.get("name") or "")
                    try:
                        arguments = json.loads(function.get("arguments") or "{}")
                    except json.JSONDecodeError:
                        arguments = {}
                    tools_called.append(name)
                    transcript.append(f"[tool {name}] {function.get('arguments')}")
                    result = self.execute(name, arguments, bash_commands)
                    transcript.append(f"[result] {result}")
                    messages.append(
                        {
                            "role": "tool",
                            "tool_call_id": call.get("id") or name,
                            "name": name,
                            "content": result,
                        }
                    )
        except Exception as exception:  # noqa: BLE001 - the result records failures
            error = f"{type(exception).__name__}: {exception}"
        else:
            error = None
        return {
            "answer": "\n".join(transcript).strip(),
            "assistant_output": "\n".join(assistant_output).strip(),
            "tools_called": tools_called,
            "bash_commands": bash_commands,
            "error": error,
        }

    def execute(self, name, arguments, bash_commands):
        if name == "bash" and arguments.get("command"):
            command = str(arguments["command"])
            bash_commands.append(command)
            process = subprocess.run(
                ["/bin/sh", "-c", command],
                cwd=self.sandbox,
                capture_output=True,
                text=True,
                timeout=60,
                env={
                    **os.environ,
                    "FORMAL_AI_DESKTOP_DIR": str(Path(self.sandbox) / "Desktop"),
                },
                check=False,
            )
            return (process.stdout + process.stderr).strip() or "(no output)"
        if name == "websearch":
            return (
                fixture_for_query(arguments.get("query", ""), self.documents)
                or "(no fixture for this query)"
            )
        return "(tool not executed by harness)"


def validate_nodes(nodes):
    errors = []
    seen = set()
    for position, node in enumerate(nodes, start=1):
        task_id = node.get("id")
        if not isinstance(task_id, str) or not task_id:
            errors.append(f"task {position} has no stable id")
        elif task_id in seen:
            errors.append(f"duplicate task id {task_id}")
        else:
            seen.add(task_id)
        for field in (
            "expect",
            "expect_any",
            "forbid",
            "expect_tool",
            "forbid_tool",
            "command_forbid",
        ):
            if field in node and not isinstance(node[field], list):
                errors.append(f"{task_id or position}: {field} must be a list")
    return errors


def summarize(results):
    passed = sum(result["pass"] for result in results)
    summary = {
        "total": len(results),
        "passed": passed,
        "failed": len(results) - passed,
        "by_level": {},
        "by_seed": {},
    }
    for result in results:
        level = summary["by_level"].setdefault(
            f"L{result['level']}", {"passed": 0, "total": 0}
        )
        level["total"] += 1
        level["passed"] += int(result["pass"])
        seed = summary["by_seed"].setdefault(
            result["seed"], {"passed": 0, "total": 0}
        )
        seed["total"] += 1
        seed["passed"] += int(result["pass"])
    return summary


def reason_for(result):
    for key, label in (
        ("error", "error"),
        ("refused", "refused"),
        ("capability_menu", "capability menu"),
        ("missing_tools", "missing tools"),
        ("forbidden_tools", "forbidden tools"),
        ("missing_expect", "missing"),
        ("unmet_expect_any", "none of expect_any"),
        ("leaked_forbid", "leaked"),
        ("bad_commands", "bad commands"),
    ):
        if result.get(key):
            return f"  {label}={result[key]}"
    return ""


def run(arguments):
    data = json.loads(Path(arguments.tasks).read_text(encoding="utf-8"))
    all_nodes = data.get("tasks", [])
    validation_errors = validate_nodes(all_nodes)
    if validation_errors:
        for error in validation_errors:
            print(f"INVALID DATASET: {error}", file=sys.stderr)
        return 2
    nodes = [
        node
        for node in all_nodes
        if not arguments.only or arguments.only in node["id"]
    ]
    loop = AgentLoop(arguments.port, arguments.sandbox, load_documents(arguments.fixtures))
    results = []
    for node in nodes:
        observation = loop.ask(node["prompt"])
        verdict = judge(
            node,
            observation["assistant_output"],
            observation["tools_called"],
            observation["bash_commands"],
            observation["error"],
        )
        result = {
            "id": node["id"],
            "level": node["level"],
            "seed": node["seed"],
            "lang": node["lang"],
            "prompt": node["prompt"],
            "note": node.get("note", ""),
            **observation,
            **verdict,
        }
        results.append(result)
        print(
            f"{'PASS' if result['pass'] else 'FAIL'}  "
            f"{node['id']:<12} L{node['level']}  "
            f"{node['prompt'][:58]!r}{reason_for(result)}"
        )

    payload = {"schema_version": 2, "summary": summarize(results), "results": results}
    Path(arguments.out).write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    learning = learning_proposals(payload)
    if arguments.learning_out:
        Path(arguments.learning_out).write_text(
            json.dumps(learning, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
    summary = payload["summary"]
    print(f"\nTOTAL {summary['passed']}/{summary['total']} passed")
    for level, value in sorted(summary["by_level"].items()):
        print(f"  {level}: {value['passed']}/{value['total']}")
    for seed, value in sorted(summary["by_seed"].items()):
        print(f"  #{seed}: {value['passed']}/{value['total']}")
    print(f"\nwrote {arguments.out}")
    if arguments.learning_out:
        print(
            f"wrote {arguments.learning_out} "
            f"({len(learning['candidates'])} review-gated candidate(s))"
        )

    if arguments.baseline:
        baseline = json.loads(Path(arguments.baseline).read_text(encoding="utf-8"))
        errors = ratchet_errors(baseline, payload)
        print(
            f"baseline {baseline['summary']['passed']}/"
            f"{baseline['summary']['total']} -> now "
            f"{summary['passed']}/{summary['total']}"
        )
        for error in errors:
            print(f"REGRESSION: {error}", file=sys.stderr)
        if errors:
            return 1
    return 0


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tasks", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--port", required=True)
    parser.add_argument("--only", default="")
    parser.add_argument("--sandbox", required=True)
    parser.add_argument("--fixtures", required=True)
    parser.add_argument("--baseline", default="")
    parser.add_argument("--learning-out", default="")
    return parser.parse_args()


if __name__ == "__main__":
    raise SystemExit(run(parse_args()))
