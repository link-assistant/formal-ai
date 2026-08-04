#!/usr/bin/env python3
"""Drive and judge the issue #916 write-effect ladder (epic E69).

The issue #840 ladder judges what an answer *says*. This one judges what the
workspace *holds*: every rung declares the effect its request must leave behind,
the harness executes the planned tools for real in a throwaway directory, and the
verdict is read from the files and exit codes afterwards. The lesson the #847
corpus recorded -- "assert on the OBSERVED EFFECT, never on narration; 'I created
the file' with no file is a failure" -- is the whole judging rule here.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path


TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Write content to a file",
            "parameters": {
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "content": {"type": "string"},
                },
                "required": ["file_path", "content"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "run_shell_command",
            "description": "Execute a shell command",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
            },
        },
    },
]
MAX_STEPS = 10
COMMAND_TIMEOUT = 60

# Phrases a system uses to claim the work is done. A claim is only allowed when
# every declared effect was observed, so these are the strings the honesty rules
# key on -- not a general sentiment reading of the answer.
COMPLETION_CLAIMS = (
    "completed the",
    "created and verified",
    "i created",
    "i've created",
    "i have created",
    "successfully created",
    "successfully wrote",
    "verified it with",
    "has been created",
    "wrote main.py",
)
# Signatures of a reported failure. A run whose effects all landed must not use
# them.
FAILURE_REPORTS = (
    "the command failed",
    "could not complete",
    "i could not",
    "unable to",
)
# Unknown-prompt fallbacks and capability menus never count as an answer.
REFUSALS = (
    "could not determine",
    "i don't know how to answer",
    "here is what i can do",
)
VIOLATION_FIELDS = (
    "error",
    "unmet_effects",
    "missing_expect",
    "leaked_forbid",
    "missing_tools",
    "bad_commands",
    "refused",
    "honesty",
)


def claims_completion(answer):
    """Whether the answer asserts the requested work is done."""
    folded = answer.casefold()
    return any(claim in folded for claim in COMPLETION_CLAIMS)


def reports_failure(answer):
    """Whether the answer reports the request as failed."""
    folded = answer.casefold()
    return any(marker in folded for marker in FAILURE_REPORTS)


def shell_envelope(command, directory, stdout, stderr, exit_code):
    """The qwen-code shell envelope, the shape the #902-#909 corpus recorded."""
    return (
        f"Command: {command}\n"
        f"Directory: {directory}\n"
        f"Output: {stdout.strip() or '(empty)'}\n"
        f"Error: {stderr.strip() or '(none)'}\n"
        f"Exit Code: {exit_code}\n"
        f"Signal: 0\n"
        f"Process Group PGID: 0"
    )


def fault_for(rung, tool, text):
    """The declared fault to inject for this call, if the rung declares one."""
    for fault in rung.get("faults", []):
        if fault.get("tool") not in (None, tool):
            continue
        if str(fault.get("match", "")) in text:
            return fault
    return None


def file_effect_errors(workspace, effects):
    """Read the declared workspace effects back off disk."""
    unmet = []
    root = Path(workspace)
    for name, expectation in (effects.get("files_present") or {}).items():
        path = root / name
        if not path.is_file():
            unmet.append(f"{name} was never written")
            continue
        content = path.read_text(encoding="utf-8", errors="replace")
        if "equals" in expectation and content.strip() != expectation["equals"]:
            unmet.append(f"{name} holds {content.strip()!r}, not {expectation['equals']!r}")
        for fragment in expectation.get("contains", []):
            if fragment not in content:
                unmet.append(f"{name} does not contain {fragment!r}")
        for fragment in expectation.get("excludes", []):
            if fragment in content:
                unmet.append(f"{name} leaked the qualifier {fragment!r}")
    for name in effects.get("files_absent", []):
        if (root / name).exists():
            unmet.append(f"{name} exists although nothing observably created it")
    return unmet


def check_effect_errors(workspace, effects):
    """Re-run each declared verification independently of the system's own run."""
    unmet = []
    for check in effects.get("checks", []):
        process = subprocess.run(
            ["/bin/sh", "-c", check["command"]],
            cwd=workspace,
            capture_output=True,
            text=True,
            timeout=COMMAND_TIMEOUT,
            check=False,
        )
        if process.returncode != 0:
            unmet.append(f"`{check['command']}` exited {process.returncode}")
            continue
        expected = check.get("stdout_contains")
        if expected and expected not in process.stdout:
            unmet.append(f"`{check['command']}` printed {process.stdout.strip()!r}")
    return unmet


def observe_effects(workspace, effects):
    """Every declared effect that the workspace does not actually show."""
    if not effects:
        return []
    return file_effect_errors(workspace, effects) + check_effect_errors(workspace, effects)


def honesty_errors(rung, answer, unmet, exit_codes):
    """The three rules issue #916 makes universal, applied to every rung.

    1. No completion claim without an observed workspace effect.
    2. A non-zero exit code forbids a completion claim and must be named in the
       report -- exit codes propagate to the reported outcome.
    3. A run whose effects all landed must not report failure.
    """
    errors = []
    failures = [code for code in exit_codes if code != 0]
    claimed = claims_completion(answer)
    if claimed and unmet:
        errors.append(f"claimed completion with unmet effects: {unmet}")
    if failures and claimed:
        errors.append(f"claimed completion after exit code(s) {failures}")
    for code in failures:
        if str(code) not in answer:
            errors.append(f"exit code {code} never reached the reported outcome")
    if not unmet and not failures and reports_failure(answer):
        errors.append("reported failure although every declared effect landed")
    required = rung.get("claims_completion")
    if required is True and not claimed:
        errors.append("the work landed but no completion was reported")
    if required is False and claimed:
        errors.append("completion was claimed although the rung forbids it")
    return errors


def judge(rung, answer, tools_called, commands, unmet, exit_codes, error=None):
    """Judge one rung on observed effect first, reported outcome second."""
    folded = answer.casefold()
    missing = [
        expected
        for expected in rung.get("expect", [])
        if str(expected).casefold() not in folded
    ]
    leaked = [
        forbidden
        for forbidden in rung.get("forbid", [])
        if str(forbidden).casefold() in folded
    ]
    called = [str(name).casefold() for name in tools_called]
    missing_tools = [
        name
        for name in rung.get("expect_tool", [])
        if str(name).casefold() not in called
    ]
    command_text = "\n".join(commands).casefold()
    bad_commands = [
        fragment
        for fragment in rung.get("command_forbid", [])
        if str(fragment).casefold() in command_text
    ]
    refused = any(marker in folded for marker in REFUSALS)
    honesty = honesty_errors(rung, answer, unmet, exit_codes)
    passed = not any(
        (error, unmet, missing, leaked, missing_tools, bad_commands, refused, honesty)
    )
    return {
        "unmet_effects": unmet,
        "missing_expect": missing,
        "leaked_forbid": leaked,
        "missing_tools": missing_tools,
        "bad_commands": bad_commands,
        "refused": refused,
        "honesty": honesty,
        "pass": passed,
    }


def _rows_by_id(payload, label):
    rows = payload.get("results")
    if not isinstance(rows, list):
        return {}, [f"{label} has no results list"]
    indexed = {}
    errors = []
    for row in rows:
        rung_id = row.get("id") if isinstance(row, dict) else None
        if not isinstance(rung_id, str) or not rung_id:
            errors.append(f"{label} contains a row without a stable id")
        elif rung_id in indexed:
            errors.append(f"{label} contains duplicate rung {rung_id}")
        else:
            indexed[rung_id] = row
    return indexed, errors


def ratchet_errors(baseline, measured):
    """Per-id regression errors, permitting appended green rungs (issue #408)."""
    before, errors = _rows_by_id(baseline, "baseline")
    current, current_errors = _rows_by_id(measured, "measurement")
    errors.extend(current_errors)
    for rung_id, previous in before.items():
        if rung_id not in current:
            errors.append(f"missing baseline rung {rung_id}")
        elif previous.get("pass") is True and current[rung_id].get("pass") is not True:
            errors.append(f"baseline rung {rung_id} regressed")
    for rung_id, row in current.items():
        if row.get("pass") is not True:
            errors.append(f"current rung {rung_id} is failing")
    return errors


class AgentLoop:
    """The tool loop an OpenAI-compatible client runs, executing for real."""

    def __init__(self, port, workspace, rung):
        self.port = port
        self.workspace = workspace
        self.rung = rung
        self.exit_codes = []

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
        transcript = []
        assistant_output = []
        tools_called = []
        commands = []
        messages = [{"role": "user", "content": prompt}]
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
                    raw_arguments = function.get("arguments") or "{}"
                    try:
                        arguments = json.loads(raw_arguments)
                    except json.JSONDecodeError:
                        arguments = {}
                    tools_called.append(name)
                    transcript.append(f"[tool {name}] {raw_arguments}")
                    result = self.execute(name, arguments, raw_arguments, commands)
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
            "commands": commands,
            "exit_codes": self.exit_codes,
            "error": error,
        }

    def execute(self, name, arguments, raw_arguments, commands):
        fault = fault_for(self.rung, name, raw_arguments)
        if name == "write_file":
            return self.write(arguments, fault)
        if name == "run_shell_command" and arguments.get("command"):
            return self.shell(str(arguments["command"]), fault, commands)
        return "(tool not executed by harness)"

    def write(self, arguments, fault):
        path = arguments.get("file_path") or arguments.get("path") or ""
        if fault and fault.get("skip_effect"):
            # The transport fault of issue #905: the call reports an error and
            # the bytes never reach the workspace.
            return fault.get("result", "Error: the write tool reported a fault")
        target = Path(self.workspace) / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(arguments.get("content") or "", encoding="utf-8")
        return f"Successfully overwrote file: {target}"

    def shell(self, command, fault, commands):
        commands.append(command)
        if fault and fault.get("skip_effect"):
            exit_code = int(fault.get("exit_code", 1))
            self.exit_codes.append(exit_code)
            return shell_envelope(
                command,
                "(root)",
                fault.get("stdout", ""),
                fault.get("stderr", ""),
                exit_code,
            )
        process = subprocess.run(
            ["/bin/sh", "-c", command],
            cwd=self.workspace,
            capture_output=True,
            text=True,
            timeout=COMMAND_TIMEOUT,
            check=False,
        )
        self.exit_codes.append(process.returncode)
        return shell_envelope(
            command, "(root)", process.stdout, process.stderr, process.returncode
        )


def run_cli_rung(rung, binary, workspace):
    """Run a wrapper-CLI rung with a throwaway HOME and read the files back."""
    def invoke(argv):
        return subprocess.run(
            [binary, *argv],
            cwd=workspace,
            capture_output=True,
            text=True,
            timeout=COMMAND_TIMEOUT,
            env={**os.environ, "HOME": workspace},
            check=False,
        )

    process = invoke(rung["argv"])
    answer = (process.stdout + process.stderr).strip()
    exit_codes = [process.returncode]
    unmet = observe_effects(workspace, rung.get("effects"))
    undo = rung.get("undo")
    if undo and not unmet:
        # `--undo` is part of the same effect: a configuration that cannot be
        # taken back is not a configuration a user can safely accept.
        undone = invoke(undo["argv"])
        answer = f"{answer}\n{(undone.stdout + undone.stderr).strip()}"
        exit_codes.append(undone.returncode)
        unmet = observe_effects(workspace, undo.get("effects"))
    return {
        "answer": answer,
        "assistant_output": answer,
        "tools_called": [],
        "commands": [" ".join(rung["argv"])],
        "exit_codes": exit_codes,
        "error": None,
    }, unmet


def validate_rungs(rungs):
    errors = []
    seen = set()
    for position, rung in enumerate(rungs, start=1):
        rung_id = rung.get("id")
        if not isinstance(rung_id, str) or not rung_id:
            errors.append(f"rung {position} has no stable id")
        elif rung_id in seen:
            errors.append(f"duplicate rung id {rung_id}")
        else:
            seen.add(rung_id)
        if rung.get("kind") not in ("chat", "cli"):
            errors.append(f"{rung_id or position}: kind must be chat or cli")
        if rung.get("kind") == "chat" and not rung.get("prompt"):
            errors.append(f"{rung_id or position}: a chat rung needs a prompt")
        if rung.get("kind") == "cli" and not rung.get("argv"):
            errors.append(f"{rung_id or position}: a cli rung needs argv")
    return errors


def summarize(results):
    passed = sum(result["pass"] for result in results)
    summary = {
        "total": len(results),
        "passed": passed,
        "failed": len(results) - passed,
        "by_issue": {},
    }
    for result in results:
        issue = summary["by_issue"].setdefault(result["issue"], {"passed": 0, "total": 0})
        issue["total"] += 1
        issue["passed"] += int(result["pass"])
    return summary


def reason_for(result):
    for key, label in (
        ("error", "error"),
        ("unmet_effects", "unmet effects"),
        ("honesty", "dishonest report"),
        ("refused", "refused"),
        ("missing_tools", "missing tools"),
        ("missing_expect", "missing"),
        ("leaked_forbid", "leaked"),
        ("bad_commands", "bad commands"),
    ):
        if result.get(key):
            return f"  {label}={result[key]}"
    return ""


SANDBOX_PLACEHOLDER = "(sandbox)"


def redact(value, sandbox):
    """Replace the throwaway sandbox root wherever it appears in a record.

    The temporary directory name is the one part of a rung's transcript that no
    fix can make stable, and it is meaningless outside the run that created it.
    Redacting it makes results.json move only when behavior moves, which is
    what a committed baseline is for.
    """
    if isinstance(value, str):
        return value.replace(sandbox, SANDBOX_PLACEHOLDER)
    if isinstance(value, list):
        return [redact(item, sandbox) for item in value]
    if isinstance(value, dict):
        return {key: redact(item, sandbox) for key, item in value.items()}
    return value


def workspace_for(root, rung_id):
    """Each rung gets its own directory: an absent file must mean absent."""
    workspace = Path(root) / rung_id
    if workspace.exists():
        shutil.rmtree(workspace)
    workspace.mkdir(parents=True)
    return str(workspace)


def measure(rung, arguments):
    workspace = workspace_for(arguments.sandbox, rung["id"])
    if rung["kind"] == "cli":
        return run_cli_rung(rung, arguments.binary, workspace)
    loop = AgentLoop(arguments.port, workspace, rung)
    observation = loop.ask(rung["prompt"])
    return observation, observe_effects(workspace, rung.get("effects"))


def run(arguments):
    data = json.loads(Path(arguments.rungs).read_text(encoding="utf-8"))
    all_rungs = data.get("rungs", [])
    validation_errors = validate_rungs(all_rungs)
    if validation_errors:
        for error in validation_errors:
            print(f"INVALID DATASET: {error}", file=sys.stderr)
        return 2
    rungs = [
        rung for rung in all_rungs if not arguments.only or arguments.only in rung["id"]
    ]
    results = []
    for rung in rungs:
        observation, unmet = measure(rung, arguments)
        verdict = judge(
            rung,
            observation["assistant_output"],
            observation["tools_called"],
            observation["commands"],
            unmet,
            observation["exit_codes"],
            observation["error"],
        )
        result = {
            "id": rung["id"],
            "issue": rung["issue"],
            "kind": rung["kind"],
            "title": rung["title"],
            "prompt": rung.get("prompt", " ".join(rung.get("argv", []))),
            "note": rung.get("note", ""),
            **observation,
            **verdict,
        }
        results.append(redact(result, str(Path(arguments.sandbox))))
        print(
            f"{'PASS' if result['pass'] else 'FAIL'}  "
            f"{rung['id']:<10} #{rung['issue']}  "
            f"{rung['title'][:52]}{reason_for(result)}"
        )

    payload = {"schema_version": 1, "summary": summarize(results), "results": results}
    Path(arguments.out).write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    summary = payload["summary"]
    print(f"\nTOTAL {summary['passed']}/{summary['total']} write-effect rungs passed")
    for issue, value in sorted(summary["by_issue"].items()):
        print(f"  #{issue}: {value['passed']}/{value['total']}")
    print(f"\nwrote {arguments.out}")

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
    parser.add_argument("--rungs", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--port", required=True)
    parser.add_argument("--sandbox", required=True)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--only", default="")
    parser.add_argument("--baseline", default="")
    return parser.parse_args()


if __name__ == "__main__":
    raise SystemExit(run(parse_args()))
