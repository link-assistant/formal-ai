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

The one reading it does make beyond counting is whose words it is judging. A
proof that hands back matched lines is quoting other files, and a failure
phrase inside a quotation is that file's, not this node's report about its own
step -- so the markers below are read against what the proof says before it
starts citing places.

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
# A tool that came back with nothing is a step that did not happen. Reporting
# it reads like a finding -- "research completed for X, but the tool returned no
# content" is a whole sentence about work -- and proves exactly as much as an
# empty file. This is not the same as a search that ran and matched nothing:
# "no matches" is an observation about the workspace and stays a valid proof.
FAILURE_MARKERS = (
    "could not be recorded",
    "no such file",
    "the write step failed",
    "error:",
    "returned no content",
    # What the renderer itself says when it reads a step as failed
    # (`response_tool_result_failed_*` in `data/seed/multilingual-responses.lino`),
    # in each language it says it in. A node whose whole proof is that sentence
    # has recorded the step that did not happen as its evidence.
    "the command failed",
    "команда завершилась с ошибкой",
    "कमांड विफल रही",
    "命令失败",
)

# How many cited lines it takes before a body counts as quoting rather than
# reporting. One is not enough: `HTTP/1.1 404: Not Found` cites a place by the
# letter of `citation_offset` and is a diagnosis. A body of quotations is a list.
CITED_LINES_THAT_MAKE_A_QUOTATION = 2


def citation_offset(line: str):
    """Where in the line it stops speaking and starts naming what it quotes.

    A search hands back other files' text and says where each line was found:
    `./scripts/install.sh:260:    log ...` from one harness, `  Line 65: ...`
    under a file heading from another. Both put a decimal number immediately
    before the colon that introduces the quoted text, and in both the number
    stands as a token of its own. Asking about the number's position keeps this
    free of any wording, in any language. The citation begins where the word
    carrying it begins, so a line that reports a failure and then points at a
    place -- "the command failed: ./install.sh:260:..." -- keeps its own half.
    """
    for index, character in enumerate(line):
        if character != ":":
            continue
        before = line[:index]
        stem = before.rstrip("0123456789")
        if len(stem) == len(before):
            continue
        if not stem or stem[-1] in ":/." or stem[-1].isspace():
            return len(before) - len(before.split()[-1]) if before.split() else 0
    return None


def own_words(text: str) -> str:
    """The part of a body the proof is saying itself, before it starts quoting.

    A proof that delivers fifty matched lines is proving something, and the
    failure vocabulary inside those lines belongs to the files they came from --
    `except Exception as error:` is a line of the script that was searched, not
    a report that this node's step failed. Judging the markers against the
    quotation is the same misreading `src/agentic_coding/tool_result.rs` had to
    stop making, and the offline run that first showed a proof surviving that
    fix is the run that exposed it here.
    """
    consumed = 0
    first_citation = None
    cited = 0
    for line in text.splitlines(keepends=True):
        offset = citation_offset(line)
        if offset is not None:
            cited += 1
            if first_citation is None:
                first_citation = consumed + offset
        consumed += len(line)
    if first_citation is not None and cited >= CITED_LINES_THAT_MAKE_A_QUOTATION:
        return text[:first_citation]
    return text


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
    if any(marker in own_words(lowered) for marker in FAILURE_MARKERS):
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
