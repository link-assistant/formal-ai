#!/usr/bin/env python3
"""Survey what a *member literal* actually looks like in this repository.

`is_member_literal` refuses any value containing whitespace, which makes an
ordinary list of multi-word members (greetings, prepositional cues) uneditable.
Before relaxing the rule, measure the real population: every string literal that
sits directly inside a bracketed slice/array in the tracked Rust sources. The
bound the route enforces should come from that data rather than from a guess.
"""
import re
import subprocess
from collections import Counter

files = subprocess.run(
    ["git", "ls-files", "src/*.rs"], capture_output=True, text=True, check=True
).stdout.split()

LIST = re.compile(r"&?\[((?:\s*\"(?:[^\"\\]|\\.)*\"\s*,?)+)\s*\]", re.S)
LITERAL = re.compile(r"\"((?:[^\"\\]|\\.)*)\"")

words = Counter()
lengths = Counter()
examples = {}
total = 0
for path in files:
    source = open(path, encoding="utf-8").read()
    for listing in LIST.finditer(source):
        for literal in LITERAL.finditer(listing.group(1)):
            value = literal.group(1)
            if not value:
                continue
            total += 1
            count = len(value.split())
            words[count] += 1
            lengths[len(value)] += 1
            examples.setdefault(count, []).append(value)

print(f"member literals found: {total}")
print("words per member (count -> members):")
for count in sorted(words):
    sample = ", ".join(repr(v) for v in examples[count][:4])
    print(f"  {count:>2}: {words[count]:>5}   e.g. {sample}")
print(f"longest member: {max(lengths)} bytes")
covered = 0
for count in sorted(words):
    covered += words[count]
    print(f"  <= {count} words covers {100 * covered / total:.3f}% of members")

whitespace = [v for c in examples for v in examples[c] if any(ch.isspace() for ch in v)]
boundary = [v for v in whitespace if v != v.strip()]
multiline = [v for v in whitespace if "\n" in v or "\t" in v]
print(f"members holding whitespace: {len(whitespace)} ({100 * len(whitespace) / total:.1f}%)")
print(f"  of those, boundary-space only at an edge: {len(boundary)}")
print(f"  of those, spanning lines: {len(multiline)}")

allv = [v for c in examples for v in examples[c]]
clause = [v for v in allv if ", " in v]
sentence_end = [v for v in allv if v.rstrip().endswith(".")]
print(f"members holding a comma-space: {len(clause)} ({100 * len(clause) / total:.2f}%)  e.g. {[v for v in clause[:3]]}")
print(f"members ending in a period: {len(sentence_end)} ({100 * len(sentence_end) / total:.2f}%)  e.g. {[v for v in sentence_end[:3]]}")
