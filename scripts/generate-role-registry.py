#!/usr/bin/env python3
"""Generate the canonical reserved-role registry `data/seed/roles.lino`.

Issue #398 (PR #399 review comment 4668342875, point #2 and CI check #3): a
`role <name>` value used in `data/seed/meanings*.lino` is a *predicate over
meanings*, not a free token. Every role must therefore be declared once in a
single, documented registry so it cannot silently collide with — or diverge
from — the meaning graph.

This script mines every distinct `role` value from the meaning seed, classifies
each as either:

* `meaning` — the role name is *also* a defined meaning slug (a category
  meaning that doubles as the role it confers, e.g. `ontology_category`), or
* `predicate` — a reserved role-only identifier with no meaning of the same
  name,

and writes them, sorted, to `data/seed/roles.lino`. The matching CI tests in
`tests/unit/data_files.rs` assert the registry stays in lockstep with usage.

The transform is deterministic and idempotent: re-running it reproduces the
file byte-for-byte. Run with `python3 scripts/generate-role-registry.py`.
"""
import argparse
import os
import re
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SLUG_RE = re.compile(r"^[a-z][a-z0-9_-]*$")


def indent(line):
    return len(line) - len(line.lstrip(" "))


def strip_comment(stripped):
    quote = 0
    for k, ch in enumerate(stripped):
        if ch in "\"'`":
            quote ^= 1
        if ch == "#" and not quote and k > 0 and stripped[k - 1] == " ":
            return stripped[:k].rstrip()
    return stripped


def meaning_files(root):
    """Return the seed files the lexicon loads: `MEANING_FILES` in the registry.

    The constant lists `*_LINO` names; each is bound to its file by an
    `include_str!` of the embedded mirror, whose data/seed twin is read here.
    """
    registry = os.path.join(root, "rust/src/seed/embedded_registry.rs")
    with open(registry, encoding="utf-8") as fh:
        source = fh.read()
    paths = dict(
        re.findall(
            r'pub const (\w+): &str =\s*include_str!\("\.\./\.\./embedded/(data/seed/[^"]+)"\)',
            source,
        )
    )
    block = source.split("pub const MEANING_FILES: &[&str] = &[", 1)[1].split("];", 1)[0]
    names = re.findall(r"\b(\w+_LINO)\b", block)
    return sorted(os.path.join(root, paths[name]) for name in names)


def collect(root):
    """Return (defined_meaning_slugs, distinct_role_values)."""
    files = meaning_files(root)
    defined = set()
    roles = set()
    for path in files:
        with open(path, encoding="utf-8") as fh:
            lines = fh.read().split("\n")
        stack = []
        for raw in lines:
            if not raw.strip():
                continue
            ind = indent(raw)
            stripped = strip_comment(raw.strip())
            if not stripped:
                continue
            while stack and stack[-1][0] >= ind:
                stack.pop()
            head = stripped.split(" ", 1)[0]
            if head.endswith(":"):
                head = head[:-1]
            if stack and stack[-1][1] == "meanings":
                defined.add(head)
            if head == "role" and " " in stripped:
                val = stripped.split(" ", 1)[1].strip()
                tok = val.split(" ", 1)[0]
                if SLUG_RE.match(tok):
                    roles.add(tok)
            stack.append((ind, head))
    return defined, roles


def render(defined, roles):
    out = [
        "# Canonical reserved-role registry (issue #398, PR #399).",
        "#",
        "# Every `role <name>` value used anywhere in data/seed/meanings*.lino is",
        "# declared here exactly once. A role is a reserved predicate over",
        "# meanings; this registry is its single definition so it can never",
        "# collide with — or drift from — the meaning graph. Regenerate with",
        "# `python3 scripts/generate-role-registry.py`; CI keeps it in lockstep.",
        "#",
        "# kind meaning   -> the role name is also a defined meaning slug",
        "# kind predicate -> the role name is a reserved role-only identifier",
        "roles",
    ]
    for name in sorted(roles):
        kind = "meaning" if name in defined else "predicate"
        out.append(f"  {name}")
        out.append(f"    kind {kind}")
    return "\n".join(out) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", default=REPO_ROOT)
    parser.add_argument("--check", action="store_true", help="Compare without changing any bytes")
    parser.add_argument("--output", help="Explicit generated projection destination")
    arguments = parser.parse_args()
    defined, roles = collect(arguments.root)
    text = render(defined, roles)
    target = arguments.output or os.path.join(arguments.root, "data/seed/roles.lino")
    if arguments.check:
        try:
            with open(target, encoding="utf-8") as fh:
                actual = fh.read()
        except FileNotFoundError:
            actual = None
        if actual != text:
            print(f"role registry differs: {target}", file=sys.stderr)
            return 1
    else:
        with open(target, "w", encoding="utf-8") as fh:
            fh.write(text)
    both = sum(1 for r in roles if r in defined)
    print(f"roles: {len(roles)} ({both} also meanings, {len(roles) - both} predicate-only)")
    print(f"{'checked' if arguments.check else 'wrote'} {target}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
