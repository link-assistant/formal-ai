#!/usr/bin/env python3
"""Do the two seed cue families co-occur inside one request sentence?

`evidence_record::parse_obligation` decides, sentence by sentence, whether the
file a sentence names is where an answer gets delivered or the operand the work
acts on. The obvious rule -- "a sentence that mentions an edit cue is doing
work, not delivering" -- is only sound if the families do not co-occur. This
measures that over every request sentence this repository records, which are
the labelled examples the planner is actually held to.

Run: python3 experiments/issue_1069_delivery_vs_operand/cue-order-survey.py
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
    examples = []
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
    print("sentences carrying both families, and the cue each leads with:")
    for lead, name, sentence in examples:
        print(f"  [{lead:5}] {name}: {sentence}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
