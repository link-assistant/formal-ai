#!/usr/bin/env python3
"""Measure total-closure surface: distinct word-like VALUE tokens that resolve
to no defined meaning / grounded source / override. Throwaway measurement."""
import glob
import os
import re
import sys
from collections import Counter

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
SLUG = re.compile(r"^[a-z][a-z0-9_-]*$")
QID = re.compile(r"^[QLP][0-9]+$")
LANGS = {"en", "ru", "hi", "zh"}


def indent(line):
    return len(line) - len(line.lstrip(" "))


def strip_comment_and_quotes(s):
    """Return (head, value_tokens) with quoted spans removed."""
    # remove trailing comment
    out = []
    i = 0
    q = 0
    buf = ""
    # tokenize, dropping quoted spans entirely
    while i < len(s):
        c = s[i]
        if c in "\"'`":
            j = s.find(c, i + 1)
            if j == -1:
                break
            i = j + 1
            buf += " "  # placeholder so head/value split still works
            continue
        if c == "#" and i > 0 and s[i - 1] == " ":
            break
        buf += c
        i += 1
    toks = buf.split()
    return toks


def defined_slugs(files):
    defined = set()
    for path in files:
        lines = open(path, encoding="utf-8").read().split("\n")
        stack = []
        for raw in lines:
            if not raw.strip():
                continue
            ind = indent(raw)
            stripped = raw.strip()
            while stack and stack[-1][0] >= ind:
                stack.pop()
            head = stripped.split(" ", 1)[0]
            if head.endswith(":"):
                head = head[:-1]
            if stack and stack[-1][1] == "meanings":
                defined.add(head)
            stack.append((ind, head))
    return defined


def main():
    files = sorted(glob.glob(os.path.join(ROOT, "data/seed/*.lino")))
    defined = defined_slugs(files)
    wikt = {os.path.splitext(os.path.basename(p))[0]
            for p in glob.glob(os.path.join(ROOT, "data/cache/wiktionary/en/*.json"))}
    wikt |= {os.path.splitext(os.path.basename(p))[0]
             for p in glob.glob(os.path.join(ROOT, "data/cache/wiktionary/*/*.json"))}
    wikt -= {"reference"}
    wordnet = {os.path.splitext(os.path.basename(p))[0]
               for p in glob.glob(os.path.join(ROOT, "data/cache/wordnet/en/*.json"))}
    qids = {os.path.splitext(os.path.basename(p))[0]
            for p in glob.glob(os.path.join(ROOT, "data/cache/wikidata/*/*.json"))}

    vals = Counter()
    en_surface = Counter()
    for path in files:
        lines = open(path, encoding="utf-8").read().split("\n")
        lex_lang = None
        for raw in lines:
            if not raw.strip():
                continue
            toks = strip_comment_and_quotes(raw.strip())
            if not toks:
                continue
            head, values = toks[0], toks[1:]
            if head == "lexeme" and values:
                lex_lang = values[0]
            # expand combos on '+'
            expanded = []
            for v in values:
                expanded.extend(v.split("+"))
            for v in expanded:
                if not SLUG.match(v):
                    continue
                if v in LANGS:
                    continue
                # only English content: skip surfaces under non-en lexeme
                if head == "text" and lex_lang not in (None, "en"):
                    continue
                vals[v] += 1

    def resolves(t):
        return (t in defined or t in wikt or t in wordnet or QID.match(t))

    unresolved = {t: c for t, c in vals.items() if not resolves(t)}
    print(f"defined meaning slugs: {len(defined)}")
    print(f"wiktionary lemmas cached: {len(wikt)}  wordnet: {len(wordnet)}  qids: {len(qids)}")
    print(f"distinct word-like value tokens: {len(vals)}")
    print(f"UNRESOLVED distinct: {len(unresolved)}  occurrences: {sum(unresolved.values())}")
    print("\n=== top unresolved ===")
    for t, c in sorted(unresolved.items(), key=lambda x: (-x[1], x[0]))[:80]:
        print(f"{c:4d}  {t}")


if __name__ == "__main__":
    main()
