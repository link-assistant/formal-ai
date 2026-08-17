#!/usr/bin/env python3
"""Render the conversational variation corpus from the probed candidate list.

Issue #933 asks for a machine-checkable "at least five wording variations per
language for every conversational test case". The candidates in
`candidates.tsv` were each run through the deterministic engine (see
`examples/issue_933_variation_probe.rs`); only prompts that already route to
their case's intent are written out, so the corpus is evidence, not intent.

    python3 experiments/issue_933_variation_floor/build-corpus.py
"""

import collections
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
HERE = pathlib.Path(__file__).resolve().parent
LANGUAGES = ["en", "ru", "hi", "zh"]

# case -> (intent, response evidence link, human title)
CASES = {
    "greeting": ("greeting", "response:greeting", "Greeting"),
    "wellbeing": ("wellbeing", "response:wellbeing", "Wellbeing small talk"),
    "farewell": ("farewell", "response:farewell", "Farewell"),
    "courtesy_response": (
        "courtesy_response",
        "response:courtesy_response",
        "Courtesy response",
    ),
    "identity": ("identity", "response:identity", "Identity question"),
    "capabilities": ("capabilities", "response:capabilities", "Capability question"),
    "test_status": ("test_status", "response:test_status", "Test status probe"),
    "assistant_free_time": (
        "assistant_free_time",
        "response:assistant_free_time",
        "Assistant free-time small talk",
    ),
    "assistant_name": (
        "assistant_name",
        "response:assistant_name",
        "Assistant name question",
    ),
    "calculation": ("calculation", "response:calculation", "Arithmetic request"),
}

# The arithmetic cases assert the computed value too: a routed-but-wrong sum
# would otherwise pass. Every other case is fully pinned by intent + evidence.
CALCULATION_VALUES = {
    "two plus two": "4",
    "three plus five": "8",
    "seven minus four": "3",
    "six times seven": "42",
    "ten divided by two": "5",
    "eight modulo three": "2",
    "два плюс два": "4",
    "три плюс пять": "8",
    "семь минус четыре": "3",
    "шесть умножить на семь": "42",
    "десять разделить на два": "5",
    "दो जोड़ दो": "4",
    "दो जमा दो": "4",
    "सात घटा चार": "3",
    "छह गुणा सात": "42",
    "दस भाग दो": "5",
    "二 加上 二": "4",
    "七 减去 四": "3",
    "六 乘以 七": "42",
    "十 除以 二": "5",
    "八 取模 三": "2",
}


def quote(value):
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def probed_answers():
    """prompt -> the answer the engine gave it, from `probe-04.tsv`.

    The probe (`examples/issue_933_variation_probe.rs`) prints multi-line
    answers with the newlines replaced by " ⏎ ", so an answer carrying that
    marker is one the corpus records by its opening line rather than verbatim.

    `probe-03.tsv` is the same run against the engine before the
    question-necessity parity fix; it is kept because it is the evidence that
    sixteen Hindi and Chinese prompts answered with an empty string.
    """
    answers = {}
    for line in (HERE / "probe-04.tsv").read_text(encoding="utf8").splitlines():
        if not line.strip():
            continue
        prompt, _intent, _links, answer = line.split("\t")
        answers[prompt] = answer
    return answers


def main():
    rows = []
    for line in (HERE / "candidates.tsv").read_text(encoding="utf8").splitlines():
        if not line.strip():
            continue
        case, language, prompt = line.split("\t")
        rows.append((case, language, prompt))

    answers = probed_answers()
    by_language = collections.defaultdict(list)
    counters = collections.Counter()
    total = 0
    for case, language, prompt in rows:
        intent, evidence, _title = CASES[case]
        counters[(case, language)] += 1
        identifier = f"{case}_{language}_{counters[(case, language)]:02d}"
        lines = [
            f"conversational_variation_case_{identifier}",
            '  record_type "conversational_variation_case"',
            f"  id {quote(identifier)}",
            f"  case {quote(case)}",
            f"  language {quote(language)}",
            '  source "self_authored_multilingual_variation"',
            f"  prompt {quote(prompt)}",
            f"  expected_intent {quote(intent)}",
            f"  expected_evidence {quote(evidence)}",
        ]
        # The recorded answer is what makes the corpus documentation as well as
        # a floor: a reader sees the exact text each wording produces (R234-2).
        # The capability answer is a multi-paragraph markdown listing, so those
        # records pin its opening line instead of inlining the whole page 21
        # times.
        answer = answers[prompt]
        if " ⏎ " in answer:
            lines.append(
                f"  expected_answer_contains {quote(answer.split(' ⏎ ')[0])}"
            )
        else:
            lines.append(f"  expected_answer {quote(answer)}")
        if case == "calculation":
            lines.append(
                f"  expected_answer_contains {quote(CALCULATION_VALUES[prompt])}"
            )
        by_language[language].append("\n".join(lines))
        total += 1

    partitions = ROOT / "data/benchmarks/conversational-variations"
    partitions.mkdir(parents=True, exist_ok=True)
    for language in LANGUAGES:
        (partitions / f"{language}.lino").write_text(
            "\n".join(by_language[language]) + "\n", encoding="utf8"
        )

    minimum = min(counters.values())
    manifest = [
        "conversational_variation_suite_issue_933",
        '  record_type "conversational_variation_suite"',
        '  id "issue_933_conversational_wording_variations"',
        '  title "Issue 933 conversational wording variation suite"',
        '  purpose "Hold at least five verified wording variations per supported'
        ' language for every conversational test case."',
        '  runner "cargo test --test unit'
        ' conversational_variation_benchmark_routes_every_case -- --nocapture"',
        '  floor_checker "npm run --prefix tests/e2e check:variation-floor"',
        f'  minimum_variations_per_language "5"',
        f'  minimum_pass_count "{total}"',
        '  languages "en|ru|hi|zh"',
    ]
    for case in CASES:
        manifest.append(f"  case {quote(case)}")
    for language in LANGUAGES:
        manifest.append(f'  member_file "conversational-variations/{language}.lino"')
    manifest += [
        '  source_policy "Self-authored prompt variations, each verified against'
        ' the deterministic engine before it was recorded."',
        '  ratchet_policy "CI fails when any case holds fewer than'
        ' minimum_variations_per_language distinct prompts in any listed language."',
    ]
    (ROOT / "data/benchmarks/conversational-variations-suite.lino").write_text(
        "\n".join(manifest) + "\n", encoding="utf8"
    )

    print(f"cases={len(CASES)} prompts={total} smallest_group={minimum}")
    for case in CASES:
        print(case, {language: counters[(case, language)] for language in LANGUAGES})


if __name__ == "__main__":
    main()
