#!/usr/bin/env python3
"""Keep raw diagnostics; classify only a deliberately missing E2E command.

The usual stderr policy receives all other lines unchanged. No arbitrary error
or tool result is permitted, including a failure of a different command.
"""
import json
import re
import sys
from pathlib import Path


def expected_missing_command(line: str, command: str) -> bool:
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        return False
    if not isinstance(event, dict):
        return False
    output = event.get("output")
    return (
        event.get("type") == "tool_result"
        and event.get("status") == "error"
        and isinstance(output, str)
        and output.startswith("Exit code 127\n")
        and re.search(r"(?:^|[ :])" + re.escape(command) + r": (?:not found|command not found)(?:\n|$)", output) is not None
    )


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: filter-expected-agent-tool-failure.py RAW_LOG POLICY_LOG EXPECTED_COMMAND")
    raw, policy, command = sys.argv[1:]
    if not re.fullmatch(r"[A-Za-z0-9_-]+", command):
        raise SystemExit("expected command must be a literal executable name")
    if Path(raw).resolve() == Path(policy).resolve():
        raise SystemExit("raw evidence must not be overwritten")
    lines = Path(raw).read_text().splitlines(keepends=True)
    retained = [line for line in lines if not expected_missing_command(line, command)]
    Path(policy).write_text("".join(retained))
    print(f"Classified {len(lines) - len(retained)} deliberate missing-command tool event(s); raw evidence retained.")


if __name__ == "__main__":
    main()
