#!/usr/bin/env python3
"""Extract the assistant-visible answer and tool calls from an Agent CLI log.

The Agent CLI emits a stream of pretty-printed JSON objects. Only a few of them
carry what a reader of a wave F transcript needs: the `text` parts (what Formal
AI actually answered), the `tool` parts (what it asked the CLI to do), and the
`step_finish` reason. Everything else is CLI bookkeeping.

Usage: extract_answer.py <agent.log> <answer.txt>
"""

import json
import sys

decoder = json.JSONDecoder()


def objects(blob):
    i = 0
    n = len(blob)
    while i < n:
        while i < n and blob[i] not in "{[":
            i += 1
        if i >= n:
            return
        try:
            obj, end = decoder.raw_decode(blob, i)
        except ValueError:
            i += 1
            continue
        yield obj
        i = end


def main():
    src, dst = sys.argv[1], sys.argv[2]
    with open(src, encoding="utf-8", errors="replace") as handle:
        blob = handle.read()

    lines = []
    for obj in objects(blob):
        if not isinstance(obj, dict):
            continue
        kind = obj.get("type")
        part = obj.get("part") or {}
        if kind == "text" and part.get("text"):
            lines.append("--- assistant text ---")
            lines.append(part["text"])
        elif kind == "tool":
            state = part.get("state") or {}
            lines.append(
                "--- tool call: %s (%s) ---"
                % (part.get("tool", "?"), state.get("status", "?"))
            )
            if state.get("input"):
                lines.append("input: " + json.dumps(state["input"], ensure_ascii=False))
            if state.get("output"):
                lines.append("output: " + str(state["output"])[:4000])
            if state.get("error"):
                lines.append("error: " + str(state["error"])[:2000])
        elif kind == "step_finish":
            lines.append("--- step finish: %s ---" % part.get("reason", "?"))
        elif kind == "log" and obj.get("level") == "error":
            lines.append("--- cli error (%s) ---" % obj.get("service", "?"))
            lines.append(json.dumps(obj.get("error", {}), ensure_ascii=False)[:2000])

    with open(dst, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
        handle.write("\n")


if __name__ == "__main__":
    main()
