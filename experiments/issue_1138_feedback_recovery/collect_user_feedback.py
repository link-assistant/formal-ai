#!/usr/bin/env python3
"""Recover genuine user feedback and requirements from Claude Code session transcripts.

Reads one or more session .jsonl files, keeps only human-authored user
messages, strips harness wrappers (system reminders, task notifications,
local-command output), drops continuation summaries, and collapses exact
duplicates (recurring cron checkpoints) to one entry with a repeat count.
Output is chronological markdown so requirements can be audited against the
repository's requirements docs. Reusable: pass any session jsonl paths.

Usage: collect_user_feedback.py <session.jsonl> [...] > feedback.md
"""

import json
import re
import sys

TAG_BLOCK = re.compile(
    r"<(system-reminder|task-notification|local-command-caveat|"
    r"command-name|command-message|command-args|local-command-stdout)"
    r"\b[^>]*>.*?</\1>",
    re.DOTALL,
)
CONTINUATION_PREFIX = "This session is being continued from a previous conversation"
HARNESS_MARKERS = (
    "[SYSTEM NOTIFICATION",
    "[Request interrupted by user for tool use]",
    "[Request interrupted by user]",
)


def message_text(entry):
    content = entry.get("message", {}).get("content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = [block.get("text", "") for block in content if block.get("type") == "text"]
        return "\n".join(parts)
    return ""


def clean(text):
    text = TAG_BLOCK.sub("", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip()


def genuine(text):
    if not text:
        return False
    if text.startswith(CONTINUATION_PREFIX):
        return False
    if text.startswith(HARNESS_MARKERS):
        return False
    if text.startswith(("Caveat:", "[System Instruction]", "CronDelete")):
        return False
    if text.lstrip().startswith(("/", "! ")):
        return False
    return True


def main():
    messages = []
    for path in sys.argv[1:]:
        with open(path, "r", encoding="utf-8") as handle:
            for line in handle:
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if entry.get("type") != "user":
                    continue
                text = clean(message_text(entry))
                if genuine(text):
                    messages.append((entry.get("timestamp", ""), text))

    seen = {}
    ordered = []
    for timestamp, text in messages:
        if text in seen:
            seen[text]["repeats"] += 1
            continue
        record = {"timestamp": timestamp, "text": text, "repeats": 0}
        seen[text] = record
        ordered.append(record)

    for record in ordered:
        repeat_note = f" *(repeated {record['repeats']}x)*" if record["repeats"] else ""
        print(f"## {record['timestamp']}{repeat_note}\n")
        print(record["text"])
        print()


if __name__ == "__main__":
    main()
