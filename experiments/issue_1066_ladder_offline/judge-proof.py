#!/usr/bin/env python3
"""Judge whether a ladder proof file proves anything.

The ladder's own criterion is mechanical: the file exists, is non-empty, and
opens with `node_path=<id>`. Sixty-three nodes met it and thirty-two of the
files said nothing -- a heading with no list, a single word naming the work
product, or a report that the step had failed. A file that says nothing is
still a non-empty file, so a harness that only counts bytes reports a green
ladder over hollow evidence, which is the failure issue #1066 exists to stop.

This adds the judgement the mechanical check cannot make. It is deliberately
about the *shape* of what a proof says and never about any node's wording: a
check keyed to the prompts would pass the ladder by describing it.

Usage: judge-proof.py PROOF_PATH NODE_ID
Prints `ok` on success, or a one-word reason on failure, and exits 1.
"""

import sys
from pathlib import Path

# The fewest words a claim about a task can be made in. "It is atomic" is four
# and is a real verdict; anything shorter is a label, not a finding.
MINIMUM_WORDS = 4

# Punctuation that promises an enumeration. The full-width form is here because
# Chinese and Japanese introduce a list with it.
LIST_PROMISE = (":", "：")

# Phrases that name a work product instead of delivering one, and phrasings that
# report the step failed. Matching English alone would be a hole, so each is
# listed in the languages the ladder's prompts and the seed both cover.
EMPTY_BODIES = {
    "the result", "the finding", "the answer", "the outcome", "the evidence",
    "результат", "вывод", "ответ", "结果", "結果", "परिणाम",
}
FAILURE_MARKERS = (
    "could not be recorded",
    "no such file",
    "the write step failed",
    "error:",
)


def judge(text: str, node_id: str) -> str:
    lines = text.splitlines()
    if not lines or lines[0].strip() != f"node_path={node_id}":
        return "bad_proof_marker"
    body = "\n".join(lines[1:]).strip()
    if not body:
        return "hollow_empty_body"
    if body.endswith(LIST_PROMISE):
        return "hollow_unmade_list"
    lowered = body.lower().strip(" .")
    if lowered in EMPTY_BODIES:
        return "hollow_named_work_product"
    if any(marker in lowered for marker in FAILURE_MARKERS):
        return "hollow_reported_failure"
    if len(body.split()) < MINIMUM_WORDS:
        return "hollow_too_short"
    return "ok"


def main() -> int:
    proof, node_id = Path(sys.argv[1]), sys.argv[2]
    if not proof.exists():
        print("missing_proof")
        return 1
    verdict = judge(proof.read_text(encoding="utf-8", errors="replace"), node_id)
    print(verdict)
    return 0 if verdict == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())
