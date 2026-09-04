#!/usr/bin/env python3
"""Is a named file a delivery destination or the operand of the work?

`evidence_record::parse_obligation` decides that sentence by sentence. Two
candidate rules are measured here over every request sentence this repository
records, which are the labelled examples the planner is actually held to:

  1. "a sentence that mentions an edit cue is doing work, not delivering",
     which is only sound if the two seed cue families do not co-occur; and
  2. "the path of a delivery follows the action that composes what goes in it",
     which is the rule adopted -- `delivered_write_target`; and
  3. among the sentences rule 2 admits, how many name their path as a
     double-quoted literal, which is the second half of that rule.

Run: python3 experiments/issue_1069_paths_in_prose/cue-order-survey.py
"""

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def surfaces(role: str) -> list[str]:
    """The English bare surfaces the seed declares for `role`."""
    found: list[str] = []
    for seed_file in sorted((ROOT / "data" / "seed").glob("*.lino")):
        text = seed_file.read_text(encoding="utf-8")
        block = re.search(rf"^\s*role {role}\n(.*?)(?=^\s{{0,4}}\w+\n\s+defined-by)",
                          text, re.S | re.M)
        if not block:
            continue
        english = re.search(r"lexeme en\n(.*?)(?=\n    lexeme |\Z)", block.group(1), re.S)
        if not english:
            continue
        found += re.findall(r'text "?([^"\n]+?)"?\s*$', english.group(1), re.M)
    return sorted({word.strip().lower() for word in found if word.strip()})


WRITE = surfaces("file_write_action_cue")
EDIT = surfaces("file_edit_action_cue")

PATH = re.compile(r"[A-Za-z0-9_./-]+\.[A-Za-z][A-Za-z0-9]{0,7}\b")
WORD = re.compile(r"[A-Za-z_][A-Za-z_-]*")


def sentences(text: str):
    return [part.strip() for part in re.split(r"(?<=[.!?])\s+|\n", text) if part.strip()]


def path_follows_action(sentence: str):
    """Where the first path sits relative to the first write-action cue.

    This is `delivered_write_target` in Python: the delivery clause opens at the
    action, so a destination is named after it and an operand before it.
    Returns None when the sentence carries no write cue at all.
    """
    action = None
    for match in WORD.finditer(sentence.lower()):
        if match.group(0) in WRITE:
            action = match.start()
            break
    if action is None:
        return None
    path = PATH.search(sentence)
    return path is not None and path.start() > action


def quotes_its_path(sentence: str) -> bool:
    """Whether the first path in the sentence is written as a quoted literal.

    This is `quoted_as_value` in Python: a path is mentioned bare or in the
    backticks this repository uses for code, while a double-quoted token is a
    value the sentence hands to something else.
    """
    match = PATH.search(sentence)
    if match is None:
        return False
    return (sentence[match.start() - 1: match.start()] == '"'
            and sentence[match.end(): match.end() + 1] == '"')


def first_family(sentence: str):
    """Which cue family the sentence leads with, and every family it mentions."""
    lead = None
    mentioned = set()
    for match in WORD.finditer(sentence.lower()):
        word = match.group(0)
        family = "write" if word in WRITE else "edit" if word in EDIT else None
        if family is None:
            continue
        mentioned.add(family)
        if lead is None:
            lead = family
    return lead, mentioned


def corpus():
    """Every string literal in the test suite, which is where the recorded
    request wordings live."""
    files = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files", "tests", "experiments"],
        capture_output=True, text=True, check=True).stdout.split()
    for name in files:
        path = ROOT / name
        if path.suffix not in {".rs", ".sh"} or not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for literal in re.findall(r'"((?:[^"\\]|\\.){20,})"', text):
            yield name, literal.replace("\\\n", " ").replace('\\"', '"')


def main() -> int:
    total = both = lead_write = lead_edit = 0
    after = before = no_write_cue = 0
    examples = []
    quoted_paths = []
    for name, literal in corpus():
        for sentence in sentences(literal):
            if not PATH.search(sentence):
                continue
            lead, mentioned = first_family(sentence)
            if lead is None:
                continue
            total += 1
            if lead == "write":
                lead_write += 1
            else:
                lead_edit += 1
            follows = path_follows_action(sentence)
            if follows is None:
                no_write_cue += 1
            elif follows:
                after += 1
                if quotes_its_path(sentence):
                    quoted_paths.append((name, " ".join(sentence.split())[:150]))
            else:
                before += 1
            if len(mentioned) == 2:
                both += 1
                if len(examples) < 40:
                    examples.append((lead, name, " ".join(sentence.split())[:150]))

    print(f"write-action cues: {len(WRITE)} -> {WRITE}")
    print(f"edit-action cues:  {len(EDIT)} -> {EDIT}")
    print()
    print(f"recorded request sentences naming a file and carrying a cue: {total}")
    print(f"  lead with a write cue: {lead_write} ({lead_write / total:.2%})")
    print(f"  lead with an edit cue: {lead_edit} ({lead_edit / total:.2%})")
    print(f"  mention BOTH families: {both} ({both / total:.2%})")
    print()
    print("rule 2 -- position of the path relative to the first write-action cue:")
    print(f"  path follows the action (delivery destination): {after} ({after / total:.2%})")
    print(f"  path precedes it (operand of the work):         {before} ({before / total:.2%})")
    print(f"  no write cue at all (an edit, never delivery):  {no_write_cue} "
          f"({no_write_cue / total:.2%})")
    print()
    print("rule 3 -- how the sentences rule 2 admits write their path:")
    print(f"  double-quoted (a value, not a destination): {len(quoted_paths)} "
          f"({len(quoted_paths) / after:.2%} of {after})")
    print(f"  bare or backticked (a mention):             {after - len(quoted_paths)}")
    for name, sentence in quoted_paths:
        print(f"    {name}: {sentence}")
    print()
    print("sentences carrying both families, and the cue each leads with:")
    for lead, name, sentence in examples:
        print(f"  [{lead:5}] {name}: {sentence}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
