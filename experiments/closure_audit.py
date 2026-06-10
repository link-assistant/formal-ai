#!/usr/bin/env python3
"""Audit meaning-layer reference closure across data/seed/meanings*.lino.

A *defined* meaning slug is a direct child of a top-level `meanings` block.
A *reference* is any meaning-slug-shaped value of a definitional field
(`defined-by`, `role`, `notation`, `annotation`, `denotation`, `connotation`).
Reports references that resolve to no defined slug.
"""
import glob
import os
import re
import sys
from collections import Counter

REF_FIELDS = {
    "defined-by",
    "role",
    "notation",
    "annotation",
    "denotation",
    "connotation",
}

# Values that are not meaning-slug references.
QID_RE = re.compile(r"^[QLP][0-9]+$")
SLUG_RE = re.compile(r"^[a-z][a-z0-9_-]*$")


def indent(line):
    return len(line) - len(line.lstrip(" "))


def tokens(value):
    """Split a field value into reference tokens, dropping quoted scalars."""
    out = []
    i = 0
    s = value.strip()
    while i < len(s):
        c = s[i]
        if c in "\"'`":
            j = s.find(c, i + 1)
            if j == -1:
                break
            i = j + 1
            continue
        if c in "()":
            i += 1
            continue
        if c == " ":
            i += 1
            continue
        j = i
        while j < len(s) and s[j] not in " ()":
            j += 1
        out.append(s[i:j])
        i = j
    return out


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    files = sorted(glob.glob(os.path.join(root, "data/seed/meanings*.lino")))
    defined = set()
    refs = Counter()
    ref_locs = {}
    by_field = {f: Counter() for f in REF_FIELDS}

    for path in files:
        with open(path, encoding="utf-8") as fh:
            lines = fh.read().split("\n")
        stack = []  # (indent, head)
        for lineno, raw in enumerate(lines, 1):
            if not raw.strip():
                continue
            ind = indent(raw)
            stripped = raw.strip()
            # strip trailing ' # comment' (only when not inside a quote span)
            if " #" in stripped:
                q = 0
                for k, ch in enumerate(stripped):
                    if ch in "\"'`":
                        q ^= 1
                    if ch == "#" and not q and k > 0 and stripped[k - 1] == " ":
                        stripped = stripped[:k].rstrip()
                        break
            while stack and stack[-1][0] >= ind:
                stack.pop()
            head = stripped.split(" ", 1)[0]
            # colon-head definition: `slug: expression` (no space before colon)
            colon_expr = None
            if head.endswith(":"):
                head = head[:-1]
                colon_expr = stripped.split(":", 1)[1] if ":" in stripped else ""
            # defined meaning: direct child of a `meanings` block
            if stack and stack[-1][1] == "meanings":
                defined.add(head)
                # the colon definition expression is a `defined-by` reference set
                if colon_expr:
                    for tok in tokens(colon_expr):
                        if QID_RE.match(tok) or SLUG_RE.match(tok) is None:
                            continue
                        refs[tok] += 1
                        by_field["defined-by"][tok] += 1
            # reference fields
            if head in REF_FIELDS and " " in stripped:
                val = stripped.split(" ", 1)[1]
                for tok in tokens(val):
                    if QID_RE.match(tok) or SLUG_RE.match(tok) is None:
                        continue
                    refs[tok] += 1
                    by_field[head][tok] += 1
                    ref_locs.setdefault(tok, []).append(f"{os.path.basename(path)}:{lineno}")
            stack.append((ind, head))

    undefined = {t: c for t, c in refs.items() if t not in defined}
    print(f"defined meaning slugs: {len(defined)}")
    print(f"distinct referenced slugs (def-fields): {len(refs)}")
    print(f"undefined referenced slugs: {len(undefined)}")
    print(f"total undefined occurrences: {sum(undefined.values())}")
    print("\n=== undefined by source field ===")
    for f in sorted(REF_FIELDS):
        u = {t: c for t, c in by_field[f].items() if t not in defined}
        print(f"{f:14s} undefined-distinct={len(u):3d} occ={sum(u.values())}")
    print("\n=== undefined ONLY ever used as a role (role-ontology candidates) ===")
    role_only = []
    other = []
    for t in undefined:
        fields = {f for f in REF_FIELDS if t in by_field[f]}
        (role_only if fields == {"role"} else other).append(t)
    print("role-only:", len(role_only))
    print("other (defined-by/facet refs):", len(other))
    print("\n=== undefined used in defined-by/facets (true meaning gaps) ===")
    for t in sorted(other, key=lambda x: (-refs[x], x)):
        print(f"{refs[x] if False else refs[t]:4d}  {t}")


if __name__ == "__main__":
    main()
