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
import glob
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


def collect(root):
    """Return (defined_meaning_slugs, distinct_role_values)."""
    files = sorted(glob.glob(os.path.join(root, "data/seed/meanings*.lino")))
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
    root = sys.argv[1] if len(sys.argv) > 1 else REPO_ROOT
    defined, roles = collect(root)
    text = render(defined, roles)
    target = os.path.join(root, "data/seed/roles.lino")
    with open(target, "w", encoding="utf-8") as fh:
        fh.write(text)
    both = sum(1 for r in roles if r in defined)
    print(f"roles: {len(roles)} ({both} also meanings, {len(roles) - both} predicate-only)")
    print(f"wrote {target}")


if __name__ == "__main__":
    main()
