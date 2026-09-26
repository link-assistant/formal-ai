#!/usr/bin/env python3
"""Drive total reference-closure to zero by defining every dangling token (issue #398, PR #399).

PR #399 review (comment 4668929105) requires *total* closure: every semantic
reference anywhere in ``data/seed/**.lino`` must resolve to a defined meaning, a
grounded source id with a cache record, or an override — and the build fails the
instant one does not. WordNet/Wiktionary grounding (``ground-wordnet.py``,
``ground-wiktionary.py``) closes the plain-English dictionary words. What remains
is the *internal* vocabulary: intent names, task names, source kinds,
programming-language tags, and similar identifiers that name concepts of this
system itself. Declaration identities and literal matcher operands are not
references; their schema classification comes from ``audit-total-closure.py``.
The reviewer's standard is explicit:
"Every word used in a description/intent/definition is either a defined meaning, a
grounded lexeme/sense, or it must be **made** one." This migration *makes* them.

It is a re-runnable migration, preserved in the repo per the review's "use
automated scripts for mass actions … preserve them" instruction:

  1. Compute the unresolved value tokens over every ``data/seed/*.lino``
     file. The ``closure-generated-*.lino`` filter in ``base_files`` is kept as
     a guard: nothing writes those files any more, and nothing may start.
  2. For each unresolved token, derive a parent meaning from the predicate (head)
     it most often appears under — ``intent`` → the ``intent`` concept, ``task`` →
     ``task``, ``pattern`` → ``pattern``, ``language`` → ``programming_language``,
     and so on — so the work list is a real two-level taxonomy rather than a
     flat dump, and a parent category that does not resolve either is listed
     alongside its members.
  3. Print each token as a line of a grounding work list, under the parent
     category it would belong to, with the surface a human reader would give it.

This script **writes nothing**. It used to emit
``data/seed/closure-generated-NN.lino``, and ``audit-total-closure.py`` then
read those files back as definitions, so the closure metric measured this
script's own output and reported zero by construction while 17,791 lines of
English-only glosses no runtime loads stood in for grounding (issue #1138 B9,
plan 09 leaf 8). A generated gloss is not grounding. The generator may propose
work; only an authored meanings file the runtime loads may satisfy the gate, and
``data/meta/closure-audit.lino`` records how much work is left.

Run ``python3 scripts/close-total.py`` for the work list and
``python3 scripts/audit-total-closure.py`` for the honest count.
"""
from __future__ import annotations

import glob
import importlib.util
import os
import re
from collections import defaultdict
from pathlib import Path

SEED_DIR = Path("data/seed")
GENERATED_PREFIX = "closure-generated-"
QID = re.compile(r"^[QLP][0-9]+$")
LANG_CODES = {"en", "ru", "hi", "zh"}

# Predicate (head) -> parent meaning for the tokens that appear under it. The
# parent names a real category; tokens become children of that category.
PARENT_BY_HEAD = {
    "intent": "intent",
    "expected_intent": "intent",
    "pattern": "prompt_pattern",
    "source_kind": "source_kind",
    "task": "task",
    "response": "response_template",
    "name": "name",
    "display_name": "name",
    "language": "programming_language",
    "code_fence": "programming_language",
    "category": "category",
    "kind": "kind",
    "tool": "tool",
    "operation": "operation",
    "isolation": "isolation_mode",
    "field": "field",
    "flow": "flow",
    "rule": "rule",
    "context": "context",
    "context_links": "context",
    "environment": "environment",
    "value": "value",
    "term": "term",
    "topic": "topic",
    "org": "organization",
    "slug": "concept",
    "token": "token",
    "coverage_group": "coverage_group",
    "text": "word_surface",
    "phrase": "phrase",
    "trace_prefix": "identifier",
    "source": "source",
    "timeline": "release_timeline",
    "release_timeline": "release_timeline",
}
DEFAULT_PARENT = "concept"

# How to root any parent category that does not already resolve in the base seed.
PARENT_DEFINED_BY = {
    "intent": "concept",
    "release_timeline": "concept",
    "prompt_pattern": "concept",
    "source_kind": "source",
    "task": "concept",
    "response_template": "response",
    "programming_language": "language",
    "category": "concept",
    "operation": "action",
    "isolation_mode": "concept",
    "field": "concept",
    "flow": "concept",
    "rule": "concept",
    "context": "concept",
    "environment": "concept",
    "term": "concept",
    "organization": "entity",
    "coverage_group": "concept",
}


def _load_audit():
    spec = importlib.util.spec_from_file_location(
        "audit_total_closure", "scripts/audit-total-closure.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def base_files() -> list[str]:
    return [
        p
        for p in sorted(glob.glob(str(SEED_DIR / "*.lino")))
        if not os.path.basename(p).startswith(GENERATED_PREFIX)
    ]


def base_defined_slugs(audit, files: list[str]) -> set[str]:
    """Slugs defined (either syntax) in the given base files."""
    defined: set[str] = set()
    for path in files:
        lines = Path(path).read_text(encoding="utf-8").split("\n")
        stack: list[tuple[int, str]] = []
        for raw in lines:
            if not raw.strip():
                continue
            ind = audit._indent(raw)
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


def base_tokens(audit, files: list[str]):
    """Return the canonical audit's counts and dominant predicate per token."""
    del files  # file filtering is centralized in the canonical inventory
    counts, dominant, _ = audit.semantic_reference_inventory(
        ".", include_generated=False
    )
    return counts, dominant


def surface_for(token: str) -> str:
    return token.replace("_", " ").replace("-", " ").strip()


def main() -> int:
    audit = _load_audit()
    files = base_files()
    resolver = audit.Resolver(".")  # caches/roles/ids are unaffected by generated files
    defined_base = base_defined_slugs(audit, files)
    counts, dominant = base_tokens(audit, files)

    def resolves_base(token: str) -> bool:
        if token in LANG_CODES:
            return True
        if QID.match(token):
            return token in resolver.ids
        if token in defined_base or token in resolver.roles:
            return True
        if token in resolver.wiktionary or token in resolver.wordnet:
            return True
        return False

    unresolved = sorted(t for t in counts if not resolves_base(t))
    print(f"unresolved tokens to define: {len(unresolved)}")

    # Determine the parent for each token, and which parents need defining.
    token_parent: dict[str, str] = {}
    needed_parents: set[str] = set()
    for token in unresolved:
        parent = PARENT_BY_HEAD.get(dominant.get(token, ""), DEFAULT_PARENT)
        # A token cannot be its own parent; fall back to the default.
        if parent == token:
            parent = DEFAULT_PARENT
        token_parent[token] = parent
        if not resolves_base(parent):
            needed_parents.add(parent)

    # Parents are defined first (and may themselves need a parent definition).
    parent_defs: dict[str, str] = {}
    for parent in sorted(needed_parents):
        parent_defs[parent] = PARENT_DEFINED_BY.get(parent, "concept")

    # Build the full ordered list of (slug, parent) blocks: parents first so a
    # reader meets a category before its members.
    blocks: list[tuple[str, str]] = []
    for parent in sorted(parent_defs):
        blocks.append((parent, parent_defs[parent]))
    for token in unresolved:
        if token in parent_defs:
            continue  # already emitted as a parent
        blocks.append((token, token_parent[token]))

    # Issue #1138 B9, plan 09 leaf 8. This script used to write
    # `data/seed/closure-generated-NN.lino` here, and
    # `scripts/audit-total-closure.py` then read those files back as
    # definitions -- so the closure metric measured this script's own output and
    # reported zero by construction. 17,791 lines of English-only glosses no
    # runtime loads stood in for grounding, and the generator satisfied the gate
    # it was supposed to be measured by.
    #
    # The generator now *proposes work* and satisfies nothing. It prints the
    # tokens that resolve to no defined meaning, no grounded source and no
    # override, grouped by the parent category each would belong under, so the
    # list can be worked through by grounding tokens in an authored meanings
    # file the runtime loads. The number it prints is the number
    # `data/meta/closure-audit.lino` ratchets down.
    by_parent: dict[str, list[str]] = defaultdict(list)
    for slug, parent in blocks:
        by_parent[parent].append(slug)

    print()
    print("=== grounding work list ===")
    for parent in sorted(by_parent):
        members = sorted(by_parent[parent])
        print(f"{parent} ({len(members)})")
        for slug in members:
            print(f"  {slug}    # suggested surface: {surface_for(slug)!r}")

    print()
    print(
        f"{len(unresolved)} unresolved token(s) across {len(by_parent)} proposed "
        f"parent categories."
    )
    print(
        "Ground them in an authored meanings file the runtime loads. This script "
        "writes nothing: a generated gloss is not grounding, and the ratchet in "
        "data/meta/closure-audit.lino measures what is."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
